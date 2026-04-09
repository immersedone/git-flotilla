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

#[derive(Debug, Deserialize)]
pub struct GitHubPullRequest {
    pub number: u64,
    pub html_url: String,
    pub state: String,
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct GitHubRefObject {
    pub sha: String,
}

#[derive(Debug, Deserialize)]
pub struct GitHubRef {
    #[serde(rename = "ref")]
    pub ref_name: String,
    pub object: GitHubRefObject,
}

/// Response from the Contents API PUT (create or update file).
#[derive(Debug, Deserialize)]
pub struct GitHubContentWriteResponse {
    pub content: GitHubContentResponseSummary,
}

#[derive(Debug, Deserialize)]
pub struct GitHubContentResponseSummary {
    pub name: String,
    pub path: String,
    pub sha: String,
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

    async fn post<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &impl serde::Serialize,
    ) -> AppResult<(T, HeaderMap)> {
        let url = format!("https://api.github.com{path}");
        let resp = self
            .client
            .post(&url)
            .headers(self.auth_headers()?)
            .json(body)
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
                422 => Err(AppError::GitHub(format!("Validation failed: {text}"))),
                _ => Err(AppError::GitHub(format!("HTTP {status}: {text}"))),
            };
        }
        let headers = resp.headers().clone();
        let parsed = resp.json::<T>().await?;
        Ok((parsed, headers))
    }

    async fn put<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &impl serde::Serialize,
    ) -> AppResult<(T, HeaderMap)> {
        let url = format!("https://api.github.com{path}");
        let resp = self
            .client
            .put(&url)
            .headers(self.auth_headers()?)
            .json(body)
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
                409 => Err(AppError::GitHub(format!(
                    "Conflict (SHA mismatch?): {text}"
                ))),
                422 => Err(AppError::GitHub(format!("Validation failed: {text}"))),
                _ => Err(AppError::GitHub(format!("HTTP {status}: {text}"))),
            };
        }
        let headers = resp.headers().clone();
        let parsed = resp.json::<T>().await?;
        Ok((parsed, headers))
    }

    async fn patch_no_body_parse(
        &self,
        path: &str,
        body: &impl serde::Serialize,
    ) -> AppResult<HeaderMap> {
        let url = format!("https://api.github.com{path}");
        let resp = self
            .client
            .patch(&url)
            .headers(self.auth_headers()?)
            .json(body)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::GitHub(format!("HTTP {status}: {text}")));
        }
        Ok(resp.headers().clone())
    }

    async fn post_no_body_parse(
        &self,
        path: &str,
        body: &impl serde::Serialize,
    ) -> AppResult<HeaderMap> {
        let url = format!("https://api.github.com{path}");
        let resp = self
            .client
            .post(&url)
            .headers(self.auth_headers()?)
            .json(body)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::GitHub(format!("HTTP {status}: {text}")));
        }
        Ok(resp.headers().clone())
    }

    async fn delete_ref(&self, path: &str) -> AppResult<HeaderMap> {
        let url = format!("https://api.github.com{path}");
        let resp = self
            .client
            .delete(&url)
            .headers(self.auth_headers()?)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return match status.as_u16() {
                404 => Err(AppError::NotFound(format!("Not found: {path}"))),
                422 => Err(AppError::GitHub(format!("Validation failed: {text}"))),
                _ => Err(AppError::GitHub(format!("HTTP {status}: {text}"))),
            };
        }
        Ok(resp.headers().clone())
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

    // ── Branch management ─────────────────────────────────────────────────

    /// Get the SHA of a branch's HEAD commit.
    pub async fn get_branch_sha(
        &self,
        owner: &str,
        repo: &str,
        branch: &str,
    ) -> AppResult<(String, Option<RateLimitSnapshot>)> {
        let path = format!("/repos/{owner}/{repo}/git/ref/heads/{branch}");
        let (git_ref, headers): (GitHubRef, _) = self.get(&path).await?;
        let rate_limit = extract_rate_limit(&headers);
        Ok((git_ref.object.sha, rate_limit))
    }

    /// Create a new branch from a given SHA.
    pub async fn create_branch(
        &self,
        owner: &str,
        repo: &str,
        branch_name: &str,
        from_sha: &str,
    ) -> AppResult<Option<RateLimitSnapshot>> {
        let path = format!("/repos/{owner}/{repo}/git/refs");
        let body = serde_json::json!({
            "ref": format!("refs/heads/{branch_name}"),
            "sha": from_sha,
        });
        let (_ref_resp, headers): (GitHubRef, _) = self.post(&path, &body).await?;
        Ok(extract_rate_limit(&headers))
    }

    /// Delete a branch (ref) from a repo.
    pub async fn delete_branch(
        &self,
        owner: &str,
        repo: &str,
        branch: &str,
    ) -> AppResult<Option<RateLimitSnapshot>> {
        let path = format!("/repos/{owner}/{repo}/git/refs/heads/{branch}");
        let headers = self.delete_ref(&path).await?;
        Ok(extract_rate_limit(&headers))
    }

    // ── File operations ───────────────────────────────────────────────────

    /// Create or update a file in a repo via the Contents API.
    ///
    /// The `content` parameter should be raw content (will be base64-encoded).
    /// Pass `sha` of the existing file when updating; `None` when creating.
    #[allow(clippy::too_many_arguments)]
    pub async fn create_or_update_file(
        &self,
        owner: &str,
        repo: &str,
        path: &str,
        content: &str,
        message: &str,
        branch: &str,
        sha: Option<&str>,
    ) -> AppResult<(GitHubContentWriteResponse, Option<RateLimitSnapshot>)> {
        let encoded = base64::engine::general_purpose::STANDARD.encode(content.as_bytes());
        let api_path = format!("/repos/{owner}/{repo}/contents/{path}");

        let mut body = serde_json::json!({
            "message": message,
            "content": encoded,
            "branch": branch,
        });

        if let Some(existing_sha) = sha {
            body.as_object_mut()
                .expect("json! always creates an object")
                .insert(
                    "sha".to_string(),
                    serde_json::Value::String(existing_sha.to_string()),
                );
        }

        let (resp, headers): (GitHubContentWriteResponse, _) = self.put(&api_path, &body).await?;
        Ok((resp, extract_rate_limit(&headers)))
    }

    // ── Pull request operations ───────────────────────────────────────────

    /// Create a pull request.
    #[allow(clippy::too_many_arguments)]
    pub async fn create_pull_request(
        &self,
        owner: &str,
        repo: &str,
        title: &str,
        body: &str,
        head: &str,
        base: &str,
        draft: bool,
        labels: &[String],
    ) -> AppResult<(GitHubPullRequest, Option<RateLimitSnapshot>)> {
        let path = format!("/repos/{owner}/{repo}/pulls");
        let pr_body = serde_json::json!({
            "title": title,
            "body": body,
            "head": head,
            "base": base,
            "draft": draft,
        });

        let (pr, headers): (GitHubPullRequest, _) = self.post(&path, &pr_body).await?;
        let rate_limit = extract_rate_limit(&headers);

        // Add labels if any were provided
        if !labels.is_empty() {
            let labels_path = format!("/repos/{owner}/{repo}/issues/{}/labels", pr.number);
            let labels_body = serde_json::json!({ "labels": labels });
            // Best-effort label assignment — don't fail the whole operation if labelling fails
            if let Err(e) = self.post_no_body_parse(&labels_path, &labels_body).await {
                tracing::warn!("Failed to add labels to PR #{}: {e}", pr.number);
            }
        }

        Ok((pr, rate_limit))
    }

    /// Close a pull request, optionally leaving a comment.
    pub async fn close_pull_request(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
        comment: Option<&str>,
    ) -> AppResult<Option<RateLimitSnapshot>> {
        // Post comment first if provided
        if let Some(text) = comment {
            let comment_path = format!("/repos/{owner}/{repo}/issues/{pr_number}/comments");
            let comment_body = serde_json::json!({ "body": text });
            if let Err(e) = self.post_no_body_parse(&comment_path, &comment_body).await {
                tracing::warn!("Failed to comment on PR #{pr_number}: {e}");
            }
        }

        // Close the PR
        let path = format!("/repos/{owner}/{repo}/pulls/{pr_number}");
        let body = serde_json::json!({ "state": "closed" });
        let headers = self.patch_no_body_parse(&path, &body).await?;
        Ok(extract_rate_limit(&headers))
    }

    /// List pull requests, optionally filtered by head branch and state.
    pub async fn list_pull_requests(
        &self,
        owner: &str,
        repo: &str,
        head: Option<&str>,
        state: Option<&str>,
    ) -> AppResult<(Vec<GitHubPullRequest>, Option<RateLimitSnapshot>)> {
        let mut query_parts = vec![format!("per_page=100")];
        if let Some(h) = head {
            // GitHub expects head in "owner:branch" format
            query_parts.push(format!("head={owner}:{h}"));
        }
        if let Some(s) = state {
            query_parts.push(format!("state={s}"));
        }
        let query = query_parts.join("&");
        let path = format!("/repos/{owner}/{repo}/pulls?{query}");
        let (prs, headers): (Vec<GitHubPullRequest>, _) = self.get(&path).await?;
        let rate_limit = extract_rate_limit(&headers);
        Ok((prs, rate_limit))
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

    #[test]
    fn deserialize_pull_request() {
        let json = r#"{
            "number": 42,
            "html_url": "https://github.com/owner/repo/pull/42",
            "state": "open",
            "title": "fix: patch CVE-2024-1234"
        }"#;
        let pr: GitHubPullRequest = serde_json::from_str(json).expect("parse PR");
        assert_eq!(pr.number, 42);
        assert_eq!(pr.html_url, "https://github.com/owner/repo/pull/42");
        assert_eq!(pr.state, "open");
        assert_eq!(pr.title, "fix: patch CVE-2024-1234");
    }

    #[test]
    fn deserialize_pull_request_list() {
        let json = r#"[
            {"number": 1, "html_url": "https://github.com/o/r/pull/1", "state": "open", "title": "PR 1"},
            {"number": 2, "html_url": "https://github.com/o/r/pull/2", "state": "closed", "title": "PR 2"}
        ]"#;
        let prs: Vec<GitHubPullRequest> = serde_json::from_str(json).expect("parse PRs");
        assert_eq!(prs.len(), 2);
        assert_eq!(prs[0].number, 1);
        assert_eq!(prs[1].state, "closed");
    }

    #[test]
    fn deserialize_git_ref() {
        let json = r#"{
            "ref": "refs/heads/main",
            "object": { "sha": "abc123def456" }
        }"#;
        let git_ref: GitHubRef = serde_json::from_str(json).expect("parse ref");
        assert_eq!(git_ref.ref_name, "refs/heads/main");
        assert_eq!(git_ref.object.sha, "abc123def456");
    }

    #[test]
    fn deserialize_content_write_response() {
        let json = r#"{
            "content": {
                "name": ".nvmrc",
                "path": ".nvmrc",
                "sha": "new_sha_123"
            },
            "commit": {
                "sha": "commit_sha_456"
            }
        }"#;
        let resp: GitHubContentWriteResponse =
            serde_json::from_str(json).expect("parse content write");
        assert_eq!(resp.content.name, ".nvmrc");
        assert_eq!(resp.content.sha, "new_sha_123");
    }

    #[test]
    fn base64_encode_content() {
        use base64::Engine;
        let raw = "node 20.1.0\n";
        let encoded = base64::engine::general_purpose::STANDARD.encode(raw.as_bytes());
        // Verify round-trip
        let decoded_bytes = base64::engine::general_purpose::STANDARD
            .decode(&encoded)
            .expect("decode");
        let decoded = String::from_utf8(decoded_bytes).expect("utf8");
        assert_eq!(decoded, raw);
        // Verify the encoded value is correct
        assert_eq!(encoded, "bm9kZSAyMC4xLjAK");
    }
}
