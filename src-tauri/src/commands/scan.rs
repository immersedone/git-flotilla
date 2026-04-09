use crate::commands::auth::KEYCHAIN_SERVICE;
use crate::error::{AppError, AppResult};
use crate::models::{RepoPackage, ScanResult};
use crate::services::github::{decode_base64_content, GitHubClient};
use crate::services::scanner::{
    compute_health_score, detect_floating_action_tags, detect_node_version, detect_package_manager,
    discover_manifests, discover_workflows, extract_package_manager_field, extract_php_version,
    file_exists, parse_composer_json, parse_package_json, HealthScoreInput,
};
use crate::{db, services::rate_limiter};
use chrono::Utc;
use keyring::Entry;
use serde::Serialize;
use sqlx::Row;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::Mutex;

// ── Batch scan types ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgressEvent {
    pub repo_id: String,
    pub status: String,
    pub current: usize,
    pub total: usize,
    pub error: Option<String>,
}

static SCAN_ABORT: AtomicBool = AtomicBool::new(false);

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Fetch a file via the GitHub content API, returning `Ok(None)` on 404.
async fn fetch_file_content_optional(
    client: &GitHubClient,
    owner: &str,
    repo: &str,
    file_path: &str,
    git_ref: &str,
) -> AppResult<Option<String>> {
    match client
        .get_file_content(owner, repo, file_path, git_ref)
        .await
    {
        Ok((content_resp, rl)) => {
            if let Some(snapshot) = rl {
                rate_limiter::update_github(snapshot);
            }
            let decoded = decode_base64_content(&content_resp.content)?;
            Ok(Some(decoded))
        }
        Err(AppError::NotFound(_)) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Convert a SQLite row to a `ScanResult` model.
fn scan_row_to_model(row: &sqlx::sqlite::SqliteRow) -> ScanResult {
    let manifest_json: String = row.get("manifest_paths");
    let workflow_json: String = row.get("workflow_files");
    let flags_json: String = row.get("flags");

    ScanResult {
        repo_id: row.get("repo_id"),
        scanned_at: row.get("scanned_at"),
        manifest_paths: serde_json::from_str(&manifest_json).unwrap_or_default(),
        node_version: row.get("node_version"),
        node_version_source: row.get("node_version_source"),
        php_version: row.get("php_version"),
        package_manager: row.get("package_manager"),
        package_manager_version: row.get("package_manager_version"),
        has_develop: row.get::<i64, _>("has_develop") != 0,
        last_pushed: row.get("last_pushed"),
        has_dot_env_example: row.get::<i64, _>("has_dot_env_example") != 0,
        workflow_files: serde_json::from_str(&workflow_json).unwrap_or_default(),
        health_score: row.get::<i64, _>("health_score") as u32,
        flags: serde_json::from_str(&flags_json).unwrap_or_default(),
        excluded: row.get::<i64, _>("excluded") != 0,
        exclude_reason: row.get("exclude_reason"),
    }
}

// ── Commands ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn scan_repo(repo_id: String) -> AppResult<ScanResult> {
    let pool = db::pool()?;

    // 1. Look up repo in DB
    let repo_row =
        sqlx::query("SELECT provider, owner, name, default_branch FROM repos WHERE id = ?")
            .bind(&repo_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Repo not found: {repo_id}")))?;

    let provider: String = repo_row.get("provider");
    let owner: String = repo_row.get("owner");
    let name: String = repo_row.get("name");
    let default_branch: String = repo_row.get("default_branch");

    // 2. Find a matching account
    let account_row = sqlx::query("SELECT id FROM accounts WHERE provider = ? LIMIT 1")
        .bind(&provider)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| {
            AppError::Auth(format!(
                "No {provider} account configured. Add one in Settings."
            ))
        })?;

    let account_id: String = account_row.get("id");

    // 3. Get token from keychain
    let token = Entry::new(KEYCHAIN_SERVICE, &account_id)
        .map_err(AppError::from)?
        .get_password()
        .map_err(|e| AppError::Keychain(format!("Failed to read token for {account_id}: {e}")))?;

    // 4. Create GitHub client
    let client = GitHubClient::new(&token);

    // 5. Fetch repo tree
    let (tree, rl) = client.get_repo_tree(&owner, &name, &default_branch).await?;
    if let Some(snapshot) = rl {
        rate_limiter::update_github(snapshot);
    }
    if tree.truncated {
        tracing::warn!("Tree for {repo_id} was truncated — scan results may be incomplete");
    }

    // 6-8. Discover manifests, workflows, detect package manager from tree
    let manifest_paths = discover_manifests(&tree);
    let workflow_files = discover_workflows(&tree);
    let lockfile_pm = detect_package_manager(&tree);

    // 9. List branches, check for develop
    let (branches, rl) = client.list_branches(&owner, &name).await?;
    if let Some(snapshot) = rl {
        rate_limiter::update_github(snapshot);
    }
    let has_develop = branches.iter().any(|b| b.name == "develop");

    // 10-11. Fetch version detection files and manifest contents
    // Collect files for node version detection
    let mut version_files: Vec<(String, String)> = Vec::new();

    // Fetch .nvmrc, .node-version, .tool-versions only if present in tree
    for file_name in &[".nvmrc", ".node-version", ".tool-versions"] {
        if file_exists(&tree, file_name) {
            if let Some(content) =
                fetch_file_content_optional(&client, &owner, &name, file_name, &default_branch)
                    .await?
            {
                version_files.push((file_name.to_string(), content));
            }
        }
    }

    // Fetch and parse each manifest file, collecting packages
    let mut all_packages: Vec<RepoPackage> = Vec::new();
    let mut root_package_json_content: Option<String> = None;
    let mut root_composer_json_content: Option<String> = None;

    for manifest_path in &manifest_paths {
        let content =
            fetch_file_content_optional(&client, &owner, &name, manifest_path, &default_branch)
                .await?;

        let Some(content) = content else {
            continue;
        };

        let file_name = manifest_path
            .rsplit('/')
            .next()
            .unwrap_or(manifest_path.as_str());

        match file_name {
            "package.json" => {
                let pkgs = parse_package_json(&content, &repo_id);
                all_packages.extend(pkgs);
                // Keep root package.json for version detection
                if manifest_path == "package.json" {
                    root_package_json_content = Some(content);
                }
            }
            "composer.json" => {
                let pkgs = parse_composer_json(&content, &repo_id);
                all_packages.extend(pkgs);
                // Keep root composer.json for PHP version extraction
                if manifest_path == "composer.json" {
                    root_composer_json_content = Some(content);
                }
            }
            // TODO: requirements.txt, Cargo.toml, go.mod parsing in future tasks
            _ => {}
        }
    }

    // 12. Detect Node version
    // Add root package.json to version files for engines.node detection
    if let Some(ref pj) = root_package_json_content {
        version_files.push(("package.json".to_string(), pj.clone()));
    }
    let version_file_refs: Vec<(&str, &str)> = version_files
        .iter()
        .map(|(p, c)| (p.as_str(), c.as_str()))
        .collect();
    let node_version_info = detect_node_version(&version_file_refs);
    let node_version = node_version_info.as_ref().map(|(v, _)| v.clone());
    let node_version_source = node_version_info.map(|(_, s)| s);

    // 13. Extract PHP version from root composer.json
    let php_version = root_composer_json_content
        .as_deref()
        .and_then(extract_php_version);

    // 14. Determine package manager and version
    let mut package_manager: Option<String> = lockfile_pm.as_ref().map(|(pm, _)| pm.to_string());
    let mut package_manager_version: Option<String> = None;

    // Try packageManager field from root package.json
    if let Some(ref pj) = root_package_json_content {
        if let Some((pm_name, pm_ver)) = extract_package_manager_field(pj) {
            // packageManager field is authoritative for version
            package_manager_version = Some(pm_ver);
            // If no lockfile detection, use this for manager name too
            if package_manager.is_none() {
                package_manager = Some(pm_name);
            }
        }
    }

    // Also detect composer if we have a composer lockfile
    if package_manager.is_none() && file_exists(&tree, "composer.lock") {
        package_manager = Some("composer".to_string());
    }

    // 15. Check file presence
    let has_dot_env_example = file_exists(&tree, ".env.example");
    let has_codeowners =
        file_exists(&tree, "CODEOWNERS") || file_exists(&tree, ".github/CODEOWNERS");
    let has_security_md = file_exists(&tree, "SECURITY.md");
    let has_editorconfig = file_exists(&tree, ".editorconfig");

    // 16. Detect floating action tags in each workflow file
    let mut floating_action_count: usize = 0;
    for wf_path in &workflow_files {
        if let Some(content) =
            fetch_file_content_optional(&client, &owner, &name, wf_path, &default_branch).await?
        {
            floating_action_count += detect_floating_action_tags(&content).len();
        }
    }

    // 17. Auto-exclude
    let excluded = manifest_paths.is_empty();
    let exclude_reason = if excluded {
        Some("No package manifests found".to_string())
    } else {
        None
    };

    // 18. Compute health score
    let (health_score, flags) = compute_health_score(&HealthScoreInput {
        has_codeowners,
        has_security_md,
        has_dot_env_example,
        has_editorconfig,
        floating_action_count,
        has_known_cves: false, // CVE matching done separately
        node_version_current: node_version.is_some(),
    });

    // Build the result
    let now = Utc::now().to_rfc3339();

    let result = ScanResult {
        repo_id: repo_id.clone(),
        scanned_at: now.clone(),
        manifest_paths: manifest_paths.clone(),
        node_version,
        node_version_source,
        php_version,
        package_manager,
        package_manager_version,
        has_develop,
        last_pushed: None, // TODO: populate from repo push timestamp
        has_dot_env_example,
        workflow_files: workflow_files.clone(),
        health_score,
        flags: flags.clone(),
        excluded,
        exclude_reason: exclude_reason.clone(),
    };

    // 19. INSERT into scan_results
    let manifest_paths_json =
        serde_json::to_string(&result.manifest_paths).unwrap_or_else(|_| "[]".to_string());
    let workflow_files_json =
        serde_json::to_string(&result.workflow_files).unwrap_or_else(|_| "[]".to_string());
    let flags_json = serde_json::to_string(&result.flags).unwrap_or_else(|_| "[]".to_string());

    sqlx::query(
        r#"
        INSERT INTO scan_results (
            repo_id, scanned_at, manifest_paths, node_version, node_version_source,
            php_version, package_manager, package_manager_version, has_develop,
            last_pushed, has_dot_env_example, workflow_files, health_score, flags,
            excluded, exclude_reason
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&result.repo_id)
    .bind(&result.scanned_at)
    .bind(&manifest_paths_json)
    .bind(&result.node_version)
    .bind(&result.node_version_source)
    .bind(&result.php_version)
    .bind(&result.package_manager)
    .bind(&result.package_manager_version)
    .bind(result.has_develop as i64)
    .bind(&result.last_pushed)
    .bind(result.has_dot_env_example as i64)
    .bind(&workflow_files_json)
    .bind(result.health_score as i64)
    .bind(&flags_json)
    .bind(result.excluded as i64)
    .bind(&result.exclude_reason)
    .execute(pool)
    .await?;

    // 20. DELETE old repo_packages, INSERT new ones
    sqlx::query("DELETE FROM repo_packages WHERE repo_id = ?")
        .bind(&repo_id)
        .execute(pool)
        .await?;

    for pkg in &all_packages {
        sqlx::query(
            r#"
            INSERT INTO repo_packages (repo_id, ecosystem, name, version, is_dev, scanned_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&pkg.repo_id)
        .bind(&pkg.ecosystem)
        .bind(&pkg.name)
        .bind(&pkg.version)
        .bind(pkg.is_dev as i64)
        .bind(&pkg.scanned_at)
        .execute(pool)
        .await?;
    }

    // 21. UPDATE repos.last_scanned_at
    sqlx::query("UPDATE repos SET last_scanned_at = ? WHERE id = ?")
        .bind(&now)
        .bind(&repo_id)
        .execute(pool)
        .await?;

    tracing::info!(
        "Scan complete for {repo_id}: {} manifests, {} packages, score {}",
        result.manifest_paths.len(),
        all_packages.len(),
        result.health_score
    );

    Ok(result)
}

#[tauri::command]
pub async fn get_scan_result(repo_id: String) -> AppResult<ScanResult> {
    let pool = db::pool()?;

    let row = sqlx::query(
        r#"
        SELECT repo_id, scanned_at, manifest_paths, node_version, node_version_source,
               php_version, package_manager, package_manager_version, has_develop,
               last_pushed, has_dot_env_example, workflow_files, health_score, flags,
               excluded, exclude_reason
        FROM scan_results
        WHERE repo_id = ?
        ORDER BY scanned_at DESC
        LIMIT 1
        "#,
    )
    .bind(&repo_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("No scan results for repo: {repo_id}")))?;

    Ok(scan_row_to_model(&row))
}

#[tauri::command]
pub async fn list_scan_results(repo_list_id: Option<String>) -> AppResult<Vec<ScanResult>> {
    let pool = db::pool()?;

    let rows = if let Some(list_id) = repo_list_id {
        sqlx::query(
            r#"
            SELECT s.repo_id, s.scanned_at, s.manifest_paths, s.node_version,
                   s.node_version_source, s.php_version, s.package_manager,
                   s.package_manager_version, s.has_develop, s.last_pushed,
                   s.has_dot_env_example, s.workflow_files, s.health_score, s.flags,
                   s.excluded, s.exclude_reason
            FROM scan_results s
            INNER JOIN (
                SELECT repo_id, MAX(scanned_at) AS max_scanned_at
                FROM scan_results
                GROUP BY repo_id
            ) latest ON s.repo_id = latest.repo_id AND s.scanned_at = latest.max_scanned_at
            INNER JOIN repo_list_members rlm ON s.repo_id = rlm.repo_id
            WHERE rlm.list_id = ?
            ORDER BY s.repo_id
            "#,
        )
        .bind(&list_id)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query(
            r#"
            SELECT s.repo_id, s.scanned_at, s.manifest_paths, s.node_version,
                   s.node_version_source, s.php_version, s.package_manager,
                   s.package_manager_version, s.has_develop, s.last_pushed,
                   s.has_dot_env_example, s.workflow_files, s.health_score, s.flags,
                   s.excluded, s.exclude_reason
            FROM scan_results s
            INNER JOIN (
                SELECT repo_id, MAX(scanned_at) AS max_scanned_at
                FROM scan_results
                GROUP BY repo_id
            ) latest ON s.repo_id = latest.repo_id AND s.scanned_at = latest.max_scanned_at
            ORDER BY s.repo_id
            "#,
        )
        .fetch_all(pool)
        .await?
    };

    Ok(rows.iter().map(scan_row_to_model).collect())
}

#[tauri::command]
pub async fn scan_repo_list(list_id: String, app: tauri::AppHandle) -> AppResult<String> {
    // Reset abort flag
    SCAN_ABORT.store(false, Ordering::SeqCst);

    // Fetch repo IDs belonging to this list
    let pool = db::pool()?;
    let rows = sqlx::query("SELECT repo_id FROM repo_list_members WHERE list_id = ?")
        .bind(&list_id)
        .fetch_all(pool)
        .await?;

    if rows.is_empty() {
        return Err(AppError::InvalidInput(format!(
            "Repo list '{list_id}' is empty or does not exist"
        )));
    }

    let repo_ids: Vec<String> = rows.iter().map(|r| r.get("repo_id")).collect();
    let total = repo_ids.len();

    // Concurrency control: 5 simultaneous scans
    let semaphore = Arc::new(tokio::sync::Semaphore::new(5));
    let counter = Arc::new(AtomicUsize::new(0));
    let succeeded = Arc::new(AtomicUsize::new(0));
    let failed_count = Arc::new(AtomicUsize::new(0));
    let errors: Arc<Mutex<Vec<(String, String)>>> = Arc::new(Mutex::new(Vec::new()));

    let mut handles = Vec::with_capacity(total);

    for repo_id in repo_ids {
        let sem = Arc::clone(&semaphore);
        let ctr = Arc::clone(&counter);
        let ok_count = Arc::clone(&succeeded);
        let fail_count = Arc::clone(&failed_count);
        let errs = Arc::clone(&errors);
        let app_handle = app.clone();

        let handle = tokio::spawn(async move {
            // Check abort before acquiring permit
            if SCAN_ABORT.load(Ordering::SeqCst) {
                return;
            }

            // Acquire semaphore permit
            let _permit = match sem.acquire().await {
                Ok(permit) => permit,
                Err(_) => return, // Semaphore closed
            };

            // Check abort after acquiring permit
            if SCAN_ABORT.load(Ordering::SeqCst) {
                return;
            }

            let current = ctr.fetch_add(1, Ordering::SeqCst) + 1;

            // Emit "scanning" event
            let _ = app_handle.emit(
                "scan-progress",
                ScanProgressEvent {
                    repo_id: repo_id.clone(),
                    status: "scanning".to_string(),
                    current,
                    total,
                    error: None,
                },
            );

            // Perform the scan
            match scan_repo(repo_id.clone()).await {
                Ok(_) => {
                    ok_count.fetch_add(1, Ordering::SeqCst);
                    let _ = app_handle.emit(
                        "scan-progress",
                        ScanProgressEvent {
                            repo_id,
                            status: "done".to_string(),
                            current,
                            total,
                            error: None,
                        },
                    );
                }
                Err(e) => {
                    fail_count.fetch_add(1, Ordering::SeqCst);
                    let err_msg = e.to_string();
                    errs.lock().await.push((repo_id.clone(), err_msg.clone()));
                    let _ = app_handle.emit(
                        "scan-progress",
                        ScanProgressEvent {
                            repo_id,
                            status: "failed".to_string(),
                            current,
                            total,
                            error: Some(err_msg),
                        },
                    );
                }
            }

            // Inter-request delay (200ms)
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        let _ = handle.await;
    }

    let ok = succeeded.load(Ordering::SeqCst);
    let fail = failed_count.load(Ordering::SeqCst);
    let aborted = SCAN_ABORT.load(Ordering::SeqCst);

    tracing::info!(
        "Batch scan of list '{list_id}' finished: {ok} succeeded, {fail} failed, aborted={aborted}"
    );

    Ok(list_id)
}

#[tauri::command]
pub async fn abort_scan(operation_id: String) -> AppResult<()> {
    let _ = operation_id;
    SCAN_ABORT.store(true, Ordering::SeqCst);
    tracing::info!("Scan abort requested");
    Ok(())
}
