// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Initialise logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("git_flotilla=debug".parse().unwrap()),
        )
        .with_target(false)
        .init();

    tracing::info!("Git Flotilla starting");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Initialise DB on startup
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = git_flotilla::db::init(&app_handle).await {
                    tracing::error!("DB initialisation failed: {e}");
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Auth
            git_flotilla::commands::auth::add_account,
            git_flotilla::commands::auth::remove_account,
            git_flotilla::commands::auth::list_accounts,
            git_flotilla::commands::auth::validate_token,
            // Repos
            git_flotilla::commands::repos::discover_repos,
            git_flotilla::commands::repos::list_repos,
            git_flotilla::commands::repos::get_repo,
            // Repo lists
            git_flotilla::commands::repos::create_repo_list,
            git_flotilla::commands::repos::update_repo_list,
            git_flotilla::commands::repos::delete_repo_list,
            git_flotilla::commands::repos::list_repo_lists,
            git_flotilla::commands::repos::add_repos_to_list,
            git_flotilla::commands::repos::remove_repos_from_list,
            git_flotilla::commands::repos::set_repo_tags,
            git_flotilla::commands::repos::export_repo_list,
            git_flotilla::commands::repos::import_repo_list,
            // Scanning
            git_flotilla::commands::scan::scan_repo,
            git_flotilla::commands::scan::scan_repo_list,
            git_flotilla::commands::scan::get_scan_result,
            git_flotilla::commands::scan::list_scan_results,
            git_flotilla::commands::scan::abort_scan,
            // Packages
            git_flotilla::commands::packages::get_dependency_matrix,
            git_flotilla::commands::packages::get_package_changelog,
            git_flotilla::commands::packages::export_matrix_csv,
            // CVE
            git_flotilla::commands::cve::check_cves,
            git_flotilla::commands::cve::list_cve_alerts,
            git_flotilla::commands::cve::acknowledge_cve,
            git_flotilla::commands::cve::dismiss_cve,
            git_flotilla::commands::cve::snooze_cve,
            git_flotilla::commands::cve::get_cve_incident,
            git_flotilla::commands::cve::get_blast_radius,
            git_flotilla::commands::cve::add_to_watchlist,
            git_flotilla::commands::cve::remove_from_watchlist,
            git_flotilla::commands::cve::list_watchlist,
            // Operations
            git_flotilla::commands::operations::create_operation,
            git_flotilla::commands::operations::run_operation,
            git_flotilla::commands::operations::abort_operation,
            git_flotilla::commands::operations::list_operations,
            git_flotilla::commands::operations::get_operation,
            git_flotilla::commands::operations::validate_operation,
            git_flotilla::commands::operations::rollback_operation,
            // Merge queue
            git_flotilla::commands::merge_queue::list_flotilla_prs,
            git_flotilla::commands::merge_queue::merge_pr,
            git_flotilla::commands::merge_queue::merge_all_green,
            // Scripts
            git_flotilla::commands::scripts::run_script,
            git_flotilla::commands::scripts::abort_script,
            git_flotilla::commands::scripts::list_presets,
            git_flotilla::commands::scripts::save_preset,
            git_flotilla::commands::scripts::delete_preset,
            // Compliance
            git_flotilla::commands::compliance::scan_secrets,
            git_flotilla::commands::compliance::scan_licences,
            git_flotilla::commands::compliance::audit_branch_protection,
            git_flotilla::commands::compliance::archive_repos,
            // Settings
            git_flotilla::commands::settings::get_settings,
            git_flotilla::commands::settings::save_settings,
            git_flotilla::commands::settings::get_rate_limit_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Git Flotilla");
}
