use crate::db;
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use tokio::sync::Semaphore;

/// Global abort flag for script runs.
static ABORT_FLAG: AtomicBool = AtomicBool::new(false);

/// Maximum concurrent repos processed in parallel.
const MAX_CONCURRENT: usize = 5;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScriptPreset {
    pub id: String,
    pub name: String,
    pub command: String,
    pub description: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScriptRepoResult {
    pub repo_id: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
}

/// Run a command across multiple repos (dry-run style for now).
///
/// For each repo_id, looks up the repo in the DB, then executes the command
/// in a temporary directory. Uses a semaphore to limit concurrency.
#[tauri::command]
pub async fn run_script(
    command: String,
    repo_ids: Vec<String>,
    _parallel: u32,
) -> AppResult<Vec<ScriptRepoResult>> {
    if command.trim().is_empty() {
        return Err(AppError::InvalidInput("Command must not be empty".into()));
    }
    if repo_ids.is_empty() {
        return Err(AppError::InvalidInput(
            "At least one repo must be selected".into(),
        ));
    }

    // Reset the abort flag at the start of a new run
    ABORT_FLAG.store(false, Ordering::SeqCst);

    let pool = db::pool()?;
    let semaphore = std::sync::Arc::new(Semaphore::new(MAX_CONCURRENT));
    let mut handles = Vec::new();

    for repo_id in repo_ids {
        let cmd = command.clone();
        let sem = semaphore.clone();
        let pool = pool.clone();

        let handle = tokio::spawn(async move {
            // Check abort before acquiring permit
            if ABORT_FLAG.load(Ordering::SeqCst) {
                return ScriptRepoResult {
                    repo_id,
                    exit_code: -1,
                    stdout: String::new(),
                    stderr: "Aborted before execution".into(),
                    duration_ms: 0,
                };
            }

            let _permit = match sem.acquire().await {
                Ok(p) => p,
                Err(_) => {
                    return ScriptRepoResult {
                        repo_id,
                        exit_code: -1,
                        stdout: String::new(),
                        stderr: "Failed to acquire semaphore".into(),
                        duration_ms: 0,
                    };
                }
            };

            // Check abort after acquiring permit
            if ABORT_FLAG.load(Ordering::SeqCst) {
                return ScriptRepoResult {
                    repo_id,
                    exit_code: -1,
                    stdout: String::new(),
                    stderr: "Aborted".into(),
                    duration_ms: 0,
                };
            }

            // Look up repo URL from DB
            let repo_url: Option<String> = sqlx::query_scalar("SELECT url FROM repos WHERE id = ?")
                .bind(&repo_id)
                .fetch_optional(&pool)
                .await
                .ok()
                .flatten();

            let start = Instant::now();

            // Create a temp dir for execution
            let tmp_dir = match tempfile::tempdir() {
                Ok(d) => d,
                Err(e) => {
                    return ScriptRepoResult {
                        repo_id,
                        exit_code: -1,
                        stdout: String::new(),
                        stderr: format!("Failed to create temp dir: {e}"),
                        duration_ms: start.elapsed().as_millis() as u64,
                    };
                }
            };

            // Execute the command in the temp dir
            let output = tokio::process::Command::new("sh")
                .arg("-c")
                .arg(&cmd)
                .current_dir(tmp_dir.path())
                .env("FLOTILLA_REPO_ID", &repo_id)
                .env(
                    "FLOTILLA_REPO_URL",
                    repo_url.as_deref().unwrap_or("unknown"),
                )
                .output()
                .await;

            let duration_ms = start.elapsed().as_millis() as u64;

            match output {
                Ok(out) => ScriptRepoResult {
                    repo_id,
                    exit_code: out.status.code().unwrap_or(-1),
                    stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
                    stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
                    duration_ms,
                },
                Err(e) => ScriptRepoResult {
                    repo_id,
                    exit_code: -1,
                    stdout: String::new(),
                    stderr: format!("Failed to execute command: {e}"),
                    duration_ms,
                },
            }
        });

        handles.push(handle);
    }

    let mut results = Vec::with_capacity(handles.len());
    for handle in handles {
        match handle.await {
            Ok(result) => results.push(result),
            Err(e) => {
                // JoinError — task panicked or was cancelled
                results.push(ScriptRepoResult {
                    repo_id: "unknown".into(),
                    exit_code: -1,
                    stdout: String::new(),
                    stderr: format!("Task failed: {e}"),
                    duration_ms: 0,
                });
            }
        }
    }

    // Log to audit
    let _ = sqlx::query(
        "INSERT INTO audit_log (id, action_type, repo_ids, outcome, detail)
         VALUES (lower(hex(randomblob(16))), 'script_run', ?, 'completed', ?)",
    )
    .bind(
        serde_json::to_string(&results.iter().map(|r| &r.repo_id).collect::<Vec<_>>())
            .unwrap_or_default(),
    )
    .bind(&command)
    .execute(pool)
    .await;

    Ok(results)
}

/// Set the abort flag so running scripts stop at the next check point.
#[tauri::command]
pub async fn abort_script() -> AppResult<()> {
    ABORT_FLAG.store(true, Ordering::SeqCst);
    Ok(())
}

/// List all saved script presets from the database.
#[tauri::command]
pub async fn list_presets() -> AppResult<Vec<ScriptPreset>> {
    let pool = db::pool()?;

    let rows = sqlx::query(
        "SELECT id, name, command, description, created_at FROM script_presets ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;

    let presets = rows
        .into_iter()
        .map(|row| {
            use sqlx::Row;
            ScriptPreset {
                id: row.get("id"),
                name: row.get("name"),
                command: row.get("command"),
                description: row.get("description"),
                created_at: row.get("created_at"),
            }
        })
        .collect();

    Ok(presets)
}

/// Save a new script preset.
#[tauri::command]
pub async fn save_preset(
    name: String,
    command: String,
    description: String,
) -> AppResult<ScriptPreset> {
    if name.trim().is_empty() {
        return Err(AppError::InvalidInput(
            "Preset name must not be empty".into(),
        ));
    }
    if command.trim().is_empty() {
        return Err(AppError::InvalidInput(
            "Preset command must not be empty".into(),
        ));
    }

    let pool = db::pool()?;

    let row = sqlx::query(
        r#"INSERT INTO script_presets (id, name, command, description)
           VALUES (lower(hex(randomblob(16))), ?, ?, ?)
           RETURNING id, name, command, description, created_at"#,
    )
    .bind(&name)
    .bind(&command)
    .bind(&description)
    .fetch_one(pool)
    .await?;

    use sqlx::Row;
    Ok(ScriptPreset {
        id: row.get("id"),
        name: row.get("name"),
        command: row.get("command"),
        description: row.get("description"),
        created_at: row.get("created_at"),
    })
}

/// Delete a script preset by ID.
#[tauri::command]
pub async fn delete_preset(id: String) -> AppResult<()> {
    let pool = db::pool()?;

    let result = sqlx::query("DELETE FROM script_presets WHERE id = ?")
        .bind(&id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!(
            "Script preset '{id}' not found"
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_preset_serializes_camel_case() {
        let preset = ScriptPreset {
            id: "abc123".into(),
            name: "lint".into(),
            command: "npm run lint".into(),
            description: "Run linter".into(),
            created_at: "2026-01-01T00:00:00".into(),
        };
        let json = serde_json::to_value(&preset).expect("serialize");
        assert!(
            json.get("createdAt").is_some(),
            "should have camelCase key createdAt"
        );
        assert!(
            json.get("created_at").is_none(),
            "should not have snake_case key"
        );
    }

    #[test]
    fn script_repo_result_serializes_camel_case() {
        let result = ScriptRepoResult {
            repo_id: "github:org/repo".into(),
            exit_code: 0,
            stdout: "ok".into(),
            stderr: String::new(),
            duration_ms: 123,
        };
        let json = serde_json::to_value(&result).expect("serialize");
        assert!(json.get("repoId").is_some(), "should have camelCase repoId");
        assert!(
            json.get("exitCode").is_some(),
            "should have camelCase exitCode"
        );
        assert!(
            json.get("durationMs").is_some(),
            "should have camelCase durationMs"
        );
        assert!(json.get("repo_id").is_none(), "should not have snake_case");
    }
}
