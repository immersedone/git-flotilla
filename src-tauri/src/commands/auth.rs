use crate::{
    db,
    error::{AppError, AppResult},
    services::github::GitHubClient,
};
use keyring::Entry;
use serde::{Deserialize, Serialize};

const KEYCHAIN_SERVICE: &str = "git-flotilla";

const REQUIRED_GITHUB_SCOPES: &[&str] = &["repo", "workflow", "read:org"];

/// Info about a connected account returned to the frontend.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    pub id:             String,   // "github:{username}"
    pub provider:       String,
    pub username:       String,
    pub avatar_url:     Option<String>,
    pub scopes:         Vec<String>,
    /// Required scopes that are absent — warning, not error.
    pub missing_scopes: Vec<String>,
}

fn account_id(provider: &str, username: &str) -> String {
    format!("{}:{}", provider, username)
}

fn keychain_entry(id: &str) -> AppResult<Entry> {
    Entry::new(KEYCHAIN_SERVICE, id).map_err(AppError::from)
}

fn check_missing_scopes(provider: &str, scopes: &[String]) -> Vec<String> {
    if provider != "github" {
        return vec![];
    }
    REQUIRED_GITHUB_SCOPES
        .iter()
        .filter(|required| !scopes.iter().any(|s| s == *required))
        .map(|s| s.to_string())
        .collect()
}

/// Validate a token against the GitHub API without storing it.
/// Returns AccountInfo including granted scopes and any missing required scopes.
#[tauri::command]
pub async fn validate_token(provider: String, token: String) -> AppResult<AccountInfo> {
    if provider != "github" {
        return Err(AppError::InvalidInput(format!(
            "Unsupported provider: {provider}. Currently only 'github' is supported."
        )));
    }

    let client = GitHubClient::new(&token);
    let (user, scopes, rate_limit) = client.get_authenticated_user().await?;

    // Update global rate limit state
    if let Some(rl) = rate_limit {
        crate::services::rate_limiter::update_github(rl);
    }

    let missing_scopes = check_missing_scopes(&provider, &scopes);

    Ok(AccountInfo {
        id:             account_id(&provider, &user.login),
        provider,
        username:       user.login,
        avatar_url:     user.avatar_url,
        scopes,
        missing_scopes,
    })
}

/// Validate token, store in OS keychain, persist account record in SQLite.
#[tauri::command]
pub async fn add_account(provider: String, token: String) -> AppResult<AccountInfo> {
    let info = validate_token(provider.clone(), token.clone()).await?;

    // Store token in OS keychain
    keychain_entry(&info.id)?
        .set_password(&token)
        .map_err(AppError::from)?;

    // Upsert account record (no token) into SQLite
    let pool = db::pool()?;
    sqlx::query(
        r#"
        INSERT INTO accounts (id, provider, username, avatar_url)
        VALUES (?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
            username   = excluded.username,
            avatar_url = excluded.avatar_url,
            updated_at = datetime('now')
        "#,
    )
    .bind(&info.id)
    .bind(&info.provider)
    .bind(&info.username)
    .bind(&info.avatar_url)
    .execute(pool)
    .await?;

    // Audit log entry
    if let Err(e) = sqlx::query(
        "INSERT INTO audit_log (action_type, repo_ids, outcome, detail) VALUES (?, '[]', 'success', ?)",
    )
    .bind("account_added")
    .bind(&info.id)
    .execute(pool)
    .await
    {
        tracing::warn!("Failed to write audit log for account_added {}: {e}", info.id);
    }

    tracing::info!("Account added: {}", info.id);
    Ok(info)
}

/// Remove an account: delete from keychain and SQLite.
#[tauri::command]
pub async fn remove_account(id: String) -> AppResult<()> {
    // Remove from keychain (ignore if already gone)
    if let Ok(entry) = keychain_entry(&id) {
        let _ = entry.delete_credential();
    }

    let pool = db::pool()?;
    sqlx::query("DELETE FROM accounts WHERE id = ?")
        .bind(&id)
        .execute(pool)
        .await?;

    if let Err(e) = sqlx::query(
        "INSERT INTO audit_log (action_type, repo_ids, outcome, detail) VALUES (?, '[]', 'success', ?)",
    )
    .bind("account_removed")
    .bind(&id)
    .execute(pool)
    .await
    {
        tracing::warn!("Failed to write audit log for account_removed {}: {e}", id);
    }

    tracing::info!("Account removed: {id}");
    Ok(())
}

/// List all accounts from SQLite. Filters out any whose keychain entry is missing.
#[tauri::command]
pub async fn list_accounts() -> AppResult<Vec<AccountInfo>> {
    let pool = db::pool()?;

    #[derive(sqlx::FromRow)]
    struct Row {
        id:         String,
        provider:   String,
        username:   String,
        avatar_url: Option<String>,
    }

    let rows = sqlx::query_as::<_, Row>(
        "SELECT id, provider, username, avatar_url FROM accounts ORDER BY added_at",
    )
    .fetch_all(pool)
    .await?;

    let mut accounts = Vec::new();
    for row in rows {
        match keychain_entry(&row.id).and_then(|e| e.get_password().map_err(AppError::from)) {
            Ok(_) => accounts.push(AccountInfo {
                id:             row.id,
                provider:       row.provider,
                username:       row.username,
                avatar_url:     row.avatar_url,
                scopes:         vec![],          // not re-validated on list
                missing_scopes: vec![],
            }),
            Err(_) => {
                // Keychain entry gone — clean up stale DB record
                tracing::warn!("Keychain entry missing for {}, removing DB record", row.id);
                let _ = sqlx::query("DELETE FROM accounts WHERE id = ?")
                    .bind(&row.id)
                    .execute(pool)
                    .await;
            }
        }
    }

    Ok(accounts)
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_id_format() {
        assert_eq!(account_id("github", "octocat"), "github:octocat");
        assert_eq!(account_id("gitlab", "alice"),   "gitlab:alice");
    }

    #[test]
    fn missing_scopes_all_present() {
        let scopes = vec!["repo".to_string(), "workflow".to_string(), "read:org".to_string()];
        assert!(check_missing_scopes("github", &scopes).is_empty());
    }

    #[test]
    fn missing_scopes_partial() {
        let scopes = vec!["repo".to_string()];
        let missing = check_missing_scopes("github", &scopes);
        assert!(missing.contains(&"workflow".to_string()));
        assert!(missing.contains(&"read:org".to_string()));
        assert!(!missing.contains(&"repo".to_string()));
    }

    #[test]
    fn missing_scopes_non_github_provider_always_empty() {
        let scopes = vec![];
        assert!(check_missing_scopes("gitlab", &scopes).is_empty());
    }
}
