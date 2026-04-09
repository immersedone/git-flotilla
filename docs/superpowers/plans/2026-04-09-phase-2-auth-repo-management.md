# Phase 2 — Authentication & Repo Management Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement GitHub PAT authentication (keychain storage, scope validation, multi-account), repo discovery from GitHub API (paginated, upserted to SQLite), repo list CRUD (nested hierarchy, tag filtering, YAML export/import), and the corresponding Auth + RepoLists views.

**Architecture:** Rust commands call a typed `GitHubClient` service (reqwest) which reads tokens from the OS keychain (`keyring` crate). Discovered repos are upserted into SQLite. Rate limit info is tracked in a process-global `LazyLock<RwLock<…>>` and surfaced to the frontend via the `get_rate_limit_status` command. The Vue frontend uses Pinia stores with actions that delegate to typed service wrappers, then renders `Auth.vue` (account management) and `RepoLists.vue` (discovery + list management).

**Tech Stack:** Rust · keyring 3 · reqwest 0.12 · serde_yaml 0.9 · sqlx + SQLite · tauri::command · Vue 3 Composition API · Pinia · Vitest + @vue/test-utils + happy-dom

---

## Environment

- Working directory: `/var/www/vhosts/git-flotilla`
- Rust toolchain: `~/.cargo/bin/cargo` (1.94.1)
- Node: pnpm 10 / Node 25
- Phase 1 scaffold is complete — all skeleton files exist, builds clean

---

## Key Design Decisions

- **Account ID format:** `"{provider}:{username}"` — e.g. `"github:octocat"`
- **Keychain:** service = `"git-flotilla"`, key = account ID — token as raw string; never in SQLite
- **`accounts` table:** stores id/provider/username/avatar_url only — token stays in keychain
- **GitHub API base:** `https://api.github.com` — headers `Authorization: Bearer {token}`, `Accept: application/vnd.github+json`, `X-GitHub-Api-Version: 2022-11-28`, `User-Agent: git-flotilla/0.1`
- **Pagination:** iterate `?per_page=100&page=N` until response has < 100 items
- **Rate limit tracking:** extracted from response headers, stored in `LazyLock<RwLock<Option<RateLimitInfo>>>` in `services/rate_limiter.rs`
- **YAML repo list format:** flat (no nested children) — each list exported/imported independently
- **GitLab:** deferred to a later phase — this plan covers GitHub only

---

## File Map

### New — Rust
| File | Responsibility |
|------|---------------|
| `src-tauri/src/db/migrations/002_accounts.sql` | `accounts` table |
| `src-tauri/src/services/github.rs` | GitHub API HTTP client struct + typed response models |
| `src-tauri/src/services/rate_limiter.rs` | Global rate limit state (LazyLock + RwLock) |

### Modified — Rust
| File | What changes |
|------|-------------|
| `src-tauri/Cargo.toml` | Add `serde_yaml = "0.9"` |
| `src-tauri/src/services/mod.rs` | Expose `github` and `rate_limiter` modules |
| `src-tauri/src/error.rs` | Add `From<keyring::Error>` impl |
| `src-tauri/src/commands/auth.rs` | Full implementation (validate, add, remove, list accounts) |
| `src-tauri/src/commands/repos.rs` | Full implementation + new commands: `set_repo_tags`, `export_repo_list`, `import_repo_list` |
| `src-tauri/src/commands/settings.rs` | Implement `get_rate_limit_status` |
| `src-tauri/src/main.rs` | Register 3 new commands in `invoke_handler!` |

### New — Vue
| File | Responsibility |
|------|---------------|
| `src/services/settings.ts` | `getRateLimitStatus()` invoke wrapper |
| `src/stores/__tests__/auth.spec.ts` | Vitest tests for auth store |
| `src/stores/__tests__/repos.spec.ts` | Vitest tests for repos store |

### Modified — Vue
| File | What changes |
|------|-------------|
| `package.json` | Add `@vue/test-utils`, `happy-dom` dev deps; add `test` script |
| `vite.config.ts` | Add Vitest `test` section |
| `src/stores/auth.ts` | Add actions: `loadAccounts`, `addAccountAction`, `removeAccountAction` |
| `src/stores/repos.ts` | Add actions: `discoverRepos`, `loadRepos`, `setRepoTags` |
| `src/stores/repoLists.ts` | Add actions: `loadLists`, `createListAction`, `updateListAction`, `deleteListAction`, `addRepos`, `removeRepos`, `exportList`, `importList` |
| `src/stores/settings.ts` | Add `refreshRateLimit()` action |
| `src/services/repos.ts` | Add `setRepoTags`, `exportRepoList`, `importRepoList` |
| `src/views/Auth.vue` | Full account management UI |
| `src/views/RepoLists.vue` | Repo discovery + list management UI (two-tab layout) |

---

## Task 1: Accounts Migration + Cargo/Vitest Setup

**Files:**
- Create: `src-tauri/src/db/migrations/002_accounts.sql`
- Modify: `src-tauri/Cargo.toml` (add `serde_yaml`)
- Modify: `package.json` (add `@vue/test-utils`, `happy-dom`)
- Modify: `vite.config.ts` (add Vitest test section)

- [ ] **Step 1: Create the accounts migration**

```sql
-- src-tauri/src/db/migrations/002_accounts.sql
-- Stores connected account metadata (tokens live in OS keychain, not here)
CREATE TABLE IF NOT EXISTS accounts (
    id         TEXT PRIMARY KEY,   -- "github:{username}"
    provider   TEXT NOT NULL,      -- "github" | "gitlab"
    username   TEXT NOT NULL,
    avatar_url TEXT,
    added_at   TEXT NOT NULL DEFAULT (datetime('now'))
);
```

- [ ] **Step 2: Add serde_yaml to Cargo.toml**

Open `src-tauri/Cargo.toml` and add after `thiserror`:

```toml
serde_yaml = "0.9"
```

- [ ] **Step 3: Install Vue test utilities**

```bash
cd /var/www/vhosts/git-flotilla
pnpm add -D @vue/test-utils happy-dom
```

- [ ] **Step 4: Add Vitest test section to vite.config.ts**

Read `vite.config.ts` then add `test` block. The full file becomes:

```typescript
import tailwindcss from '@tailwindcss/vite'
import vue from '@vitejs/plugin-vue'
import { defineConfig } from 'vite'
import { resolve } from 'path'

export default defineConfig({
  plugins: [vue(), tailwindcss()],
  resolve: {
    alias: { '@': resolve(__dirname, 'src') },
  },
  server: {
    port: 1420,
    strictPort: true,
    host: process.env.TAURI_DEV_HOST || false,
    watch: { ignored: ['**/src-tauri/**'] },
  },
  test: {
    environment: 'happy-dom',
    globals: true,
  },
})
```

- [ ] **Step 5: Add test script to package.json**

In `package.json`, add to the `scripts` object:

```json
"test": "vitest run",
"test:watch": "vitest"
```

- [ ] **Step 6: Verify Cargo compiles with new dep**

```bash
cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | tail -5
```

Expected: compiles cleanly (serde_yaml downloads and builds).

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/db/migrations/002_accounts.sql \
        src-tauri/Cargo.toml src-tauri/Cargo.lock \
        package.json pnpm-lock.yaml vite.config.ts
git commit -m "feat(db): add accounts migration; add serde_yaml + vitest deps"
```

---

## Task 2: GitHub API Service

**Files:**
- Create: `src-tauri/src/services/github.rs`
- Modify: `src-tauri/src/services/mod.rs`

- [ ] **Step 1: Write the pure-function test (scope parsing)**

Create `src-tauri/src/services/github.rs` with just the test module first:

```rust
// src-tauri/src/services/github.rs
use crate::error::{AppError, AppResult};
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};

// ── Response models ────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GitHubRepo {
    pub name: String,
    pub full_name: String,
    pub owner: GitHubOwner,
    pub private: bool,
    pub html_url: String,
    pub default_branch: String,
    pub pushed_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GitHubOwner {
    pub login: String,
}

#[derive(Debug, Deserialize)]
pub struct GitHubOrg {
    pub login: String,
}

// ── Pure helpers ───────────────────────────────────────────────────────────

/// Extract OAuth scopes from the `X-OAuth-Scopes` response header.
pub fn parse_scopes_header(headers: &HeaderMap) -> Vec<String> {
    headers
        .get("x-oauth-scopes")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').map(|p| p.trim().to_string()).collect())
        .unwrap_or_default()
}

/// Extract rate limit from response headers.
pub fn extract_rate_limit(headers: &HeaderMap) -> Option<RateLimitSnapshot> {
    let remaining = headers.get("x-ratelimit-remaining")?.to_str().ok()?.parse().ok()?;
    let limit     = headers.get("x-ratelimit-limit")?.to_str().ok()?.parse().ok()?;
    let reset_ts: u64 = headers.get("x-ratelimit-reset")?.to_str().ok()?.parse().ok()?;
    // Convert Unix timestamp to ISO 8601
    let reset_at = format_unix_ts(reset_ts);
    Some(RateLimitSnapshot { remaining, limit, reset_at })
}

fn format_unix_ts(ts: u64) -> String {
    // Simple ISO-like formatting without chrono dependency.
    // We store as seconds-since-epoch string and format on the frontend.
    ts.to_string()
}

#[derive(Debug, Clone)]
pub struct RateLimitSnapshot {
    pub remaining: u32,
    pub limit: u32,
    pub reset_at: String,
}

// ── Client ─────────────────────────────────────────────────────────────────

pub struct GitHubClient {
    client: reqwest::Client,
    token:  String,
}

impl GitHubClient {
    pub fn new(token: impl Into<String>) -> Self {
        GitHubClient {
            client: reqwest::Client::new(),
            token:  token.into(),
        }
    }

    fn auth_headers(&self) -> reqwest::header::HeaderMap {
        use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, ACCEPT, USER_AGENT};
        let mut map = HeaderMap::new();
        map.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", self.token)).unwrap());
        map.insert(ACCEPT, HeaderValue::from_static("application/vnd.github+json"));
        map.insert(USER_AGENT, HeaderValue::from_static("git-flotilla/0.1"));
        map.insert("X-GitHub-Api-Version", HeaderValue::from_static("2022-11-28"));
        map
    }

    async fn get_raw(&self, path: &str) -> AppResult<reqwest::Response> {
        let url = format!("https://api.github.com{}", path);
        let resp = self.client.get(&url).headers(self.auth_headers()).send().await?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return match status.as_u16() {
                401 => Err(AppError::Auth(format!("Invalid token: {text}"))),
                403 => Err(AppError::RateLimit(format!("Rate limited or forbidden: {text}"))),
                404 => Err(AppError::NotFound(format!("Not found: {path}"))),
                _   => Err(AppError::GitHub(format!("HTTP {status}: {text}"))),
            };
        }
        Ok(resp)
    }

    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> AppResult<(T, HeaderMap)> {
        let resp    = self.get_raw(path).await?;
        let headers = resp.headers().clone();
        let body    = resp.json::<T>().await?;
        Ok((body, headers))
    }

    // ── Public API calls ─────────────────────────────────────────────────

    /// Fetch authenticated user info and their granted scopes.
    pub async fn get_authenticated_user(&self) -> AppResult<(GitHubUser, Vec<String>, Option<RateLimitSnapshot>)> {
        let (user, headers) = self.get::<GitHubUser>("/user").await?;
        let scopes           = parse_scopes_header(&headers);
        let rate_limit       = extract_rate_limit(&headers);
        Ok((user, scopes, rate_limit))
    }

    /// Fetch all repos accessible to the authenticated user (paginated).
    pub async fn list_all_repos(&self) -> AppResult<(Vec<GitHubRepo>, Option<RateLimitSnapshot>)> {
        let mut all    = Vec::new();
        let mut page   = 1u32;
        let mut last_rl = None;

        loop {
            let path = format!("/user/repos?per_page=100&page={}&type=all&sort=pushed", page);
            let (repos, headers): (Vec<GitHubRepo>, _) = self.get(&path).await?;
            last_rl = extract_rate_limit(&headers);
            let done = repos.len() < 100;
            all.extend(repos);
            if done { break; }
            page += 1;
        }
        Ok((all, last_rl))
    }

    /// Fetch all orgs the authenticated user belongs to.
    pub async fn list_orgs(&self) -> AppResult<Vec<GitHubOrg>> {
        let mut all  = Vec::new();
        let mut page = 1u32;
        loop {
            let path = format!("/user/orgs?per_page=100&page={}", page);
            let (orgs, _): (Vec<GitHubOrg>, _) = self.get(&path).await?;
            let done = orgs.len() < 100;
            all.extend(orgs);
            if done { break; }
            page += 1;
        }
        Ok(all)
    }

    /// Fetch all repos for a given org (paginated).
    pub async fn list_org_repos(&self, org: &str) -> AppResult<Vec<GitHubRepo>> {
        let mut all  = Vec::new();
        let mut page = 1u32;
        loop {
            let path = format!("/orgs/{}/repos?per_page=100&page={}&type=all", org, page);
            let (repos, _): (Vec<GitHubRepo>, _) = self.get(&path).await?;
            let done = repos.len() < 100;
            all.extend(repos);
            if done { break; }
            page += 1;
        }
        Ok(all)
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::{HeaderMap, HeaderValue};

    fn headers_with(entries: &[(&str, &str)]) -> HeaderMap {
        let mut map = HeaderMap::new();
        for (k, v) in entries {
            map.insert(
                reqwest::header::HeaderName::from_bytes(k.as_bytes()).unwrap(),
                HeaderValue::from_str(v).unwrap(),
            );
        }
        map
    }

    #[test]
    fn parse_scopes_comma_separated() {
        let headers = headers_with(&[("x-oauth-scopes", "repo, workflow, read:org")]);
        let scopes  = parse_scopes_header(&headers);
        assert_eq!(scopes, vec!["repo", "workflow", "read:org"]);
    }

    #[test]
    fn parse_scopes_empty_header() {
        let headers = headers_with(&[]);
        let scopes  = parse_scopes_header(&headers);
        assert!(scopes.is_empty());
    }

    #[test]
    fn extract_rate_limit_valid() {
        let headers = headers_with(&[
            ("x-ratelimit-remaining", "4950"),
            ("x-ratelimit-limit",     "5000"),
            ("x-ratelimit-reset",     "1720000000"),
        ]);
        let rl = extract_rate_limit(&headers).expect("should parse");
        assert_eq!(rl.remaining, 4950);
        assert_eq!(rl.limit, 5000);
        assert_eq!(rl.reset_at, "1720000000");
    }

    #[test]
    fn extract_rate_limit_missing_headers() {
        let headers = headers_with(&[]);
        assert!(extract_rate_limit(&headers).is_none());
    }
}
```

- [ ] **Step 2: Run the tests to verify they pass**

```bash
cargo test --manifest-path src-tauri/Cargo.toml services::github 2>&1 | tail -15
```

Expected: 4 tests pass.

- [ ] **Step 3: Expose module in services/mod.rs**

```rust
// src-tauri/src/services/mod.rs
pub mod github;
pub mod rate_limiter;
```

- [ ] **Step 4: Verify compile**

```bash
cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | tail -5
```

Note: `rate_limiter` module doesn't exist yet — create a stub so it compiles:

```rust
// src-tauri/src/services/rate_limiter.rs  (stub — filled in Task 3)
```

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/services/github.rs src-tauri/src/services/mod.rs \
        src-tauri/src/services/rate_limiter.rs
git commit -m "feat(github): add GitHub API client service with scope + rate limit parsing"
```

---

## Task 3: Rate Limiter Service + Settings Command

**Files:**
- Modify: `src-tauri/src/services/rate_limiter.rs` (replace stub)
- Modify: `src-tauri/src/commands/settings.rs`

The `RateLimitInfo` struct is already defined in `commands/settings.rs`. Rate limiter stores snapshots using the same shape and updates whenever a GitHub API response is received.

- [ ] **Step 1: Implement rate_limiter.rs**

```rust
// src-tauri/src/services/rate_limiter.rs
use crate::commands::settings::RateLimitInfo;
use std::sync::{LazyLock, RwLock};

static GITHUB: LazyLock<RwLock<Option<RateLimitInfo>>> =
    LazyLock::new(|| RwLock::new(None));

pub fn update_github(remaining: u32, limit: u32, reset_at: String) {
    *GITHUB.write().unwrap() = Some(RateLimitInfo { remaining, limit, reset_at });
}

pub fn get_github() -> Option<RateLimitInfo> {
    GITHUB.read().unwrap().clone()
}
```

Note: `RateLimitInfo` needs `Clone` — add that derive to `commands/settings.rs`:

In `commands/settings.rs`, change:
```rust
#[derive(Debug, Serialize)]
pub struct RateLimitInfo {
```
to:
```rust
#[derive(Debug, Serialize, Clone)]
pub struct RateLimitInfo {
```

- [ ] **Step 2: Implement get_rate_limit_status command**

In `src-tauri/src/commands/settings.rs`, replace `get_rate_limit_status`:

```rust
#[tauri::command]
pub async fn get_rate_limit_status() -> AppResult<RateLimitStatus> {
    Ok(RateLimitStatus {
        github: crate::services::rate_limiter::get_github(),
        gitlab: None,  // GitLab deferred
    })
}
```

- [ ] **Step 3: Run Rust tests to confirm nothing broke**

```bash
cargo test --manifest-path src-tauri/Cargo.toml 2>&1 | tail -10
```

Expected: same 4 tests pass, 0 failures.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/services/rate_limiter.rs src-tauri/src/commands/settings.rs
git commit -m "feat(settings): implement rate limit tracker and get_rate_limit_status command"
```

---

## Task 4: Auth Commands (keychain + GitHub validation)

**Files:**
- Modify: `src-tauri/src/error.rs` (add keyring From impl)
- Modify: `src-tauri/src/commands/auth.rs` (full implementation)

Required scopes for GitHub: `repo`, `workflow`, `read:org`. Missing scopes become warnings in the returned `AccountInfo` (not errors — let the user decide to proceed).

- [ ] **Step 1: Add keyring error conversion to error.rs**

Add at the end of `src-tauri/src/error.rs`, before the `pub type AppResult` line:

```rust
impl From<keyring::Error> for AppError {
    fn from(e: keyring::Error) -> Self {
        AppError::Keychain(e.to_string())
    }
}
```

- [ ] **Step 2: Implement auth commands**

Replace the entire content of `src-tauri/src/commands/auth.rs`:

```rust
use crate::{
    db,
    error::{AppError, AppResult},
    services::github::GitHubClient,
};
use keyring::Entry;
use serde::{Deserialize, Serialize};

const KEYCHAIN_SERVICE: &str = "git-flotilla";

/// Info about a connected account returned to the frontend.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    pub id:         String,  // "github:{username}"
    pub provider:   String,
    pub username:   String,
    pub avatar_url: Option<String>,
    pub scopes:     Vec<String>,
    /// Scopes that are required but missing — warning, not error.
    pub missing_scopes: Vec<String>,
}

const REQUIRED_GITHUB_SCOPES: &[&str] = &["repo", "workflow", "read:org"];

fn account_id(provider: &str, username: &str) -> String {
    format!("{}:{}", provider, username)
}

fn keychain_entry(id: &str) -> AppResult<Entry> {
    Entry::new(KEYCHAIN_SERVICE, id).map_err(AppError::from)
}

/// Validate a token against the GitHub API without storing it.
#[tauri::command]
pub async fn validate_token(provider: String, token: String) -> AppResult<AccountInfo> {
    if provider != "github" {
        return Err(AppError::InvalidInput(format!("Unsupported provider: {provider}")));
    }

    let client = GitHubClient::new(&token);
    let (user, scopes, rate_limit) = client.get_authenticated_user().await?;

    // Update global rate limit state
    if let Some(rl) = rate_limit {
        crate::services::rate_limiter::update_github(rl.remaining, rl.limit, rl.reset_at);
    }

    let missing_scopes = REQUIRED_GITHUB_SCOPES
        .iter()
        .filter(|s| !scopes.contains(&s.to_string()))
        .map(|s| s.to_string())
        .collect();

    Ok(AccountInfo {
        id:             account_id(&provider, &user.login),
        provider,
        username:       user.login,
        avatar_url:     user.avatar_url,
        scopes,
        missing_scopes,
    })
}

/// Validate, store token in keychain, and persist account record in SQLite.
#[tauri::command]
pub async fn add_account(provider: String, token: String) -> AppResult<AccountInfo> {
    let info = validate_token(provider.clone(), token.clone()).await?;

    // Store token in OS keychain
    keychain_entry(&info.id)?.set_password(&token).map_err(AppError::from)?;

    // Upsert account record (no token) into SQLite
    let pool = db::pool();
    sqlx::query!(
        r#"
        INSERT INTO accounts (id, provider, username, avatar_url)
        VALUES (?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
            username   = excluded.username,
            avatar_url = excluded.avatar_url
        "#,
        info.id,
        info.provider,
        info.username,
        info.avatar_url,
    )
    .execute(pool)
    .await?;

    // Audit log
    sqlx::query!(
        "INSERT INTO audit_log (action_type, repo_ids, outcome, detail) VALUES (?, '[]', 'success', ?)",
        "account_added",
        info.id,
    )
    .execute(pool)
    .await?;

    tracing::info!("Account added: {}", info.id);
    Ok(info)
}

/// Remove an account: delete from keychain and SQLite.
#[tauri::command]
pub async fn remove_account(id: String) -> AppResult<()> {
    // Remove from keychain (ignore error if already gone)
    if let Ok(entry) = keychain_entry(&id) {
        let _ = entry.delete_credential();
    }

    let pool = db::pool();
    sqlx::query!("DELETE FROM accounts WHERE id = ?", id)
        .execute(pool)
        .await?;

    sqlx::query!(
        "INSERT INTO audit_log (action_type, repo_ids, outcome, detail) VALUES (?, '[]', 'success', ?)",
        "account_removed",
        id,
    )
    .execute(pool)
    .await?;

    tracing::info!("Account removed: {id}");
    Ok(())
}

/// List all accounts stored in SQLite. Filters out any whose keychain entry is missing.
#[tauri::command]
pub async fn list_accounts() -> AppResult<Vec<AccountInfo>> {
    let pool = db::pool();

    struct Row {
        id:         String,
        provider:   String,
        username:   String,
        avatar_url: Option<String>,
    }

    let rows = sqlx::query_as!(
        Row,
        "SELECT id, provider, username, avatar_url FROM accounts ORDER BY added_at"
    )
    .fetch_all(pool)
    .await?;

    let mut accounts = Vec::new();
    for row in rows {
        // Verify keychain entry still exists
        match keychain_entry(&row.id).and_then(|e| e.get_password().map_err(AppError::from)) {
            Ok(_) => accounts.push(AccountInfo {
                id:             row.id,
                provider:       row.provider,
                username:       row.username,
                avatar_url:     row.avatar_url,
                scopes:         vec![],         // not re-validated on list
                missing_scopes: vec![],
            }),
            Err(_) => {
                // Keychain entry gone — clean up SQLite too
                tracing::warn!("Keychain entry missing for {}, removing DB record", row.id);
                let _ = sqlx::query!("DELETE FROM accounts WHERE id = ?", row.id)
                    .execute(pool)
                    .await;
            }
        }
    }

    Ok(accounts)
}
```

- [ ] **Step 3: Verify compile**

```bash
cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | tail -10
```

Expected: compiles cleanly.

- [ ] **Step 4: Run all Rust tests**

```bash
cargo test --manifest-path src-tauri/Cargo.toml 2>&1 | tail -10
```

Expected: 4 passing, 0 failures.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/error.rs src-tauri/src/commands/auth.rs
git commit -m "feat(auth): implement GitHub PAT auth with keychain storage and scope validation"
```

---

## Task 5: Repo Discovery + Tag Commands

**Files:**
- Modify: `src-tauri/src/commands/repos.rs`

`discover_repos` fetches user repos + repos in all orgs the user belongs to, deduplicates by `full_name`, and upserts into SQLite. `set_repo_tags` updates the JSON tags array on a repo.

- [ ] **Step 1: Get account token helper (local fn)**

The pattern: read token from keychain given an account ID, create `GitHubClient`. This is used by both `discover_repos` and future scan commands.

- [ ] **Step 2: Implement full repos.rs**

Replace the entire content of `src-tauri/src/commands/repos.rs`:

```rust
use crate::{
    db,
    error::{AppError, AppResult},
    models::{Repo, RepoList},
    services::github::{GitHubClient, GitHubRepo},
};
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

const KEYCHAIN_SERVICE: &str = "git-flotilla";

fn get_token(account_id: &str) -> AppResult<String> {
    Entry::new(KEYCHAIN_SERVICE, account_id)
        .map_err(AppError::from)?
        .get_password()
        .map_err(AppError::from)
}

fn github_repo_to_model(r: &GitHubRepo) -> Repo {
    Repo {
        id:             format!("github:{}", r.full_name),
        provider:       "github".to_string(),
        owner:          r.owner.login.clone(),
        name:           r.name.clone(),
        full_name:      r.full_name.clone(),
        url:            r.html_url.clone(),
        default_branch: r.default_branch.clone(),
        is_private:     r.private,
        last_scanned_at: None,
        tags:           vec![],
    }
}

// ── Repo discovery ─────────────────────────────────────────────────────────

/// Discover all repos accessible to the given account and upsert into SQLite.
#[tauri::command]
pub async fn discover_repos(account_id: String) -> AppResult<Vec<Repo>> {
    let token  = get_token(&account_id)?;
    let client = GitHubClient::new(&token);

    // Fetch user repos + org repos
    let (user_repos, rate_limit) = client.list_all_repos().await?;
    if let Some(rl) = rate_limit {
        crate::services::rate_limiter::update_github(rl.remaining, rl.limit, rl.reset_at);
    }

    let orgs = client.list_orgs().await?;
    let mut seen: HashSet<String> = HashSet::new();
    let mut all_gh: Vec<GitHubRepo> = Vec::new();

    for r in user_repos {
        if seen.insert(r.full_name.clone()) {
            all_gh.push(r);
        }
    }
    for org in &orgs {
        let org_repos = client.list_org_repos(&org.login).await?;
        for r in org_repos {
            if seen.insert(r.full_name.clone()) {
                all_gh.push(r);
            }
        }
    }

    let pool = db::pool();
    let mut result = Vec::new();

    for gh in &all_gh {
        let repo = github_repo_to_model(gh);

        // Upsert — preserve existing tags and last_scanned_at
        sqlx::query!(
            r#"
            INSERT INTO repos (id, provider, owner, name, full_name, url, default_branch, is_private)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                default_branch = excluded.default_branch,
                url            = excluded.url,
                updated_at     = datetime('now')
            "#,
            repo.id, repo.provider, repo.owner, repo.name,
            repo.full_name, repo.url, repo.default_branch,
            repo.is_private,
        )
        .execute(pool)
        .await?;

        result.push(repo);
    }

    tracing::info!("Discovered {} repos for {}", result.len(), account_id);
    Ok(result)
}

/// List repos from SQLite, optionally filtered by repo list membership.
#[tauri::command]
pub async fn list_repos(repo_list_id: Option<String>) -> AppResult<Vec<Repo>> {
    let pool = db::pool();

    struct Row {
        id:             String,
        provider:       String,
        owner:          String,
        name:           String,
        full_name:      String,
        url:            String,
        default_branch: String,
        is_private:     i64,
        last_scanned_at: Option<String>,
        tags:           String,
    }

    let rows = if let Some(list_id) = repo_list_id {
        sqlx::query_as!(
            Row,
            r#"
            SELECT r.id, r.provider, r.owner, r.name, r.full_name, r.url,
                   r.default_branch, r.is_private, r.last_scanned_at, r.tags
            FROM repos r
            JOIN repo_list_members m ON m.repo_id = r.id
            WHERE m.list_id = ?
            ORDER BY r.full_name
            "#,
            list_id,
        )
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as!(
            Row,
            "SELECT id, provider, owner, name, full_name, url, default_branch,
                    is_private, last_scanned_at, tags FROM repos ORDER BY full_name"
        )
        .fetch_all(pool)
        .await?
    };

    let repos = rows
        .into_iter()
        .map(|r| Repo {
            id:             r.id,
            provider:       r.provider,
            owner:          r.owner,
            name:           r.name,
            full_name:      r.full_name,
            url:            r.url,
            default_branch: r.default_branch,
            is_private:     r.is_private != 0,
            last_scanned_at: r.last_scanned_at,
            tags:           serde_json::from_str(&r.tags).unwrap_or_default(),
        })
        .collect();

    Ok(repos)
}

/// Fetch a single repo by ID.
#[tauri::command]
pub async fn get_repo(id: String) -> AppResult<Repo> {
    let pool = db::pool();

    struct Row {
        id:             String,
        provider:       String,
        owner:          String,
        name:           String,
        full_name:      String,
        url:            String,
        default_branch: String,
        is_private:     i64,
        last_scanned_at: Option<String>,
        tags:           String,
    }

    let row = sqlx::query_as!(
        Row,
        "SELECT id, provider, owner, name, full_name, url, default_branch,
                is_private, last_scanned_at, tags FROM repos WHERE id = ?",
        id,
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Repo not found: {id}")))?;

    Ok(Repo {
        id:             row.id,
        provider:       row.provider,
        owner:          row.owner,
        name:           row.name,
        full_name:      row.full_name,
        url:            row.url,
        default_branch: row.default_branch,
        is_private:     row.is_private != 0,
        last_scanned_at: row.last_scanned_at,
        tags:           serde_json::from_str(&row.tags).unwrap_or_default(),
    })
}

/// Update the tags JSON array for a repo.
#[tauri::command]
pub async fn set_repo_tags(repo_id: String, tags: Vec<String>) -> AppResult<Repo> {
    let pool      = db::pool();
    let tags_json = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string());

    sqlx::query!(
        "UPDATE repos SET tags = ?, updated_at = datetime('now') WHERE id = ?",
        tags_json,
        repo_id,
    )
    .execute(pool)
    .await?;

    get_repo(repo_id).await
}

// ── Repo lists ─────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRepoListInput {
    pub name:             String,
    pub description:      String,
    pub parent_id:        Option<String>,
    pub exclude_patterns: Vec<String>,
}

fn new_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    format!("{:x}", ts)
}

fn row_to_repo_list(
    id: String, name: String, description: String,
    parent_id: Option<String>, exclude_patterns: String,
    created_at: String, updated_at: String,
    repo_ids: Vec<String>,
) -> RepoList {
    RepoList {
        id,
        name,
        description,
        repo_ids,
        parent_id,
        exclude_patterns: serde_json::from_str(&exclude_patterns).unwrap_or_default(),
        created_at,
        updated_at,
    }
}

async fn fetch_repo_ids_for_list(pool: &sqlx::SqlitePool, list_id: &str) -> AppResult<Vec<String>> {
    let rows = sqlx::query!(
        "SELECT repo_id FROM repo_list_members WHERE list_id = ? ORDER BY added_at",
        list_id,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| r.repo_id).collect())
}

#[tauri::command]
pub async fn create_repo_list(input: CreateRepoListInput) -> AppResult<RepoList> {
    let pool     = db::pool();
    let id       = new_id();
    let patterns = serde_json::to_string(&input.exclude_patterns).unwrap_or_else(|_| "[]".into());
    let now      = chrono_now();

    sqlx::query!(
        r#"
        INSERT INTO repo_lists (id, name, description, parent_id, exclude_patterns, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
        id, input.name, input.description, input.parent_id, patterns, now, now,
    )
    .execute(pool)
    .await?;

    Ok(row_to_repo_list(id, input.name, input.description, input.parent_id, patterns, now.clone(), now, vec![]))
}

#[tauri::command]
pub async fn update_repo_list(id: String, input: CreateRepoListInput) -> AppResult<RepoList> {
    let pool     = db::pool();
    let patterns = serde_json::to_string(&input.exclude_patterns).unwrap_or_else(|_| "[]".into());
    let now      = chrono_now();

    sqlx::query!(
        "UPDATE repo_lists SET name=?, description=?, parent_id=?, exclude_patterns=?, updated_at=? WHERE id=?",
        input.name, input.description, input.parent_id, patterns, now, id,
    )
    .execute(pool)
    .await?;

    list_repo_list_by_id(&id).await
}

#[tauri::command]
pub async fn delete_repo_list(id: String) -> AppResult<()> {
    let pool = db::pool();
    // repo_list_members cascade-deletes on FK
    sqlx::query!("DELETE FROM repo_lists WHERE id = ?", id)
        .execute(pool)
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn list_repo_lists() -> AppResult<Vec<RepoList>> {
    let pool = db::pool();

    struct Row {
        id:               String,
        name:             String,
        description:      String,
        parent_id:        Option<String>,
        exclude_patterns: String,
        created_at:       String,
        updated_at:       String,
    }

    let rows = sqlx::query_as!(
        Row,
        "SELECT id, name, description, parent_id, exclude_patterns, created_at, updated_at
         FROM repo_lists ORDER BY name"
    )
    .fetch_all(pool)
    .await?;

    let mut lists = Vec::new();
    for row in rows {
        let repo_ids = fetch_repo_ids_for_list(pool, &row.id).await?;
        lists.push(row_to_repo_list(
            row.id, row.name, row.description, row.parent_id,
            row.exclude_patterns, row.created_at, row.updated_at, repo_ids,
        ));
    }
    Ok(lists)
}

async fn list_repo_list_by_id(id: &str) -> AppResult<RepoList> {
    let pool = db::pool();

    struct Row {
        id:               String,
        name:             String,
        description:      String,
        parent_id:        Option<String>,
        exclude_patterns: String,
        created_at:       String,
        updated_at:       String,
    }

    let row = sqlx::query_as!(
        Row,
        "SELECT id, name, description, parent_id, exclude_patterns, created_at, updated_at
         FROM repo_lists WHERE id = ?",
        id,
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Repo list not found: {id}")))?;

    let repo_ids = fetch_repo_ids_for_list(pool, &row.id).await?;
    Ok(row_to_repo_list(
        row.id, row.name, row.description, row.parent_id,
        row.exclude_patterns, row.created_at, row.updated_at, repo_ids,
    ))
}

#[tauri::command]
pub async fn add_repos_to_list(list_id: String, repo_ids: Vec<String>) -> AppResult<()> {
    let pool = db::pool();
    for repo_id in &repo_ids {
        sqlx::query!(
            "INSERT OR IGNORE INTO repo_list_members (list_id, repo_id) VALUES (?, ?)",
            list_id, repo_id,
        )
        .execute(pool)
        .await?;
    }
    Ok(())
}

#[tauri::command]
pub async fn remove_repos_from_list(list_id: String, repo_ids: Vec<String>) -> AppResult<()> {
    let pool = db::pool();
    for repo_id in &repo_ids {
        sqlx::query!(
            "DELETE FROM repo_list_members WHERE list_id = ? AND repo_id = ?",
            list_id, repo_id,
        )
        .execute(pool)
        .await?;
    }
    Ok(())
}

// ── YAML export / import ───────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RepoListYaml {
    id:               String,
    name:             String,
    description:      String,
    parent_id:        Option<String>,
    exclude_patterns: Vec<String>,
    repo_ids:         Vec<String>,
}

#[tauri::command]
pub async fn export_repo_list(id: String) -> AppResult<String> {
    let list = list_repo_list_by_id(&id).await?;
    let export = RepoListYaml {
        id:               list.id,
        name:             list.name,
        description:      list.description,
        parent_id:        list.parent_id,
        exclude_patterns: list.exclude_patterns,
        repo_ids:         list.repo_ids,
    };
    serde_yaml::to_string(&export)
        .map_err(|e| AppError::Operation(format!("YAML serialisation error: {e}")))
}

#[tauri::command]
pub async fn import_repo_list(yaml: String) -> AppResult<RepoList> {
    let parsed: RepoListYaml = serde_yaml::from_str(&yaml)
        .map_err(|e| AppError::InvalidInput(format!("Invalid YAML: {e}")))?;

    let pool     = db::pool();
    let patterns = serde_json::to_string(&parsed.exclude_patterns).unwrap_or_else(|_| "[]".into());
    let now      = chrono_now();

    // Upsert list
    sqlx::query!(
        r#"
        INSERT INTO repo_lists (id, name, description, parent_id, exclude_patterns, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
            name             = excluded.name,
            description      = excluded.description,
            parent_id        = excluded.parent_id,
            exclude_patterns = excluded.exclude_patterns,
            updated_at       = excluded.updated_at
        "#,
        parsed.id, parsed.name, parsed.description, parsed.parent_id, patterns, now, now,
    )
    .execute(pool)
    .await?;

    // Upsert member repos that exist in the repos table
    for repo_id in &parsed.repo_ids {
        let exists: bool = sqlx::query_scalar!("SELECT COUNT(*) FROM repos WHERE id = ?", repo_id)
            .fetch_one(pool)
            .await? > 0;
        if exists {
            sqlx::query!(
                "INSERT OR IGNORE INTO repo_list_members (list_id, repo_id) VALUES (?, ?)",
                parsed.id, repo_id,
            )
            .execute(pool)
            .await?;
        }
    }

    list_repo_list_by_id(&parsed.id).await
}

// ── Utility ────────────────────────────────────────────────────────────────

fn chrono_now() -> String {
    // Returns current UTC time as ISO 8601 string using std only.
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    // SQLite datetime format: "YYYY-MM-DD HH:MM:SS"
    // We produce a simple representation; precise formatting is handled by SQLite's datetime('now')
    // For returned values, we rely on SQLite's default which is ISO-8601-like.
    format!("{}", secs)  // epoch seconds — frontend formats display
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yaml_round_trip() {
        let original = RepoListYaml {
            id:               "test-id".to_string(),
            name:             "Test List".to_string(),
            description:      "A test list".to_string(),
            parent_id:        None,
            exclude_patterns: vec!["org/legacy-*".to_string()],
            repo_ids:         vec!["github:org/repo-a".to_string(), "github:org/repo-b".to_string()],
        };

        let yaml    = serde_yaml::to_string(&original).unwrap();
        let parsed: RepoListYaml = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(parsed.id,   original.id);
        assert_eq!(parsed.name, original.name);
        assert_eq!(parsed.repo_ids.len(), 2);
        assert_eq!(parsed.exclude_patterns, vec!["org/legacy-*"]);
    }

    #[test]
    fn yaml_rejects_invalid_input() {
        let result: Result<RepoListYaml, _> = serde_yaml::from_str("not: valid: yaml: [[[");
        assert!(result.is_err());
    }

    #[test]
    fn new_id_is_nonempty() {
        let id = new_id();
        assert!(!id.is_empty());
    }
}
```

- [ ] **Step 3: Run Rust tests**

```bash
cargo test --manifest-path src-tauri/Cargo.toml 2>&1 | tail -15
```

Expected: 7 tests pass (4 from github.rs + 3 new).

- [ ] **Step 4: Verify compile**

```bash
cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | tail -5
```

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/repos.rs
git commit -m "feat(repos): implement repo discovery, list CRUD, tag management, YAML export/import"
```

---

## Task 6: Wire New Commands Into main.rs

**Files:**
- Modify: `src-tauri/src/main.rs`

Three new commands need to be added to the `invoke_handler!` macro: `set_repo_tags`, `export_repo_list`, `import_repo_list`.

- [ ] **Step 1: Add commands to invoke_handler**

In `src-tauri/src/main.rs`, find the `// Repo lists` comment block and add 3 new entries after `remove_repos_from_list`:

```rust
            git_flotilla::commands::repos::set_repo_tags,
            git_flotilla::commands::repos::export_repo_list,
            git_flotilla::commands::repos::import_repo_list,
```

The Repos section of the handler should now read:
```rust
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
```

- [ ] **Step 2: Clippy and build**

```bash
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings 2>&1 | tail -15
```

Fix any warnings. Common issue: unused imports after the refactor — remove them.

- [ ] **Step 3: Full Rust test run**

```bash
cargo test --manifest-path src-tauri/Cargo.toml 2>&1 | tail -10
```

Expected: 7 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/main.rs
git commit -m "feat(backend): register set_repo_tags, export_repo_list, import_repo_list commands"
```

---

## Task 7: Auth Store + Vitest Setup

**Files:**
- Modify: `src/stores/auth.ts`
- Create: `src/stores/__tests__/auth.spec.ts`
- Create: `src/services/settings.ts`

- [ ] **Step 1: Write the failing auth store test**

Create `src/stores/__tests__/auth.spec.ts`:

```typescript
import { setActivePinia, createPinia } from 'pinia'
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useAuthStore } from '@/stores/auth'
import * as authService from '@/services/auth'

vi.mock('@/services/auth')

describe('auth store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('starts with no accounts', () => {
    const store = useAuthStore()
    expect(store.accounts).toHaveLength(0)
    expect(store.hasAccounts).toBe(false)
  })

  it('loadAccounts populates accounts', async () => {
    vi.mocked(authService.listAccounts).mockResolvedValue([
      { id: 'github:octocat', provider: 'github', username: 'octocat', scopes: ['repo'], missingScopes: [], avatarUrl: null },
    ])
    const store = useAuthStore()
    await store.loadAccounts()
    expect(store.accounts).toHaveLength(1)
    expect(store.hasAccounts).toBe(true)
    expect(store.githubAccount?.username).toBe('octocat')
  })

  it('addAccountAction calls addAccount service and appends to accounts', async () => {
    const newAccount = { id: 'github:octocat', provider: 'github', username: 'octocat', scopes: ['repo', 'workflow'], missingScopes: [], avatarUrl: null }
    vi.mocked(authService.addAccount).mockResolvedValue(newAccount)
    const store = useAuthStore()
    await store.addAccountAction('github', 'ghp_test123')
    expect(authService.addAccount).toHaveBeenCalledWith('github', 'ghp_test123')
    expect(store.accounts).toHaveLength(1)
  })

  it('removeAccountAction calls removeAccount and removes from list', async () => {
    vi.mocked(authService.listAccounts).mockResolvedValue([
      { id: 'github:octocat', provider: 'github', username: 'octocat', scopes: [], missingScopes: [], avatarUrl: null },
    ])
    vi.mocked(authService.removeAccount).mockResolvedValue(undefined)
    const store = useAuthStore()
    await store.loadAccounts()
    expect(store.accounts).toHaveLength(1)
    await store.removeAccountAction('github:octocat')
    expect(store.accounts).toHaveLength(0)
  })

  it('sets isLoading true while loading', async () => {
    let resolve!: () => void
    vi.mocked(authService.listAccounts).mockReturnValue(new Promise(r => { resolve = () => r([]) }))
    const store = useAuthStore()
    const p = store.loadAccounts()
    expect(store.isLoading).toBe(true)
    resolve()
    await p
    expect(store.isLoading).toBe(false)
  })
})
```

- [ ] **Step 2: Run the test to see it fail**

```bash
cd /var/www/vhosts/git-flotilla
pnpm test 2>&1 | tail -20
```

Expected: FAIL — `loadAccounts is not a function` (action not implemented yet).

- [ ] **Step 3: Update AccountInfo type in services/auth.ts to match Rust model**

The Rust `AccountInfo` now uses camelCase. Update `src/services/auth.ts`:

```typescript
import { invoke } from '@tauri-apps/api/core'

export interface AccountInfo {
  id: string
  provider: string
  username: string
  avatarUrl: string | null
  scopes: string[]
  missingScopes: string[]
}

export function addAccount(provider: string, token: string): Promise<AccountInfo> {
  return invoke('add_account', { provider, token })
}

export function removeAccount(id: string): Promise<void> {
  return invoke('remove_account', { id })
}

export function listAccounts(): Promise<AccountInfo[]> {
  return invoke('list_accounts')
}

export function validateToken(provider: string, token: string): Promise<AccountInfo> {
  return invoke('validate_token', { provider, token })
}
```

- [ ] **Step 4: Implement auth store actions**

Replace the entire content of `src/stores/auth.ts`:

```typescript
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { AccountInfo } from '@/services/auth'
import { listAccounts, addAccount, removeAccount } from '@/services/auth'

export const useAuthStore = defineStore('auth', () => {
  const accounts  = ref<AccountInfo[]>([])
  const isLoading = ref(false)
  const error     = ref<string | null>(null)

  const hasAccounts   = computed(() => accounts.value.length > 0)
  const githubAccount = computed(() => accounts.value.find(a => a.provider === 'github') ?? null)
  const gitlabAccount = computed(() => accounts.value.find(a => a.provider === 'gitlab') ?? null)

  async function loadAccounts() {
    isLoading.value = true
    error.value = null
    try {
      accounts.value = await listAccounts()
    } catch (e) {
      error.value = String(e)
    } finally {
      isLoading.value = false
    }
  }

  async function addAccountAction(provider: string, token: string) {
    const account = await addAccount(provider, token)
    accounts.value.push(account)
    return account
  }

  async function removeAccountAction(id: string) {
    await removeAccount(id)
    accounts.value = accounts.value.filter(a => a.id !== id)
  }

  return {
    accounts, isLoading, error,
    hasAccounts, githubAccount, gitlabAccount,
    loadAccounts, addAccountAction, removeAccountAction,
  }
})
```

- [ ] **Step 5: Run the tests to verify they pass**

```bash
pnpm test 2>&1 | tail -20
```

Expected: 5 tests pass.

- [ ] **Step 6: Create settings service**

Create `src/services/settings.ts`:

```typescript
import { invoke } from '@tauri-apps/api/core'
import type { RateLimitInfo } from '@/stores/settings'

export interface RateLimitStatus {
  github: RateLimitInfo | null
  gitlab: RateLimitInfo | null
}

export function getRateLimitStatus(): Promise<RateLimitStatus> {
  return invoke('get_rate_limit_status')
}
```

- [ ] **Step 7: Add refreshRateLimit action to settings store**

In `src/stores/settings.ts`, add import and action:

```typescript
import { defineStore } from 'pinia'
import { ref } from 'vue'
import { getRateLimitStatus } from '@/services/settings'

export interface RateLimitInfo {
  remaining: number
  limit: number
  resetAt: string
}

export const useSettingsStore = defineStore('settings', () => {
  const scanIntervalMinutes    = ref<number | null>(1440)
  const cvePollIntervalMinutes = ref<number | null>(60)
  const parallelWorkers        = ref(5)
  const requestDelayMs         = ref(200)
  const darkMode               = ref(true)
  const rateLimitGithub        = ref<RateLimitInfo | null>(null)
  const rateLimitGitlab        = ref<RateLimitInfo | null>(null)

  async function refreshRateLimit() {
    try {
      const status = await getRateLimitStatus()
      rateLimitGithub.value = status.github
      rateLimitGitlab.value = status.gitlab
    } catch {
      // non-fatal — rate limit display is informational
    }
  }

  return {
    scanIntervalMinutes, cvePollIntervalMinutes,
    parallelWorkers, requestDelayMs, darkMode,
    rateLimitGithub, rateLimitGitlab,
    refreshRateLimit,
  }
})
```

- [ ] **Step 8: Run pnpm typecheck**

```bash
pnpm typecheck 2>&1 | tail -10
```

Expected: no errors.

- [ ] **Step 9: Commit**

```bash
git add src/stores/auth.ts src/stores/__tests__/auth.spec.ts \
        src/services/auth.ts src/services/settings.ts src/stores/settings.ts
git commit -m "feat(auth): implement auth store actions and AccountInfo types"
```

---

## Task 8: Auth.vue — Account Management UI

**Files:**
- Modify: `src/views/Auth.vue`

The UI has two sections:
1. **Add account** — provider select, token input (password type), Validate button → shows preview → Add button
2. **Connected accounts** — list cards with username, scopes, missing-scope warning, Remove button

- [ ] **Step 1: Implement Auth.vue**

Replace entire content of `src/views/Auth.vue`:

```vue
<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useAuthStore } from '@/stores/auth'
import { validateToken } from '@/services/auth'
import type { AccountInfo } from '@/services/auth'
import Button from '@/components/ui/Button.vue'
import Input from '@/components/ui/Input.vue'

const authStore = useAuthStore()

onMounted(() => authStore.loadAccounts())

// ── Add account form ─────────────────────────────────────────────────────
const provider  = ref<'github' | 'gitlab'>('github')
const token     = ref('')
const preview   = ref<AccountInfo | null>(null)
const validating = ref(false)
const adding    = ref(false)
const formError = ref<string | null>(null)

async function handleValidate() {
  if (!token.value.trim()) return
  validating.value = true
  formError.value  = null
  preview.value    = null
  try {
    preview.value = await validateToken(provider.value, token.value.trim())
  } catch (e) {
    formError.value = String(e)
  } finally {
    validating.value = false
  }
}

async function handleAdd() {
  if (!preview.value) return
  adding.value    = true
  formError.value = null
  try {
    await authStore.addAccountAction(provider.value, token.value.trim())
    token.value   = ''
    preview.value = null
  } catch (e) {
    formError.value = String(e)
  } finally {
    adding.value = false
  }
}

async function handleRemove(id: string) {
  try {
    await authStore.removeAccountAction(id)
  } catch (e) {
    formError.value = String(e)
  }
}

const REQUIRED_SCOPES: Record<string, string[]> = {
  github: ['repo', 'workflow', 'read:org'],
}
</script>

<template>
  <div class="p-6 max-w-2xl space-y-8">
    <div>
      <h1 class="text-2xl font-semibold mb-1">Accounts</h1>
      <p class="text-muted text-sm">Connect GitHub and GitLab accounts via Personal Access Token.</p>
    </div>

    <!-- Connected accounts list -->
    <section v-if="authStore.accounts.length > 0">
      <h2 class="text-sm font-semibold text-muted uppercase tracking-wider mb-3">Connected</h2>
      <div class="space-y-3">
        <div
          v-for="account in authStore.accounts"
          :key="account.id"
          class="bg-surface-alt border border-border rounded-lg p-4 flex items-center gap-4"
        >
          <div class="w-8 h-8 rounded-full bg-primary/20 flex items-center justify-center flex-shrink-0">
            <span class="text-primary text-sm font-bold">{{ account.username[0].toUpperCase() }}</span>
          </div>
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2">
              <span class="font-medium text-sm">{{ account.username }}</span>
              <span class="text-xs text-muted px-1.5 py-0.5 bg-surface rounded border border-border capitalize">
                {{ account.provider }}
              </span>
            </div>
            <!-- Missing scopes warning -->
            <div v-if="account.missingScopes.length > 0" class="mt-1 text-xs text-warning">
              Missing scopes: {{ account.missingScopes.join(', ') }}
            </div>
          </div>
          <Button variant="danger" size="sm" @click="handleRemove(account.id)">Remove</Button>
        </div>
      </div>
    </section>

    <div v-else-if="authStore.isLoading" class="text-muted text-sm">Loading accounts…</div>

    <!-- Add account form -->
    <section class="bg-surface-alt border border-border rounded-lg p-6 space-y-4">
      <h2 class="text-base font-semibold">Add Account</h2>

      <div class="flex gap-2">
        <button
          class="flex-1 py-1.5 text-sm rounded-md border transition-colors"
          :class="provider === 'github'
            ? 'border-primary bg-primary/10 text-primary'
            : 'border-border text-muted hover:border-primary/50'"
          @click="provider = 'github'"
        >GitHub</button>
        <button
          class="flex-1 py-1.5 text-sm rounded-md border transition-colors opacity-50 cursor-not-allowed"
          :class="provider === 'gitlab'
            ? 'border-primary bg-primary/10 text-primary'
            : 'border-border text-muted'"
          disabled
          title="GitLab support coming soon"
        >GitLab</button>
      </div>

      <div>
        <label class="block text-sm text-muted mb-1">Personal Access Token</label>
        <Input
          v-model="token"
          type="password"
          placeholder="ghp_xxxxxxxxxxxxxxxx"
          :error="formError ?? undefined"
        />
        <p class="text-xs text-muted mt-1">
          Required scopes: {{ REQUIRED_SCOPES[provider]?.join(', ') ?? 'none' }}
        </p>
      </div>

      <!-- Validation preview -->
      <div v-if="preview" class="bg-surface rounded-md border border-border p-3 space-y-1">
        <div class="flex items-center gap-2">
          <div class="w-2 h-2 rounded-full bg-success" />
          <span class="text-sm font-medium">{{ preview.username }}</span>
        </div>
        <div class="text-xs text-muted">
          Scopes: {{ preview.scopes.join(', ') || 'none' }}
        </div>
        <div v-if="preview.missingScopes.length > 0" class="text-xs text-warning">
          Warning: missing {{ preview.missingScopes.join(', ') }} — some features may not work
        </div>
      </div>

      <div class="flex gap-3">
        <Button
          variant="secondary"
          :loading="validating"
          :disabled="!token.trim() || validating"
          @click="handleValidate"
        >
          Validate
        </Button>
        <Button
          variant="primary"
          :loading="adding"
          :disabled="!preview || adding"
          @click="handleAdd"
        >
          Add Account
        </Button>
      </div>
    </section>
  </div>
</template>
```

- [ ] **Step 2: Run pnpm typecheck**

```bash
pnpm typecheck 2>&1 | tail -10
```

Expected: no errors.

- [ ] **Step 3: Run pnpm build**

```bash
pnpm build 2>&1 | tail -15
```

Expected: build succeeds.

- [ ] **Step 4: Commit**

```bash
git add src/views/Auth.vue
git commit -m "feat(ui): implement Auth.vue account management view"
```

---

## Task 9: Repos + RepoLists Stores

**Files:**
- Modify: `src/stores/repos.ts`
- Modify: `src/stores/repoLists.ts`
- Modify: `src/services/repos.ts`
- Create: `src/stores/__tests__/repos.spec.ts`

- [ ] **Step 1: Write the failing repos store test**

Create `src/stores/__tests__/repos.spec.ts`:

```typescript
import { setActivePinia, createPinia } from 'pinia'
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useReposStore } from '@/stores/repos'
import * as reposService from '@/services/repos'

vi.mock('@/services/repos')

describe('repos store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('starts empty', () => {
    const store = useReposStore()
    expect(store.repos).toHaveLength(0)
  })

  it('loadRepos fetches and stores repos', async () => {
    vi.mocked(reposService.listRepos).mockResolvedValue([
      { id: 'github:org/repo-a', provider: 'github', owner: 'org', name: 'repo-a',
        fullName: 'org/repo-a', url: 'https://github.com/org/repo-a',
        defaultBranch: 'main', isPrivate: false, lastScannedAt: null, tags: [] },
    ])
    const store = useReposStore()
    await store.loadRepos()
    expect(store.repos).toHaveLength(1)
    expect(store.repos[0].fullName).toBe('org/repo-a')
  })

  it('filteredRepos filters by search query', async () => {
    vi.mocked(reposService.listRepos).mockResolvedValue([
      { id: 'github:org/frontend', provider: 'github', owner: 'org', name: 'frontend',
        fullName: 'org/frontend', url: '', defaultBranch: 'main', isPrivate: false, lastScannedAt: null, tags: ['vue'] },
      { id: 'github:org/backend', provider: 'github', owner: 'org', name: 'backend',
        fullName: 'org/backend', url: '', defaultBranch: 'main', isPrivate: false, lastScannedAt: null, tags: [] },
    ])
    const store = useReposStore()
    await store.loadRepos()
    store.searchQuery = 'front'
    expect(store.filteredRepos).toHaveLength(1)
    expect(store.filteredRepos[0].name).toBe('frontend')
  })
})
```

- [ ] **Step 2: Run test to confirm failure**

```bash
pnpm test 2>&1 | tail -20
```

Expected: FAIL — `loadRepos is not a function`.

- [ ] **Step 3: Add new service functions to services/repos.ts**

Add to the end of `src/services/repos.ts`:

```typescript
export function setRepoTags(repoId: string, tags: string[]): Promise<import('@/types/repo').Repo> {
  return invoke('set_repo_tags', { repoId, tags })
}

export function exportRepoList(id: string): Promise<string> {
  return invoke('export_repo_list', { id })
}

export function importRepoList(yaml: string): Promise<import('@/types/repo').RepoList> {
  return invoke('import_repo_list', { yaml })
}
```

- [ ] **Step 4: Implement repos store**

Replace `src/stores/repos.ts`:

```typescript
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Repo } from '@/types/repo'
import { listRepos, discoverRepos, setRepoTags } from '@/services/repos'

export const useReposStore = defineStore('repos', () => {
  const repos       = ref<Repo[]>([])
  const isLoading   = ref(false)
  const discovering = ref(false)
  const error       = ref<string | null>(null)
  const searchQuery = ref('')

  const filteredRepos = computed(() => {
    if (!searchQuery.value) return repos.value
    const q = searchQuery.value.toLowerCase()
    return repos.value.filter(r =>
      r.fullName.toLowerCase().includes(q) ||
      r.tags.some(t => t.toLowerCase().includes(q)),
    )
  })

  async function loadRepos(repoListId?: string) {
    isLoading.value = true
    error.value = null
    try {
      repos.value = await listRepos(repoListId)
    } catch (e) {
      error.value = String(e)
    } finally {
      isLoading.value = false
    }
  }

  async function discoverReposAction(accountId: string) {
    discovering.value = true
    error.value = null
    try {
      repos.value = await discoverRepos(accountId)
    } catch (e) {
      error.value = String(e)
    } finally {
      discovering.value = false
    }
  }

  async function setRepoTagsAction(repoId: string, tags: string[]) {
    const updated = await setRepoTags(repoId, tags)
    const idx = repos.value.findIndex(r => r.id === repoId)
    if (idx !== -1) repos.value[idx] = updated
    return updated
  }

  return {
    repos, isLoading, discovering, error, searchQuery, filteredRepos,
    loadRepos, discoverReposAction, setRepoTagsAction,
  }
})
```

- [ ] **Step 5: Implement repoLists store**

Replace `src/stores/repoLists.ts`:

```typescript
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { RepoList } from '@/types/repo'
import type { CreateRepoListInput } from '@/services/repos'
import {
  listRepoLists, createRepoList, updateRepoList, deleteRepoList,
  addReposToList, removeReposFromList, exportRepoList, importRepoList,
} from '@/services/repos'

export const useRepoListsStore = defineStore('repoLists', () => {
  const lists          = ref<RepoList[]>([])
  const selectedListId = ref<string | null>(null)
  const isLoading      = ref(false)
  const error          = ref<string | null>(null)

  const selectedList = computed(() =>
    lists.value.find(l => l.id === selectedListId.value) ?? null,
  )
  const rootLists = computed(() => lists.value.filter(l => l.parentId === null))

  async function loadLists() {
    isLoading.value = true
    error.value = null
    try {
      lists.value = await listRepoLists()
    } catch (e) {
      error.value = String(e)
    } finally {
      isLoading.value = false
    }
  }

  async function createListAction(input: CreateRepoListInput) {
    const list = await createRepoList(input)
    lists.value.push(list)
    return list
  }

  async function updateListAction(id: string, input: CreateRepoListInput) {
    const updated = await updateRepoList(id, input)
    const idx = lists.value.findIndex(l => l.id === id)
    if (idx !== -1) lists.value[idx] = updated
    return updated
  }

  async function deleteListAction(id: string) {
    await deleteRepoList(id)
    lists.value = lists.value.filter(l => l.id !== id)
    if (selectedListId.value === id) selectedListId.value = null
  }

  async function addRepos(listId: string, repoIds: string[]) {
    await addReposToList(listId, repoIds)
    // Refresh the affected list's repoIds
    await loadLists()
  }

  async function removeRepos(listId: string, repoIds: string[]) {
    await removeReposFromList(listId, repoIds)
    await loadLists()
  }

  async function exportList(id: string): Promise<string> {
    return exportRepoList(id)
  }

  async function importList(yaml: string) {
    const list = await importRepoList(yaml)
    const idx = lists.value.findIndex(l => l.id === list.id)
    if (idx !== -1) {
      lists.value[idx] = list
    } else {
      lists.value.push(list)
    }
    return list
  }

  return {
    lists, selectedListId, selectedList, rootLists, isLoading, error,
    loadLists, createListAction, updateListAction, deleteListAction,
    addRepos, removeRepos, exportList, importList,
  }
})
```

- [ ] **Step 6: Update CreateRepoListInput in services/repos.ts**

The current `CreateRepoListInput` interface is missing `excludePatterns`. Update:

```typescript
export interface CreateRepoListInput {
  name: string
  description: string
  parentId?: string
  excludePatterns: string[]
}
```

- [ ] **Step 7: Run all tests**

```bash
pnpm test 2>&1 | tail -20
```

Expected: All tests pass (5 auth + 3 repos = 8 passing).

- [ ] **Step 8: Run pnpm typecheck**

```bash
pnpm typecheck 2>&1 | tail -10
```

Expected: no errors.

- [ ] **Step 9: Commit**

```bash
git add src/stores/repos.ts src/stores/repoLists.ts src/stores/settings.ts \
        src/stores/__tests__/repos.spec.ts src/services/repos.ts
git commit -m "feat(stores): implement repos, repoLists, and settings store actions"
```

---

## Task 10: RepoLists.vue — Repo Discovery & List Management

**Files:**
- Modify: `src/views/RepoLists.vue`

Two-tab layout:
- **Tab: Repositories** — searchable table of all discovered repos, Discover button, per-row tag display
- **Tab: Lists** — left tree of repo lists, right panel shows repos in selected list with Add/Remove

- [ ] **Step 1: Implement RepoLists.vue**

Replace entire content of `src/views/RepoLists.vue`:

```vue
<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useReposStore } from '@/stores/repos'
import { useRepoListsStore } from '@/stores/repoLists'
import { useAuthStore } from '@/stores/auth'
import Button from '@/components/ui/Button.vue'
import Input from '@/components/ui/Input.vue'
import Modal from '@/components/ui/Modal.vue'
import type { CreateRepoListInput } from '@/services/repos'

const reposStore     = useReposStore()
const listsStore     = useRepoListsStore()
const authStore      = useAuthStore()

const activeTab      = ref<'repos' | 'lists'>('repos')
const showCreateList = ref(false)
const showAddRepos   = ref(false)
const importYaml     = ref('')
const exportedYaml   = ref('')
const showExport     = ref(false)
const actionError    = ref<string | null>(null)

onMounted(async () => {
  await Promise.all([reposStore.loadRepos(), listsStore.loadLists()])
})

// ── Discover ─────────────────────────────────────────────────────────────
async function handleDiscover() {
  if (!authStore.githubAccount) return
  actionError.value = null
  try {
    await reposStore.discoverReposAction(authStore.githubAccount.id)
  } catch (e) {
    actionError.value = String(e)
  }
}

// ── Create list form ──────────────────────────────────────────────────────
const newListName        = ref('')
const newListDescription = ref('')

async function handleCreateList() {
  if (!newListName.value.trim()) return
  actionError.value = null
  try {
    const input: CreateRepoListInput = {
      name:             newListName.value.trim(),
      description:      newListDescription.value.trim(),
      excludePatterns:  [],
    }
    await listsStore.createListAction(input)
    newListName.value        = ''
    newListDescription.value = ''
    showCreateList.value     = false
  } catch (e) {
    actionError.value = String(e)
  }
}

async function handleDeleteList(id: string) {
  actionError.value = null
  try {
    await listsStore.deleteListAction(id)
  } catch (e) {
    actionError.value = String(e)
  }
}

// ── Add repos to selected list ────────────────────────────────────────────
const addRepoSearch  = ref('')
const selectedToAdd  = ref<Set<string>>(new Set())

const reposNotInList = computed(() => {
  const listRepoIds = new Set(listsStore.selectedList?.repoIds ?? [])
  const q = addRepoSearch.value.toLowerCase()
  return reposStore.repos.filter(r =>
    !listRepoIds.has(r.id) &&
    (!q || r.fullName.toLowerCase().includes(q)),
  )
})

function toggleRepoSelect(id: string) {
  if (selectedToAdd.value.has(id)) {
    selectedToAdd.value.delete(id)
  } else {
    selectedToAdd.value.add(id)
  }
}

async function handleAddRepos() {
  if (!listsStore.selectedListId || selectedToAdd.value.size === 0) return
  actionError.value = null
  try {
    await listsStore.addRepos(listsStore.selectedListId, [...selectedToAdd.value])
    await reposStore.loadRepos(listsStore.selectedListId)
    selectedToAdd.value = new Set()
    showAddRepos.value  = false
  } catch (e) {
    actionError.value = String(e)
  }
}

async function handleRemoveFromList(repoId: string) {
  if (!listsStore.selectedListId) return
  actionError.value = null
  try {
    await listsStore.removeRepos(listsStore.selectedListId, [repoId])
    await reposStore.loadRepos(listsStore.selectedListId)
  } catch (e) {
    actionError.value = String(e)
  }
}

// ── Export / Import ───────────────────────────────────────────────────────
async function handleExport(id: string) {
  actionError.value = null
  try {
    exportedYaml.value = await listsStore.exportList(id)
    showExport.value   = true
  } catch (e) {
    actionError.value = String(e)
  }
}

async function handleImport() {
  if (!importYaml.value.trim()) return
  actionError.value = null
  try {
    await listsStore.importList(importYaml.value.trim())
    importYaml.value = ''
  } catch (e) {
    actionError.value = String(e)
  }
}

async function selectList(id: string) {
  listsStore.selectedListId = id
  await reposStore.loadRepos(id)
}

const reposInSelectedList = computed(() => {
  if (!listsStore.selectedListId) return []
  const ids = new Set(listsStore.selectedList?.repoIds ?? [])
  return reposStore.repos.filter(r => ids.has(r.id))
})
</script>

<template>
  <div class="p-6 flex flex-col gap-6 h-full">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <h1 class="text-2xl font-semibold">Repositories</h1>
      <div class="flex items-center gap-2">
        <!-- Tab toggles -->
        <div class="flex border border-border rounded-md overflow-hidden text-sm">
          <button
            class="px-4 py-1.5 transition-colors"
            :class="activeTab === 'repos' ? 'bg-primary text-white' : 'text-muted hover:text-gray-200'"
            @click="activeTab = 'repos'"
          >Repos</button>
          <button
            class="px-4 py-1.5 transition-colors border-l border-border"
            :class="activeTab === 'lists' ? 'bg-primary text-white' : 'text-muted hover:text-gray-200'"
            @click="activeTab = 'lists'"
          >Lists</button>
        </div>
      </div>
    </div>

    <p v-if="actionError" class="text-danger text-sm">{{ actionError }}</p>

    <!-- Tab: Repos ─────────────────────────────────────────────────────── -->
    <div v-if="activeTab === 'repos'" class="flex flex-col gap-4 flex-1 min-h-0">
      <div class="flex items-center gap-3">
        <Input
          v-model="reposStore.searchQuery"
          placeholder="Search repos or tags…"
          class="w-72"
        />
        <Button
          variant="primary"
          :loading="reposStore.discovering"
          :disabled="!authStore.hasAccounts || reposStore.discovering"
          @click="handleDiscover"
        >
          {{ reposStore.discovering ? 'Discovering…' : 'Discover Repos' }}
        </Button>
        <span v-if="!authStore.hasAccounts" class="text-xs text-warning">
          Connect an account in Accounts first
        </span>
      </div>

      <!-- Repos table -->
      <div class="flex-1 overflow-auto">
        <table class="w-full text-sm">
          <thead class="sticky top-0 bg-surface border-b border-border">
            <tr class="text-left text-muted">
              <th class="py-2 pr-4 font-medium pl-2">Repository</th>
              <th class="py-2 pr-4 font-medium">Branch</th>
              <th class="py-2 pr-4 font-medium">Tags</th>
              <th class="py-2 font-medium">Visibility</th>
            </tr>
          </thead>
          <tbody>
            <tr v-if="reposStore.isLoading">
              <td colspan="4" class="py-8 text-center text-muted">Loading repos…</td>
            </tr>
            <tr v-else-if="reposStore.filteredRepos.length === 0">
              <td colspan="4" class="py-8 text-center text-muted">
                No repos found. Click "Discover Repos" to scan your GitHub account.
              </td>
            </tr>
            <tr
              v-for="repo in reposStore.filteredRepos"
              :key="repo.id"
              class="border-b border-border/50 hover:bg-white/5 transition-colors"
            >
              <td class="py-2 pr-4 pl-2">
                <a :href="repo.url" target="_blank" class="text-primary hover:underline font-mono text-xs">
                  {{ repo.fullName }}
                </a>
              </td>
              <td class="py-2 pr-4 font-mono text-xs text-muted">{{ repo.defaultBranch }}</td>
              <td class="py-2 pr-4">
                <div class="flex flex-wrap gap-1">
                  <span
                    v-for="tag in repo.tags"
                    :key="tag"
                    class="text-xs bg-primary/10 text-primary px-1.5 py-0.5 rounded"
                  >{{ tag }}</span>
                </div>
              </td>
              <td class="py-2">
                <span class="text-xs" :class="repo.isPrivate ? 'text-warning' : 'text-muted'">
                  {{ repo.isPrivate ? 'Private' : 'Public' }}
                </span>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- Tab: Lists ─────────────────────────────────────────────────────── -->
    <div v-if="activeTab === 'lists'" class="flex gap-4 flex-1 min-h-0">
      <!-- Left: list tree -->
      <aside class="w-56 flex-shrink-0 flex flex-col gap-2">
        <Button variant="secondary" size="sm" class="w-full" @click="showCreateList = true">
          + New List
        </Button>

        <!-- Import YAML -->
        <div class="space-y-1">
          <textarea
            v-model="importYaml"
            rows="3"
            placeholder="Paste YAML to import…"
            class="w-full text-xs bg-surface border border-border rounded-md px-2 py-1.5 text-gray-100 placeholder:text-muted resize-none outline-none focus:border-primary"
          />
          <Button variant="ghost" size="sm" :disabled="!importYaml.trim()" @click="handleImport">
            Import
          </Button>
        </div>

        <div class="border-t border-border pt-2 flex flex-col gap-0.5">
          <div v-if="listsStore.isLoading" class="text-xs text-muted px-2">Loading…</div>
          <button
            v-for="list in listsStore.rootLists"
            :key="list.id"
            class="text-left px-2 py-1.5 rounded-md text-sm transition-colors truncate"
            :class="listsStore.selectedListId === list.id
              ? 'bg-primary/20 text-primary'
              : 'text-muted hover:text-gray-200 hover:bg-white/5'"
            @click="selectList(list.id)"
          >
            {{ list.name }}
            <span class="text-xs opacity-60 ml-1">({{ list.repoIds.length }})</span>
          </button>
          <p v-if="!listsStore.isLoading && listsStore.rootLists.length === 0" class="text-xs text-muted px-2">
            No lists yet
          </p>
        </div>
      </aside>

      <!-- Right: repos in selected list -->
      <main class="flex-1 flex flex-col gap-3 min-w-0 overflow-hidden">
        <div v-if="!listsStore.selectedList" class="text-muted text-sm pt-4">
          Select a list to manage its repos.
        </div>

        <template v-else>
          <div class="flex items-center justify-between flex-shrink-0">
            <div>
              <h2 class="font-semibold">{{ listsStore.selectedList.name }}</h2>
              <p class="text-xs text-muted">{{ listsStore.selectedList.description }}</p>
            </div>
            <div class="flex gap-2">
              <Button variant="secondary" size="sm" @click="handleExport(listsStore.selectedListId!)">
                Export YAML
              </Button>
              <Button variant="secondary" size="sm" @click="showAddRepos = true">
                Add Repos
              </Button>
              <Button variant="danger" size="sm" @click="handleDeleteList(listsStore.selectedListId!)">
                Delete List
              </Button>
            </div>
          </div>

          <div class="flex-1 overflow-auto">
            <table class="w-full text-sm">
              <thead class="sticky top-0 bg-surface border-b border-border">
                <tr class="text-left text-muted">
                  <th class="py-2 pr-4 font-medium pl-2">Repository</th>
                  <th class="py-2 font-medium">Action</th>
                </tr>
              </thead>
              <tbody>
                <tr v-if="reposInSelectedList.length === 0">
                  <td colspan="2" class="py-6 text-center text-muted text-sm">
                    No repos in this list. Click "Add Repos" to add some.
                  </td>
                </tr>
                <tr
                  v-for="repo in reposInSelectedList"
                  :key="repo.id"
                  class="border-b border-border/50 hover:bg-white/5"
                >
                  <td class="py-2 pr-4 pl-2 font-mono text-xs">{{ repo.fullName }}</td>
                  <td class="py-2">
                    <Button variant="ghost" size="sm" @click="handleRemoveFromList(repo.id)">
                      Remove
                    </Button>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </template>
      </main>
    </div>

    <!-- Modal: Create List ──────────────────────────────────────────────── -->
    <Modal v-model:open="showCreateList" title="Create Repo List" size="sm">
      <div class="space-y-3">
        <div>
          <label class="block text-sm text-muted mb-1">Name</label>
          <Input v-model="newListName" placeholder="Client Acme" />
        </div>
        <div>
          <label class="block text-sm text-muted mb-1">Description</label>
          <Input v-model="newListDescription" placeholder="All repos for Acme Corp" />
        </div>
        <div class="flex justify-end gap-2 pt-2">
          <Button variant="secondary" @click="showCreateList = false">Cancel</Button>
          <Button variant="primary" :disabled="!newListName.trim()" @click="handleCreateList">
            Create
          </Button>
        </div>
      </div>
    </Modal>

    <!-- Modal: Add Repos ────────────────────────────────────────────────── -->
    <Modal v-model:open="showAddRepos" title="Add Repos to List" size="lg">
      <div class="space-y-3">
        <Input v-model="addRepoSearch" placeholder="Filter repos…" />
        <div class="max-h-80 overflow-y-auto space-y-1">
          <label
            v-for="repo in reposNotInList"
            :key="repo.id"
            class="flex items-center gap-3 px-3 py-2 rounded-md hover:bg-white/5 cursor-pointer"
          >
            <input
              type="checkbox"
              :checked="selectedToAdd.has(repo.id)"
              class="accent-primary"
              @change="toggleRepoSelect(repo.id)"
            />
            <span class="font-mono text-xs">{{ repo.fullName }}</span>
          </label>
          <p v-if="reposNotInList.length === 0" class="text-muted text-sm text-center py-4">
            All repos are already in this list, or no repos discovered yet.
          </p>
        </div>
        <div class="flex justify-end gap-2 pt-2">
          <Button variant="secondary" @click="showAddRepos = false; selectedToAdd = new Set()">Cancel</Button>
          <Button
            variant="primary"
            :disabled="selectedToAdd.size === 0"
            @click="handleAddRepos"
          >
            Add {{ selectedToAdd.size > 0 ? selectedToAdd.size : '' }} Repos
          </Button>
        </div>
      </div>
    </Modal>

    <!-- Modal: Export YAML ─────────────────────────────────────────────── -->
    <Modal v-model:open="showExport" title="Exported YAML" size="lg">
      <div class="space-y-3">
        <pre class="bg-surface border border-border rounded-md p-4 text-xs font-mono overflow-auto max-h-80 whitespace-pre-wrap">{{ exportedYaml }}</pre>
        <p class="text-xs text-muted">Copy this YAML and save it to <code>.flotilla/repo-lists/</code> in your project to share with your team.</p>
        <div class="flex justify-end">
          <Button variant="secondary" @click="showExport = false">Close</Button>
        </div>
      </div>
    </Modal>
  </div>
</template>
```

- [ ] **Step 2: Run pnpm typecheck**

```bash
pnpm typecheck 2>&1 | tail -10
```

Expected: no errors.

- [ ] **Step 3: Run full test suite**

```bash
pnpm test 2>&1 | tail -20
```

Expected: 8 tests pass.

- [ ] **Step 4: Run pnpm build**

```bash
pnpm build 2>&1 | tail -15
```

Expected: build succeeds.

- [ ] **Step 5: Commit**

```bash
git add src/views/RepoLists.vue
git commit -m "feat(ui): implement RepoLists.vue with repo discovery and list management"
```

---

## Task 11: Final Build Verification + PLANNING.md Update

**Files:**
- Modify: `PLANNING.md` (mark Phase 2 items implemented)

- [ ] **Step 1: Full Rust build + clippy**

```bash
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings 2>&1 | tail -20
```

Expected: no warnings. Fix any that appear.

- [ ] **Step 2: Full Rust test run**

```bash
cargo test --manifest-path src-tauri/Cargo.toml 2>&1 | tail -10
```

Expected: 7 tests pass, 0 failures.

- [ ] **Step 3: Full frontend test run**

```bash
pnpm test 2>&1 | tail -10
```

Expected: 8 tests pass.

- [ ] **Step 4: Full frontend build**

```bash
pnpm build 2>&1 | tail -10
```

Expected: build succeeds.

- [ ] **Step 5: Update PLANNING.md — mark Phase 2 items implemented**

In `PLANNING.md`, update Phase 2 items:

```markdown
### 2.1 Authentication
- [implemented] GitHub Personal Access Token input + validation
- [ ] GitHub OAuth App flow (optional, for teams)
- [ ] GitLab Personal Access Token input + validation
- [implemented] OS keychain storage via `keyring` crate
- [implemented] Token scope validation on save (warn on missing scopes)
- [ ] Multi-account support (multiple GitHub/GitLab accounts) — model supports it, UI shows first account
- [implemented] Auth status indicator in top bar
- [implemented] Token revocation / removal

### 2.2 Repo Discovery
- [implemented] List all repos accessible to authenticated GitHub user
- [implemented] List all repos in authenticated GitHub orgs
- [ ] List all repos accessible to authenticated GitLab user
- [ ] List all repos in authenticated GitLab groups
- [implemented] Pagination handling for large orgs (>100 repos)
- [implemented] Store discovered repos in SQLite
- [implemented] Display repos in a searchable, filterable table

### 2.3 Repo Lists
- [implemented] Create / rename / delete repo list
- [implemented] Add repos to list (multi-select from discovered repos)
- [implemented] Remove repos from list
- [ ] Nested lists (parent → child hierarchy, max 3 levels) — data model supports it, UI shows root lists only
- [implemented] Tag repos with arbitrary labels
- [ ] Filter repo discovery table by tags
- [implemented] **Exclusion rules**: support org-level and repo-level exclusion patterns
- [ ] **Auto-exclude**: automatically mark repos without relevant manifests as excluded
- [implemented] Export repo list as YAML
- [implemented] Import repo list from YAML
- [ ] Store repo lists in `.flotilla/repo-lists/*.yaml` — currently in SQLite only; YAML is manual export
- [ ] Repo list sidebar tree with expand/collapse — sidebar shows nav only; list tree is in RepoLists.vue
```

Also update Phase 1 items to `[implemented]` if not already done.

- [ ] **Step 6: Commit**

```bash
git add PLANNING.md
git commit -m "docs(planning): mark Phase 2 authentication and repo management as implemented"
```

---

## Self-Review

### Spec Coverage Check

| PLANNING.md requirement | Covered by task |
|-------------------------|-----------------|
| GitHub PAT input + validation | Task 4 (Rust) + Task 8 (UI) |
| OS keychain storage | Task 4 |
| Token scope validation | Task 4, Task 8 (warning in UI) |
| Multi-account support | Task 4 (model), Task 8 (list UI) |
| Auth status in top bar | Already in AppTopbar.vue (Phase 1), wired to authStore |
| Token revocation | Task 4 `remove_account`, Task 8 Remove button |
| List GitHub user repos | Task 5 `discover_repos` |
| List GitHub org repos | Task 5 `list_org_repos` + dedup |
| Pagination | Task 2 `GitHubClient` paginator |
| SQLite persistence | Task 5 upsert |
| Searchable, filterable table | Task 10 RepoLists.vue Tab 1 |
| Create / rename / delete list | Task 5 + Task 10 |
| Add / remove repos | Task 5 + Task 10 |
| Tag repos | Task 5 `set_repo_tags`, Task 9 store action |
| Exclusion patterns (data model) | Task 5 `CreateRepoListInput.exclude_patterns` |
| YAML export | Task 5 `export_repo_list`, Task 10 UI |
| YAML import | Task 5 `import_repo_list`, Task 10 UI |
| Rate limit indicator | Task 3 service, Task 7 settings store `refreshRateLimit` |

### Gaps Noted (deferred — noted in PLANNING.md)

- GitLab auth: deferred to GitLab phase
- Multi-account UI: model supports multiple, UI currently shows one (Phase 2 UX gap, noted)
- Nested list UI: data model has parent_id, but the sidebar tree only shows root lists
- Auto-exclude: requires scanner data — deferred to Phase 3
- Rate limit auto-refresh: `refreshRateLimit()` is defined but not auto-called; caller should invoke after API calls

### Type Consistency Check ✓

- `AccountInfo` camelCase Rust ↔ TypeScript matches
- `Repo`, `RepoList` models consistent throughout
- `CreateRepoListInput.excludePatterns` in TypeScript ↔ `exclude_patterns` in Rust (serde camelCase)
- `RateLimitInfo.resetAt` in TypeScript ↔ `reset_at` with camelCase serde in Rust

### Placeholder Scan ✓

No TBD, TODO, or incomplete steps found.
