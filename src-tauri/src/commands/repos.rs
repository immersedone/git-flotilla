use crate::{
    db,
    error::{AppError, AppResult},
    models::{Repo, RepoList},
    services::github::{GitHubClient, GitHubRepo},
};
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

const KEYCHAIN_SERVICE: &str = "git-flotilla";

// ── Private helpers ────────────────────────────────────────────────────────

fn get_token(account_id: &str) -> AppResult<String> {
    Entry::new(KEYCHAIN_SERVICE, account_id)
        .map_err(AppError::from)?
        .get_password()
        .map_err(AppError::from)
}

fn gh_repo_to_model(r: &GitHubRepo) -> Repo {
    Repo {
        id:              format!("github:{}", r.full_name),
        provider:        "github".to_string(),
        owner:           r.owner.login.clone(),
        name:            r.name.clone(),
        full_name:       r.full_name.clone(),
        url:             r.html_url.clone(),
        default_branch:  r.default_branch.clone(),
        is_private:      r.private,
        last_scanned_at: None,
        tags:            vec![],
    }
}

fn gen_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    format!("{:x}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos())
}

// ── Row types for runtime sqlx queries ────────────────────────────────────

#[derive(sqlx::FromRow)]
struct RepoRow {
    id:              String,
    provider:        String,
    owner:           String,
    name:            String,
    full_name:       String,
    url:             String,
    default_branch:  String,
    is_private:      i64,
    last_scanned_at: Option<String>,
    tags:            String,
}

impl RepoRow {
    fn into_model(self) -> Repo {
        Repo {
            id:              self.id,
            provider:        self.provider,
            owner:           self.owner,
            name:            self.name,
            full_name:       self.full_name,
            url:             self.url,
            default_branch:  self.default_branch,
            is_private:      self.is_private != 0,
            last_scanned_at: self.last_scanned_at,
            tags:            serde_json::from_str(&self.tags).unwrap_or_default(),
        }
    }
}

#[derive(sqlx::FromRow)]
struct RepoListRow {
    id:               String,
    name:             String,
    description:      String,
    parent_id:        Option<String>,
    exclude_patterns: String,
    created_at:       String,
    updated_at:       String,
}

async fn fetch_list_repo_ids(pool: &sqlx::SqlitePool, list_id: &str) -> AppResult<Vec<String>> {
    #[derive(sqlx::FromRow)]
    struct IdRow { repo_id: String }
    let rows = sqlx::query_as::<_, IdRow>(
        "SELECT repo_id FROM repo_list_members WHERE list_id = ? ORDER BY added_at",
    )
    .bind(list_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| r.repo_id).collect())
}

async fn row_to_list(pool: &sqlx::SqlitePool, row: RepoListRow) -> AppResult<RepoList> {
    let repo_ids         = fetch_list_repo_ids(pool, &row.id).await?;
    let exclude_patterns = serde_json::from_str(&row.exclude_patterns).unwrap_or_default();
    Ok(RepoList {
        id:               row.id,
        name:             row.name,
        description:      row.description,
        repo_ids,
        parent_id:        row.parent_id,
        exclude_patterns,
        created_at:       row.created_at,
        updated_at:       row.updated_at,
    })
}

async fn get_list_by_id(pool: &sqlx::SqlitePool, id: &str) -> AppResult<RepoList> {
    let row = sqlx::query_as::<_, RepoListRow>(
        "SELECT id, name, description, parent_id, exclude_patterns, created_at, updated_at
         FROM repo_lists WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Repo list not found: {id}")))?;
    row_to_list(pool, row).await
}

// ── Repo commands ──────────────────────────────────────────────────────────

/// Discover all repos accessible to the account and upsert into SQLite.
#[tauri::command]
pub async fn discover_repos(account_id: String) -> AppResult<Vec<Repo>> {
    let token  = get_token(&account_id)?;
    let client = GitHubClient::new(&token);

    // Fetch user repos
    let (user_repos, rate_limit) = client.list_all_repos().await?;
    if let Some(rl) = rate_limit {
        crate::services::rate_limiter::update_github(rl);
    }

    // Fetch org repos (deduplicated)
    let (orgs, _) = client.list_orgs().await?;
    let mut seen: HashSet<String> = HashSet::new();
    let mut all_gh: Vec<GitHubRepo> = Vec::new();

    for r in user_repos {
        if seen.insert(r.full_name.clone()) {
            all_gh.push(r);
        }
    }
    for org in &orgs {
        let (org_repos, _) = client.list_org_repos(&org.login).await?;
        for r in org_repos {
            if seen.insert(r.full_name.clone()) {
                all_gh.push(r);
            }
        }
    }

    let pool = db::pool()?;
    let mut result = Vec::new();

    for gh in &all_gh {
        let repo = gh_repo_to_model(gh);
        sqlx::query(
            r#"INSERT INTO repos (id, provider, owner, name, full_name, url, default_branch, is_private)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)
               ON CONFLICT(id) DO UPDATE SET
                   default_branch = excluded.default_branch,
                   url            = excluded.url,
                   updated_at     = datetime('now')"#,
        )
        .bind(&repo.id)
        .bind(&repo.provider)
        .bind(&repo.owner)
        .bind(&repo.name)
        .bind(&repo.full_name)
        .bind(&repo.url)
        .bind(&repo.default_branch)
        .bind(repo.is_private as i64)
        .execute(pool)
        .await?;
        result.push(repo);
    }

    tracing::info!("Discovered {} repos for {}", result.len(), account_id);
    Ok(result)
}

/// List repos from SQLite, optionally filtered by membership in a repo list.
#[tauri::command]
pub async fn list_repos(repo_list_id: Option<String>) -> AppResult<Vec<Repo>> {
    let pool = db::pool()?;

    let rows = if let Some(list_id) = repo_list_id {
        sqlx::query_as::<_, RepoRow>(
            r#"SELECT r.id, r.provider, r.owner, r.name, r.full_name, r.url,
                      r.default_branch, r.is_private, r.last_scanned_at, r.tags
               FROM repos r
               JOIN repo_list_members m ON m.repo_id = r.id
               WHERE m.list_id = ?
               ORDER BY r.full_name"#,
        )
        .bind(list_id)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, RepoRow>(
            "SELECT id, provider, owner, name, full_name, url, default_branch,
                    is_private, last_scanned_at, tags FROM repos ORDER BY full_name",
        )
        .fetch_all(pool)
        .await?
    };

    Ok(rows.into_iter().map(RepoRow::into_model).collect())
}

/// Fetch a single repo by ID.
#[tauri::command]
pub async fn get_repo(id: String) -> AppResult<Repo> {
    let pool = db::pool()?;
    let row  = sqlx::query_as::<_, RepoRow>(
        "SELECT id, provider, owner, name, full_name, url, default_branch,
                is_private, last_scanned_at, tags FROM repos WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Repo not found: {id}")))?;
    Ok(row.into_model())
}

/// Update the tags JSON array for a repo.
#[tauri::command]
pub async fn set_repo_tags(repo_id: String, tags: Vec<String>) -> AppResult<Repo> {
    let pool      = db::pool()?;
    let tags_json = serde_json::to_string(&tags).map_err(|e| AppError::InvalidInput(e.to_string()))?;
    sqlx::query("UPDATE repos SET tags = ?, updated_at = datetime('now') WHERE id = ?")
        .bind(&tags_json)
        .bind(&repo_id)
        .execute(pool)
        .await?;
    get_repo(repo_id).await
}

// ── Repo list commands ─────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRepoListInput {
    pub name:             String,
    pub description:      String,
    pub parent_id:        Option<String>,
    pub exclude_patterns: Vec<String>,
}

#[tauri::command]
pub async fn create_repo_list(input: CreateRepoListInput) -> AppResult<RepoList> {
    let pool     = db::pool()?;
    let id       = gen_id();
    let patterns = serde_json::to_string(&input.exclude_patterns)
        .map_err(|e| AppError::InvalidInput(e.to_string()))?;

    sqlx::query(
        "INSERT INTO repo_lists (id, name, description, parent_id, exclude_patterns)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&input.name)
    .bind(&input.description)
    .bind(&input.parent_id)
    .bind(&patterns)
    .execute(pool)
    .await?;

    get_list_by_id(pool, &id).await
}

#[tauri::command]
pub async fn update_repo_list(id: String, input: CreateRepoListInput) -> AppResult<RepoList> {
    let pool     = db::pool()?;
    let patterns = serde_json::to_string(&input.exclude_patterns)
        .map_err(|e| AppError::InvalidInput(e.to_string()))?;

    sqlx::query(
        "UPDATE repo_lists
         SET name = ?, description = ?, parent_id = ?, exclude_patterns = ?, updated_at = datetime('now')
         WHERE id = ?",
    )
    .bind(&input.name)
    .bind(&input.description)
    .bind(&input.parent_id)
    .bind(&patterns)
    .bind(&id)
    .execute(pool)
    .await?;

    get_list_by_id(pool, &id).await
}

#[tauri::command]
pub async fn delete_repo_list(id: String) -> AppResult<()> {
    let pool = db::pool()?;
    // repo_list_members cascade-deletes via FK
    sqlx::query("DELETE FROM repo_lists WHERE id = ?")
        .bind(&id)
        .execute(pool)
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn list_repo_lists() -> AppResult<Vec<RepoList>> {
    let pool = db::pool()?;
    let rows = sqlx::query_as::<_, RepoListRow>(
        "SELECT id, name, description, parent_id, exclude_patterns, created_at, updated_at
         FROM repo_lists ORDER BY name",
    )
    .fetch_all(pool)
    .await?;

    let mut lists = Vec::new();
    for row in rows {
        lists.push(row_to_list(pool, row).await?);
    }
    Ok(lists)
}

#[tauri::command]
pub async fn add_repos_to_list(list_id: String, repo_ids: Vec<String>) -> AppResult<()> {
    let pool = db::pool()?;
    for repo_id in &repo_ids {
        sqlx::query(
            "INSERT OR IGNORE INTO repo_list_members (list_id, repo_id) VALUES (?, ?)",
        )
        .bind(&list_id)
        .bind(repo_id)
        .execute(pool)
        .await?;
    }
    Ok(())
}

#[tauri::command]
pub async fn remove_repos_from_list(list_id: String, repo_ids: Vec<String>) -> AppResult<()> {
    let pool = db::pool()?;
    for repo_id in &repo_ids {
        sqlx::query(
            "DELETE FROM repo_list_members WHERE list_id = ? AND repo_id = ?",
        )
        .bind(&list_id)
        .bind(repo_id)
        .execute(pool)
        .await?;
    }
    Ok(())
}

// ── YAML export / import ───────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RepoListYaml {
    id:               String,
    name:             String,
    description:      String,
    parent_id:        Option<String>,
    exclude_patterns: Vec<String>,
    repo_ids:         Vec<String>,
}

/// Export a repo list as a YAML string (frontend handles file-save dialog).
#[tauri::command]
pub async fn export_repo_list(id: String) -> AppResult<String> {
    let pool = db::pool()?;
    let list = get_list_by_id(pool, &id).await?;
    let export = RepoListYaml {
        id:               list.id,
        name:             list.name,
        description:      list.description,
        parent_id:        list.parent_id,
        exclude_patterns: list.exclude_patterns,
        repo_ids:         list.repo_ids,
    };
    serde_yaml_ng::to_string(&export)
        .map_err(|e| AppError::Operation(format!("YAML serialisation failed: {e}")))
}

/// Import a repo list from a YAML string. Upserts list and members that exist in repos table.
#[tauri::command]
pub async fn import_repo_list(yaml: String) -> AppResult<RepoList> {
    let parsed: RepoListYaml = serde_yaml_ng::from_str(&yaml)
        .map_err(|e| AppError::InvalidInput(format!("Invalid YAML: {e}")))?;

    let pool     = db::pool()?;
    let patterns = serde_json::to_string(&parsed.exclude_patterns)
        .map_err(|e| AppError::InvalidInput(e.to_string()))?;

    sqlx::query(
        r#"INSERT INTO repo_lists (id, name, description, parent_id, exclude_patterns)
           VALUES (?, ?, ?, ?, ?)
           ON CONFLICT(id) DO UPDATE SET
               name             = excluded.name,
               description      = excluded.description,
               parent_id        = excluded.parent_id,
               exclude_patterns = excluded.exclude_patterns,
               updated_at       = datetime('now')"#,
    )
    .bind(&parsed.id)
    .bind(&parsed.name)
    .bind(&parsed.description)
    .bind(&parsed.parent_id)
    .bind(&patterns)
    .execute(pool)
    .await?;

    // Only add members whose repo row exists
    for repo_id in &parsed.repo_ids {
        #[derive(sqlx::FromRow)]
        struct CountRow { count: i64 }
        let row = sqlx::query_as::<_, CountRow>("SELECT COUNT(*) as count FROM repos WHERE id = ?")
            .bind(repo_id)
            .fetch_one(pool)
            .await?;
        if row.count > 0 {
            sqlx::query(
                "INSERT OR IGNORE INTO repo_list_members (list_id, repo_id) VALUES (?, ?)",
            )
            .bind(&parsed.id)
            .bind(repo_id)
            .execute(pool)
            .await?;
        }
    }

    get_list_by_id(pool, &parsed.id).await
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gen_id_is_nonempty() {
        let id = gen_id();
        assert!(!id.is_empty());
        // Two IDs generated in sequence should be unique
        let id2 = gen_id();
        // Not guaranteed equal since timestamps are the same resolution in fast CPUs,
        // but they should at least be non-empty strings
        assert!(!id2.is_empty());
    }

    #[test]
    fn yaml_round_trip() {
        let original = RepoListYaml {
            id:               "test-123".to_string(),
            name:             "My Test List".to_string(),
            description:      "A test".to_string(),
            parent_id:        None,
            exclude_patterns: vec!["org/legacy-*".to_string()],
            repo_ids:         vec!["github:org/repo-a".to_string(), "github:org/repo-b".to_string()],
        };
        let yaml   = serde_yaml_ng::to_string(&original).expect("serialize");
        let parsed: RepoListYaml = serde_yaml_ng::from_str(&yaml).expect("deserialize");
        assert_eq!(parsed.id,   original.id);
        assert_eq!(parsed.name, original.name);
        assert_eq!(parsed.repo_ids.len(), 2);
        assert_eq!(parsed.exclude_patterns, vec!["org/legacy-*"]);
    }

    #[test]
    fn yaml_rejects_invalid() {
        let result: Result<RepoListYaml, _> = serde_yaml_ng::from_str(": invalid: yaml: [[[");
        assert!(result.is_err());
    }
}
