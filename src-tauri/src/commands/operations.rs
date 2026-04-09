use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::{BatchOperation, OperationResult};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::Emitter;

// ── Abort flag ──────────────────────────────────────────────────────────────

static OPS_ABORT: AtomicBool = AtomicBool::new(false);

// ── Input / output types ────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOperationInput {
    pub operation_type: String,
    pub mode: Option<String>,
    pub target_repo_ids: Vec<String>,
    pub package_name: Option<String>,
    pub target_version: Option<String>,
    pub version_map: Option<HashMap<String, String>>,
    pub file_path: Option<String>,
    pub file_content: Option<String>,
    pub pr_title_template: Option<String>,
    pub pr_body_template: Option<String>,
    pub branch_prefix: Option<String>,
    pub label: Option<String>,
    pub is_dry_run: bool,
    pub skip_ci: bool,
    pub also_target_branches: Vec<String>,
    pub divergence_check: bool,
    pub divergence_threshold: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidateResult {
    pub repo_id: String,
    pub is_applied: bool,
    pub current_version: Option<String>,
    pub has_overrides: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationProgressEvent {
    pub operation_id: String,
    pub repo_id: String,
    pub status: String,
    pub current: usize,
    pub total: usize,
    pub error: Option<String>,
}

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Generate a random hex ID (32 hex chars = 16 bytes).
fn gen_id() -> String {
    use std::fmt::Write;
    let mut bytes = [0u8; 16];
    getrandom(&mut bytes);
    let mut s = String::with_capacity(32);
    for b in &bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// Fill buffer with random bytes using a simple fallback approach.
fn getrandom(buf: &mut [u8]) {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    // Use std's RandomState which is seeded from OS entropy
    for chunk in buf.chunks_mut(8) {
        let h = RandomState::new().build_hasher().finish();
        let bytes = h.to_le_bytes();
        let len = chunk.len().min(8);
        chunk[..len].copy_from_slice(&bytes[..len]);
    }
}

/// Convert a SQLite row from `batch_operations` to a `BatchOperation` model.
fn operation_row_to_model(
    row: &sqlx::sqlite::SqliteRow,
    results: Vec<OperationResult>,
) -> BatchOperation {
    let target_json: String = row.get("target_repo_ids");
    let completed_json: String = row.get("completed_repo_ids");
    let version_map_raw: Option<String> = row.get("version_map");

    let target_repo_ids: Vec<String> = serde_json::from_str(&target_json).unwrap_or_default();
    let completed_repo_ids: Vec<String> = serde_json::from_str(&completed_json).unwrap_or_default();
    let version_map: Option<HashMap<String, String>> =
        version_map_raw.and_then(|v| serde_json::from_str(&v).ok());

    BatchOperation {
        id: row.get("id"),
        operation_type: row.get("operation_type"),
        mode: row.get("mode"),
        status: row.get("status"),
        target_repo_ids,
        completed_repo_ids,
        version_map,
        created_at: row.get("created_at"),
        completed_at: row.get("completed_at"),
        results,
        is_dry_run: row.get::<i64, _>("is_dry_run") != 0,
        skip_ci: row.get::<i64, _>("skip_ci") != 0,
    }
}

/// Fetch all `operation_results` for a given operation ID.
async fn fetch_operation_results(
    pool: &sqlx::SqlitePool,
    operation_id: &str,
) -> AppResult<Vec<OperationResult>> {
    let rows = sqlx::query(
        r#"
        SELECT repo_id, status, pr_url, error, diff
        FROM operation_results
        WHERE operation_id = ?
        ORDER BY created_at ASC
        "#,
    )
    .bind(operation_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .iter()
        .map(|r| OperationResult {
            repo_id: r.get("repo_id"),
            status: r.get("status"),
            pr_url: r.get("pr_url"),
            error: r.get("error"),
            diff: r.get("diff"),
        })
        .collect())
}

/// Write an entry to the audit log.
async fn write_audit_log(
    pool: &sqlx::SqlitePool,
    action_type: &str,
    repo_ids: &[String],
    operation_id: Option<&str>,
    outcome: &str,
    detail: Option<&str>,
) -> AppResult<()> {
    let repo_ids_json = serde_json::to_string(repo_ids).unwrap_or_else(|_| "[]".to_string());
    sqlx::query(
        r#"
        INSERT INTO audit_log (id, timestamp, action_type, repo_ids, operation_id, outcome, detail)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(gen_id())
    .bind(Utc::now().to_rfc3339())
    .bind(action_type)
    .bind(&repo_ids_json)
    .bind(operation_id)
    .bind(outcome)
    .bind(detail)
    .execute(pool)
    .await?;
    Ok(())
}

// ── Commands ────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn create_operation(input: CreateOperationInput) -> AppResult<BatchOperation> {
    let pool = db::pool()?;
    let id = gen_id();
    let now = Utc::now().to_rfc3339();

    let target_json =
        serde_json::to_string(&input.target_repo_ids).unwrap_or_else(|_| "[]".to_string());
    let completed_json = "[]".to_string();
    let version_map_json: Option<String> = input
        .version_map
        .as_ref()
        .and_then(|vm| serde_json::to_string(vm).ok());

    sqlx::query(
        r#"
        INSERT INTO batch_operations
            (id, operation_type, mode, status, target_repo_ids, completed_repo_ids,
             version_map, is_dry_run, skip_ci, created_at, completed_at)
        VALUES (?, ?, ?, 'pending', ?, ?, ?, ?, ?, ?, NULL)
        "#,
    )
    .bind(&id)
    .bind(&input.operation_type)
    .bind(&input.mode)
    .bind(&target_json)
    .bind(&completed_json)
    .bind(&version_map_json)
    .bind(input.is_dry_run as i64)
    .bind(input.skip_ci as i64)
    .bind(&now)
    .execute(pool)
    .await?;

    write_audit_log(
        pool,
        "operation_created",
        &input.target_repo_ids,
        Some(&id),
        "success",
        Some(&format!(
            "type={}, dry_run={}",
            input.operation_type, input.is_dry_run
        )),
    )
    .await?;

    Ok(BatchOperation {
        id,
        operation_type: input.operation_type,
        mode: input.mode,
        status: "pending".to_string(),
        target_repo_ids: input.target_repo_ids,
        completed_repo_ids: Vec::new(),
        version_map: input.version_map,
        created_at: now,
        completed_at: None,
        results: Vec::new(),
        is_dry_run: input.is_dry_run,
        skip_ci: input.skip_ci,
    })
}

#[tauri::command]
pub async fn list_operations() -> AppResult<Vec<BatchOperation>> {
    let pool = db::pool()?;

    let rows = sqlx::query(
        r#"
        SELECT id, operation_type, mode, status, target_repo_ids, completed_repo_ids,
               version_map, is_dry_run, skip_ci, created_at, completed_at
        FROM batch_operations
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut ops = Vec::with_capacity(rows.len());
    for row in &rows {
        let op_id: String = row.get("id");
        let results = fetch_operation_results(pool, &op_id).await?;
        ops.push(operation_row_to_model(row, results));
    }

    Ok(ops)
}

#[tauri::command]
pub async fn get_operation(id: String) -> AppResult<BatchOperation> {
    let pool = db::pool()?;

    let row = sqlx::query(
        r#"
        SELECT id, operation_type, mode, status, target_repo_ids, completed_repo_ids,
               version_map, is_dry_run, skip_ci, created_at, completed_at
        FROM batch_operations
        WHERE id = ?
        "#,
    )
    .bind(&id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Operation not found: {id}")))?;

    let results = fetch_operation_results(pool, &id).await?;
    Ok(operation_row_to_model(&row, results))
}

#[tauri::command]
pub async fn run_operation(id: String, app: tauri::AppHandle) -> AppResult<()> {
    let pool = db::pool()?;

    // Load operation
    let row = sqlx::query(
        r#"
        SELECT id, operation_type, mode, status, target_repo_ids, completed_repo_ids,
               version_map, is_dry_run, skip_ci, created_at, completed_at
        FROM batch_operations
        WHERE id = ?
        "#,
    )
    .bind(&id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Operation not found: {id}")))?;

    let status: String = row.get("status");
    if status != "pending" && status != "paused" {
        return Err(AppError::Operation(format!(
            "Cannot run operation with status '{status}' — must be 'pending' or 'paused'"
        )));
    }

    let target_json: String = row.get("target_repo_ids");
    let completed_json: String = row.get("completed_repo_ids");
    let is_dry_run = row.get::<i64, _>("is_dry_run") != 0;
    let op_type: String = row.get("operation_type");

    let target_repo_ids: Vec<String> = serde_json::from_str(&target_json).unwrap_or_default();
    let already_completed: Vec<String> = serde_json::from_str(&completed_json).unwrap_or_default();

    // Filter out already-completed repos (resumability)
    let pending_repos: Vec<String> = target_repo_ids
        .iter()
        .filter(|r| !already_completed.contains(r))
        .cloned()
        .collect();

    let total = target_repo_ids.len();

    // Reset abort flag, update status to "running"
    OPS_ABORT.store(false, Ordering::SeqCst);
    sqlx::query("UPDATE batch_operations SET status = 'running' WHERE id = ?")
        .bind(&id)
        .execute(pool)
        .await?;

    // Concurrency control: 5 simultaneous workers
    let semaphore = Arc::new(tokio::sync::Semaphore::new(5));
    let counter = Arc::new(std::sync::atomic::AtomicUsize::new(already_completed.len()));
    let completed_ids = Arc::new(tokio::sync::Mutex::new(already_completed));

    let mut handles = Vec::with_capacity(pending_repos.len());

    for repo_id in pending_repos {
        let sem = Arc::clone(&semaphore);
        let ctr = Arc::clone(&counter);
        let completed = Arc::clone(&completed_ids);
        let app_handle = app.clone();
        let op_id = id.clone();
        let op_type_clone = op_type.clone();
        let is_dry = is_dry_run;

        let handle = tokio::spawn(async move {
            // Check abort before acquiring permit
            if OPS_ABORT.load(Ordering::SeqCst) {
                return;
            }

            let _permit = match sem.acquire().await {
                Ok(permit) => permit,
                Err(_) => return,
            };

            // Check abort after acquiring permit
            if OPS_ABORT.load(Ordering::SeqCst) {
                return;
            }

            let current = ctr.fetch_add(1, Ordering::SeqCst) + 1;

            // Emit "processing" event
            let _ = app_handle.emit(
                "operation-progress",
                OperationProgressEvent {
                    operation_id: op_id.clone(),
                    repo_id: repo_id.clone(),
                    status: "processing".to_string(),
                    current,
                    total,
                    error: None,
                },
            );

            // Execute the operation for this repo
            let (result_status, diff, error) = if is_dry {
                // Dry run: generate a placeholder diff
                let diff_text = format!(
                    "--- a/{repo_id}\n+++ b/{repo_id}\n@@ dry run @@\n \
                     # Would apply {op_type_clone} to {repo_id}"
                );
                ("completed".to_string(), Some(diff_text), None::<String>)
            } else {
                // Real execution placeholder — actual GitHub API calls will be added
                // in subsequent phases (PR creation, file updates, etc.)
                ("completed".to_string(), None::<String>, None::<String>)
            };

            // Insert operation_result
            let insert_result = async {
                let pool = db::pool()?;
                sqlx::query(
                    r#"
                    INSERT INTO operation_results
                        (id, operation_id, repo_id, status, pr_url, pre_change_sha, error, diff, created_at)
                    VALUES (?, ?, ?, ?, NULL, NULL, ?, ?, ?)
                    "#,
                )
                .bind(gen_id())
                .bind(&op_id)
                .bind(&repo_id)
                .bind(&result_status)
                .bind(&error)
                .bind(&diff)
                .bind(Utc::now().to_rfc3339())
                .execute(pool)
                .await?;

                // Update completed_repo_ids
                let mut ids = completed.lock().await;
                ids.push(repo_id.clone());
                let updated_json =
                    serde_json::to_string(&*ids).unwrap_or_else(|_| "[]".to_string());
                sqlx::query(
                    "UPDATE batch_operations SET completed_repo_ids = ? WHERE id = ?",
                )
                .bind(&updated_json)
                .bind(&op_id)
                .execute(pool)
                .await?;

                AppResult::Ok(())
            }
            .await;

            let event_status = match &insert_result {
                Ok(_) => result_status,
                Err(e) => {
                    tracing::error!("Failed to record result for {repo_id}: {e}");
                    "failed".to_string()
                }
            };

            let _ = app_handle.emit(
                "operation-progress",
                OperationProgressEvent {
                    operation_id: op_id,
                    repo_id,
                    status: event_status,
                    current,
                    total,
                    error: insert_result.err().map(|e| e.to_string()),
                },
            );
        });

        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        let _ = handle.await;
    }

    // Determine final status
    let aborted = OPS_ABORT.load(Ordering::SeqCst);
    let final_status = if aborted { "paused" } else { "completed" };
    let now = Utc::now().to_rfc3339();

    sqlx::query("UPDATE batch_operations SET status = ?, completed_at = ? WHERE id = ?")
        .bind(final_status)
        .bind(if aborted { None } else { Some(&now) })
        .bind(&id)
        .execute(pool)
        .await?;

    // Audit log
    write_audit_log(
        pool,
        "operation_executed",
        &target_repo_ids,
        Some(&id),
        final_status,
        Some(&format!("type={op_type}, dry_run={is_dry_run}")),
    )
    .await?;

    tracing::info!("Operation {id} finished with status={final_status}");

    Ok(())
}

#[tauri::command]
pub async fn abort_operation(id: String) -> AppResult<()> {
    OPS_ABORT.store(true, Ordering::SeqCst);

    let pool = db::pool()?;
    sqlx::query(
        "UPDATE batch_operations SET status = 'paused' WHERE id = ? AND status = 'running'",
    )
    .bind(&id)
    .execute(pool)
    .await?;

    write_audit_log(pool, "operation_aborted", &[], Some(&id), "paused", None).await?;

    tracing::info!("Operation {id} abort requested");
    Ok(())
}

#[tauri::command]
pub async fn validate_operation(
    package_name: String,
    target_version: String,
    repo_ids: Vec<String>,
) -> AppResult<Vec<ValidateResult>> {
    let pool = db::pool()?;
    let mut results = Vec::with_capacity(repo_ids.len());

    for repo_id in &repo_ids {
        // Check current version of the package in this repo
        let pkg_row = sqlx::query(
            r#"
            SELECT version FROM repo_packages
            WHERE repo_id = ? AND name = ?
            ORDER BY scanned_at DESC
            LIMIT 1
            "#,
        )
        .bind(repo_id)
        .bind(&package_name)
        .fetch_optional(pool)
        .await?;

        let current_version: Option<String> = pkg_row.as_ref().map(|r| r.get("version"));
        let is_applied = current_version
            .as_deref()
            .map(|v| v == target_version)
            .unwrap_or(false);

        // Check for overrides in root package.json scan data
        // For now, we cannot detect overrides without re-scanning — default to false
        let has_overrides = false;

        results.push(ValidateResult {
            repo_id: repo_id.clone(),
            is_applied,
            current_version,
            has_overrides,
        });
    }

    Ok(results)
}

#[tauri::command]
pub async fn rollback_operation(id: String) -> AppResult<()> {
    let pool = db::pool()?;

    // Verify the operation exists
    let row = sqlx::query("SELECT id, status, target_repo_ids FROM batch_operations WHERE id = ?")
        .bind(&id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Operation not found: {id}")))?;

    let status: String = row.get("status");
    if status == "rolled_back" {
        return Err(AppError::Operation(
            "Operation is already rolled back".to_string(),
        ));
    }

    let target_json: String = row.get("target_repo_ids");
    let target_repo_ids: Vec<String> = serde_json::from_str(&target_json).unwrap_or_default();

    sqlx::query("UPDATE batch_operations SET status = 'rolled_back' WHERE id = ?")
        .bind(&id)
        .execute(pool)
        .await?;

    write_audit_log(
        pool,
        "operation_rollback",
        &target_repo_ids,
        Some(&id),
        "rolled_back",
        None,
    )
    .await?;

    tracing::info!("Operation {id} marked as rolled_back");
    Ok(())
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gen_id_is_unique() {
        let a = gen_id();
        let b = gen_id();
        assert_ne!(a, b);
        assert_eq!(a.len(), 32);
        assert_eq!(b.len(), 32);
        // All lowercase hex
        assert!(a
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
        assert!(b
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }

    #[test]
    fn validate_result_serializes_camel_case() {
        let vr = ValidateResult {
            repo_id: "github:org/repo".to_string(),
            is_applied: true,
            current_version: Some("1.2.3".to_string()),
            has_overrides: false,
        };
        let json = serde_json::to_string(&vr).expect("serialize");
        assert!(
            json.contains("\"repoId\""),
            "expected camelCase repoId in: {json}"
        );
        assert!(
            json.contains("\"isApplied\""),
            "expected camelCase isApplied in: {json}"
        );
        assert!(
            json.contains("\"currentVersion\""),
            "expected camelCase currentVersion in: {json}"
        );
        assert!(
            json.contains("\"hasOverrides\""),
            "expected camelCase hasOverrides in: {json}"
        );
    }

    #[test]
    fn create_operation_input_deserializes_camel_case() {
        let json = r#"{
            "operationType": "package_pin",
            "mode": "pin",
            "targetRepoIds": ["github:org/repo"],
            "packageName": "lodash",
            "targetVersion": "4.17.21",
            "versionMap": null,
            "filePath": null,
            "fileContent": null,
            "prTitleTemplate": null,
            "prBodyTemplate": null,
            "branchPrefix": null,
            "label": null,
            "isDryRun": true,
            "skipCi": false,
            "alsoTargetBranches": [],
            "divergenceCheck": false,
            "divergenceThreshold": null
        }"#;
        let input: CreateOperationInput =
            serde_json::from_str(json).expect("deserialize camelCase input");
        assert_eq!(input.operation_type, "package_pin");
        assert_eq!(input.target_repo_ids, vec!["github:org/repo"]);
        assert!(input.is_dry_run);
    }

    #[test]
    fn operation_progress_event_serializes_camel_case() {
        let evt = OperationProgressEvent {
            operation_id: "abc".to_string(),
            repo_id: "r1".to_string(),
            status: "processing".to_string(),
            current: 1,
            total: 5,
            error: None,
        };
        let json = serde_json::to_string(&evt).expect("serialize");
        assert!(
            json.contains("\"operationId\""),
            "expected camelCase in: {json}"
        );
        assert!(json.contains("\"repoId\""), "expected camelCase in: {json}");
    }
}
