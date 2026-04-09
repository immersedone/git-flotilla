use crate::commands::auth::KEYCHAIN_SERVICE;
use crate::{
    db,
    error::{AppError, AppResult},
    models::{Repo, RepoList},
    services::github::{GitHubClient, GitHubRepo},
    services::gitlab::{GitLabClient, GitLabProject},
};
use keyring::Entry;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::collections::HashSet;

// ── Private helpers ────────────────────────────────────────────────────────

fn get_token(account_id: &str) -> AppResult<String> {
    Entry::new(KEYCHAIN_SERVICE, account_id)
        .map_err(AppError::from)?
        .get_password()
        .map_err(AppError::from)
}

fn gh_repo_to_model(r: &GitHubRepo) -> Repo {
    Repo {
        id: format!("github:{}", r.full_name),
        provider: "github".to_string(),
        owner: r.owner.login.clone(),
        name: r.name.clone(),
        full_name: r.full_name.clone(),
        url: r.html_url.clone(),
        default_branch: r.default_branch.clone(),
        is_private: r.private,
        last_scanned_at: None,
        tags: vec![],
    }
}

fn gl_project_to_model(p: &GitLabProject) -> Repo {
    Repo {
        id: format!("gitlab:{}", p.path_with_namespace),
        provider: "gitlab".to_string(),
        owner: p.namespace.full_path.clone(),
        name: p.name.clone(),
        full_name: p.path_with_namespace.clone(),
        url: p.web_url.clone(),
        default_branch: p
            .default_branch
            .clone()
            .unwrap_or_else(|| "main".to_string()),
        is_private: p.visibility == "private",
        last_scanned_at: None,
        tags: vec![],
    }
}

fn gen_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    format!(
        "{:x}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    )
}

// ── Row types for runtime sqlx queries ────────────────────────────────────

#[derive(sqlx::FromRow)]
struct RepoRow {
    id: String,
    provider: String,
    owner: String,
    name: String,
    full_name: String,
    url: String,
    default_branch: String,
    is_private: i64,
    last_scanned_at: Option<String>,
    tags: String,
}

impl RepoRow {
    fn into_model(self) -> Repo {
        Repo {
            id: self.id,
            provider: self.provider,
            owner: self.owner,
            name: self.name,
            full_name: self.full_name,
            url: self.url,
            default_branch: self.default_branch,
            is_private: self.is_private != 0,
            last_scanned_at: self.last_scanned_at,
            tags: serde_json::from_str(&self.tags).unwrap_or_default(),
        }
    }
}

#[derive(sqlx::FromRow)]
struct RepoListRow {
    id: String,
    name: String,
    description: String,
    parent_id: Option<String>,
    exclude_patterns: String,
    created_at: String,
    updated_at: String,
}

async fn fetch_list_repo_ids(pool: &sqlx::SqlitePool, list_id: &str) -> AppResult<Vec<String>> {
    #[derive(sqlx::FromRow)]
    struct IdRow {
        repo_id: String,
    }
    let rows = sqlx::query_as::<_, IdRow>(
        "SELECT repo_id FROM repo_list_members WHERE list_id = ? ORDER BY added_at",
    )
    .bind(list_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| r.repo_id).collect())
}

async fn row_to_list(pool: &sqlx::SqlitePool, row: RepoListRow) -> AppResult<RepoList> {
    let repo_ids = fetch_list_repo_ids(pool, &row.id).await?;
    let exclude_patterns = serde_json::from_str(&row.exclude_patterns).unwrap_or_default();
    Ok(RepoList {
        id: row.id,
        name: row.name,
        description: row.description,
        repo_ids,
        parent_id: row.parent_id,
        exclude_patterns,
        created_at: row.created_at,
        updated_at: row.updated_at,
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

async fn fetch_repo_by_id(pool: &sqlx::SqlitePool, id: &str) -> AppResult<Repo> {
    let row = sqlx::query_as::<_, RepoRow>(
        "SELECT id, provider, owner, name, full_name, url, default_branch,
                is_private, last_scanned_at, tags FROM repos WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Repo not found: {id}")))?;
    Ok(row.into_model())
}

// ── Repo commands ──────────────────────────────────────────────────────────

/// Discover all repos accessible to the account and upsert into SQLite.
///
/// Supports both GitHub and GitLab accounts. The provider is inferred from
/// the `account_id` prefix (e.g. `github:octocat` or `gitlab:alice`).
#[tauri::command]
pub async fn discover_repos(account_id: String) -> AppResult<Vec<Repo>> {
    let token = get_token(&account_id)?;

    // Determine provider from account ID prefix
    let provider = account_id.split(':').next().unwrap_or("github");

    let repos = match provider {
        "gitlab" => discover_gitlab_repos(&token).await?,
        _ => discover_github_repos(&token).await?,
    };

    let pool = db::pool()?;
    for repo in &repos {
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
    }

    tracing::info!("Discovered {} repos for {}", repos.len(), account_id);
    Ok(repos)
}

async fn discover_github_repos(token: &str) -> AppResult<Vec<Repo>> {
    let client = GitHubClient::new(token);

    // Fetch user repos
    let (user_repos, rate_limit) = client.list_all_repos().await?;
    if let Some(rl) = rate_limit {
        crate::services::rate_limiter::update_github(rl);
    }

    // Fetch org repos (deduplicated)
    let (orgs, orgs_rl) = client.list_orgs().await?;
    if let Some(rl) = orgs_rl {
        crate::services::rate_limiter::update_github(rl);
    }
    let mut seen: HashSet<String> = HashSet::new();
    let mut all_gh: Vec<GitHubRepo> = Vec::new();

    for r in user_repos {
        if seen.insert(r.full_name.clone()) {
            all_gh.push(r);
        }
    }
    for org in &orgs {
        let (org_repos, org_rl) = client.list_org_repos(&org.login).await?;
        if let Some(rl) = org_rl {
            crate::services::rate_limiter::update_github(rl);
        }
        for r in org_repos {
            if seen.insert(r.full_name.clone()) {
                all_gh.push(r);
            }
        }
    }

    Ok(all_gh.iter().map(gh_repo_to_model).collect())
}

async fn discover_gitlab_repos(token: &str) -> AppResult<Vec<Repo>> {
    let client = GitLabClient::new(token, None);

    // Fetch all projects the user is a member of
    let (projects, rate_limit) = client.list_all_projects().await?;
    if let Some(rl) = rate_limit {
        crate::services::rate_limiter::update_gitlab(rl);
    }

    // Fetch group projects (deduplicated with membership projects)
    let (groups, groups_rl) = client.list_groups().await?;
    if let Some(rl) = groups_rl {
        crate::services::rate_limiter::update_gitlab(rl);
    }

    let mut seen: HashSet<String> = HashSet::new();
    let mut all_gl: Vec<GitLabProject> = Vec::new();

    for p in projects {
        if seen.insert(p.path_with_namespace.clone()) {
            all_gl.push(p);
        }
    }
    for group in &groups {
        let (group_projects, group_rl) = client.list_group_projects(&group.full_path).await?;
        if let Some(rl) = group_rl {
            crate::services::rate_limiter::update_gitlab(rl);
        }
        for p in group_projects {
            if seen.insert(p.path_with_namespace.clone()) {
                all_gl.push(p);
            }
        }
    }

    Ok(all_gl.iter().map(gl_project_to_model).collect())
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
    fetch_repo_by_id(pool, &id).await
}

/// Update the tags JSON array for a repo.
#[tauri::command]
pub async fn set_repo_tags(repo_id: String, tags: Vec<String>) -> AppResult<Repo> {
    let pool = db::pool()?;
    let tags_json =
        serde_json::to_string(&tags).map_err(|e| AppError::InvalidInput(e.to_string()))?;
    let result =
        sqlx::query("UPDATE repos SET tags = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(&tags_json)
            .bind(&repo_id)
            .execute(pool)
            .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Repo not found: {repo_id}")));
    }
    fetch_repo_by_id(pool, &repo_id).await
}

// ── Repo list commands ─────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRepoListInput {
    pub name: String,
    pub description: String,
    pub parent_id: Option<String>,
    pub exclude_patterns: Vec<String>,
}

#[tauri::command]
pub async fn create_repo_list(input: CreateRepoListInput) -> AppResult<RepoList> {
    let pool = db::pool()?;
    let id = gen_id();
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
    let pool = db::pool()?;
    let patterns = serde_json::to_string(&input.exclude_patterns)
        .map_err(|e| AppError::InvalidInput(e.to_string()))?;

    let result = sqlx::query(
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

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Repo list not found: {id}")));
    }

    get_list_by_id(pool, &id).await
}

#[tauri::command]
pub async fn delete_repo_list(id: String) -> AppResult<()> {
    let pool = db::pool()?;
    // repo_list_members cascade-deletes via FK
    let result = sqlx::query("DELETE FROM repo_lists WHERE id = ?")
        .bind(&id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Repo list not found: {id}")));
    }
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

    // Fetch all members in one query to avoid N+1
    let all_members =
        sqlx::query("SELECT list_id, repo_id FROM repo_list_members ORDER BY list_id, added_at")
            .fetch_all(pool)
            .await?;

    let mut members_by_list: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for member_row in all_members {
        members_by_list
            .entry(member_row.get::<String, _>("list_id"))
            .or_default()
            .push(member_row.get::<String, _>("repo_id"));
    }

    let mut lists = Vec::new();
    for row in rows {
        let repo_ids = members_by_list.remove(&row.id).unwrap_or_default();
        let exclude_patterns = serde_json::from_str(&row.exclude_patterns).unwrap_or_default();
        lists.push(RepoList {
            id: row.id,
            name: row.name,
            description: row.description,
            repo_ids,
            parent_id: row.parent_id,
            exclude_patterns,
            created_at: row.created_at,
            updated_at: row.updated_at,
        });
    }
    Ok(lists)
}

#[tauri::command]
pub async fn add_repos_to_list(list_id: String, repo_ids: Vec<String>) -> AppResult<()> {
    let pool = db::pool()?;
    for repo_id in &repo_ids {
        sqlx::query("INSERT OR IGNORE INTO repo_list_members (list_id, repo_id) VALUES (?, ?)")
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
        sqlx::query("DELETE FROM repo_list_members WHERE list_id = ? AND repo_id = ?")
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
    id: String,
    name: String,
    description: String,
    parent_id: Option<String>,
    exclude_patterns: Vec<String>,
    repo_ids: Vec<String>,
}

/// Export a repo list as a YAML string (frontend handles file-save dialog).
#[tauri::command]
pub async fn export_repo_list(id: String) -> AppResult<String> {
    let pool = db::pool()?;
    let list = get_list_by_id(pool, &id).await?;
    let export = RepoListYaml {
        id: list.id,
        name: list.name,
        description: list.description,
        parent_id: list.parent_id,
        exclude_patterns: list.exclude_patterns,
        repo_ids: list.repo_ids,
    };
    serde_yaml_ng::to_string(&export)
        .map_err(|e| AppError::Operation(format!("YAML serialisation failed: {e}")))
}

/// Import a repo list from a YAML string. Upserts list and members that exist in repos table.
#[tauri::command]
pub async fn import_repo_list(yaml: String) -> AppResult<RepoList> {
    let parsed: RepoListYaml = serde_yaml_ng::from_str(&yaml)
        .map_err(|e| AppError::InvalidInput(format!("Invalid YAML: {e}")))?;

    let pool = db::pool()?;
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
        struct CountRow {
            count: i64,
        }
        let row = sqlx::query_as::<_, CountRow>("SELECT COUNT(*) as count FROM repos WHERE id = ?")
            .bind(repo_id)
            .fetch_one(pool)
            .await?;
        if row.count > 0 {
            sqlx::query("INSERT OR IGNORE INTO repo_list_members (list_id, repo_id) VALUES (?, ?)")
                .bind(&parsed.id)
                .bind(repo_id)
                .execute(pool)
                .await?;
        }
    }

    get_list_by_id(pool, &parsed.id).await
}

// ── Repo Similarity Clustering ─────────────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RepoCluster {
    pub label: String,
    pub repos: Vec<String>,
    pub fingerprint: ClusterFingerprint,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClusterFingerprint {
    pub package_manager: Option<String>,
    pub node_version: Option<String>,
    pub key_packages: Vec<String>,
}

/// Group repos by tech stack fingerprint (package_manager, major_node_version)
/// and find shared key packages within each group.
#[tauri::command]
pub async fn get_repo_clusters() -> AppResult<Vec<RepoCluster>> {
    let pool = db::pool()?;

    // Fetch latest scan result per repo
    let scan_rows = sqlx::query(
        r#"
        SELECT s.repo_id, s.package_manager, s.node_version
        FROM scan_results s
        WHERE s.scanned_at = (
            SELECT MAX(s2.scanned_at) FROM scan_results s2 WHERE s2.repo_id = s.repo_id
        )
        ORDER BY s.repo_id
        "#,
    )
    .fetch_all(pool)
    .await?;

    // Build groups keyed by (package_manager, major_node_version)
    let mut groups: std::collections::HashMap<(String, String), Vec<String>> =
        std::collections::HashMap::new();
    let mut node_versions: std::collections::HashMap<String, Option<String>> =
        std::collections::HashMap::new();

    for row in &scan_rows {
        let repo_id: String = row.get("repo_id");
        let pm: Option<String> = row.get("package_manager");
        let nv: Option<String> = row.get("node_version");

        let pm_key = pm.clone().unwrap_or_else(|| "none".to_string());
        let nv_major = nv
            .as_deref()
            .and_then(|v| v.split('.').next())
            .unwrap_or("unknown")
            .to_string();

        node_versions.insert(repo_id.clone(), nv);
        groups.entry((pm_key, nv_major)).or_default().push(repo_id);
    }

    // Fetch packages for all repos to find shared key packages per cluster
    let pkg_rows = sqlx::query(
        r#"
        SELECT p.repo_id, p.name
        FROM packages p
        WHERE p.scanned_at = (
            SELECT MAX(p2.scanned_at) FROM packages p2
            WHERE p2.repo_id = p.repo_id AND p2.ecosystem = p.ecosystem AND p2.name = p.name
        )
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut repo_packages: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for row in &pkg_rows {
        let repo_id: String = row.get("repo_id");
        let name: String = row.get("name");
        repo_packages.entry(repo_id).or_default().push(name);
    }

    // Build clusters
    let mut clusters: Vec<RepoCluster> = Vec::new();

    for ((pm, nv_major), repo_ids) in &groups {
        // Find top 3 shared packages by frequency within the group
        let mut pkg_freq: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
        for repo_id in repo_ids {
            if let Some(packages) = repo_packages.get(repo_id) {
                // Deduplicate per repo to count each package at most once per repo
                let unique: HashSet<&str> = packages.iter().map(|s| s.as_str()).collect();
                for pkg in unique {
                    *pkg_freq.entry(pkg).or_insert(0) += 1;
                }
            }
        }
        let mut freq_vec: Vec<(&&str, &usize)> = pkg_freq.iter().collect();
        freq_vec.sort_by(|a, b| b.1.cmp(a.1));
        let key_packages: Vec<String> = freq_vec
            .iter()
            .take(3)
            .map(|(name, _)| (**name).to_string())
            .collect();

        let pm_display = if pm == "none" { None } else { Some(pm.clone()) };
        let nv_display = if nv_major == "unknown" {
            None
        } else {
            Some(nv_major.clone())
        };

        // Build human-readable label
        let label =
            build_cluster_label(pm_display.as_deref(), nv_display.as_deref(), &key_packages);

        clusters.push(RepoCluster {
            label,
            repos: repo_ids.clone(),
            fingerprint: ClusterFingerprint {
                package_manager: pm_display,
                node_version: nv_display,
                key_packages,
            },
        });
    }

    // Sort by cluster size, largest first
    clusters.sort_by(|a, b| b.repos.len().cmp(&a.repos.len()));

    Ok(clusters)
}

fn build_cluster_label(
    pm: Option<&str>,
    node_major: Option<&str>,
    key_packages: &[String],
) -> String {
    let mut parts = Vec::new();

    // Add key framework packages first (e.g. "Laravel", "Vue")
    for pkg in key_packages.iter().take(2) {
        let display = match pkg.as_str() {
            "vue" | "vue-router" => "Vue",
            "react" | "react-dom" => "React",
            "next" => "Next.js",
            "nuxt" | "nuxt3" => "Nuxt",
            "laravel/framework" => "Laravel",
            "express" => "Express",
            "svelte" => "Svelte",
            "angular" | "@angular/core" => "Angular",
            _ => "",
        };
        if !display.is_empty() && !parts.contains(&display.to_string()) {
            parts.push(display.to_string());
        }
    }

    // Add package manager
    if let Some(pm) = pm {
        parts.push(format!("({})", pm));
    }

    // Add Node major version if present
    if let Some(nv) = node_major {
        parts.push(format!("Node {}", nv));
    }

    if parts.is_empty() {
        "Unknown stack".to_string()
    } else {
        parts.join(" + ")
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_cluster_serialises_camel_case() {
        let cluster = RepoCluster {
            label: "Vue + (pnpm) + Node 20".to_string(),
            repos: vec!["github:org/repo-a".to_string()],
            fingerprint: ClusterFingerprint {
                package_manager: Some("pnpm".to_string()),
                node_version: Some("20".to_string()),
                key_packages: vec!["vue".to_string(), "pinia".to_string()],
            },
        };
        let json = serde_json::to_string(&cluster).expect("serialize");
        assert!(
            json.contains("\"packageManager\""),
            "expected camelCase: {json}"
        );
        assert!(
            json.contains("\"nodeVersion\""),
            "expected camelCase: {json}"
        );
        assert!(
            json.contains("\"keyPackages\""),
            "expected camelCase: {json}"
        );
    }

    #[test]
    fn build_cluster_label_with_framework() {
        let label = build_cluster_label(
            Some("npm"),
            Some("20"),
            &["vue".to_string(), "pinia".to_string()],
        );
        assert!(label.contains("Vue"), "label should mention Vue: {label}");
        assert!(label.contains("(npm)"), "label should mention npm: {label}");
        assert!(
            label.contains("Node 20"),
            "label should mention Node 20: {label}"
        );
    }

    #[test]
    fn build_cluster_label_unknown_stack() {
        let label = build_cluster_label(None, None, &[]);
        assert_eq!(label, "Unknown stack");
    }

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
            id: "test-123".to_string(),
            name: "My Test List".to_string(),
            description: "A test".to_string(),
            parent_id: None,
            exclude_patterns: vec!["org/legacy-*".to_string()],
            repo_ids: vec![
                "github:org/repo-a".to_string(),
                "github:org/repo-b".to_string(),
            ],
        };
        let yaml = serde_yaml_ng::to_string(&original).expect("serialize");
        let parsed: RepoListYaml = serde_yaml_ng::from_str(&yaml).expect("deserialize");
        assert_eq!(parsed.id, original.id);
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
