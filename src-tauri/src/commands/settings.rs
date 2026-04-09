use crate::db;
use crate::error::AppResult;
use crate::models::RateLimitInfo;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::sync::{LazyLock, RwLock};

// ── Notification Store ────────────────────────────────────────────────────

static NOTIFICATIONS: LazyLock<RwLock<Vec<AppNotification>>> =
    LazyLock::new(|| RwLock::new(Vec::new()));

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppNotification {
    pub id: String,
    pub notification_type: String,
    pub title: String,
    pub message: String,
    pub timestamp: String,
    pub is_read: bool,
}

/// Internal helper — adds a notification to the in-memory store.
/// Not exposed as a Tauri command.
pub fn push_notification(notification_type: &str, title: &str, message: &str) {
    let notification = AppNotification {
        id: gen_id(),
        notification_type: notification_type.to_string(),
        title: title.to_string(),
        message: message.to_string(),
        timestamp: Utc::now().to_rfc3339(),
        is_read: false,
    };
    if let Ok(mut store) = NOTIFICATIONS.write() {
        store.insert(0, notification);
        // Cap at 200 notifications to prevent unbounded growth
        store.truncate(200);
    }
}

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

// ── Notification Commands ──────────────────────────────────────────────────

/// Return all notifications, newest first.
#[tauri::command]
pub async fn list_notifications() -> AppResult<Vec<AppNotification>> {
    let store = NOTIFICATIONS
        .read()
        .map_err(|e| crate::error::AppError::Operation(format!("Lock poisoned: {e}")))?;
    Ok(store.clone())
}

/// Mark a single notification as read.
#[tauri::command]
pub async fn mark_notification_read(id: String) -> AppResult<()> {
    let mut store = NOTIFICATIONS
        .write()
        .map_err(|e| crate::error::AppError::Operation(format!("Lock poisoned: {e}")))?;
    if let Some(notif) = store.iter_mut().find(|n| n.id == id) {
        notif.is_read = true;
    }
    Ok(())
}

/// Clear all notifications.
#[tauri::command]
pub async fn clear_notifications() -> AppResult<()> {
    let mut store = NOTIFICATIONS
        .write()
        .map_err(|e| crate::error::AppError::Operation(format!("Lock poisoned: {e}")))?;
    store.clear();
    Ok(())
}

// ── CSV Export Commands ───────────────────────────────────────────────────

/// Export the audit log as a CSV string.
/// The frontend is responsible for triggering the file download.
#[tauri::command]
pub async fn export_audit_log_csv() -> AppResult<String> {
    let pool = db::pool()?;

    let rows = sqlx::query(
        r#"
        SELECT timestamp, action_type, outcome, detail, repo_ids
        FROM audit_log
        ORDER BY timestamp DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut csv = String::from("timestamp,action_type,outcome,detail,repo_ids\n");
    for row in &rows {
        let timestamp: String = row.get("timestamp");
        let action_type: String = row.get("action_type");
        let outcome: String = row.get("outcome");
        let detail: Option<String> = row.get("detail");
        let repo_ids: String = row.get("repo_ids");

        csv.push_str(&format!(
            "{},{},{},{},{}\n",
            csv_escape(&timestamp),
            csv_escape(&action_type),
            csv_escape(&outcome),
            csv_escape(&detail.unwrap_or_default()),
            csv_escape(&repo_ids),
        ));
    }

    Ok(csv)
}

/// Export the latest health report as CSV, optionally filtered by repo list.
#[tauri::command]
pub async fn export_health_report_csv(repo_list_id: Option<String>) -> AppResult<String> {
    let pool = db::pool()?;

    let rows = if let Some(ref list_id) = repo_list_id {
        sqlx::query(
            r#"
            SELECT s.repo_id, s.health_score, s.node_version, s.package_manager,
                   s.manifest_paths, s.flags, s.excluded
            FROM scan_results s
            JOIN repo_list_members m ON m.repo_id = s.repo_id
            WHERE m.list_id = ?
            AND s.scanned_at = (
                SELECT MAX(s2.scanned_at) FROM scan_results s2 WHERE s2.repo_id = s.repo_id
            )
            ORDER BY s.repo_id
            "#,
        )
        .bind(list_id)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query(
            r#"
            SELECT s.repo_id, s.health_score, s.node_version, s.package_manager,
                   s.manifest_paths, s.flags, s.excluded
            FROM scan_results s
            WHERE s.scanned_at = (
                SELECT MAX(s2.scanned_at) FROM scan_results s2 WHERE s2.repo_id = s.repo_id
            )
            ORDER BY s.repo_id
            "#,
        )
        .fetch_all(pool)
        .await?
    };

    let mut csv = String::from(
        "repo_id,health_score,node_version,package_manager,manifest_count,flags_count,excluded\n",
    );
    for row in &rows {
        let repo_id: String = row.get("repo_id");
        let health_score: i64 = row.get("health_score");
        let node_version: Option<String> = row.get("node_version");
        let package_manager: Option<String> = row.get("package_manager");
        let manifest_paths: String = row.get("manifest_paths");
        let flags: String = row.get("flags");
        let excluded: bool = row.get("excluded");

        let manifest_count = serde_json::from_str::<Vec<serde_json::Value>>(&manifest_paths)
            .map(|v| v.len())
            .unwrap_or(0);
        let flags_count = serde_json::from_str::<Vec<serde_json::Value>>(&flags)
            .map(|v| v.len())
            .unwrap_or(0);

        csv.push_str(&format!(
            "{},{},{},{},{},{},{}\n",
            csv_escape(&repo_id),
            health_score,
            csv_escape(&node_version.unwrap_or_default()),
            csv_escape(&package_manager.unwrap_or_default()),
            manifest_count,
            flags_count,
            excluded,
        ));
    }

    Ok(csv)
}

/// Export CVE alerts as CSV with affected repo counts.
#[tauri::command]
pub async fn export_cve_report_csv() -> AppResult<String> {
    let pool = db::pool()?;

    let rows = sqlx::query(
        r#"
        SELECT c.id, c.package_name, c.ecosystem, c.severity, c.status,
               c.fixed_version, c.published_at,
               COUNT(a.repo_id) AS affected_repos_count
        FROM cve_alerts c
        LEFT JOIN cve_affected_repos a ON a.cve_id = c.id
        GROUP BY c.id
        ORDER BY c.published_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut csv = String::from(
        "cve_id,package,ecosystem,severity,status,affected_repos_count,fixed_version,published_at\n",
    );
    for row in &rows {
        let cve_id: String = row.get("id");
        let package: String = row.get("package_name");
        let ecosystem: String = row.get("ecosystem");
        let severity: String = row.get("severity");
        let status: String = row.get("status");
        let affected_repos_count: i64 = row.get("affected_repos_count");
        let fixed_version: Option<String> = row.get("fixed_version");
        let published_at: String = row.get("published_at");

        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{}\n",
            csv_escape(&cve_id),
            csv_escape(&package),
            csv_escape(&ecosystem),
            csv_escape(&severity),
            csv_escape(&status),
            affected_repos_count,
            csv_escape(&fixed_version.unwrap_or_default()),
            csv_escape(&published_at),
        ));
    }

    Ok(csv)
}

/// Escape a value for CSV: if it contains commas, quotes, or newlines,
/// wrap in quotes and double any existing quotes.
fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

// ── Config Import / Export Commands ───────────────────────────────────────

/// Export current settings as a YAML config string.
#[tauri::command]
pub async fn export_config() -> AppResult<String> {
    let settings = get_settings().await?;
    let config = crate::services::config::FlotillaConfig::from_app_settings(&settings);
    crate::services::config::export_config(&config)
}

/// Import settings from a YAML config string, validate, and save.
#[tauri::command]
pub async fn import_config(yaml: String) -> AppResult<()> {
    let config = crate::services::config::import_config(&yaml)?;
    let errors = crate::services::config::validate_config(&config);
    if !errors.is_empty() {
        return Err(crate::error::AppError::InvalidInput(format!(
            "Config validation errors: {}",
            errors.join("; ")
        )));
    }
    let settings = config.to_app_settings();
    save_settings(settings).await
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_escape_plain_value() {
        assert_eq!(csv_escape("hello"), "hello");
    }

    #[test]
    fn csv_escape_with_comma() {
        assert_eq!(csv_escape("hello,world"), "\"hello,world\"");
    }

    #[test]
    fn csv_escape_with_quotes() {
        assert_eq!(csv_escape("say \"hi\""), "\"say \"\"hi\"\"\"");
    }

    #[test]
    fn csv_escape_with_newline() {
        assert_eq!(csv_escape("line1\nline2"), "\"line1\nline2\"");
    }

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
    fn push_notification_adds_to_store() {
        // Clear any existing notifications first
        if let Ok(mut store) = NOTIFICATIONS.write() {
            store.clear();
        }
        push_notification("scan_complete", "Scan Done", "Scanned 5 repos");
        let store = NOTIFICATIONS.read().expect("read lock");
        assert!(!store.is_empty());
        let n = &store[0];
        assert_eq!(n.notification_type, "scan_complete");
        assert_eq!(n.title, "Scan Done");
        assert_eq!(n.message, "Scanned 5 repos");
        assert!(!n.is_read);
        assert!(!n.id.is_empty());
    }

    #[test]
    fn notification_serialises_camel_case() {
        let n = AppNotification {
            id: "abc".to_string(),
            notification_type: "cve_found".to_string(),
            title: "CVE".to_string(),
            message: "msg".to_string(),
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            is_read: false,
        };
        let json = serde_json::to_string(&n).expect("serialize");
        assert!(
            json.contains("\"notificationType\""),
            "expected camelCase: {json}"
        );
        assert!(json.contains("\"isRead\""), "expected camelCase: {json}");
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
