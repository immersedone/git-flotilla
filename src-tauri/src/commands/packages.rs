use crate::{
    commands::auth::KEYCHAIN_SERVICE,
    db,
    error::{AppError, AppResult},
    services::{changelog, github::GitHubClient, rate_limiter},
};
use keyring::Entry;
use serde::Serialize;
use sqlx::Row;
use std::collections::{BTreeMap, BTreeSet, HashMap};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DependencyMatrix {
    pub packages: Vec<PackageRow>,
    pub repo_ids: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PackageRow {
    pub name: String,
    pub ecosystem: String,
    pub versions_by_repo: HashMap<String, String>,
    pub latest_version: Option<String>,
    pub repo_count: usize,
    pub has_drift: bool,
    pub is_dev_only: bool,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChangelogEntry {
    pub version: String,
    pub body: String,
    pub published_at: String,
    pub is_breaking: bool,
}

/// Aggregate flat package tuples into a DependencyMatrix.
///
/// Each tuple is (repo_id, ecosystem, name, version, is_dev).
/// Groups by (ecosystem, name), computes drift, repo count, and dev-only status.
/// Uses BTreeMap/BTreeSet for deterministic ordering.
fn aggregate_packages(rows: &[(&str, &str, &str, &str, bool)]) -> DependencyMatrix {
    // (ecosystem, name) -> per-package accumulator
    struct Accum {
        versions_by_repo: BTreeMap<String, String>,
        unique_versions: BTreeSet<String>,
        all_dev: bool,
    }

    let mut map: BTreeMap<(String, String), Accum> = BTreeMap::new();
    let mut all_repo_ids: BTreeSet<String> = BTreeSet::new();

    for &(repo_id, ecosystem, name, version, is_dev) in rows {
        all_repo_ids.insert(repo_id.to_string());

        let key = (ecosystem.to_string(), name.to_string());
        let accum = map.entry(key).or_insert_with(|| Accum {
            versions_by_repo: BTreeMap::new(),
            unique_versions: BTreeSet::new(),
            all_dev: true,
        });

        accum
            .versions_by_repo
            .insert(repo_id.to_string(), version.to_string());
        accum.unique_versions.insert(version.to_string());
        if !is_dev {
            accum.all_dev = false;
        }
    }

    let packages: Vec<PackageRow> = map
        .into_iter()
        .map(|((ecosystem, name), accum)| {
            let repo_count = accum.versions_by_repo.len();
            let has_drift = accum.unique_versions.len() > 1;
            PackageRow {
                name,
                ecosystem,
                versions_by_repo: accum.versions_by_repo.into_iter().collect(),
                latest_version: None,
                repo_count,
                has_drift,
                is_dev_only: accum.all_dev,
            }
        })
        .collect();

    let repo_ids: Vec<String> = all_repo_ids.into_iter().collect();

    DependencyMatrix { packages, repo_ids }
}

#[tauri::command]
pub async fn get_dependency_matrix(
    repo_list_id: Option<String>,
    ecosystem: Option<String>,
) -> AppResult<DependencyMatrix> {
    let pool = db::pool()?;

    let rows = match (&repo_list_id, &ecosystem) {
        (Some(list_id), Some(eco)) => {
            sqlx::query(
                "SELECT rp.repo_id, rp.ecosystem, rp.name, rp.version, rp.is_dev
                 FROM repo_packages rp
                 JOIN repo_list_members rlm ON rp.repo_id = rlm.repo_id
                 WHERE rlm.list_id = ?1 AND rp.ecosystem = ?2
                 ORDER BY rp.name, rp.repo_id",
            )
            .bind(list_id)
            .bind(eco)
            .fetch_all(pool)
            .await?
        }
        (Some(list_id), None) => {
            sqlx::query(
                "SELECT rp.repo_id, rp.ecosystem, rp.name, rp.version, rp.is_dev
                 FROM repo_packages rp
                 JOIN repo_list_members rlm ON rp.repo_id = rlm.repo_id
                 WHERE rlm.list_id = ?1
                 ORDER BY rp.name, rp.repo_id",
            )
            .bind(list_id)
            .fetch_all(pool)
            .await?
        }
        (None, Some(eco)) => {
            sqlx::query(
                "SELECT repo_id, ecosystem, name, version, is_dev
                 FROM repo_packages
                 WHERE ecosystem = ?1
                 ORDER BY name, repo_id",
            )
            .bind(eco)
            .fetch_all(pool)
            .await?
        }
        (None, None) => {
            sqlx::query(
                "SELECT repo_id, ecosystem, name, version, is_dev
                 FROM repo_packages
                 ORDER BY name, repo_id",
            )
            .fetch_all(pool)
            .await?
        }
    };

    let tuples: Vec<(String, String, String, String, bool)> = rows
        .iter()
        .map(|row| {
            let repo_id: String = row.get("repo_id");
            let ecosystem: String = row.get("ecosystem");
            let name: String = row.get("name");
            let version: String = row.get("version");
            let is_dev: bool = row.get::<i32, _>("is_dev") != 0;
            (repo_id, ecosystem, name, version, is_dev)
        })
        .collect();

    let refs: Vec<(&str, &str, &str, &str, bool)> = tuples
        .iter()
        .map(|(r, e, n, v, d)| (r.as_str(), e.as_str(), n.as_str(), v.as_str(), *d))
        .collect();

    let matrix = aggregate_packages(&refs);

    tracing::info!(
        "Dependency matrix: {} packages across {} repos",
        matrix.packages.len(),
        matrix.repo_ids.len()
    );

    Ok(matrix)
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        let escaped = s.replace('"', "\"\"");
        format!("\"{}\"", escaped)
    } else {
        s.to_string()
    }
}

fn matrix_to_csv(matrix: &DependencyMatrix) -> String {
    let mut lines: Vec<String> = Vec::new();

    // Header
    let mut header_parts = vec![
        "Package".to_string(),
        "Ecosystem".to_string(),
        "Repos".to_string(),
        "Drift".to_string(),
    ];
    for repo_id in &matrix.repo_ids {
        header_parts.push(csv_escape(repo_id));
    }
    lines.push(header_parts.join(","));

    // Data rows
    for pkg in &matrix.packages {
        let mut row_parts = vec![
            csv_escape(&pkg.name),
            csv_escape(&pkg.ecosystem),
            pkg.repo_count.to_string(),
            if pkg.has_drift {
                "yes".to_string()
            } else {
                "no".to_string()
            },
        ];
        for repo_id in &matrix.repo_ids {
            let version = pkg
                .versions_by_repo
                .get(repo_id)
                .cloned()
                .unwrap_or_default();
            row_parts.push(csv_escape(&version));
        }
        lines.push(row_parts.join(","));
    }

    lines.join("\n")
}

#[tauri::command]
pub async fn export_matrix_csv(repo_list_id: Option<String>) -> AppResult<String> {
    let matrix = get_dependency_matrix(repo_list_id, None).await?;
    Ok(matrix_to_csv(&matrix))
}

#[tauri::command]
pub async fn get_package_changelog(
    package_name: String,
    ecosystem: String,
    from_version: String,
    to_version: String,
) -> AppResult<Vec<ChangelogEntry>> {
    if ecosystem != "npm" {
        return Err(AppError::InvalidInput(format!(
            "Changelog fetching is only supported for npm packages (got \"{ecosystem}\")"
        )));
    }

    let http_client = reqwest::Client::builder()
        .user_agent("git-flotilla/0.1")
        .build()
        .map_err(|e| AppError::Operation(format!("Failed to create HTTP client: {e}")))?;

    // Resolve npm package -> GitHub repo
    let (owner, repo) = changelog::npm_package_to_github_repo(&http_client, &package_name)
        .await
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "Could not find a GitHub repository for npm package \"{package_name}\""
            ))
        })?;

    tracing::info!("Resolved {package_name} -> {owner}/{repo}");

    // Try to get an authenticated GitHub token
    let pool = db::pool()?;
    let maybe_account = sqlx::query("SELECT id FROM accounts WHERE provider = 'github' LIMIT 1")
        .fetch_optional(pool)
        .await?;

    let releases = if let Some(account_row) = maybe_account {
        let account_id: String = account_row.get("id");
        let token = Entry::new(KEYCHAIN_SERVICE, &account_id)
            .map_err(AppError::from)?
            .get_password()
            .map_err(|e| {
                AppError::Keychain(format!("Failed to read token for {account_id}: {e}"))
            })?;

        let gh = GitHubClient::new(&token);
        let (releases, rl) = gh.list_releases(&owner, &repo).await?;
        if let Some(snapshot) = rl {
            rate_limiter::update_github(snapshot);
        }
        releases
    } else {
        // Unauthenticated fallback
        tracing::warn!("No GitHub account configured — using unauthenticated API request");
        let url = format!("https://api.github.com/repos/{owner}/{repo}/releases?per_page=100");
        let resp = http_client
            .get(&url)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await
            .map_err(|e| AppError::GitHub(format!("Failed to fetch releases: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::GitHub(format!(
                "GitHub API returned HTTP {status}: {text}"
            )));
        }

        resp.json()
            .await
            .map_err(|e| AppError::GitHub(format!("Failed to parse releases: {e}")))?
    };

    let entries = changelog::releases_to_changelog(&releases, &from_version, &to_version);

    tracing::info!(
        "Changelog for {package_name} ({from_version} -> {to_version}): {} entries",
        entries.len()
    );

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_matrix_from_flat_packages() {
        let rows: Vec<(&str, &str, &str, &str, bool)> = vec![
            ("repo-a", "npm", "vue", "3.3.0", false),
            ("repo-b", "npm", "vue", "3.4.0", false),
            ("repo-a", "npm", "lodash", "4.17.21", false),
            ("repo-b", "npm", "axios", "1.6.0", false),
        ];

        let matrix = aggregate_packages(&rows);

        assert_eq!(matrix.packages.len(), 3);
        assert_eq!(matrix.repo_ids.len(), 2);

        let vue = matrix.packages.iter().find(|p| p.name == "vue");
        assert!(vue.is_some());
        let vue = vue.expect("vue should exist");
        assert!(vue.has_drift);
        assert_eq!(vue.repo_count, 2);
        assert_eq!(
            vue.versions_by_repo.get("repo-a"),
            Some(&"3.3.0".to_string())
        );
        assert_eq!(
            vue.versions_by_repo.get("repo-b"),
            Some(&"3.4.0".to_string())
        );
    }

    #[test]
    fn build_matrix_no_drift_when_same_version() {
        let rows: Vec<(&str, &str, &str, &str, bool)> = vec![
            ("repo-a", "npm", "vue", "3.4.0", false),
            ("repo-b", "npm", "vue", "3.4.0", false),
        ];

        let matrix = aggregate_packages(&rows);

        let vue = matrix.packages.iter().find(|p| p.name == "vue");
        assert!(vue.is_some());
        let vue = vue.expect("vue should exist");
        assert!(!vue.has_drift);
        assert_eq!(vue.repo_count, 2);
    }

    #[test]
    fn build_matrix_dev_only_tracking() {
        let rows: Vec<(&str, &str, &str, &str, bool)> = vec![
            ("repo-a", "npm", "vitest", "1.0.0", true),
            ("repo-b", "npm", "vitest", "1.1.0", true),
            ("repo-a", "npm", "vue", "3.4.0", false),
        ];

        let matrix = aggregate_packages(&rows);

        let vitest = matrix.packages.iter().find(|p| p.name == "vitest");
        assert!(vitest.is_some());
        assert!(vitest.expect("vitest should exist").is_dev_only);

        let vue = matrix.packages.iter().find(|p| p.name == "vue");
        assert!(vue.is_some());
        assert!(!vue.expect("vue should exist").is_dev_only);
    }

    #[test]
    fn csv_export_format() {
        let rows: Vec<(&str, &str, &str, &str, bool)> = vec![
            ("repo-a", "npm", "vue", "3.3.0", false),
            ("repo-b", "npm", "vue", "3.4.0", false),
        ];

        let matrix = aggregate_packages(&rows);
        let csv = matrix_to_csv(&matrix);
        let lines: Vec<&str> = csv.lines().collect();

        assert!(lines[0].starts_with("Package,Ecosystem,Repos,Drift,"));
        // Data row for vue
        assert!(lines[1].starts_with("vue,npm,2,yes,"));
    }
}
