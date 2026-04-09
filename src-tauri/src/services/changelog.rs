use crate::commands::packages::ChangelogEntry;
use crate::services::github::GitHubRelease;
use serde::Deserialize;

// ── npm registry lookup ───────────────────────────────────────────────────

#[derive(Deserialize)]
struct NpmPackageInfo {
    repository: Option<NpmRepository>,
}

#[derive(Deserialize)]
struct NpmRepository {
    url: Option<String>,
}

/// Look up an npm package on the registry and extract its GitHub (owner, repo).
pub async fn npm_package_to_github_repo(
    client: &reqwest::Client,
    package_name: &str,
) -> Option<(String, String)> {
    let url = format!("https://registry.npmjs.org/{package_name}");
    let resp = client.get(&url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let info: NpmPackageInfo = resp.json().await.ok()?;
    let repo_url = info.repository?.url?;
    parse_github_url(&repo_url)
}

// ── npm latest version lookup ────────────────────────────────────────────

#[derive(Deserialize)]
struct NpmLatestResponse {
    version: Option<String>,
}

/// Fetch the latest version of an npm package from the registry.
///
/// Returns `None` on any network or parse failure — this is a best-effort lookup.
pub async fn get_npm_latest_version(
    client: &reqwest::Client,
    package_name: &str,
) -> Option<String> {
    let url = format!("https://registry.npmjs.org/{package_name}/latest");
    let resp = client.get(&url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let info: NpmLatestResponse = resp.json().await.ok()?;
    info.version
}

// ── URL parsing ───────────────────────────────────────────────────────────

/// Parse a GitHub URL into (owner, repo).
///
/// Handles: `git+https://github.com/owner/repo.git`,
///          `https://github.com/owner/repo`,
///          `ssh://git@github.com/owner/repo.git`,
///          `git://github.com/owner/repo.git`
fn parse_github_url(url: &str) -> Option<(String, String)> {
    let cleaned = url
        .trim()
        .replace("git+https://", "https://")
        .replace("git+ssh://", "ssh://")
        .replace("git://", "https://")
        .replace("ssh://git@github.com/", "https://github.com/")
        .replace("ssh://git@github.com:", "https://github.com/");

    // Strip trailing .git
    let cleaned = cleaned.strip_suffix(".git").unwrap_or(&cleaned);

    // Now expect https://github.com/owner/repo
    let path = cleaned.strip_prefix("https://github.com/")?;
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() >= 2 && !parts[0].is_empty() && !parts[1].is_empty() {
        Some((parts[0].to_string(), parts[1].to_string()))
    } else {
        None
    }
}

// ── Release filtering ─────────────────────────────────────────────────────

/// Strip leading 'v' from a version tag.
fn normalize_version_tag(version: &str) -> String {
    version.strip_prefix('v').unwrap_or(version).to_string()
}

/// Filter GitHub releases to those between `from_version` and `to_version` (inclusive).
///
/// Releases come newest-first from GitHub. We walk through them, start collecting
/// when we hit `to_version`, and stop after we collect `from_version`.
/// Prereleases are skipped. Breaking changes are detected by searching the body
/// for "breaking" or "BREAKING".
pub fn releases_to_changelog(
    releases: &[GitHubRelease],
    from_version: &str,
    to_version: &str,
) -> Vec<ChangelogEntry> {
    let from_norm = normalize_version_tag(from_version);
    let to_norm = normalize_version_tag(to_version);

    let mut entries = Vec::new();
    let mut collecting = false;

    for release in releases {
        if release.prerelease {
            continue;
        }

        let tag_norm = normalize_version_tag(&release.tag_name);

        if tag_norm == to_norm {
            collecting = true;
        }

        if collecting {
            let body = release.body.clone().unwrap_or_default();
            let body_lower = body.to_lowercase();
            let is_breaking = body_lower.contains("breaking change")
                || body_lower.contains("breaking:")
                || body.contains("BREAKING CHANGE")
                || body.contains("⚠");

            entries.push(ChangelogEntry {
                version: tag_norm.clone(),
                body,
                published_at: release.published_at.clone().unwrap_or_default(),
                is_breaking,
            });

            if tag_norm == from_norm {
                break;
            }
        }
    }

    entries
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_github_url_https() {
        let result = parse_github_url("https://github.com/vuejs/core");
        assert_eq!(result, Some(("vuejs".to_string(), "core".to_string())));
    }

    #[test]
    fn parse_github_url_git_plus() {
        let result = parse_github_url("git+https://github.com/axios/axios.git");
        assert_eq!(result, Some(("axios".to_string(), "axios".to_string())));
    }

    #[test]
    fn parse_github_url_ssh() {
        let result = parse_github_url("ssh://git@github.com/org/repo.git");
        assert_eq!(result, Some(("org".to_string(), "repo".to_string())));
    }

    #[test]
    fn normalize_strips_v_prefix() {
        assert_eq!(normalize_version_tag("v3.4.0"), "3.4.0");
        assert_eq!(normalize_version_tag("3.4.0"), "3.4.0");
    }

    #[test]
    fn releases_to_changelog_filters_range() {
        let releases = vec![
            GitHubRelease {
                tag_name: "v3.4.0".to_string(),
                name: Some("3.4.0".to_string()),
                body: Some("New feature".to_string()),
                published_at: Some("2026-03-01T00:00:00Z".to_string()),
                prerelease: false,
            },
            GitHubRelease {
                tag_name: "v3.3.0".to_string(),
                name: Some("3.3.0".to_string()),
                body: Some("BREAKING change here".to_string()),
                published_at: Some("2026-02-01T00:00:00Z".to_string()),
                prerelease: false,
            },
            GitHubRelease {
                tag_name: "v3.2.0".to_string(),
                name: Some("3.2.0".to_string()),
                body: Some("Bug fix".to_string()),
                published_at: Some("2026-01-01T00:00:00Z".to_string()),
                prerelease: false,
            },
        ];

        let entries = releases_to_changelog(&releases, "3.2.0", "3.4.0");
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].version, "3.4.0");
        assert!(!entries[0].is_breaking);
        assert_eq!(entries[1].version, "3.3.0");
        assert!(entries[1].is_breaking);
        assert_eq!(entries[2].version, "3.2.0");
        assert!(!entries[2].is_breaking);
    }

    #[test]
    fn releases_skips_prereleases() {
        let releases = vec![
            GitHubRelease {
                tag_name: "v3.4.0".to_string(),
                name: Some("3.4.0".to_string()),
                body: Some("Stable".to_string()),
                published_at: Some("2026-03-01T00:00:00Z".to_string()),
                prerelease: false,
            },
            GitHubRelease {
                tag_name: "v3.4.0-beta.1".to_string(),
                name: Some("3.4.0-beta.1".to_string()),
                body: Some("Beta".to_string()),
                published_at: Some("2026-02-15T00:00:00Z".to_string()),
                prerelease: true,
            },
            GitHubRelease {
                tag_name: "v3.3.0".to_string(),
                name: Some("3.3.0".to_string()),
                body: Some("Previous stable".to_string()),
                published_at: Some("2026-02-01T00:00:00Z".to_string()),
                prerelease: false,
            },
        ];

        let entries = releases_to_changelog(&releases, "3.3.0", "3.4.0");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].version, "3.4.0");
        assert_eq!(entries[1].version, "3.3.0");
    }
}
