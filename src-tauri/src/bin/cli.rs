use clap::{Parser, Subcommand};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::Row;
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(
    name = "git-flotilla",
    about = "Git Flotilla CLI — manage repos at scale",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output as JSON instead of human-readable format
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// List and inspect repos
    Repo {
        #[command(subcommand)]
        action: RepoAction,
    },
    /// Scan repos for dependencies and health
    Scan {
        /// Repo ID to scan (e.g. "github:org/repo")
        #[arg(long)]
        repo: Option<String>,
        /// Repo list ID to scan all repos in
        #[arg(long)]
        list: Option<String>,
    },
    /// Check CVEs against scanned packages
    Cve {
        #[command(subcommand)]
        action: CveAction,
    },
    /// Show health summary across all repos
    Report,
}

#[derive(Subcommand)]
enum RepoAction {
    /// List all known repos
    List,
}

#[derive(Subcommand)]
enum CveAction {
    /// Run CVE check against scanned packages
    Check,
    /// List current CVE alerts
    List {
        /// Filter by severity: critical, high, medium, low
        #[arg(long)]
        severity: Option<String>,
    },
}

/// Resolve the path to the Flotilla SQLite database.
///
/// Priority:
///   1. FLOTILLA_DB_PATH environment variable (explicit override)
///   2. Platform-specific Tauri data directory (matches where the GUI stores the DB)
fn resolve_db_path() -> Result<PathBuf, String> {
    // Explicit override
    if let Ok(p) = std::env::var("FLOTILLA_DB_PATH") {
        return Ok(PathBuf::from(p));
    }

    // Follow the same convention Tauri uses for app_data_dir.
    // Tauri v2 uses the identifier from tauri.conf.json — "com.gitflotilla.desktop"
    // which Tauri maps to:
    //   Linux:   ~/.local/share/com.gitflotilla.desktop/
    //   macOS:   ~/Library/Application Support/com.gitflotilla.desktop/
    //   Windows: %APPDATA%/com.gitflotilla.desktop/
    let data_dir = dirs::data_dir().ok_or_else(|| {
        "Could not determine platform data directory. Set FLOTILLA_DB_PATH instead.".to_string()
    })?;

    // Try the Tauri-style identifier first, then a simpler fallback
    let tauri_dir = data_dir.join("com.gitflotilla.desktop");
    if tauri_dir.join("flotilla.db").exists() {
        return Ok(tauri_dir.join("flotilla.db"));
    }

    let simple_dir = data_dir.join("git-flotilla");
    if simple_dir.join("flotilla.db").exists() {
        return Ok(simple_dir.join("flotilla.db"));
    }

    // Default to the Tauri-style path even if the file does not exist yet
    Ok(tauri_dir.join("flotilla.db"))
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let db_path = resolve_db_path().map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;

    if !db_path.exists() {
        return Err(format!(
            "Database not found at {}. Launch the Git Flotilla GUI first to initialise the database, or set FLOTILLA_DB_PATH.",
            db_path.display()
        )
        .into());
    }

    let db_url = format!("sqlite://{}?mode=ro", db_path.display());
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&db_url)
        .await
        .map_err(|e| format!("Failed to open database at {}: {e}", db_path.display()))?;

    match cli.command {
        Commands::Repo { action } => handle_repo(action, &pool, cli.json).await?,
        Commands::Scan { repo, list } => handle_scan(repo, list, &pool, cli.json).await?,
        Commands::Cve { action } => handle_cve(action, &pool, cli.json).await?,
        Commands::Report => handle_report(&pool, cli.json).await?,
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Repo
// ---------------------------------------------------------------------------

async fn handle_repo(
    action: RepoAction,
    pool: &sqlx::SqlitePool,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        RepoAction::List => {
            let rows = sqlx::query(
                "SELECT id, full_name, provider, default_branch, is_private, last_scanned_at \
                 FROM repos ORDER BY full_name",
            )
            .fetch_all(pool)
            .await?;

            if json {
                let repos: Vec<serde_json::Value> = rows
                    .iter()
                    .map(|r| {
                        serde_json::json!({
                            "id": r.get::<String, _>("id"),
                            "fullName": r.get::<String, _>("full_name"),
                            "provider": r.get::<String, _>("provider"),
                            "defaultBranch": r.get::<String, _>("default_branch"),
                            "isPrivate": r.get::<bool, _>("is_private"),
                            "lastScannedAt": r.get::<Option<String>, _>("last_scanned_at"),
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&repos)?);
            } else {
                if rows.is_empty() {
                    println!("No repos found. Discover repos via the Git Flotilla GUI first.");
                    return Ok(());
                }
                println!(
                    "{:<50} {:<10} {:<12} LAST SCANNED",
                    "REPO", "PROVIDER", "BRANCH"
                );
                println!("{}", "-".repeat(90));
                for r in &rows {
                    let scanned: String = r
                        .get::<Option<String>, _>("last_scanned_at")
                        .unwrap_or_else(|| "never".to_string());
                    println!(
                        "{:<50} {:<10} {:<12} {}",
                        r.get::<String, _>("full_name"),
                        r.get::<String, _>("provider"),
                        r.get::<String, _>("default_branch"),
                        scanned,
                    );
                }
                println!("\n{} repo(s) total", rows.len());
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Scan
// ---------------------------------------------------------------------------

async fn handle_scan(
    repo: Option<String>,
    list: Option<String>,
    pool: &sqlx::SqlitePool,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(repo_id) = repo {
        // Show latest scan result for a single repo
        let row = sqlx::query(
            "SELECT sr.repo_id, sr.scanned_at, sr.health_score, sr.node_version, \
             sr.package_manager, sr.package_manager_version, sr.php_version, \
             sr.has_develop, sr.excluded, sr.exclude_reason, sr.manifest_paths, sr.workflow_files \
             FROM scan_results sr \
             WHERE sr.repo_id = ? \
             ORDER BY sr.scanned_at DESC LIMIT 1",
        )
        .bind(&repo_id)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(r) => {
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "repoId": r.get::<String, _>("repo_id"),
                            "scannedAt": r.get::<String, _>("scanned_at"),
                            "healthScore": r.get::<i32, _>("health_score"),
                            "nodeVersion": r.get::<Option<String>, _>("node_version"),
                            "phpVersion": r.get::<Option<String>, _>("php_version"),
                            "packageManager": r.get::<Option<String>, _>("package_manager"),
                            "packageManagerVersion": r.get::<Option<String>, _>("package_manager_version"),
                            "hasDevelop": r.get::<bool, _>("has_develop"),
                            "excluded": r.get::<bool, _>("excluded"),
                            "excludeReason": r.get::<Option<String>, _>("exclude_reason"),
                            "manifestPaths": r.get::<String, _>("manifest_paths"),
                            "workflowFiles": r.get::<String, _>("workflow_files"),
                        }))?
                    );
                } else {
                    println!("Scan result for {repo_id}");
                    println!("{}", "-".repeat(50));
                    println!("Scanned at:       {}", r.get::<String, _>("scanned_at"));
                    println!("Health score:     {}/100", r.get::<i32, _>("health_score"));
                    if let Some(nv) = r.get::<Option<String>, _>("node_version") {
                        println!("Node version:     {nv}");
                    }
                    if let Some(pv) = r.get::<Option<String>, _>("php_version") {
                        println!("PHP version:      {pv}");
                    }
                    if let Some(pm) = r.get::<Option<String>, _>("package_manager") {
                        let ver = r
                            .get::<Option<String>, _>("package_manager_version")
                            .unwrap_or_default();
                        println!("Package manager:  {pm} {ver}");
                    }
                    println!(
                        "Has develop:      {}",
                        if r.get::<bool, _>("has_develop") {
                            "yes"
                        } else {
                            "no"
                        }
                    );
                    if r.get::<bool, _>("excluded") {
                        println!(
                            "Excluded:         yes ({})",
                            r.get::<Option<String>, _>("exclude_reason")
                                .unwrap_or_default()
                        );
                    }
                }
            }
            None => {
                if json {
                    println!("null");
                } else {
                    println!("No scan results found for {repo_id}. Run a scan in the GUI first.");
                }
            }
        }
    } else if let Some(list_id) = list {
        // Show scan summary for a repo list
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM repo_list_members WHERE list_id = ?",
        )
        .bind(&list_id)
        .fetch_one(pool)
        .await?;

        let scanned = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(DISTINCT sr.repo_id) \
             FROM scan_results sr \
             INNER JOIN repo_list_members rlm ON sr.repo_id = rlm.repo_id \
             WHERE rlm.list_id = ?",
        )
        .bind(&list_id)
        .fetch_one(pool)
        .await?;

        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "listId": list_id,
                    "totalRepos": count,
                    "scannedRepos": scanned,
                }))?
            );
        } else {
            if count == 0 {
                println!("Repo list '{list_id}' not found or empty.");
            } else {
                println!("Repo list: {list_id}");
                println!("Total repos: {count}");
                println!("Scanned:     {scanned}");
            }
        }
    } else {
        eprintln!("Specify --repo <REPO_ID> or --list <LIST_ID>");
        process::exit(1);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// CVE
// ---------------------------------------------------------------------------

async fn handle_cve(
    action: CveAction,
    pool: &sqlx::SqlitePool,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        CveAction::Check => {
            // Show summary of how many packages are in DB and how many CVEs matched
            let pkg_count =
                sqlx::query_scalar::<_, i64>("SELECT COUNT(DISTINCT name) FROM repo_packages")
                    .fetch_one(pool)
                    .await?;
            let cve_count = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM cve_alerts WHERE status = 'new'",
            )
            .fetch_one(pool)
            .await?;

            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "uniquePackagesTracked": pkg_count,
                        "openCves": cve_count,
                    }))?
                );
            } else {
                println!("CVE Status");
                println!("{}", "-".repeat(40));
                println!("Unique packages tracked: {pkg_count}");
                println!("Open CVE alerts:         {cve_count}");
                if cve_count > 0 {
                    println!("\nRun `git-flotilla-cli cve list` for details.");
                }
            }
        }
        CveAction::List { severity } => {
            let rows = if let Some(ref sev) = severity {
                sqlx::query(
                    "SELECT id, package_name, ecosystem, severity, summary, status \
                     FROM cve_alerts WHERE severity = ? ORDER BY detected_at DESC",
                )
                .bind(sev)
                .fetch_all(pool)
                .await?
            } else {
                sqlx::query(
                    "SELECT id, package_name, ecosystem, severity, summary, status \
                     FROM cve_alerts ORDER BY detected_at DESC",
                )
                .fetch_all(pool)
                .await?
            };

            if json {
                let alerts: Vec<serde_json::Value> = rows
                    .iter()
                    .map(|r| {
                        serde_json::json!({
                            "id": r.get::<String, _>("id"),
                            "packageName": r.get::<String, _>("package_name"),
                            "ecosystem": r.get::<String, _>("ecosystem"),
                            "severity": r.get::<String, _>("severity"),
                            "summary": r.get::<String, _>("summary"),
                            "status": r.get::<String, _>("status"),
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&alerts)?);
            } else {
                if rows.is_empty() {
                    let qualifier = severity
                        .as_ref()
                        .map(|s| format!(" with severity '{s}'"))
                        .unwrap_or_default();
                    println!("No CVE alerts found{qualifier}.");
                    return Ok(());
                }
                println!(
                    "{:<20} {:<30} {:<12} {:<10} STATUS",
                    "CVE", "PACKAGE", "ECOSYSTEM", "SEVERITY"
                );
                println!("{}", "-".repeat(90));
                for r in &rows {
                    println!(
                        "{:<20} {:<30} {:<12} {:<10} {}",
                        r.get::<String, _>("id"),
                        r.get::<String, _>("package_name"),
                        r.get::<String, _>("ecosystem"),
                        r.get::<String, _>("severity"),
                        r.get::<String, _>("status"),
                    );
                }
                println!("\n{} alert(s) total", rows.len());
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Report
// ---------------------------------------------------------------------------

async fn handle_report(
    pool: &sqlx::SqlitePool,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let repo_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM repos")
        .fetch_one(pool)
        .await?;

    let scan_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(DISTINCT repo_id) FROM scan_results")
            .fetch_one(pool)
            .await?;

    let avg_health = sqlx::query_scalar::<_, f64>(
        "SELECT COALESCE(AVG(health_score), 0) FROM scan_results sr \
         INNER JOIN (SELECT repo_id, MAX(scanned_at) as latest FROM scan_results GROUP BY repo_id) l \
         ON sr.repo_id = l.repo_id AND sr.scanned_at = l.latest",
    )
    .fetch_one(pool)
    .await?;

    let cve_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM cve_alerts WHERE status = 'new'")
            .fetch_one(pool)
            .await?;

    let pkg_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(DISTINCT name) FROM repo_packages")
        .fetch_one(pool)
        .await?;

    let cve_critical = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM cve_alerts WHERE status = 'new' AND severity = 'critical'",
    )
    .fetch_one(pool)
    .await?;

    let cve_high = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM cve_alerts WHERE status = 'new' AND severity = 'high'",
    )
    .fetch_one(pool)
    .await?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "repos": repo_count,
                "scannedRepos": scan_count,
                "averageHealthScore": avg_health.round() as i64,
                "openCves": cve_count,
                "criticalCves": cve_critical,
                "highCves": cve_high,
                "uniquePackages": pkg_count,
            }))?
        );
    } else {
        println!("Git Flotilla — Health Report");
        println!("{}", "\u{2501}".repeat(35));
        println!("Repos:              {repo_count}");
        println!("Scanned:            {scan_count}");
        println!("Avg Health Score:   {:.0}/100", avg_health);
        println!("Open CVEs:          {cve_count}");
        if cve_critical > 0 || cve_high > 0 {
            println!("  Critical:         {cve_critical}");
            println!("  High:             {cve_high}");
        }
        println!("Unique Packages:    {pkg_count}");
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_parses_repo_list() {
        let cli = Cli::try_parse_from(["git-flotilla", "repo", "list"]).unwrap();
        assert!(matches!(
            cli.command,
            Commands::Repo {
                action: RepoAction::List
            }
        ));
    }

    #[test]
    fn cli_parses_json_flag() {
        let cli = Cli::try_parse_from(["git-flotilla", "--json", "report"]).unwrap();
        assert!(cli.json);
    }

    #[test]
    fn cli_parses_json_flag_after_command() {
        let cli = Cli::try_parse_from(["git-flotilla", "report", "--json"]).unwrap();
        assert!(cli.json);
    }

    #[test]
    fn cli_parses_scan_with_repo() {
        let cli =
            Cli::try_parse_from(["git-flotilla", "scan", "--repo", "github:org/repo"]).unwrap();
        assert!(matches!(
            cli.command,
            Commands::Scan {
                repo: Some(_),
                list: None
            }
        ));
    }

    #[test]
    fn cli_parses_scan_with_list() {
        let cli = Cli::try_parse_from(["git-flotilla", "scan", "--list", "my-list"]).unwrap();
        assert!(matches!(
            cli.command,
            Commands::Scan {
                repo: None,
                list: Some(_)
            }
        ));
    }

    #[test]
    fn cli_parses_cve_list_with_severity() {
        let cli =
            Cli::try_parse_from(["git-flotilla", "cve", "list", "--severity", "critical"]).unwrap();
        assert!(matches!(
            cli.command,
            Commands::Cve {
                action: CveAction::List { severity: Some(_) }
            }
        ));
    }

    #[test]
    fn cli_parses_cve_check() {
        let cli = Cli::try_parse_from(["git-flotilla", "cve", "check"]).unwrap();
        assert!(matches!(
            cli.command,
            Commands::Cve {
                action: CveAction::Check
            }
        ));
    }

    #[test]
    fn resolve_db_path_respects_env() {
        std::env::set_var("FLOTILLA_DB_PATH", "/tmp/test-flotilla.db");
        let path = resolve_db_path().unwrap();
        assert_eq!(path, PathBuf::from("/tmp/test-flotilla.db"));
        std::env::remove_var("FLOTILLA_DB_PATH");
    }
}
