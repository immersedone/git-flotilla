use chrono::{Duration, Utc};
use sqlx::SqlitePool;

use crate::error::AppResult;

fn days_ago(d: i64) -> String {
    (Utc::now() - Duration::days(d))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string()
}

fn hours_ago(h: i64) -> String {
    (Utc::now() - Duration::hours(h))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string()
}

/// Seeds the database with realistic demo data if it is empty (no repos).
/// This provides a polished first-run experience with data suitable for
/// screenshots and demos. The data uses the fictional "acme-corp" org.
pub async fn seed_if_empty(pool: &SqlitePool) -> AppResult<()> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM repos")
        .fetch_one(pool)
        .await?;

    if count.0 > 0 {
        return Ok(());
    }

    seed_all(pool).await?;

    tracing::info!("Seeded demo data");
    Ok(())
}

async fn seed_all(pool: &SqlitePool) -> AppResult<()> {
    seed_repos(pool).await?;
    seed_repo_list(pool).await?;
    let scanned_at = days_ago(1);
    seed_scans(pool, &scanned_at).await?;
    seed_packages(pool, &scanned_at).await?;
    seed_cves(pool).await?;
    seed_operations(pool).await?;
    seed_audit_log(pool).await?;
    seed_script_presets(pool).await?;

    Ok(())
}

async fn seed_repos(pool: &SqlitePool) -> AppResult<()> {
    let repos = [
        (
            "github:acme-corp/web-app",
            "github",
            "acme-corp",
            "web-app",
            "acme-corp/web-app",
            "https://github.com/acme-corp/web-app",
            "main",
            1i32,
            30i64,
        ),
        (
            "github:acme-corp/api-gateway",
            "github",
            "acme-corp",
            "api-gateway",
            "acme-corp/api-gateway",
            "https://github.com/acme-corp/api-gateway",
            "main",
            1,
            25,
        ),
        (
            "github:acme-corp/mobile-app",
            "github",
            "acme-corp",
            "mobile-app",
            "acme-corp/mobile-app",
            "https://github.com/acme-corp/mobile-app",
            "main",
            0,
            20,
        ),
        (
            "github:acme-corp/docs-site",
            "github",
            "acme-corp",
            "docs-site",
            "acme-corp/docs-site",
            "https://github.com/acme-corp/docs-site",
            "main",
            0,
            45,
        ),
        (
            "github:acme-corp/shared-utils",
            "github",
            "acme-corp",
            "shared-utils",
            "acme-corp/shared-utils",
            "https://github.com/acme-corp/shared-utils",
            "main",
            1,
            15,
        ),
    ];

    for (id, provider, owner, name, full_name, url, branch, is_private, offset_days) in &repos {
        let created = days_ago(*offset_days);
        sqlx::query(
            "INSERT INTO repos (id, provider, owner, name, full_name, url, default_branch, is_private, tags, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, '[]', ?, ?)"
        )
        .bind(id).bind(provider).bind(owner).bind(name).bind(full_name)
        .bind(url).bind(branch).bind(is_private).bind(&created).bind(&created)
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn seed_repo_list(pool: &SqlitePool) -> AppResult<()> {
    let list_id = "list-acme-corp-001";
    let created = days_ago(30);
    let updated = days_ago(1);
    let member_added = days_ago(30);

    sqlx::query(
        "INSERT INTO repo_lists (id, name, description, created_at, updated_at) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(list_id)
    .bind("Acme Corp")
    .bind("All Acme Corp repositories")
    .bind(&created)
    .bind(&updated)
    .execute(pool)
    .await?;

    let repo_ids = [
        "github:acme-corp/web-app",
        "github:acme-corp/api-gateway",
        "github:acme-corp/mobile-app",
        "github:acme-corp/docs-site",
        "github:acme-corp/shared-utils",
    ];

    for repo_id in &repo_ids {
        sqlx::query("INSERT INTO repo_list_members (list_id, repo_id, added_at) VALUES (?, ?, ?)")
            .bind(list_id)
            .bind(repo_id)
            .bind(&member_added)
            .execute(pool)
            .await?;
    }

    Ok(())
}

async fn seed_scans(
    pool: &SqlitePool,
    scanned_at: &str,
) -> AppResult<()> {
    let last_pushed = days_ago(2);

    // web-app: health 85
    sqlx::query(
        "INSERT INTO scan_results (id, repo_id, scanned_at, manifest_paths, node_version, node_version_source, php_version, package_manager, package_manager_version, has_develop, last_pushed, has_dot_env_example, workflow_files, health_score, flags, excluded, exclude_reason)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind("scan-001").bind("github:acme-corp/web-app").bind(scanned_at)
    .bind(r#"["package.json","resources/js/package.json"]"#)
    .bind("20.11.1").bind(".nvmrc").bind("^8.2")
    .bind("pnpm").bind("9.1.0")
    .bind(1i32).bind(&last_pushed).bind(1i32)
    .bind(r#"[".github/workflows/ci.yml",".github/workflows/deploy.yml"]"#)
    .bind(85i32)
    .bind(r#"[{"type":"floating_action_tag","detail":"actions/checkout@v3"}]"#)
    .bind(0i32).bind(Option::<&str>::None)
    .execute(pool).await?;

    // api-gateway: health 65
    sqlx::query(
        "INSERT INTO scan_results (id, repo_id, scanned_at, manifest_paths, node_version, node_version_source, php_version, package_manager, package_manager_version, has_develop, last_pushed, has_dot_env_example, workflow_files, health_score, flags, excluded, exclude_reason)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind("scan-002").bind("github:acme-corp/api-gateway").bind(scanned_at)
    .bind(r#"["package.json"]"#)
    .bind("22.4.0").bind(".node-version").bind(Option::<&str>::None)
    .bind("npm").bind("10.5.0")
    .bind(1i32).bind(&last_pushed).bind(0i32)
    .bind(r#"[".github/workflows/ci.yml",".github/workflows/deploy.yml",".github/workflows/lint.yml"]"#)
    .bind(65i32)
    .bind(r#"[{"type":"floating_action_tag","detail":"actions/checkout@v3"},{"type":"floating_action_tag","detail":"actions/setup-node@v3"}]"#)
    .bind(0i32).bind(Option::<&str>::None)
    .execute(pool).await?;

    // mobile-app: health 45
    sqlx::query(
        "INSERT INTO scan_results (id, repo_id, scanned_at, manifest_paths, node_version, node_version_source, php_version, package_manager, package_manager_version, has_develop, last_pushed, has_dot_env_example, workflow_files, health_score, flags, excluded, exclude_reason)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind("scan-003").bind("github:acme-corp/mobile-app").bind(scanned_at)
    .bind(r#"["package.json"]"#)
    .bind("20.11.1").bind(".nvmrc").bind(Option::<&str>::None)
    .bind("yarn").bind("4.1.0")
    .bind(0i32).bind(&last_pushed).bind(0i32)
    .bind(r#"[".github/workflows/build.yml"]"#)
    .bind(45i32)
    .bind(r#"[{"type":"missing_codeowners","detail":null},{"type":"missing_security_md","detail":null}]"#)
    .bind(0i32).bind(Option::<&str>::None)
    .execute(pool).await?;

    // docs-site: health 30 (Node 18 EOL)
    sqlx::query(
        "INSERT INTO scan_results (id, repo_id, scanned_at, manifest_paths, node_version, node_version_source, php_version, package_manager, package_manager_version, has_develop, last_pushed, has_dot_env_example, workflow_files, health_score, flags, excluded, exclude_reason)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind("scan-004").bind("github:acme-corp/docs-site").bind(scanned_at)
    .bind(r#"["package.json"]"#)
    .bind("18.19.0").bind("engines.node").bind(Option::<&str>::None)
    .bind("npm").bind("10.2.0")
    .bind(0i32).bind(&last_pushed).bind(0i32)
    .bind("[]")
    .bind(30i32)
    .bind(r#"[{"type":"node_eol","detail":"18.19.0"},{"type":"missing_codeowners","detail":null},{"type":"missing_security_md","detail":null},{"type":"no_workflows","detail":null}]"#)
    .bind(0i32).bind(Option::<&str>::None)
    .execute(pool).await?;

    // shared-utils: health 95
    sqlx::query(
        "INSERT INTO scan_results (id, repo_id, scanned_at, manifest_paths, node_version, node_version_source, php_version, package_manager, package_manager_version, has_develop, last_pushed, has_dot_env_example, workflow_files, health_score, flags, excluded, exclude_reason)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind("scan-005").bind("github:acme-corp/shared-utils").bind(scanned_at)
    .bind(r#"["package.json"]"#)
    .bind("20.11.1").bind(".nvmrc").bind(Option::<&str>::None)
    .bind("pnpm").bind("9.1.0")
    .bind(1i32).bind(&last_pushed).bind(1i32)
    .bind(r#"[".github/workflows/ci.yml",".github/workflows/release.yml"]"#)
    .bind(95i32)
    .bind("[]")
    .bind(0i32).bind(Option::<&str>::None)
    .execute(pool).await?;

    // Update repos.last_scanned_at
    let repo_ids = [
        "github:acme-corp/web-app",
        "github:acme-corp/api-gateway",
        "github:acme-corp/mobile-app",
        "github:acme-corp/docs-site",
        "github:acme-corp/shared-utils",
    ];
    for repo_id in &repo_ids {
        sqlx::query("UPDATE repos SET last_scanned_at = ? WHERE id = ?")
            .bind(scanned_at)
            .bind(repo_id)
            .execute(pool)
            .await?;
    }

    Ok(())
}

async fn seed_packages(pool: &SqlitePool, scanned_at: &str) -> AppResult<()> {
    let pkgs: &[(&str, &str, &str, &str, i32)] = &[
        // vue (3 repos)
        ("github:acme-corp/web-app", "npm", "vue", "3.4.0", 0),
        ("github:acme-corp/docs-site", "npm", "vue", "3.4.0", 0),
        ("github:acme-corp/shared-utils", "npm", "vue", "3.4.0", 0),
        // axios (4 repos)
        ("github:acme-corp/web-app", "npm", "axios", "1.7.2", 0),
        ("github:acme-corp/api-gateway", "npm", "axios", "1.7.2", 0),
        ("github:acme-corp/mobile-app", "npm", "axios", "1.7.2", 0),
        ("github:acme-corp/docs-site", "npm", "axios", "1.7.2", 0),
        // lodash (2 repos)
        ("github:acme-corp/web-app", "npm", "lodash", "4.17.21", 0),
        (
            "github:acme-corp/api-gateway",
            "npm",
            "lodash",
            "4.17.21",
            0,
        ),
        // typescript (5 repos)
        ("github:acme-corp/web-app", "npm", "typescript", "5.4.0", 1),
        (
            "github:acme-corp/api-gateway",
            "npm",
            "typescript",
            "5.4.0",
            1,
        ),
        (
            "github:acme-corp/mobile-app",
            "npm",
            "typescript",
            "5.4.0",
            1,
        ),
        (
            "github:acme-corp/docs-site",
            "npm",
            "typescript",
            "5.4.0",
            1,
        ),
        (
            "github:acme-corp/shared-utils",
            "npm",
            "typescript",
            "5.4.0",
            1,
        ),
        // express drift: 4.18.2 in 2 repos, 4.19.0 in 1
        (
            "github:acme-corp/api-gateway",
            "npm",
            "express",
            "4.18.2",
            0,
        ),
        ("github:acme-corp/web-app", "npm", "express", "4.18.2", 0),
        ("github:acme-corp/docs-site", "npm", "express", "4.19.0", 0),
        // react-native (mobile only)
        (
            "github:acme-corp/mobile-app",
            "npm",
            "react-native",
            "0.74.1",
            0,
        ),
        ("github:acme-corp/mobile-app", "npm", "react", "18.2.0", 0),
        // next.js (docs only)
        ("github:acme-corp/docs-site", "npm", "next", "14.2.3", 0),
        // laravel (web-app, composer)
        (
            "github:acme-corp/web-app",
            "composer",
            "laravel/framework",
            "11.5.0",
            0,
        ),
        // dev deps
        ("github:acme-corp/web-app", "npm", "vitest", "2.0.0", 1),
        ("github:acme-corp/api-gateway", "npm", "vitest", "2.0.0", 1),
        ("github:acme-corp/shared-utils", "npm", "vitest", "2.0.0", 1),
        ("github:acme-corp/web-app", "npm", "eslint", "9.0.0", 1),
        ("github:acme-corp/api-gateway", "npm", "eslint", "9.0.0", 1),
        ("github:acme-corp/mobile-app", "npm", "eslint", "9.0.0", 1),
        ("github:acme-corp/shared-utils", "npm", "eslint", "9.0.0", 1),
        ("github:acme-corp/web-app", "npm", "prettier", "3.3.0", 1),
        (
            "github:acme-corp/api-gateway",
            "npm",
            "prettier",
            "3.3.0",
            1,
        ),
        (
            "github:acme-corp/shared-utils",
            "npm",
            "prettier",
            "3.3.0",
            1,
        ),
        // tailwindcss
        ("github:acme-corp/web-app", "npm", "tailwindcss", "4.0.0", 1),
        (
            "github:acme-corp/docs-site",
            "npm",
            "tailwindcss",
            "4.0.0",
            1,
        ),
    ];

    for (repo_id, ecosystem, name, version, is_dev) in pkgs {
        sqlx::query(
            "INSERT INTO repo_packages (repo_id, ecosystem, name, version, is_dev, scanned_at)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(repo_id)
        .bind(ecosystem)
        .bind(name)
        .bind(version)
        .bind(is_dev)
        .bind(scanned_at)
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn seed_cves(pool: &SqlitePool) -> AppResult<()> {
    // CVE alerts
    let published_1 = days_ago(5);
    let published_2 = days_ago(7);
    let published_3 = days_ago(3);
    let detected = days_ago(1);

    sqlx::query(
        "INSERT INTO cve_alerts (id, package_name, ecosystem, severity, summary, affected_version_range, fixed_version, published_at, detected_at, status)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind("CVE-2024-48930").bind("lodash").bind("npm").bind("critical")
    .bind("Prototype Pollution in lodash via the merge, mergeWith, and defaultsDeep functions")
    .bind("<4.17.22").bind("4.17.22")
    .bind(&published_1).bind(&detected).bind("new")
    .execute(pool).await?;

    sqlx::query(
        "INSERT INTO cve_alerts (id, package_name, ecosystem, severity, summary, affected_version_range, fixed_version, published_at, detected_at, status)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind("CVE-2024-39338").bind("axios").bind("npm").bind("high")
    .bind("Server-Side Request Forgery (SSRF) in axios when following redirects")
    .bind("<1.7.3").bind("1.7.3")
    .bind(&published_2).bind(&detected).bind("acknowledged")
    .execute(pool).await?;

    sqlx::query(
        "INSERT INTO cve_alerts (id, package_name, ecosystem, severity, summary, affected_version_range, fixed_version, published_at, detected_at, status)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind("CVE-2024-12345").bind("express").bind("npm").bind("medium")
    .bind("Open Redirect vulnerability in express res.redirect allows URL injection")
    .bind("<4.19.1").bind("4.19.1")
    .bind(&published_3).bind(&detected).bind("new")
    .execute(pool).await?;

    // Affected repos
    let affected: &[(&str, &str, &str)] = &[
        // lodash CVE — 2 repos
        ("CVE-2024-48930", "github:acme-corp/web-app", "new"),
        ("CVE-2024-48930", "github:acme-corp/api-gateway", "new"),
        // axios CVE — 4 repos
        ("CVE-2024-39338", "github:acme-corp/web-app", "acknowledged"),
        (
            "CVE-2024-39338",
            "github:acme-corp/api-gateway",
            "acknowledged",
        ),
        (
            "CVE-2024-39338",
            "github:acme-corp/mobile-app",
            "acknowledged",
        ),
        (
            "CVE-2024-39338",
            "github:acme-corp/docs-site",
            "acknowledged",
        ),
        // express CVE — 3 repos
        ("CVE-2024-12345", "github:acme-corp/api-gateway", "new"),
        ("CVE-2024-12345", "github:acme-corp/web-app", "new"),
        ("CVE-2024-12345", "github:acme-corp/docs-site", "new"),
    ];

    for (cve_id, repo_id, status) in affected {
        sqlx::query("INSERT INTO cve_affected_repos (cve_id, repo_id, status) VALUES (?, ?, ?)")
            .bind(cve_id)
            .bind(repo_id)
            .bind(status)
            .execute(pool)
            .await?;
    }

    Ok(())
}

async fn seed_operations(pool: &SqlitePool) -> AppResult<()> {
    let op_completed_id = "op-seed-001";
    let op_pending_id = "op-seed-002";

    let all_axios_repos = r#"["github:acme-corp/web-app","github:acme-corp/api-gateway","github:acme-corp/mobile-app","github:acme-corp/docs-site"]"#;
    let completed_at = days_ago(2);
    let pending_at = hours_ago(3);

    sqlx::query(
        "INSERT INTO batch_operations (id, operation_type, mode, status, target_repo_ids, completed_repo_ids, version_map, is_dry_run, skip_ci, created_at, completed_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(op_completed_id)
    .bind("package_bump").bind("bump").bind("completed")
    .bind(all_axios_repos).bind(all_axios_repos)
    .bind(r#"{"1":"1.7.3"}"#)
    .bind(0i32).bind(0i32)
    .bind(&completed_at).bind(&completed_at)
    .execute(pool).await?;

    let nvmrc_targets = r#"["github:acme-corp/web-app","github:acme-corp/api-gateway","github:acme-corp/shared-utils"]"#;

    sqlx::query(
        "INSERT INTO batch_operations (id, operation_type, mode, status, target_repo_ids, completed_repo_ids, version_map, is_dry_run, skip_ci, created_at, completed_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(op_pending_id)
    .bind("file_update").bind(Option::<&str>::None).bind("pending")
    .bind(nvmrc_targets).bind("[]")
    .bind(Option::<&str>::None)
    .bind(1i32).bind(0i32)
    .bind(&pending_at).bind(Option::<&str>::None)
    .execute(pool).await?;

    // Operation results for the completed operation
    let result_repos = ["web-app", "api-gateway", "mobile-app", "docs-site"];
    let result_at = days_ago(2);

    for (i, repo_name) in result_repos.iter().enumerate() {
        let repo_id = format!("github:acme-corp/{repo_name}");
        let pr_url = format!("https://github.com/acme-corp/{repo_name}/pull/{}", 100 + i);
        sqlx::query(
            "INSERT INTO operation_results (operation_id, repo_id, status, pr_url, created_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(op_completed_id)
        .bind(&repo_id)
        .bind("success")
        .bind(&pr_url)
        .bind(&result_at)
        .execute(pool)
        .await?;
    }

    Ok(())
}

#[allow(clippy::type_complexity)]
async fn seed_audit_log(
    pool: &SqlitePool,
) -> AppResult<()> {
    let ts1 = days_ago(5);
    let ts2 = days_ago(3);
    let ts3 = days_ago(2);
    let ts4 = days_ago(1);
    let ts5 = hours_ago(3);

    let entries: &[(&str, &str, &str, Option<&str>, &str, &str)] = &[
        (
            &ts1,
            "scan_completed",
            r#"["github:acme-corp/web-app","github:acme-corp/api-gateway","github:acme-corp/mobile-app","github:acme-corp/docs-site","github:acme-corp/shared-utils"]"#,
            None,
            "success",
            "Scanned 5 repos in Acme Corp list",
        ),
        (
            &ts2,
            "cve_detected",
            r#"["github:acme-corp/web-app","github:acme-corp/api-gateway"]"#,
            None,
            "alert",
            "CVE-2024-48930 (critical) detected in lodash — affects 2 repos",
        ),
        (
            &ts3,
            "operation_completed",
            r#"["github:acme-corp/web-app","github:acme-corp/api-gateway","github:acme-corp/mobile-app","github:acme-corp/docs-site"]"#,
            Some("op-seed-001"),
            "success",
            "Bumped axios to 1.7.3 across 4 repos — all PRs created",
        ),
        (
            &ts4,
            "cve_acknowledged",
            r#"["github:acme-corp/web-app","github:acme-corp/api-gateway","github:acme-corp/mobile-app","github:acme-corp/docs-site"]"#,
            None,
            "info",
            "Acknowledged CVE-2024-39338 (axios SSRF) — fix in progress",
        ),
        (
            &ts5,
            "operation_created",
            r#"["github:acme-corp/web-app","github:acme-corp/api-gateway","github:acme-corp/shared-utils"]"#,
            Some("op-seed-002"),
            "info",
            "Created dry-run file_update to sync .nvmrc across 3 repos",
        ),
    ];

    for (ts, action, repo_ids, op_id, outcome, detail) in entries {
        sqlx::query(
            "INSERT INTO audit_log (timestamp, action_type, repo_ids, operation_id, outcome, detail)
             VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(ts).bind(action).bind(repo_ids).bind(op_id).bind(outcome).bind(detail)
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn seed_script_presets(pool: &SqlitePool) -> AppResult<()> {
    let created = days_ago(10);

    sqlx::query(
        "INSERT INTO script_presets (name, command, description, created_at) VALUES (?, ?, ?, ?)",
    )
    .bind("npm outdated")
    .bind("npm outdated --json")
    .bind("List outdated npm packages in JSON format")
    .bind(&created)
    .execute(pool)
    .await?;

    sqlx::query(
        "INSERT INTO script_presets (name, command, description, created_at) VALUES (?, ?, ?, ?)",
    )
    .bind("npx depcheck")
    .bind("npx depcheck --json")
    .bind("Find unused dependencies and missing declarations")
    .bind(&created)
    .execute(pool)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./src/db/migrations")
            .run(&pool)
            .await
            .unwrap();
        pool
    }

    #[tokio::test]
    async fn test_seed_populates_empty_db() {
        let pool = setup_pool().await;

        seed_if_empty(&pool).await.unwrap();

        let repo_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM repos")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(repo_count.0, 5, "should seed 5 repos");

        let list_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM repo_lists")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(list_count.0, 1, "should seed 1 repo list");

        let member_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM repo_list_members")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(member_count.0, 5, "should assign all 5 repos to the list");

        let scan_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM scan_results")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(scan_count.0, 5, "should seed 5 scan results");

        let pkg_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM repo_packages")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(
            pkg_count.0 >= 30,
            "should seed at least 30 packages, got {}",
            pkg_count.0
        );

        let cve_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM cve_alerts")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(cve_count.0, 3, "should seed 3 CVE alerts");

        let cve_repo_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM cve_affected_repos")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(
            cve_repo_count.0, 9,
            "should seed 9 CVE-affected-repo entries"
        );

        let op_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM batch_operations")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(op_count.0, 2, "should seed 2 batch operations");

        let result_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM operation_results")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(result_count.0, 4, "should seed 4 operation results");

        let audit_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM audit_log")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(audit_count.0, 5, "should seed 5 audit log entries");

        let script_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM script_presets")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(script_count.0, 2, "should seed 2 script presets");
    }

    #[tokio::test]
    async fn test_seed_is_idempotent() {
        let pool = setup_pool().await;

        seed_if_empty(&pool).await.unwrap();
        seed_if_empty(&pool).await.unwrap();

        let repo_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM repos")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(repo_count.0, 5, "second seed call should be a no-op");
    }

    #[tokio::test]
    async fn test_seed_skips_nonempty_db() {
        let pool = setup_pool().await;

        sqlx::query(
            "INSERT INTO repos (id, provider, owner, name, full_name, url, default_branch, is_private, tags)
             VALUES ('github:other/repo', 'github', 'other', 'repo', 'other/repo', 'https://github.com/other/repo', 'main', 0, '[]')"
        )
        .execute(&pool)
        .await
        .unwrap();

        seed_if_empty(&pool).await.unwrap();

        let repo_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM repos")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(repo_count.0, 1, "should not seed when repos already exist");
    }
}
