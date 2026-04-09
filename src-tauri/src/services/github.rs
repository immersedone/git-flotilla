use crate::error::{AppError, AppResult};
use base64::Engine;
use reqwest::header::HeaderMap;
use serde::Deserialize;

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

#[derive(Debug, Deserialize)]
pub struct GitHubTreeEntry {
    pub path: String,
    pub mode: String,
    #[serde(rename = "type")]
    pub entry_type: String,
    pub sha: String,
    pub size: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct GitHubTreeResponse {
    pub sha: String,
    pub url: String,
    pub tree: Vec<GitHubTreeEntry>,
    pub truncated: bool,
}

#[derive(Debug, Deserialize)]
pub struct GitHubContentResponse {
    pub name: String,
    pub path: String,
    pub sha: String,
    pub size: u64,
    pub encoding: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct GitHubBranch {
    pub name: String,
    #[serde(rename = "protected")]
    pub is_protected: bool,
}

#[derive(Debug, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub published_at: Option<String>,
    pub prerelease: bool,
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

#[derive(Debug, Clone)]
pub struct RateLimitSnapshot {
    pub remaining: u32,
    pub limit: u32,
    pub reset_epoch: u64,
}

/// Extract rate limit info from response headers.
pub fn extract_rate_limit(headers: &HeaderMap) -> Option<RateLimitSnapshot> {
    let remaining = headers
        .get("x-ratelimit-remaining")?
        .to_str()
        .ok()?
        .parse()
        .ok()?;
    let limit = headers
        .get("x-ratelimit-limit")?
        .to_str()
        .ok()?
        .parse()
        .ok()?;
    let reset_epoch: u64 = headers
        .get("x-ratelimit-reset")?
        .to_str()
        .ok()?
        .parse()
        .ok()?;
    Some(RateLimitSnapshot {
        remaining,
        limit,
        reset_epoch,
    })
}

/// Decode a base64-encoded string from GitHub's content API.
///
/// GitHub returns base64 content with embedded newlines; this function
/// strips them before decoding and converts the result to a UTF-8 string.
pub fn decode_base64_content(encoded: &str) -> AppResult<String> {
    let cleaned: String = encoded
        .chars()
        .filter(|c| *c != '\n' && *c != '\r')
        .collect();
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&cleaned)
        .map_err(|e| AppError::GitHub(format!("Base64 decode failed: {e}")))?;
    String::from_utf8(bytes).map_err(|e| AppError::GitHub(format!("UTF-8 conversion failed: {e}")))
}

// ── Client ─────────────────────────────────────────────────────────────────

pub struct GitHubClient {
    client: reqwest::Client,
    token: String,
}

impl GitHubClient {
    pub fn new(token: impl Into<String>) -> Self {
        GitHubClient {
            client: reqwest::Client::new(),
            token: token.into(),
        }
    }

    fn auth_headers(&self) -> AppResult<reqwest::header::HeaderMap> {
        use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
        let mut map = HeaderMap::new();
        let auth_val = HeaderValue::from_str(&format!("Bearer {}", self.token))
            .map_err(|_| AppError::Auth("Token contains invalid header characters".into()))?;
        map.insert(AUTHORIZATION, auth_val);
        map.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.github+json"),
        );
        map.insert(USER_AGENT, HeaderValue::from_static("git-flotilla/0.1"));
        map.insert(
            "X-GitHub-Api-Version",
            HeaderValue::from_static("2022-11-28"),
        );
        Ok(map)
    }

    async fn get_raw(&self, path: &str) -> AppResult<reqwest::Response> {
        let url = format!("https://api.github.com{}", path);
        let resp = self
            .client
            .get(&url)
            .headers(self.auth_headers()?)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return match status.as_u16() {
                401 => Err(AppError::Auth(format!("Invalid token: {text}"))),
                403 => Err(AppError::RateLimit(format!(
                    "Rate limited or forbidden: {text}"
                ))),
                404 => Err(AppError::NotFound(format!("Not found: {path}"))),
                _ => Err(AppError::GitHub(format!("HTTP {status}: {text}"))),
            };
        }
        Ok(resp)
    }

    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> AppResult<(T, HeaderMap)> {
        let resp = self.get_raw(path).await?;
        let headers = resp.headers().clone();
        let body = resp.json::<T>().await?;
        Ok((body, headers))
    }

    /// Fetch authenticated user info and their granted scopes.
    pub async fn get_authenticated_user(
        &self,
    ) -> AppResult<(GitHubUser, Vec<String>, Option<RateLimitSnapshot>)> {
        let (user, headers) = self.get::<GitHubUser>("/user").await?;
        let scopes = parse_scopes_header(&headers);
        let rate_limit = extract_rate_limit(&headers);
        Ok((user, scopes, rate_limit))
    }

    /// Fetch all repos accessible to the authenticated user (paginated).
    pub async fn list_all_repos(&self) -> AppResult<(Vec<GitHubRepo>, Option<RateLimitSnapshot>)> {
        let mut all = Vec::new();
        let mut page = 1u32;
        let mut last_rl;
        loop {
            let path = format!(
                "/user/repos?per_page=100&page={}&type=all&sort=pushed",
                page
            );
            let (repos, headers): (Vec<GitHubRepo>, _) = self.get(&path).await?;
            last_rl = extract_rate_limit(&headers);
            let done = repos.len() < 100;
            all.extend(repos);
            if done {
                break;
            }
            page += 1;
        }
        Ok((all, last_rl))
    }

    /// Fetch all orgs the authenticated user belongs to.
    pub async fn list_orgs(&self) -> AppResult<(Vec<GitHubOrg>, Option<RateLimitSnapshot>)> {
        let mut all = Vec::new();
        let mut page = 1u32;
        let mut last_rl;
        loop {
            let path = format!("/user/orgs?per_page=100&page={}", page);
            let (orgs, headers): (Vec<GitHubOrg>, _) = self.get(&path).await?;
            last_rl = extract_rate_limit(&headers);
            let done = orgs.len() < 100;
            all.extend(orgs);
            if done {
                break;
            }
            page += 1;
        }
        Ok((all, last_rl))
    }

    /// Fetch all repos for a given org (paginated).
    pub async fn list_org_repos(
        &self,
        org: &str,
    ) -> AppResult<(Vec<GitHubRepo>, Option<RateLimitSnapshot>)> {
        let mut all = Vec::new();
        let mut page = 1u32;
        let mut last_rl;
        loop {
            let path = format!("/orgs/{}/repos?per_page=100&page={}&type=all", org, page);
            let (repos, headers): (Vec<GitHubRepo>, _) = self.get(&path).await?;
            last_rl = extract_rate_limit(&headers);
            let done = repos.len() < 100;
            all.extend(repos);
            if done {
                break;
            }
            page += 1;
        }
        Ok((all, last_rl))
    }

    /// Fetch the full recursive tree for a given tree SHA.
    pub async fn get_repo_tree(
        &self,
        owner: &str,
        repo: &str,
        tree_sha: &str,
    ) -> AppResult<(GitHubTreeResponse, Option<RateLimitSnapshot>)> {
        let path = format!("/repos/{owner}/{repo}/git/trees/{tree_sha}?recursive=1");
        let (tree, headers): (GitHubTreeResponse, _) = self.get(&path).await?;
        let rate_limit = extract_rate_limit(&headers);
        Ok((tree, rate_limit))
    }

    /// Fetch a single file's content (base64-encoded) at a given ref.
    pub async fn get_file_content(
        &self,
        owner: &str,
        repo: &str,
        file_path: &str,
        git_ref: &str,
    ) -> AppResult<(GitHubContentResponse, Option<RateLimitSnapshot>)> {
        let path = format!("/repos/{owner}/{repo}/contents/{file_path}?ref={git_ref}");
        let (content, headers): (GitHubContentResponse, _) = self.get(&path).await?;
        let rate_limit = extract_rate_limit(&headers);
        Ok((content, rate_limit))
    }

    /// Fetch releases for a repo (single page, up to 100).
    pub async fn list_releases(
        &self,
        owner: &str,
        repo: &str,
    ) -> AppResult<(Vec<GitHubRelease>, Option<RateLimitSnapshot>)> {
        let path = format!("/repos/{owner}/{repo}/releases?per_page=100");
        let (releases, headers): (Vec<GitHubRelease>, _) = self.get(&path).await?;
        let rate_limit = extract_rate_limit(&headers);
        Ok((releases, rate_limit))
    }

    /// List all branches for a repo (paginated).
    pub async fn list_branches(
        &self,
        owner: &str,
        repo: &str,
    ) -> AppResult<(Vec<GitHubBranch>, Option<RateLimitSnapshot>)> {
        let mut all = Vec::new();
        let mut page = 1u32;
        let mut last_rl;
        loop {
            let path = format!("/repos/{owner}/{repo}/branches?per_page=100&page={page}");
            let (branches, headers): (Vec<GitHubBranch>, _) = self.get(&path).await?;
            last_rl = extract_rate_limit(&headers);
            let done = branches.len() < 100;
            all.extend(branches);
            if done {
                break;
            }
            page += 1;
        }
        Ok((all, last_rl))
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
        let scopes = parse_scopes_header(&headers);
        assert_eq!(scopes, vec!["repo", "workflow", "read:org"]);
    }

    #[test]
    fn parse_scopes_empty_header() {
        let headers = headers_with(&[]);
        let scopes = parse_scopes_header(&headers);
        assert!(scopes.is_empty());
    }

    #[test]
    fn extract_rate_limit_valid() {
        let headers = headers_with(&[
            ("x-ratelimit-remaining", "4950"),
            ("x-ratelimit-limit", "5000"),
            ("x-ratelimit-reset", "1720000000"),
        ]);
        let rl = extract_rate_limit(&headers).expect("should parse");
        assert_eq!(rl.remaining, 4950);
        assert_eq!(rl.limit, 5000);
        assert_eq!(rl.reset_epoch, 1720000000u64);
    }

    #[test]
    fn extract_rate_limit_missing_headers() {
        let headers = headers_with(&[]);
        assert!(extract_rate_limit(&headers).is_none());
    }

    #[test]
    fn deserialize_tree_entry() {
        let json = r#"{
            "path": "src/main.rs",
            "mode": "100644",
            "type": "blob",
            "sha": "abc123",
            "size": 1024
        }"#;
        let entry: GitHubTreeEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.path, "src/main.rs");
        assert_eq!(entry.mode, "100644");
        assert_eq!(entry.entry_type, "blob");
        assert_eq!(entry.sha, "abc123");
        assert_eq!(entry.size, Some(1024));
    }

    #[test]
    fn deserialize_tree_response() {
        let json = r#"{
            "sha": "def456",
            "url": "https://api.github.com/repos/owner/repo/git/trees/def456",
            "tree": [
                {
                    "path": "package.json",
                    "mode": "100644",
                    "type": "blob",
                    "sha": "aaa111",
                    "size": 512
                },
                {
                    "path": "src",
                    "mode": "040000",
                    "type": "tree",
                    "sha": "bbb222",
                    "size": null
                }
            ],
            "truncated": false
        }"#;
        let resp: GitHubTreeResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.sha, "def456");
        assert_eq!(resp.tree.len(), 2);
        assert!(!resp.truncated);
        assert_eq!(resp.tree[0].path, "package.json");
        assert!(resp.tree[1].size.is_none());
    }

    #[test]
    fn deserialize_content_response() {
        let json = r#"{
            "name": ".nvmrc",
            "path": ".nvmrc",
            "sha": "ccc333",
            "size": 5,
            "encoding": "base64",
            "content": "MjAuMQ==\n"
        }"#;
        let resp: GitHubContentResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.name, ".nvmrc");
        assert_eq!(resp.path, ".nvmrc");
        assert_eq!(resp.sha, "ccc333");
        assert_eq!(resp.size, 5);
        assert_eq!(resp.encoding, "base64");
    }

    #[test]
    fn deserialize_branch_response() {
        let json = r#"[
            { "name": "main", "protected": true },
            { "name": "develop", "protected": false }
        ]"#;
        let branches: Vec<GitHubBranch> = serde_json::from_str(json).unwrap();
        assert_eq!(branches.len(), 2);
        assert_eq!(branches[0].name, "main");
        assert!(branches[0].is_protected);
        assert_eq!(branches[1].name, "develop");
        assert!(!branches[1].is_protected);
    }

    #[test]
    fn deserialize_release_response() {
        let json = r#"[{"tag_name":"v3.4.0","name":"3.4.0","body":"New feature","published_at":"2026-01-01T00:00:00Z","prerelease":false}]"#;
        let releases: Vec<GitHubRelease> = serde_json::from_str(json).expect("parse");
        assert_eq!(releases.len(), 1);
        assert_eq!(releases[0].tag_name, "v3.4.0");
        assert!(!releases[0].prerelease);
    }

    #[test]
    fn decode_base64_valid() {
        use base64::Engine;
        let original = "Hello, World!";
        let encoded = base64::engine::general_purpose::STANDARD.encode(original);
        let decoded = decode_base64_content(&encoded).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn decode_base64_with_newlines() {
        // Simulate GitHub's base64 with embedded newlines
        let encoded = "SGVsbG8s\nIFdvcmxk\nIQ==\n";
        let decoded = decode_base64_content(encoded).unwrap();
        assert_eq!(decoded, "Hello, World!");
    }
}
