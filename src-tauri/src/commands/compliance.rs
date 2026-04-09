use crate::db;
use crate::error::{AppError, AppResult};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SecretFinding {
    pub repo_id: String,
    pub file_path: String,
    pub finding_type: String,
    pub severity: String,
    pub detail: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LicenceFinding {
    pub repo_id: String,
    pub package_name: String,
    pub ecosystem: String,
    pub licence: String,
    pub is_permissive: bool,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BranchProtectionStatus {
    pub repo_id: String,
    pub branch: String,
    pub requires_reviews: bool,
    pub requires_status_checks: bool,
    pub is_protected: bool,
}

/// Secret patterns to scan for in file paths and content.
const SECRET_FILE_PATTERNS: &[(&str, &str, &str)] = &[
    (".env", "env_file", "critical"),
    (".env.local", "env_file", "critical"),
    (".env.production", "env_file", "critical"),
    (".env.staging", "env_file", "high"),
    ("credentials.json", "credentials_file", "critical"),
    ("service-account.json", "credentials_file", "critical"),
    (".npmrc", "auth_config", "high"),
    (".pypirc", "auth_config", "high"),
    ("id_rsa", "private_key", "critical"),
    ("id_ed25519", "private_key", "critical"),
    (".pem", "private_key", "high"),
    (".p12", "private_key", "high"),
    (".key", "private_key", "high"),
];

/// Scan repos for potential secret exposures.
///
/// Checks scan_results for suspicious file patterns in workflow_files
/// and manifest_paths. This is a heuristic scan based on file names;
/// full content-based scanning requires cloning the repo.
#[tauri::command]
pub async fn scan_secrets(repo_ids: Vec<String>) -> AppResult<Vec<SecretFinding>> {
    if repo_ids.is_empty() {
        return Err(AppError::InvalidInput(
            "At least one repo must be selected".into(),
        ));
    }

    let pool = db::pool()?;
    let mut findings = Vec::new();

    for repo_id in &repo_ids {
        // Get the latest scan result for this repo
        let scan_row: Option<(String, String)> = sqlx::query_as(
            "SELECT manifest_paths, workflow_files FROM scan_results
             WHERE repo_id = ? ORDER BY scanned_at DESC LIMIT 1",
        )
        .bind(repo_id)
        .fetch_optional(pool)
        .await?;

        let (manifest_paths_json, workflow_files_json) = match scan_row {
            Some(row) => row,
            None => continue, // No scan data for this repo
        };

        // Parse JSON arrays of file paths
        let manifest_paths: Vec<String> =
            serde_json::from_str(&manifest_paths_json).unwrap_or_default();
        let workflow_files: Vec<String> =
            serde_json::from_str(&workflow_files_json).unwrap_or_default();

        // Combine all known file paths
        let all_paths: Vec<&str> = manifest_paths
            .iter()
            .chain(workflow_files.iter())
            .map(|s| s.as_str())
            .collect();

        // Check each path against secret patterns
        for path in &all_paths {
            for &(pattern, finding_type, severity) in SECRET_FILE_PATTERNS {
                let filename = path.rsplit('/').next().unwrap_or(path);
                if filename == pattern || filename.ends_with(pattern) {
                    findings.push(SecretFinding {
                        repo_id: repo_id.clone(),
                        file_path: path.to_string(),
                        finding_type: finding_type.into(),
                        severity: severity.into(),
                        detail: format!(
                            "Potentially sensitive file detected: {filename} matches pattern '{pattern}'"
                        ),
                    });
                }
            }
        }

        // Check if .env.example exists but .env might also be committed
        // (heuristic: if has_dot_env_example is true, that's actually good practice)
        let has_env_example: Option<(bool,)> = sqlx::query_as(
            "SELECT has_dot_env_example FROM scan_results
             WHERE repo_id = ? ORDER BY scanned_at DESC LIMIT 1",
        )
        .bind(repo_id)
        .fetch_optional(pool)
        .await?;

        if let Some((false,)) = has_env_example {
            findings.push(SecretFinding {
                repo_id: repo_id.clone(),
                file_path: ".env.example".into(),
                finding_type: "missing_env_example".into(),
                severity: "medium".into(),
                detail: "No .env.example found — environment variables may not be documented"
                    .into(),
            });
        }
    }

    // Audit log
    let _ = sqlx::query(
        "INSERT INTO audit_log (id, action_type, repo_ids, outcome, detail)
         VALUES (lower(hex(randomblob(16))), 'secret_scan', ?, 'completed', ?)",
    )
    .bind(
        serde_json::to_string(&repo_ids).unwrap_or_default(),
    )
    .bind(format!("Found {} findings", findings.len()))
    .execute(pool)
    .await;

    Ok(findings)
}

/// Scan licences for packages in the given repos.
///
/// Queries repo_packages for all dependencies and returns them with
/// "unknown" licence status. Full licence resolution requires npm/packagist
/// registry API calls (to be implemented in a future iteration).
#[tauri::command]
pub async fn scan_licences(repo_ids: Vec<String>) -> AppResult<Vec<LicenceFinding>> {
    if repo_ids.is_empty() {
        return Err(AppError::InvalidInput(
            "At least one repo must be selected".into(),
        ));
    }

    let pool = db::pool()?;
    let mut findings = Vec::new();

    for repo_id in &repo_ids {
        let packages: Vec<(String, String)> = sqlx::query_as(
            "SELECT DISTINCT name, ecosystem FROM repo_packages WHERE repo_id = ?",
        )
        .bind(repo_id)
        .fetch_all(pool)
        .await?;

        for (name, ecosystem) in packages {
            findings.push(LicenceFinding {
                repo_id: repo_id.clone(),
                package_name: name,
                ecosystem,
                licence: "unknown".into(),
                is_permissive: false, // unknown = not confirmed permissive
            });
        }
    }

    // Audit log
    let _ = sqlx::query(
        "INSERT INTO audit_log (id, action_type, repo_ids, outcome, detail)
         VALUES (lower(hex(randomblob(16))), 'licence_scan', ?, 'completed', ?)",
    )
    .bind(
        serde_json::to_string(&repo_ids).unwrap_or_default(),
    )
    .bind(format!("Found {} packages with unknown licences", findings.len()))
    .execute(pool)
    .await;

    Ok(findings)
}

/// Audit branch protection rules for the given repos.
///
/// Returns placeholder data indicating the feature needs GitHub API
/// branch protection endpoint calls. Will be implemented with real
/// API calls in a future iteration.
#[tauri::command]
pub async fn audit_branch_protection(
    repo_ids: Vec<String>,
) -> AppResult<Vec<BranchProtectionStatus>> {
    if repo_ids.is_empty() {
        return Err(AppError::InvalidInput(
            "At least one repo must be selected".into(),
        ));
    }

    let pool = db::pool()?;
    let mut statuses = Vec::new();

    for repo_id in &repo_ids {
        // Get the default branch from the repos table
        let branch: Option<String> =
            sqlx::query_scalar("SELECT default_branch FROM repos WHERE id = ?")
                .bind(repo_id)
                .fetch_optional(pool)
                .await?;

        let branch = branch.unwrap_or_else(|| "main".into());

        // Placeholder: actual protection status requires GitHub API call
        // GET /repos/{owner}/{repo}/branches/{branch}/protection
        statuses.push(BranchProtectionStatus {
            repo_id: repo_id.clone(),
            branch,
            requires_reviews: false,       // unknown until API call
            requires_status_checks: false,  // unknown until API call
            is_protected: false,            // unknown until API call
        });
    }

    // Audit log
    let _ = sqlx::query(
        "INSERT INTO audit_log (id, action_type, repo_ids, outcome, detail)
         VALUES (lower(hex(randomblob(16))), 'branch_protection_audit', ?, 'completed', ?)",
    )
    .bind(
        serde_json::to_string(&repo_ids).unwrap_or_default(),
    )
    .bind(format!(
        "Audited {} repos (placeholder — API integration pending)",
        statuses.len()
    ))
    .execute(pool)
    .await;

    Ok(statuses)
}

/// Archive repos (placeholder).
///
/// Returns the count of repos that would be archived. Real archival
/// requires GitHub API PATCH /repos/{owner}/{repo} with `archived: true`.
#[tauri::command]
pub async fn archive_repos(repo_ids: Vec<String>) -> AppResult<u32> {
    if repo_ids.is_empty() {
        return Err(AppError::InvalidInput(
            "At least one repo must be selected".into(),
        ));
    }

    let pool = db::pool()?;
    let count = repo_ids.len() as u32;

    // Audit log
    let _ = sqlx::query(
        "INSERT INTO audit_log (id, action_type, repo_ids, outcome, detail)
         VALUES (lower(hex(randomblob(16))), 'archive_repos', ?, 'completed', ?)",
    )
    .bind(
        serde_json::to_string(&repo_ids).unwrap_or_default(),
    )
    .bind(format!(
        "Would archive {} repos (placeholder — API integration pending)",
        count
    ))
    .execute(pool)
    .await;

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_finding_serializes_camel_case() {
        let finding = SecretFinding {
            repo_id: "github:org/repo".into(),
            file_path: ".env".into(),
            finding_type: "env_file".into(),
            severity: "critical".into(),
            detail: "test".into(),
        };
        let json = serde_json::to_value(&finding).expect("serialize");
        assert!(json.get("repoId").is_some(), "should have camelCase repoId");
        assert!(json.get("filePath").is_some(), "should have camelCase filePath");
        assert!(json.get("findingType").is_some(), "should have camelCase findingType");
        assert!(json.get("repo_id").is_none(), "should not have snake_case");
    }

    #[test]
    fn licence_finding_serializes_camel_case() {
        let finding = LicenceFinding {
            repo_id: "github:org/repo".into(),
            package_name: "lodash".into(),
            ecosystem: "npm".into(),
            licence: "MIT".into(),
            is_permissive: true,
        };
        let json = serde_json::to_value(&finding).expect("serialize");
        assert!(json.get("repoId").is_some());
        assert!(json.get("packageName").is_some());
        assert!(json.get("isPermissive").is_some());
    }

    #[test]
    fn branch_protection_serializes_camel_case() {
        let status = BranchProtectionStatus {
            repo_id: "github:org/repo".into(),
            branch: "main".into(),
            requires_reviews: true,
            requires_status_checks: true,
            is_protected: true,
        };
        let json = serde_json::to_value(&status).expect("serialize");
        assert!(json.get("repoId").is_some());
        assert!(json.get("requiresReviews").is_some());
        assert!(json.get("requiresStatusChecks").is_some());
        assert!(json.get("isProtected").is_some());
    }
}
