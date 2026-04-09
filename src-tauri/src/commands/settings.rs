use crate::db;
use crate::error::AppResult;
use crate::models::RateLimitInfo;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::Row;

// ── Types ──────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub scan_interval_minutes: Option<u32>,
    pub cve_poll_interval_minutes: Option<u32>,
    pub parallel_workers: u32,
    pub request_delay_ms: u32,
    pub health_score_weights: HealthScoreWeights,
    pub webhook_url: Option<String>,
    pub webhook_events: Vec<String>,
    pub dark_mode: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HealthScoreWeights {
    pub has_codeowners: u32,
    pub has_security_md: u32,
    pub has_env_example: u32,
    pub has_editorconfig: u32,
    pub no_floating_action_tags: u32,
    pub deps_up_to_date: u32,
    pub no_known_cves: u32,
    pub runtime_not_eol: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitStatus {
    pub github: Option<RateLimitInfo>,
    pub gitlab: Option<RateLimitInfo>,
}

// ── Defaults ───────────────────────────────────────────────────────────────

impl Default for HealthScoreWeights {
    fn default() -> Self {
        Self {
            has_codeowners: 10,
            has_security_md: 10,
            has_env_example: 5,
            has_editorconfig: 5,
            no_floating_action_tags: 15,
            deps_up_to_date: 20,
            no_known_cves: 20,
            runtime_not_eol: 15,
        }
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            scan_interval_minutes: Some(1440), // daily
            cve_poll_interval_minutes: Some(60),
            parallel_workers: 5,
            request_delay_ms: 200,
            health_score_weights: HealthScoreWeights::default(),
            webhook_url: None,
            webhook_events: Vec::new(),
            dark_mode: true,
        }
    }
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
    outcome: &str,
    detail: Option<&str>,
) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO audit_log (id, timestamp, action_type, repo_ids, operation_id, outcome, detail)
        VALUES (?, ?, ?, '[]', NULL, ?, ?)
        "#,
    )
    .bind(gen_id())
    .bind(Utc::now().to_rfc3339())
    .bind(action_type)
    .bind(outcome)
    .bind(detail)
    .execute(pool)
    .await?;
    Ok(())
}

// ── Commands ───────────────────────────────────────────────────────────────

/// Load all settings from the DB and return as a structured `AppSettings`.
/// Falls back to defaults for any missing keys.
#[tauri::command]
pub async fn get_settings() -> AppResult<AppSettings> {
    let pool = db::pool()?;

    let rows = sqlx::query("SELECT key, value FROM settings")
        .fetch_all(pool)
        .await?;

    // Build a map of stored values
    let mut map = std::collections::HashMap::new();
    for row in &rows {
        let k: String = row.get("key");
        let v: String = row.get("value");
        map.insert(k, v);
    }

    // If there's a full JSON blob stored under "app_settings", use it as a base
    if let Some(json_str) = map.get("app_settings") {
        if let Ok(stored) = serde_json::from_str::<AppSettings>(json_str) {
            return Ok(stored);
        }
    }

    // Otherwise, construct from individual keys (or return defaults)
    let defaults = AppSettings::default();

    let scan_interval_minutes = map
        .get("scan_interval_minutes")
        .and_then(|v| v.parse().ok())
        .or(defaults.scan_interval_minutes);
    let cve_poll_interval_minutes = map
        .get("cve_poll_interval_minutes")
        .and_then(|v| v.parse().ok())
        .or(defaults.cve_poll_interval_minutes);
    let parallel_workers = map
        .get("parallel_workers")
        .and_then(|v| v.parse().ok())
        .unwrap_or(defaults.parallel_workers);
    let request_delay_ms = map
        .get("request_delay_ms")
        .and_then(|v| v.parse().ok())
        .unwrap_or(defaults.request_delay_ms);
    let health_score_weights = map
        .get("health_score_weights")
        .and_then(|v| serde_json::from_str(v).ok())
        .unwrap_or_default();
    let webhook_url = map.get("webhook_url").cloned().or(defaults.webhook_url);
    let webhook_events = map
        .get("webhook_events")
        .and_then(|v| serde_json::from_str(v).ok())
        .unwrap_or(defaults.webhook_events);
    let dark_mode = map
        .get("dark_mode")
        .map(|v| v == "true")
        .unwrap_or(defaults.dark_mode);

    Ok(AppSettings {
        scan_interval_minutes,
        cve_poll_interval_minutes,
        parallel_workers,
        request_delay_ms,
        health_score_weights,
        webhook_url,
        webhook_events,
        dark_mode,
    })
}

/// Save settings by storing the entire AppSettings as a single JSON blob
/// under the "app_settings" key (UPSERT).
#[tauri::command]
pub async fn save_settings(settings: AppSettings) -> AppResult<()> {
    let pool = db::pool()?;
    let now = Utc::now().to_rfc3339();

    let json_str = serde_json::to_string(&settings).map_err(|e| {
        crate::error::AppError::InvalidInput(format!("Failed to serialize settings: {e}"))
    })?;

    sqlx::query(
        r#"
        INSERT INTO settings (key, value, updated_at)
        VALUES ('app_settings', ?, ?)
        ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at
        "#,
    )
    .bind(&json_str)
    .bind(&now)
    .execute(pool)
    .await?;

    write_audit_log(pool, "settings_saved", "success", Some("Settings updated")).await?;

    tracing::info!("Settings saved");
    Ok(())
}

/// Return current rate limit status for configured providers.
#[tauri::command]
pub async fn get_rate_limit_status() -> AppResult<RateLimitStatus> {
    Ok(RateLimitStatus {
        github: crate::services::rate_limiter::get_github(),
        gitlab: None,
    })
}

/// Query the audit log with optional filters.
#[tauri::command]
pub async fn list_audit_log(
    limit: Option<u32>,
    action_type: Option<String>,
) -> AppResult<Vec<serde_json::Value>> {
    let pool = db::pool()?;
    let row_limit = limit.unwrap_or(100);

    let rows = if let Some(ref at) = action_type {
        sqlx::query(
            r#"
            SELECT id, timestamp, action_type, repo_ids, operation_id, outcome, detail
            FROM audit_log
            WHERE action_type = ?
            ORDER BY timestamp DESC
            LIMIT ?
            "#,
        )
        .bind(at)
        .bind(row_limit)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query(
            r#"
            SELECT id, timestamp, action_type, repo_ids, operation_id, outcome, detail
            FROM audit_log
            ORDER BY timestamp DESC
            LIMIT ?
            "#,
        )
        .bind(row_limit)
        .fetch_all(pool)
        .await?
    };

    let entries: Vec<serde_json::Value> = rows
        .iter()
        .map(|row| {
            let repo_ids_raw: String = row.get("repo_ids");
            let repo_ids: serde_json::Value =
                serde_json::from_str(&repo_ids_raw).unwrap_or(serde_json::json!([]));

            serde_json::json!({
                "id": row.get::<String, _>("id"),
                "timestamp": row.get::<String, _>("timestamp"),
                "actionType": row.get::<String, _>("action_type"),
                "repoIds": repo_ids,
                "operationId": row.get::<Option<String>, _>("operation_id"),
                "outcome": row.get::<String, _>("outcome"),
                "detail": row.get::<Option<String>, _>("detail"),
            })
        })
        .collect();

    Ok(entries)
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_round_trip() {
        // Verify AppSettings serialises and deserialises correctly with camelCase
        let settings = AppSettings::default();
        let json = serde_json::to_string(&settings).expect("serialize");

        // Check camelCase keys
        assert!(
            json.contains("\"scanIntervalMinutes\""),
            "expected camelCase scanIntervalMinutes in: {json}"
        );
        assert!(
            json.contains("\"cvePollIntervalMinutes\""),
            "expected camelCase cvePollIntervalMinutes in: {json}"
        );
        assert!(
            json.contains("\"parallelWorkers\""),
            "expected camelCase parallelWorkers in: {json}"
        );
        assert!(
            json.contains("\"requestDelayMs\""),
            "expected camelCase requestDelayMs in: {json}"
        );
        assert!(
            json.contains("\"healthScoreWeights\""),
            "expected camelCase healthScoreWeights in: {json}"
        );
        assert!(
            json.contains("\"darkMode\""),
            "expected camelCase darkMode in: {json}"
        );

        // Deserialise back
        let restored: AppSettings = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.parallel_workers, 5);
        assert_eq!(restored.request_delay_ms, 200);
        assert!(restored.dark_mode);
        assert_eq!(restored.health_score_weights.has_codeowners, 10);
        assert_eq!(restored.health_score_weights.no_known_cves, 20);
    }

    #[test]
    fn default_health_score_weights_sum_to_100() {
        let w = HealthScoreWeights::default();
        let total = w.has_codeowners
            + w.has_security_md
            + w.has_env_example
            + w.has_editorconfig
            + w.no_floating_action_tags
            + w.deps_up_to_date
            + w.no_known_cves
            + w.runtime_not_eol;
        assert_eq!(total, 100, "Default health score weights should sum to 100");
    }
}
