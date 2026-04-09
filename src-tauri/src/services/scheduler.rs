use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::{interval, Duration};

use crate::error::{AppError, AppResult};

static SCHEDULER_RUNNING: AtomicBool = AtomicBool::new(false);

/// Default polling interval: 1 hour (in seconds).
const DEFAULT_INTERVAL_SECS: u64 = 3600;

/// Start the background scheduler. Call once during app init.
///
/// This spawns a long-lived tokio task that periodically runs CVE checks
/// against all packages in the database. The function is idempotent — calling
/// it multiple times will only start one scheduler.
pub fn start_scheduler() {
    if SCHEDULER_RUNNING.swap(true, Ordering::SeqCst) {
        tracing::info!("Scheduler already running, skipping duplicate start");
        return;
    }

    tauri::async_runtime::spawn(async move {
        tracing::info!("Background scheduler started (interval: {DEFAULT_INTERVAL_SECS}s)");

        let mut tick = interval(Duration::from_secs(DEFAULT_INTERVAL_SECS));

        // The first tick completes immediately — consume it so we don't
        // run a CVE check right at startup (the user can trigger one manually).
        tick.tick().await;

        loop {
            tick.tick().await;

            tracing::info!("Scheduler tick: checking for CVE updates");

            if let Err(e) = run_scheduled_cve_check().await {
                tracing::error!("Scheduled CVE check failed: {e}");
            }
        }
    });
}

/// Run a CVE check against all packages currently stored in the database.
///
/// This queries the `repo_packages` table for distinct (name, ecosystem, version)
/// triples and sends them to the OSV.dev batch API. Results are logged; full
/// upsert into the `cve_alerts` table will be added when the CVE processing
/// service is extracted from `commands/cve.rs`.
async fn run_scheduled_cve_check() -> AppResult<()> {
    let pool = crate::db::pool()?;

    let rows = sqlx::query_as::<_, (String, String, String)>(
        "SELECT DISTINCT name, ecosystem, version FROM repo_packages",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::Database(format!("Failed to fetch packages: {e}")))?;

    if rows.is_empty() {
        tracing::info!("No packages to check for CVEs");
        return Ok(());
    }

    tracing::info!("Checking {} distinct packages against OSV.dev", rows.len());

    let vulns = crate::services::cve_scraper::query_osv_batch(&rows).await?;

    tracing::info!(
        "Scheduled CVE check complete: {} vulnerabilities found",
        vulns.len()
    );

    // TODO: Extract shared CVE processing/upsert logic from commands/cve.rs
    // into a service function and call it here to persist results.

    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheduler_running_flag_is_atomic() {
        // Reset state for test isolation
        SCHEDULER_RUNNING.store(false, Ordering::SeqCst);
        assert!(!SCHEDULER_RUNNING.load(Ordering::SeqCst));

        // Simulate first call — swap returns false (was not running)
        let was_running = SCHEDULER_RUNNING.swap(true, Ordering::SeqCst);
        assert!(!was_running);
        assert!(SCHEDULER_RUNNING.load(Ordering::SeqCst));

        // Simulate second call — swap returns true (already running)
        let was_running = SCHEDULER_RUNNING.swap(true, Ordering::SeqCst);
        assert!(was_running);

        // Reset for other tests
        SCHEDULER_RUNNING.store(false, Ordering::SeqCst);
    }

    #[test]
    fn default_interval_is_one_hour() {
        assert_eq!(DEFAULT_INTERVAL_SECS, 3600);
    }
}
