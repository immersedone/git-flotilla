use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::CveAlert;
use crate::services::cve_scraper;
use serde::Serialize;
use sqlx::Row;

// ── Response structs ──────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlastRadius {
    pub cve_id: String,
    pub direct_repos: Vec<String>,
    pub transitive_repos: Vec<String>,
    pub dependency_paths: Vec<Vec<String>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncidentTimeline {
    pub cve_id: String,
    pub published_at: String,
    pub detected_at: String,
    pub events: Vec<IncidentEvent>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncidentEvent {
    pub timestamp: String,
    pub event_type: String,
    pub repo_id: Option<String>,
    pub detail: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchlistEntry {
    pub package_name: String,
    pub ecosystem: String,
    pub added_at: String,
}

// ── Commands ──────────────────────────────────────────────────────────────

/// Scan all packages in the database for known CVEs via OSV.dev.
#[tauri::command]
pub async fn check_cves() -> AppResult<Vec<CveAlert>> {
    let pool = db::pool()?;
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    // 1. Fetch all unique (name, ecosystem, version) from repo_packages
    let rows = sqlx::query("SELECT DISTINCT name, ecosystem, version FROM repo_packages")
        .fetch_all(pool)
        .await?;

    let packages: Vec<(String, String, String)> = rows
        .iter()
        .map(|row| {
            let name: String = row.get("name");
            let ecosystem: String = row.get("ecosystem");
            let version: String = row.get("version");
            (name, ecosystem, version)
        })
        .collect();

    if packages.is_empty() {
        return Ok(Vec::new());
    }

    // 2. Query OSV.dev
    let vulns = cve_scraper::query_osv_batch(&packages).await?;

    let mut alerts = Vec::new();

    // 3. Process each vulnerability
    for vuln in &vulns {
        let cve_id = cve_scraper::extract_cve_id(vuln);
        let severity = vuln
            .severity
            .as_ref()
            .map(|s| cve_scraper::cvss_to_severity(s))
            .unwrap_or_else(|| "medium".to_string());
        let summary = vuln
            .summary
            .clone()
            .or_else(|| {
                vuln.details.as_ref().map(|d| {
                    if d.len() > 200 {
                        format!("{}...", &d[..200])
                    } else {
                        d.clone()
                    }
                })
            })
            .unwrap_or_else(|| "No description available".to_string());
        let fixed_version = cve_scraper::extract_fixed_version(vuln);
        let affected_range = cve_scraper::extract_affected_range(vuln);
        let published_at = vuln.published.clone().unwrap_or_else(|| now.clone());

        // Extract the package name from the affected field if available
        let affected_package_name = vuln
            .affected
            .as_ref()
            .and_then(|a| a.first())
            .and_then(|a| a.package.as_ref())
            .map(|p| p.name.clone());

        let affected_ecosystem = vuln
            .affected
            .as_ref()
            .and_then(|a| a.first())
            .and_then(|a| a.package.as_ref())
            .map(|p| p.ecosystem.clone());

        // Map OSV ecosystem back to our internal names for DB matching
        let internal_ecosystem = affected_ecosystem
            .as_deref()
            .map(map_osv_ecosystem_back)
            .unwrap_or("npm");

        let pkg_name = affected_package_name
            .clone()
            .unwrap_or_else(|| "unknown".to_string());

        // 3a. UPSERT into cve_alerts
        sqlx::query(
            "INSERT INTO cve_alerts (id, package_name, ecosystem, severity, summary, affected_version_range, fixed_version, published_at, detected_at, status)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'new')
             ON CONFLICT(id) DO UPDATE SET
                severity = excluded.severity,
                summary = excluded.summary,
                affected_version_range = excluded.affected_version_range,
                fixed_version = excluded.fixed_version"
        )
        .bind(&cve_id)
        .bind(&pkg_name)
        .bind(internal_ecosystem)
        .bind(&severity)
        .bind(&summary)
        .bind(&affected_range)
        .bind(&fixed_version)
        .bind(&published_at)
        .bind(&now)
        .execute(pool)
        .await?;

        // 3b. Find affected repos
        let affected_repo_rows = sqlx::query(
            "SELECT DISTINCT repo_id FROM repo_packages WHERE name = ? AND ecosystem = ?",
        )
        .bind(&pkg_name)
        .bind(internal_ecosystem)
        .fetch_all(pool)
        .await?;

        let affected_repos: Vec<String> = affected_repo_rows
            .iter()
            .map(|r| r.get("repo_id"))
            .collect();

        // 3c. INSERT into cve_affected_repos (ON CONFLICT IGNORE)
        for repo_id in &affected_repos {
            sqlx::query(
                "INSERT INTO cve_affected_repos (cve_id, repo_id, status)
                 VALUES (?, ?, 'new')
                 ON CONFLICT(cve_id, repo_id) DO NOTHING",
            )
            .bind(&cve_id)
            .bind(repo_id)
            .execute(pool)
            .await?;
        }

        // 3d. Read back the current status
        let status_row = sqlx::query("SELECT status FROM cve_alerts WHERE id = ?")
            .bind(&cve_id)
            .fetch_optional(pool)
            .await?;
        let status: String = status_row
            .map(|r| r.get("status"))
            .unwrap_or_else(|| "new".to_string());

        alerts.push(CveAlert {
            id: cve_id,
            package_name: pkg_name,
            ecosystem: internal_ecosystem.to_string(),
            severity,
            summary,
            affected_version_range: affected_range,
            fixed_version,
            published_at,
            detected_at: now.clone(),
            affected_repos,
            status,
        });
    }

    // Log to audit log
    if !alerts.is_empty() {
        let cve_count = alerts.len();
        sqlx::query(
            "INSERT INTO audit_log (id, timestamp, action_type, repo_ids, outcome, detail)
             VALUES (lower(hex(randomblob(16))), ?, 'cve_check', '[]', 'success', ?)",
        )
        .bind(&now)
        .bind(format!("Found {cve_count} CVE(s) from OSV.dev scan"))
        .execute(pool)
        .await?;
    }

    Ok(alerts)
}

/// List CVE alerts with optional severity and status filters.
#[tauri::command]
pub async fn list_cve_alerts(
    severity: Option<String>,
    status: Option<String>,
) -> AppResult<Vec<CveAlert>> {
    let pool = db::pool()?;

    // Build query dynamically based on filters
    let mut sql = String::from("SELECT * FROM cve_alerts WHERE 1=1");
    let mut bind_values: Vec<String> = Vec::new();

    if let Some(ref sev) = severity {
        sql.push_str(" AND severity = ?");
        bind_values.push(sev.clone());
    }
    if let Some(ref st) = status {
        sql.push_str(" AND status = ?");
        bind_values.push(st.clone());
    }
    sql.push_str(" ORDER BY detected_at DESC");

    let mut query = sqlx::query(&sql);
    for val in &bind_values {
        query = query.bind(val);
    }

    let rows = query.fetch_all(pool).await?;

    let mut alerts = Vec::new();
    for row in &rows {
        let cve_id: String = row.get("id");

        // Fetch affected repos
        let repo_rows = sqlx::query("SELECT repo_id FROM cve_affected_repos WHERE cve_id = ?")
            .bind(&cve_id)
            .fetch_all(pool)
            .await?;

        let affected_repos: Vec<String> = repo_rows.iter().map(|r| r.get("repo_id")).collect();

        alerts.push(CveAlert {
            id: cve_id,
            package_name: row.get("package_name"),
            ecosystem: row.get("ecosystem"),
            severity: row.get("severity"),
            summary: row.get("summary"),
            affected_version_range: row.get("affected_version_range"),
            fixed_version: row.get("fixed_version"),
            published_at: row.get("published_at"),
            detected_at: row.get("detected_at"),
            affected_repos,
            status: row.get("status"),
        });
    }

    Ok(alerts)
}

/// Acknowledge a CVE (per-repo or globally).
#[tauri::command]
pub async fn acknowledge_cve(cve_id: String, repo_id: Option<String>) -> AppResult<()> {
    let pool = db::pool()?;
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    if let Some(ref rid) = repo_id {
        sqlx::query(
            "UPDATE cve_affected_repos SET status = 'acknowledged' WHERE cve_id = ? AND repo_id = ?",
        )
        .bind(&cve_id)
        .bind(rid)
        .execute(pool)
        .await?;
    } else {
        sqlx::query("UPDATE cve_alerts SET status = 'acknowledged' WHERE id = ?")
            .bind(&cve_id)
            .execute(pool)
            .await?;
    }

    // Audit log
    let repo_ids_json = repo_id
        .as_ref()
        .map(|r| format!("[\"{r}\"]"))
        .unwrap_or_else(|| "[]".to_string());
    sqlx::query(
        "INSERT INTO audit_log (id, timestamp, action_type, repo_ids, outcome, detail)
         VALUES (lower(hex(randomblob(16))), ?, 'cve_acknowledge', ?, 'success', ?)",
    )
    .bind(&now)
    .bind(&repo_ids_json)
    .bind(format!("Acknowledged CVE {cve_id}"))
    .execute(pool)
    .await?;

    Ok(())
}

/// Dismiss a CVE (per-repo or globally).
#[tauri::command]
pub async fn dismiss_cve(cve_id: String, repo_id: Option<String>) -> AppResult<()> {
    let pool = db::pool()?;
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    if let Some(ref rid) = repo_id {
        sqlx::query(
            "UPDATE cve_affected_repos SET status = 'dismissed' WHERE cve_id = ? AND repo_id = ?",
        )
        .bind(&cve_id)
        .bind(rid)
        .execute(pool)
        .await?;
    } else {
        sqlx::query("UPDATE cve_alerts SET status = 'dismissed' WHERE id = ?")
            .bind(&cve_id)
            .execute(pool)
            .await?;
    }

    // Audit log
    let repo_ids_json = repo_id
        .as_ref()
        .map(|r| format!("[\"{r}\"]"))
        .unwrap_or_else(|| "[]".to_string());
    sqlx::query(
        "INSERT INTO audit_log (id, timestamp, action_type, repo_ids, outcome, detail)
         VALUES (lower(hex(randomblob(16))), ?, 'cve_dismiss', ?, 'success', ?)",
    )
    .bind(&now)
    .bind(&repo_ids_json)
    .bind(format!("Dismissed CVE {cve_id}"))
    .execute(pool)
    .await?;

    Ok(())
}

/// Snooze a CVE for N days (per-repo or globally).
#[tauri::command]
pub async fn snooze_cve(cve_id: String, repo_id: Option<String>, days: u32) -> AppResult<()> {
    let pool = db::pool()?;
    let now = chrono::Utc::now();
    let now_str = now.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let snoozed_until = (now + chrono::Duration::days(i64::from(days)))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();

    if let Some(ref rid) = repo_id {
        sqlx::query(
            "UPDATE cve_affected_repos SET status = 'snoozed', snoozed_until = ? WHERE cve_id = ? AND repo_id = ?",
        )
        .bind(&snoozed_until)
        .bind(&cve_id)
        .bind(rid)
        .execute(pool)
        .await?;
    } else {
        sqlx::query("UPDATE cve_alerts SET status = 'snoozed' WHERE id = ?")
            .bind(&cve_id)
            .execute(pool)
            .await?;
    }

    // Audit log
    let repo_ids_json = repo_id
        .as_ref()
        .map(|r| format!("[\"{r}\"]"))
        .unwrap_or_else(|| "[]".to_string());
    sqlx::query(
        "INSERT INTO audit_log (id, timestamp, action_type, repo_ids, outcome, detail)
         VALUES (lower(hex(randomblob(16))), ?, 'cve_snooze', ?, 'success', ?)",
    )
    .bind(&now_str)
    .bind(&repo_ids_json)
    .bind(format!(
        "Snoozed CVE {cve_id} for {days} days until {snoozed_until}"
    ))
    .execute(pool)
    .await?;

    Ok(())
}

/// Get incident timeline for a specific CVE.
#[tauri::command]
pub async fn get_cve_incident(cve_id: String) -> AppResult<IncidentTimeline> {
    let pool = db::pool()?;

    // Fetch the CVE alert
    let row = sqlx::query("SELECT * FROM cve_alerts WHERE id = ?")
        .bind(&cve_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("CVE {cve_id} not found")))?;

    let published_at: String = row.get("published_at");
    let detected_at: String = row.get("detected_at");

    let mut events = Vec::new();

    // Event: CVE published
    events.push(IncidentEvent {
        timestamp: published_at.clone(),
        event_type: "published".to_string(),
        repo_id: None,
        detail: format!("CVE {cve_id} published"),
    });

    // Event: CVE detected by Flotilla
    events.push(IncidentEvent {
        timestamp: detected_at.clone(),
        event_type: "detected".to_string(),
        repo_id: None,
        detail: format!("CVE {cve_id} detected by Flotilla scan"),
    });

    // Fetch audit log entries related to this CVE
    let audit_rows = sqlx::query(
        "SELECT timestamp, action_type, repo_ids, outcome, detail FROM audit_log
         WHERE detail LIKE ? ORDER BY timestamp ASC",
    )
    .bind(format!("%{cve_id}%"))
    .fetch_all(pool)
    .await?;

    for audit_row in &audit_rows {
        let timestamp: String = audit_row.get("timestamp");
        let action_type: String = audit_row.get("action_type");
        let detail: String = audit_row
            .get::<Option<String>, _>("detail")
            .unwrap_or_default();
        let repo_ids_json: String = audit_row.get("repo_ids");

        // Try to extract first repo_id from JSON array
        let repo_id = serde_json::from_str::<Vec<String>>(&repo_ids_json)
            .ok()
            .and_then(|v| v.into_iter().next());

        events.push(IncidentEvent {
            timestamp,
            event_type: action_type,
            repo_id,
            detail,
        });
    }

    // Sort events by timestamp
    events.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    Ok(IncidentTimeline {
        cve_id,
        published_at,
        detected_at,
        events,
    })
}

/// Get blast radius for a CVE — which repos are directly affected.
#[tauri::command]
pub async fn get_blast_radius(cve_id: String) -> AppResult<BlastRadius> {
    let pool = db::pool()?;

    // Verify CVE exists
    let exists = sqlx::query("SELECT id FROM cve_alerts WHERE id = ?")
        .bind(&cve_id)
        .fetch_optional(pool)
        .await?;

    if exists.is_none() {
        return Err(AppError::NotFound(format!("CVE {cve_id} not found")));
    }

    // Fetch directly affected repos
    let repo_rows = sqlx::query("SELECT repo_id FROM cve_affected_repos WHERE cve_id = ?")
        .bind(&cve_id)
        .fetch_all(pool)
        .await?;

    let direct_repos: Vec<String> = repo_rows.iter().map(|r| r.get("repo_id")).collect();

    Ok(BlastRadius {
        cve_id,
        direct_repos,
        transitive_repos: Vec::new(), // Transitive analysis deferred
        dependency_paths: Vec::new(), // Deferred
    })
}

/// Add a package to the CVE watchlist.
#[tauri::command]
pub async fn add_to_watchlist(package_name: String, ecosystem: String) -> AppResult<()> {
    let pool = db::pool()?;

    sqlx::query(
        "INSERT INTO cve_watchlist (id, package_name, ecosystem, added_at)
         VALUES (lower(hex(randomblob(16))), ?, ?, datetime('now'))
         ON CONFLICT(package_name, ecosystem) DO NOTHING",
    )
    .bind(&package_name)
    .bind(&ecosystem)
    .execute(pool)
    .await?;

    Ok(())
}

/// Remove a package from the CVE watchlist.
#[tauri::command]
pub async fn remove_from_watchlist(package_name: String, ecosystem: String) -> AppResult<()> {
    let pool = db::pool()?;

    let result = sqlx::query("DELETE FROM cve_watchlist WHERE package_name = ? AND ecosystem = ?")
        .bind(&package_name)
        .bind(&ecosystem)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!(
            "Watchlist entry not found: {package_name} ({ecosystem})"
        )));
    }

    Ok(())
}

/// List all packages on the CVE watchlist.
#[tauri::command]
pub async fn list_watchlist() -> AppResult<Vec<serde_json::Value>> {
    let pool = db::pool()?;

    let rows = sqlx::query(
        "SELECT package_name, ecosystem, added_at FROM cve_watchlist ORDER BY added_at DESC",
    )
    .fetch_all(pool)
    .await?;

    let entries: Vec<serde_json::Value> = rows
        .iter()
        .map(|row| {
            serde_json::json!({
                "packageName": row.get::<String, _>("package_name"),
                "ecosystem": row.get::<String, _>("ecosystem"),
                "addedAt": row.get::<String, _>("added_at"),
            })
        })
        .collect();

    Ok(entries)
}

// ── Internal helpers ──────────────────────────────────────────────────────

/// Map OSV ecosystem names back to our internal ecosystem names.
fn map_osv_ecosystem_back(osv_ecosystem: &str) -> &str {
    match osv_ecosystem {
        "Packagist" => "composer",
        "PyPI" => "pip",
        "crates.io" => "cargo",
        "Go" => "go",
        "npm" => "npm",
        _ => osv_ecosystem,
    }
}
