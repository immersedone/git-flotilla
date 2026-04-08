use crate::error::{AppError, AppResult};
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
    pub reset_at: String,
}

/// Extract rate limit info from response headers.
pub fn extract_rate_limit(headers: &HeaderMap) -> Option<RateLimitSnapshot> {
    let remaining = headers.get("x-ratelimit-remaining")?.to_str().ok()?.parse().ok()?;
    let limit     = headers.get("x-ratelimit-limit")?.to_str().ok()?.parse().ok()?;
    let reset_ts: u64 = headers.get("x-ratelimit-reset")?.to_str().ok()?.parse().ok()?;
    Some(RateLimitSnapshot { remaining, limit, reset_at: reset_ts.to_string() })
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
        let url  = format!("https://api.github.com{}", path);
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

        #[allow(unused_assignments)]
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
