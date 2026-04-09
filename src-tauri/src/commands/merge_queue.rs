use crate::db;
use crate::error::AppResult;
use chrono::Utc;
use serde::Serialize;
use sqlx::Row;

// ── Types ──────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FlotillaPr {
    pub repo_id: String,
    pub pr_url: String,
    pub operation_id: String,
    pub operation_type: String,
    pub created_at: String,
    pub status: String,
}

// ── Helpers ────────────────────────────────────────────────────────────────

fn gen_id() -> String {
    use std::collections::hash_map::RandomState;
    use std::fmt::Write;
    use std::hash::{BuildHasher, Hasher};
    let mut bytes = [0u8; 16];
    for chunk in bytes.chunks_mut(8) {
        let h = RandomState::new().build_hasher().finish();
        let b = h.to_le_bytes();
        let len = chunk.len().min(8);
        chunk[..len].copy_from_slice(&b[..len]);
    }
    let mut s = String::with_capacity(32);
    for b in &bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

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

// ── Commands ───────────────────────────────────────────────────────────────

/// List all Flotilla-created PRs by joining operation_results (where pr_url is set)
/// with batch_operations for metadata.
#[tauri::command]
pub async fn list_flotilla_prs(operation_id: Option<String>) -> AppResult<Vec<FlotillaPr>> {
    let pool = db::pool()?;

    let rows = if let Some(ref op_id) = operation_id {
        sqlx::query(
            r#"
            SELECT
                r.repo_id,
                r.pr_url,
                r.operation_id,
                o.operation_type,
                r.created_at,
                r.status
            FROM operation_results r
            JOIN batch_operations o ON o.id = r.operation_id
            WHERE r.pr_url IS NOT NULL
              AND r.operation_id = ?
            ORDER BY r.created_at DESC
            "#,
        )
        .bind(op_id)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query(
            r#"
            SELECT
                r.repo_id,
                r.pr_url,
                r.operation_id,
                o.operation_type,
                r.created_at,
                r.status
            FROM operation_results r
            JOIN batch_operations o ON o.id = r.operation_id
            WHERE r.pr_url IS NOT NULL
            ORDER BY r.created_at DESC
            "#,
        )
        .fetch_all(pool)
        .await?
    };

    let prs = rows
        .iter()
        .map(|row| FlotillaPr {
            repo_id: row.get("repo_id"),
            pr_url: row.get("pr_url"),
            operation_id: row.get("operation_id"),
            operation_type: row.get("operation_type"),
            created_at: row.get("created_at"),
            status: row.get("status"),
        })
        .collect();

    Ok(prs)
}

/// Mark a PR as merged by updating its operation_result status.
/// Actual GitHub merge API call is deferred to a later phase.
#[tauri::command]
pub async fn merge_pr(pr_url: String) -> AppResult<()> {
    let pool = db::pool()?;

    let row = sqlx::query("SELECT repo_id, operation_id FROM operation_results WHERE pr_url = ?")
        .bind(&pr_url)
        .fetch_optional(pool)
        .await?;

    let row =
        row.ok_or_else(|| crate::error::AppError::NotFound(format!("PR not found: {pr_url}")))?;

    let repo_id: String = row.get("repo_id");
    let operation_id: String = row.get("operation_id");

    sqlx::query("UPDATE operation_results SET status = 'merged' WHERE pr_url = ?")
        .bind(&pr_url)
        .execute(pool)
        .await?;

    write_audit_log(
        pool,
        "pr_merged",
        &[repo_id],
        Some(&operation_id),
        "success",
        Some(&format!("Merged PR: {pr_url}")),
    )
    .await?;

    tracing::info!("PR marked as merged: {pr_url}");
    Ok(())
}

/// Merge all PRs with status "completed" (no errors). Returns the count merged.
#[tauri::command]
pub async fn merge_all_green() -> AppResult<u32> {
    let pool = db::pool()?;

    let rows = sqlx::query(
        r#"
        SELECT id, repo_id, pr_url, operation_id
        FROM operation_results
        WHERE pr_url IS NOT NULL
          AND status = 'completed'
          AND error IS NULL
        "#,
    )
    .fetch_all(pool)
    .await?;

    let count = rows.len() as u32;

    if count == 0 {
        return Ok(0);
    }

    // Collect IDs for batch update
    let ids: Vec<String> = rows.iter().map(|r| r.get::<String, _>("id")).collect();
    let repo_ids: Vec<String> = rows.iter().map(|r| r.get::<String, _>("repo_id")).collect();

    for id in &ids {
        sqlx::query("UPDATE operation_results SET status = 'merged' WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
    }

    write_audit_log(
        pool,
        "pr_merge_all_green",
        &repo_ids,
        None,
        "success",
        Some(&format!("Batch merged {count} PRs")),
    )
    .await?;

    tracing::info!("Batch merged {count} green PRs");
    Ok(count)
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flotilla_pr_serializes_camel_case() {
        let pr = FlotillaPr {
            repo_id: "github:org/repo".to_string(),
            pr_url: "https://github.com/org/repo/pull/42".to_string(),
            operation_id: "abc123".to_string(),
            operation_type: "package_pin".to_string(),
            created_at: "2026-04-09T00:00:00Z".to_string(),
            status: "completed".to_string(),
        };
        let json = serde_json::to_string(&pr).expect("serialize");
        assert!(
            json.contains("\"repoId\""),
            "expected camelCase repoId in: {json}"
        );
        assert!(
            json.contains("\"prUrl\""),
            "expected camelCase prUrl in: {json}"
        );
        assert!(
            json.contains("\"operationId\""),
            "expected camelCase operationId in: {json}"
        );
        assert!(
            json.contains("\"operationType\""),
            "expected camelCase operationType in: {json}"
        );
        assert!(
            json.contains("\"createdAt\""),
            "expected camelCase createdAt in: {json}"
        );
    }
}
