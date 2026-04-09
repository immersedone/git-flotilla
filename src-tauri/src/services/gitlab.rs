use crate::error::{AppError, AppResult};
use reqwest::header::HeaderMap;
use serde::Deserialize;

// ── Response models ────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct GitLabUser {
    pub username: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GitLabProject {
    pub id: u64,
    pub path_with_namespace: String,
    pub name: String,
    pub namespace: GitLabNamespace,
    pub web_url: String,
    pub default_branch: Option<String>,
    pub visibility: String,
    pub last_activity_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GitLabNamespace {
    pub full_path: String,
}

#[derive(Debug, Deserialize)]
pub struct GitLabGroup {
    pub full_path: String,
}

// ── Rate limit helpers ────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct GitLabRateLimitSnapshot {
    pub remaining: u32,
    pub limit: u32,
    pub reset_epoch: u64,
}

/// Extract rate limit info from GitLab response headers.
///
/// GitLab uses `RateLimit-Remaining`, `RateLimit-Limit`, `RateLimit-Reset`.
pub fn extract_gitlab_rate_limit(headers: &HeaderMap) -> Option<GitLabRateLimitSnapshot> {
    let remaining = headers
        .get("ratelimit-remaining")?
        .to_str()
        .ok()?
        .parse()
        .ok()?;
    let limit = headers
        .get("ratelimit-limit")?
        .to_str()
        .ok()?
        .parse()
        .ok()?;
    let reset_epoch: u64 = headers
        .get("ratelimit-reset")?
        .to_str()
        .ok()?
        .parse()
        .ok()?;
    Some(GitLabRateLimitSnapshot {
        remaining,
        limit,
        reset_epoch,
    })
}

// ── Client ─────────────────────────────────────────────────────────────────

pub struct GitLabClient {
    client: reqwest::Client,
    token: String,
    base_url: String,
}

impl GitLabClient {
    /// Create a new GitLab API client.
    ///
    /// `base_url` defaults to `https://gitlab.com/api/v4` if `None`.
    pub fn new(token: impl Into<String>, base_url: Option<String>) -> Self {
        GitLabClient {
            client: reqwest::Client::new(),
            token: token.into(),
            base_url: base_url.unwrap_or_else(|| "https://gitlab.com/api/v4".to_string()),
        }
    }

    fn auth_headers(&self) -> AppResult<reqwest::header::HeaderMap> {
        use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, USER_AGENT};
        let mut map = HeaderMap::new();
        let token_val = HeaderValue::from_str(&self.token)
            .map_err(|_| AppError::Auth("Token contains invalid header characters".into()))?;
        map.insert("PRIVATE-TOKEN", token_val);
        map.insert(ACCEPT, HeaderValue::from_static("application/json"));
        map.insert(USER_AGENT, HeaderValue::from_static("git-flotilla/0.1"));
        Ok(map)
    }

    async fn get_raw(&self, path: &str) -> AppResult<reqwest::Response> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .get(&url)
            .headers(self.auth_headers()?)
            .send()
            .await
            .map_err(|e| AppError::GitLab(e.to_string()))?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return match status.as_u16() {
                401 => Err(AppError::Auth(format!("Invalid GitLab token: {text}"))),
                403 => Err(AppError::RateLimit(format!(
                    "GitLab rate limited or forbidden: {text}"
                ))),
                404 => Err(AppError::NotFound(format!("GitLab not found: {path}"))),
                _ => Err(AppError::GitLab(format!("HTTP {status}: {text}"))),
            };
        }
        Ok(resp)
    }

    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> AppResult<(T, HeaderMap)> {
        let resp = self.get_raw(path).await?;
        let headers = resp.headers().clone();
        let body = resp
            .json::<T>()
            .await
            .map_err(|e| AppError::GitLab(e.to_string()))?;
        Ok((body, headers))
    }

    /// Fetch the authenticated user info.
    pub async fn get_authenticated_user(
        &self,
    ) -> AppResult<(GitLabUser, Option<GitLabRateLimitSnapshot>)> {
        let (user, headers) = self.get::<GitLabUser>("/user").await?;
        let rate_limit = extract_gitlab_rate_limit(&headers);
        Ok((user, rate_limit))
    }

    /// Fetch all projects accessible to the authenticated user (paginated).
    pub async fn list_all_projects(
        &self,
    ) -> AppResult<(Vec<GitLabProject>, Option<GitLabRateLimitSnapshot>)> {
        let mut all = Vec::new();
        let mut page = 1u32;
        let mut last_rl;
        loop {
            let path = format!("/projects?membership=true&per_page=100&page={}", page);
            let (projects, headers): (Vec<GitLabProject>, _) = self.get(&path).await?;
            last_rl = extract_gitlab_rate_limit(&headers);
            let done = projects.len() < 100;
            all.extend(projects);
            if done {
                break;
            }
            page += 1;
        }
        Ok((all, last_rl))
    }

    /// Fetch all groups the authenticated user belongs to (paginated).
    pub async fn list_groups(
        &self,
    ) -> AppResult<(Vec<GitLabGroup>, Option<GitLabRateLimitSnapshot>)> {
        let mut all = Vec::new();
        let mut page = 1u32;
        let mut last_rl;
        loop {
            let path = format!("/groups?per_page=100&page={}", page);
            let (groups, headers): (Vec<GitLabGroup>, _) = self.get(&path).await?;
            last_rl = extract_gitlab_rate_limit(&headers);
            let done = groups.len() < 100;
            all.extend(groups);
            if done {
                break;
            }
            page += 1;
        }
        Ok((all, last_rl))
    }

    /// Fetch all projects in a given group (paginated).
    ///
    /// `group_path` is URL-encoded automatically (e.g. "my-org/sub-group").
    pub async fn list_group_projects(
        &self,
        group_path: &str,
    ) -> AppResult<(Vec<GitLabProject>, Option<GitLabRateLimitSnapshot>)> {
        let encoded = urlencoding::encode(group_path);
        let mut all = Vec::new();
        let mut page = 1u32;
        let mut last_rl;
        loop {
            let path = format!("/groups/{}/projects?per_page=100&page={}", encoded, page);
            let (projects, headers): (Vec<GitLabProject>, _) = self.get(&path).await?;
            last_rl = extract_gitlab_rate_limit(&headers);
            let done = projects.len() < 100;
            all.extend(projects);
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
    fn deserialize_gitlab_user() {
        let json = r#"{"username":"alice","avatar_url":"https://gitlab.com/uploads/-/system/user/avatar/1/avatar.png"}"#;
        let user: GitLabUser = serde_json::from_str(json).expect("parse user");
        assert_eq!(user.username, "alice");
        assert_eq!(
            user.avatar_url.as_deref(),
            Some("https://gitlab.com/uploads/-/system/user/avatar/1/avatar.png")
        );
    }

    #[test]
    fn deserialize_gitlab_user_no_avatar() {
        let json = r#"{"username":"bob","avatar_url":null}"#;
        let user: GitLabUser = serde_json::from_str(json).expect("parse user");
        assert_eq!(user.username, "bob");
        assert!(user.avatar_url.is_none());
    }

    #[test]
    fn deserialize_gitlab_project() {
        let json = r#"{
            "id": 42,
            "path_with_namespace": "my-org/my-project",
            "name": "my-project",
            "namespace": { "full_path": "my-org" },
            "web_url": "https://gitlab.com/my-org/my-project",
            "default_branch": "main",
            "visibility": "private",
            "last_activity_at": "2026-03-15T10:00:00Z"
        }"#;
        let project: GitLabProject = serde_json::from_str(json).expect("parse project");
        assert_eq!(project.id, 42);
        assert_eq!(project.path_with_namespace, "my-org/my-project");
        assert_eq!(project.name, "my-project");
        assert_eq!(project.namespace.full_path, "my-org");
        assert_eq!(project.web_url, "https://gitlab.com/my-org/my-project");
        assert_eq!(project.default_branch.as_deref(), Some("main"));
        assert_eq!(project.visibility, "private");
        assert_eq!(
            project.last_activity_at.as_deref(),
            Some("2026-03-15T10:00:00Z")
        );
    }

    #[test]
    fn deserialize_gitlab_project_no_default_branch() {
        let json = r#"{
            "id": 99,
            "path_with_namespace": "user/empty-repo",
            "name": "empty-repo",
            "namespace": { "full_path": "user" },
            "web_url": "https://gitlab.com/user/empty-repo",
            "default_branch": null,
            "visibility": "public",
            "last_activity_at": null
        }"#;
        let project: GitLabProject = serde_json::from_str(json).expect("parse project");
        assert_eq!(project.id, 99);
        assert!(project.default_branch.is_none());
        assert!(project.last_activity_at.is_none());
    }

    #[test]
    fn deserialize_gitlab_group() {
        let json = r#"{"full_path":"my-org/sub-group"}"#;
        let group: GitLabGroup = serde_json::from_str(json).expect("parse group");
        assert_eq!(group.full_path, "my-org/sub-group");
    }

    #[test]
    fn extract_gitlab_rate_limit_valid() {
        let headers = headers_with(&[
            ("ratelimit-remaining", "1950"),
            ("ratelimit-limit", "2000"),
            ("ratelimit-reset", "1720000000"),
        ]);
        let rl = extract_gitlab_rate_limit(&headers).expect("should parse");
        assert_eq!(rl.remaining, 1950);
        assert_eq!(rl.limit, 2000);
        assert_eq!(rl.reset_epoch, 1720000000u64);
    }

    #[test]
    fn extract_gitlab_rate_limit_missing_headers() {
        let headers = headers_with(&[]);
        assert!(extract_gitlab_rate_limit(&headers).is_none());
    }

    #[test]
    fn gitlab_auth_header_format() {
        let client = GitLabClient::new("glpat-xxxxxxxxxxxxxxxxxxxx", None);
        let headers = client.auth_headers().expect("should build headers");
        let token_header = headers
            .get("PRIVATE-TOKEN")
            .expect("should have PRIVATE-TOKEN header");
        assert_eq!(token_header.to_str().unwrap(), "glpat-xxxxxxxxxxxxxxxxxxxx");
        // Should NOT have Authorization/Bearer header
        assert!(headers.get("authorization").is_none());
    }

    #[test]
    fn gitlab_client_default_base_url() {
        let client = GitLabClient::new("token", None);
        assert_eq!(client.base_url, "https://gitlab.com/api/v4");
    }

    #[test]
    fn gitlab_client_custom_base_url() {
        let client = GitLabClient::new(
            "token",
            Some("https://gitlab.example.com/api/v4".to_string()),
        );
        assert_eq!(client.base_url, "https://gitlab.example.com/api/v4");
    }
}
