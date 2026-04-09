# Phase 4: Package Intelligence — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a cross-repo dependency matrix from Phase 3's scan data, with version drift detection, ecosystem filtering, CSV export, and changelog aggregation from GitHub Releases.

**Architecture:** The `repo_packages` table (populated by scanning) is the data source. SQL aggregation queries build the matrix (GROUP BY package name + ecosystem, pivot versions per repo). A `services/changelog.rs` module fetches release notes from GitHub. The frontend `Packages.vue` renders a sortable/filterable matrix table with expandable changelog detail.

**Tech Stack:** Rust (sqlx, reqwest, serde), TypeScript (Vue 3, Pinia)

**Scope:** Phase 4.1 (Dependency Matrix) + Phase 4.2 (Changelog Aggregation). Deferred: 4.3 (Package Standardisation — requires Batch Operations from Phase 6).

---

## File Structure

### New files to create:
- `src-tauri/src/services/changelog.rs` — GitHub Releases API fetching + CHANGELOG.md fallback

### Files to modify:
- `src-tauri/src/commands/packages.rs` — replace stubs with real implementations
- `src-tauri/src/services/github.rs` — add `list_releases()` method
- `src-tauri/src/services/mod.rs` — register changelog module
- `src/stores/packages.ts` — add actions
- `src/views/Packages.vue` — full matrix UI
- `src/stores/__tests__/packages.spec.ts` — store tests (new file)

---

## Task 1: Implement `get_dependency_matrix` Command

Query the `repo_packages` table to build a cross-repo dependency matrix. This is the core of Phase 4 — everything else builds on this.

**Files:**
- Modify: `src-tauri/src/commands/packages.rs`

- [ ] **Step 1: Write test for matrix aggregation helper**

Add to `packages.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_matrix_from_flat_packages() {
        let packages = vec![
            ("github:org/app-a", "npm", "vue", "^3.4.0", false),
            ("github:org/app-b", "npm", "vue", "^3.3.0", false),
            ("github:org/app-a", "npm", "axios", "1.7.2", false),
            ("github:org/app-b", "npm", "lodash", "4.17.21", false),
        ];
        let matrix = aggregate_packages(&packages);
        assert_eq!(matrix.repo_ids.len(), 2);
        assert_eq!(matrix.packages.len(), 3); // vue, axios, lodash

        let vue_row = matrix.packages.iter().find(|p| p.name == "vue").unwrap();
        assert_eq!(vue_row.versions_by_repo.len(), 2);
        assert_eq!(vue_row.repo_count, 2);
        assert!(vue_row.has_drift); // ^3.4.0 != ^3.3.0
    }

    #[test]
    fn build_matrix_no_drift_when_same_version() {
        let packages = vec![
            ("github:org/a", "npm", "vue", "^3.4.0", false),
            ("github:org/b", "npm", "vue", "^3.4.0", false),
        ];
        let matrix = aggregate_packages(&packages);
        let vue = matrix.packages.iter().find(|p| p.name == "vue").unwrap();
        assert!(!vue.has_drift);
    }

    #[test]
    fn csv_export_format() {
        let matrix = DependencyMatrix {
            packages: vec![
                PackageRow {
                    name: "vue".to_string(),
                    ecosystem: "npm".to_string(),
                    versions_by_repo: [
                        ("github:org/a".to_string(), "^3.4.0".to_string()),
                        ("github:org/b".to_string(), "^3.3.0".to_string()),
                    ].into(),
                    latest_version: None,
                    repo_count: 2,
                    has_drift: true,
                    is_dev_only: false,
                },
            ],
            repo_ids: vec!["github:org/a".to_string(), "github:org/b".to_string()],
        };
        let csv = matrix_to_csv(&matrix);
        assert!(csv.starts_with("Package,Ecosystem,Repos,Drift,"));
        assert!(csv.contains("vue,npm,2,Yes,"));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib -- packages`
Expected: FAIL — `aggregate_packages`, `matrix_to_csv` not defined

- [ ] **Step 3: Update `DependencyMatrix` and `PackageRow` structs**

Replace the existing structs in `packages.rs`:

```rust
use crate::{db, error::{AppError, AppResult}};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DependencyMatrix {
    pub packages: Vec<PackageRow>,
    pub repo_ids: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PackageRow {
    pub name: String,
    pub ecosystem: String,
    pub versions_by_repo: HashMap<String, String>,
    pub latest_version: Option<String>,
    pub repo_count: usize,
    pub has_drift: bool,
    pub is_dev_only: bool,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChangelogEntry {
    pub version: String,
    pub body: String,
    pub published_at: String,
    pub is_breaking: bool,
}
```

- [ ] **Step 4: Implement `aggregate_packages` helper**

```rust
/// Build a DependencyMatrix from flat package tuples: (repo_id, ecosystem, name, version, is_dev).
fn aggregate_packages(packages: &[(&str, &str, &str, &str, bool)]) -> DependencyMatrix {
    let mut repo_set: BTreeSet<String> = BTreeSet::new();
    // Key: (ecosystem, name) → { repo_id → version, all_dev }
    let mut groups: BTreeMap<(String, String), (HashMap<String, String>, bool)> = BTreeMap::new();

    for &(repo_id, ecosystem, name, version, is_dev) in packages {
        repo_set.insert(repo_id.to_string());
        let entry = groups
            .entry((ecosystem.to_string(), name.to_string()))
            .or_insert_with(|| (HashMap::new(), true));
        entry.0.insert(repo_id.to_string(), version.to_string());
        if !is_dev {
            entry.1 = false;
        }
    }

    let repo_ids: Vec<String> = repo_set.into_iter().collect();

    let packages: Vec<PackageRow> = groups
        .into_iter()
        .map(|((ecosystem, name), (versions_by_repo, is_dev_only))| {
            let unique_versions: HashSet<&String> = versions_by_repo.values().collect();
            let has_drift = unique_versions.len() > 1;
            let repo_count = versions_by_repo.len();
            PackageRow {
                name,
                ecosystem,
                versions_by_repo,
                latest_version: None,
                repo_count,
                has_drift,
                is_dev_only,
            }
        })
        .collect();

    DependencyMatrix { packages, repo_ids }
}
```

- [ ] **Step 5: Implement `get_dependency_matrix` command**

```rust
#[tauri::command]
pub async fn get_dependency_matrix(
    repo_list_id: Option<String>,
    ecosystem: Option<String>,
) -> AppResult<DependencyMatrix> {
    let pool = db::pool()?;

    let rows = match (&repo_list_id, &ecosystem) {
        (Some(list_id), Some(eco)) => {
            sqlx::query(
                r#"SELECT p.repo_id, p.ecosystem, p.name, p.version, p.is_dev
                   FROM repo_packages p
                   INNER JOIN repo_list_members m ON m.repo_id = p.repo_id
                   WHERE m.list_id = ? AND p.ecosystem = ?
                   ORDER BY p.name, p.repo_id"#,
            )
            .bind(list_id)
            .bind(eco)
            .fetch_all(pool)
            .await?
        }
        (Some(list_id), None) => {
            sqlx::query(
                r#"SELECT p.repo_id, p.ecosystem, p.name, p.version, p.is_dev
                   FROM repo_packages p
                   INNER JOIN repo_list_members m ON m.repo_id = p.repo_id
                   WHERE m.list_id = ?
                   ORDER BY p.name, p.repo_id"#,
            )
            .bind(list_id)
            .fetch_all(pool)
            .await?
        }
        (None, Some(eco)) => {
            sqlx::query(
                r#"SELECT repo_id, ecosystem, name, version, is_dev
                   FROM repo_packages
                   WHERE ecosystem = ?
                   ORDER BY name, repo_id"#,
            )
            .bind(eco)
            .fetch_all(pool)
            .await?
        }
        (None, None) => {
            sqlx::query(
                r#"SELECT repo_id, ecosystem, name, version, is_dev
                   FROM repo_packages
                   ORDER BY name, repo_id"#,
            )
            .fetch_all(pool)
            .await?
        }
    };

    let flat: Vec<(&str, &str, &str, &str, bool)> = Vec::new();
    // Convert rows to tuples — need owned strings first
    let owned: Vec<(String, String, String, String, bool)> = rows
        .iter()
        .map(|r| {
            (
                r.get::<String, _>("repo_id"),
                r.get::<String, _>("ecosystem"),
                r.get::<String, _>("name"),
                r.get::<String, _>("version"),
                r.get::<i64, _>("is_dev") != 0,
            )
        })
        .collect();

    let tuples: Vec<(&str, &str, &str, &str, bool)> = owned
        .iter()
        .map(|(r, e, n, v, d)| (r.as_str(), e.as_str(), n.as_str(), v.as_str(), *d))
        .collect();

    let matrix = aggregate_packages(&tuples);
    tracing::info!(
        "Built dependency matrix: {} packages across {} repos",
        matrix.packages.len(),
        matrix.repo_ids.len()
    );
    Ok(matrix)
}
```

- [ ] **Step 6: Implement `matrix_to_csv` helper and `export_matrix_csv` command**

```rust
/// Convert matrix to CSV string. Columns: Package, Ecosystem, Repos, Drift, [repo1], [repo2], ...
fn matrix_to_csv(matrix: &DependencyMatrix) -> String {
    let mut lines = Vec::new();

    // Header
    let mut header = "Package,Ecosystem,Repos,Drift".to_string();
    for repo_id in &matrix.repo_ids {
        header.push(',');
        header.push_str(repo_id);
    }
    lines.push(header);

    // Rows
    for pkg in &matrix.packages {
        let mut line = format!(
            "{},{},{},{}",
            csv_escape(&pkg.name),
            pkg.ecosystem,
            pkg.repo_count,
            if pkg.has_drift { "Yes" } else { "No" },
        );
        for repo_id in &matrix.repo_ids {
            line.push(',');
            if let Some(ver) = pkg.versions_by_repo.get(repo_id) {
                line.push_str(&csv_escape(ver));
            }
        }
        lines.push(line);
    }

    lines.join("\n")
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[tauri::command]
pub async fn export_matrix_csv(repo_list_id: Option<String>) -> AppResult<String> {
    let matrix = get_dependency_matrix(repo_list_id, None).await?;
    Ok(matrix_to_csv(&matrix))
}
```

- [ ] **Step 7: Run tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib -- packages`
Expected: All 3 tests pass

- [ ] **Step 8: Run clippy + fmt**

Run: `cargo clippy --manifest-path src-tauri/Cargo.toml --lib -- -D warnings && cargo fmt --manifest-path src-tauri/Cargo.toml`

- [ ] **Step 9: Commit**

```bash
git add src-tauri/src/commands/packages.rs
git commit -m "feat(packages): implement dependency matrix aggregation and CSV export"
```

---

## Task 2: Update Frontend Types for Extended PackageRow

The Rust `PackageRow` now has `repoCount`, `hasDrift`, `isDevOnly` fields that the frontend types don't have yet.

**Files:**
- Modify: `src/types/package.ts`

- [ ] **Step 1: Update PackageRow interface**

```typescript
export interface PackageRow {
  name: string
  ecosystem: Ecosystem
  versionsByRepo: Record<string, string>
  latestVersion: string | null
  repoCount: number
  hasDrift: boolean
  isDevOnly: boolean
}
```

- [ ] **Step 2: Verify typecheck**

Run: `pnpm typecheck`

- [ ] **Step 3: Commit**

```bash
git add src/types/package.ts
git commit -m "feat(packages): add repoCount, hasDrift, isDevOnly to PackageRow type"
```

---

## Task 3: Implement Changelog Fetching

Fetch release notes from GitHub Releases API for a given npm/composer package. This lets users see what changed between versions before bumping.

**Files:**
- Modify: `src-tauri/src/services/github.rs` — add `list_releases()` method
- Create: `src-tauri/src/services/changelog.rs` — changelog fetching logic
- Modify: `src-tauri/src/services/mod.rs` — register module
- Modify: `src-tauri/src/commands/packages.rs` — implement `get_package_changelog`

- [ ] **Step 1: Add `GitHubRelease` struct and `list_releases` method to github.rs**

Struct:
```rust
#[derive(Debug, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub published_at: Option<String>,
    pub prerelease: bool,
}
```

Method on `GitHubClient`:
```rust
pub async fn list_releases(
    &self,
    owner: &str,
    repo: &str,
) -> AppResult<(Vec<GitHubRelease>, Option<RateLimitSnapshot>)>
```
Calls `/repos/{owner}/{repo}/releases?per_page=100` (single page — most packages have <100 releases worth checking).

- [ ] **Step 2: Create `services/changelog.rs`**

```rust
use crate::error::AppResult;
use crate::services::github::{GitHubClient, GitHubRelease};
use crate::commands::packages::ChangelogEntry;

/// Map a npm package name to its likely GitHub repo (e.g. "vue" → "vuejs/core").
/// For now, this is a best-effort lookup using the npm registry API.
/// Returns None if the package can't be mapped.
pub async fn npm_package_to_github_repo(
    client: &reqwest::Client,
    package_name: &str,
) -> Option<(String, String)> {
    let url = format!("https://registry.npmjs.org/{package_name}");
    let resp = client.get(&url).send().await.ok()?;
    let json: serde_json::Value = resp.json().await.ok()?;
    let repo_url = json.get("repository")?.get("url")?.as_str()?;
    // Parse "git+https://github.com/owner/repo.git" or "https://github.com/owner/repo"
    parse_github_url(repo_url)
}

/// Extract (owner, repo) from various GitHub URL formats.
fn parse_github_url(url: &str) -> Option<(String, String)> {
    let cleaned = url
        .replace("git+", "")
        .replace("git://", "https://")
        .replace(".git", "")
        .replace("ssh://git@github.com/", "https://github.com/");
    let path = cleaned.strip_prefix("https://github.com/")?;
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() >= 2 {
        Some((parts[0].to_string(), parts[1].to_string()))
    } else {
        None
    }
}

/// Convert GitHub releases to ChangelogEntry items, filtering to entries
/// between from_version and to_version (by tag_name matching).
pub fn releases_to_changelog(
    releases: &[GitHubRelease],
    from_version: &str,
    to_version: &str,
) -> Vec<ChangelogEntry> {
    let from_tag = normalize_version_tag(from_version);
    let to_tag = normalize_version_tag(to_version);

    let mut in_range = false;
    let mut entries = Vec::new();

    // Releases come newest-first from GitHub API
    for release in releases {
        if release.prerelease {
            continue;
        }
        let tag = normalize_version_tag(&release.tag_name);

        if tag == to_tag {
            in_range = true;
        }

        if in_range {
            let body = release.body.clone().unwrap_or_default();
            let is_breaking = body.to_lowercase().contains("breaking")
                || body.contains("BREAKING");
            entries.push(ChangelogEntry {
                version: release.tag_name.clone(),
                body,
                published_at: release.published_at.clone().unwrap_or_default(),
                is_breaking,
            });
        }

        if tag == from_tag {
            break;
        }
    }

    entries
}

/// Normalize version tags: strip "v" prefix, handle scoped packages
fn normalize_version_tag(version: &str) -> String {
    version
        .strip_prefix('v')
        .unwrap_or(version)
        .trim_start_matches(|c: char| !c.is_ascii_digit())
        .to_string()
}

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
                tag_name: "v3.4.0".into(),
                name: Some("3.4.0".into()),
                body: Some("New feature".into()),
                published_at: Some("2026-01-03".into()),
                prerelease: false,
            },
            GitHubRelease {
                tag_name: "v3.3.0".into(),
                name: Some("3.3.0".into()),
                body: Some("BREAKING: changed API".into()),
                published_at: Some("2026-01-02".into()),
                prerelease: false,
            },
            GitHubRelease {
                tag_name: "v3.2.0".into(),
                name: Some("3.2.0".into()),
                body: Some("Bugfix".into()),
                published_at: Some("2026-01-01".into()),
                prerelease: false,
            },
        ];
        let entries = releases_to_changelog(&releases, "3.2.0", "3.4.0");
        assert_eq!(entries.len(), 3);
        assert!(entries[1].is_breaking);
        assert_eq!(entries[0].version, "v3.4.0");
    }

    #[test]
    fn releases_skips_prereleases() {
        let releases = vec![
            GitHubRelease {
                tag_name: "v3.4.0".into(),
                name: None,
                body: Some("Stable".into()),
                published_at: Some("2026-01-02".into()),
                prerelease: false,
            },
            GitHubRelease {
                tag_name: "v3.4.0-beta.1".into(),
                name: None,
                body: Some("Beta".into()),
                published_at: Some("2026-01-01".into()),
                prerelease: true,
            },
            GitHubRelease {
                tag_name: "v3.3.0".into(),
                name: None,
                body: Some("Previous".into()),
                published_at: Some("2025-12-01".into()),
                prerelease: false,
            },
        ];
        let entries = releases_to_changelog(&releases, "3.3.0", "3.4.0");
        assert_eq!(entries.len(), 2); // beta skipped
    }
}
```

- [ ] **Step 3: Register changelog module**

Add `pub mod changelog;` to `src-tauri/src/services/mod.rs`.

- [ ] **Step 4: Implement `get_package_changelog` command**

In `packages.rs`:

```rust
#[tauri::command]
pub async fn get_package_changelog(
    package_name: String,
    ecosystem: String,
    from_version: String,
    to_version: String,
) -> AppResult<Vec<ChangelogEntry>> {
    if ecosystem != "npm" {
        return Err(AppError::Operation(format!(
            "Changelog fetching not yet supported for {ecosystem}"
        )));
    }

    let http_client = reqwest::Client::builder()
        .user_agent("git-flotilla/0.1")
        .build()
        .map_err(|e| AppError::Operation(e.to_string()))?;

    // Look up GitHub repo from npm registry
    let (owner, repo) = crate::services::changelog::npm_package_to_github_repo(
        &http_client,
        &package_name,
    )
    .await
    .ok_or_else(|| {
        AppError::NotFound(format!(
            "Could not find GitHub repo for npm package: {package_name}"
        ))
    })?;

    // Get token for authenticated requests (higher rate limit)
    let pool = db::pool()?;
    let token_result = sqlx::query("SELECT id FROM accounts WHERE provider = 'github' LIMIT 1")
        .fetch_optional(pool)
        .await?;

    let releases = if let Some(row) = token_result {
        let account_id: String = row.get("id");
        let token = keyring::Entry::new(
            crate::commands::auth::KEYCHAIN_SERVICE,
            &account_id,
        )
        .map_err(AppError::from)?
        .get_password()
        .map_err(AppError::from)?;

        let gh_client = crate::services::github::GitHubClient::new(&token);
        let (releases, rl) = gh_client.list_releases(&owner, &repo).await?;
        if let Some(snapshot) = rl {
            crate::services::rate_limiter::update_github(snapshot);
        }
        releases
    } else {
        // Unauthenticated fallback (lower rate limit)
        let url = format!("https://api.github.com/repos/{owner}/{repo}/releases?per_page=100");
        let resp = http_client
            .get(&url)
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "git-flotilla/0.1")
            .send()
            .await
            .map_err(|e| AppError::GitHub(e.to_string()))?;
        resp.json().await.map_err(|e| AppError::GitHub(e.to_string()))?
    };

    let entries = crate::services::changelog::releases_to_changelog(
        &releases,
        &from_version,
        &to_version,
    );

    tracing::info!(
        "Fetched {} changelog entries for {package_name} ({from_version} → {to_version})",
        entries.len()
    );

    Ok(entries)
}
```

- [ ] **Step 5: Run all tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib`

- [ ] **Step 6: Run clippy + fmt**

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/services/changelog.rs src-tauri/src/services/mod.rs src-tauri/src/services/github.rs src-tauri/src/commands/packages.rs
git commit -m "feat(packages): implement changelog fetching from GitHub Releases"
```

---

## Task 4: Frontend Packages Store

Implement the full Pinia store with matrix loading, ecosystem filtering, and changelog fetching.

**Files:**
- Modify: `src/stores/packages.ts`
- Create: `src/stores/__tests__/packages.spec.ts`

- [ ] **Step 1: Write tests**

```typescript
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { usePackagesStore } from '../packages'

vi.mock('@/services/packages', () => ({
  getDependencyMatrix: vi.fn(),
  getPackageChangelog: vi.fn(),
  exportMatrixCsv: vi.fn(),
}))

describe('usePackagesStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('initialises with empty state', () => {
    const store = usePackagesStore()
    expect(store.matrix).toBeNull()
    expect(store.isLoading).toBe(false)
    expect(store.error).toBeNull()
    expect(store.selectedEcosystem).toBeNull()
  })

  it('loads matrix', async () => {
    const { getDependencyMatrix } = await import('@/services/packages')
    const mockMatrix = {
      packages: [{ name: 'vue', ecosystem: 'npm', repoCount: 3, hasDrift: true }],
      repoIds: ['a', 'b', 'c'],
    }
    vi.mocked(getDependencyMatrix).mockResolvedValueOnce(mockMatrix as any)

    const store = usePackagesStore()
    await store.loadMatrix()
    expect(store.matrix).toEqual(mockMatrix)
  })

  it('handles loadMatrix error', async () => {
    const { getDependencyMatrix } = await import('@/services/packages')
    vi.mocked(getDependencyMatrix).mockRejectedValueOnce(new Error('fail'))

    const store = usePackagesStore()
    await expect(store.loadMatrix()).rejects.toThrow('fail')
    expect(store.error).toBe('Error: fail')
  })

  it('filteredPackages filters by ecosystem', () => {
    const store = usePackagesStore()
    store.matrix = {
      packages: [
        { name: 'vue', ecosystem: 'npm', repoCount: 2, hasDrift: false, isDevOnly: false, versionsByRepo: {}, latestVersion: null },
        { name: 'laravel', ecosystem: 'composer', repoCount: 1, hasDrift: false, isDevOnly: false, versionsByRepo: {}, latestVersion: null },
      ],
      repoIds: [],
    } as any
    store.selectedEcosystem = 'npm'
    expect(store.filteredPackages.length).toBe(1)
    expect(store.filteredPackages[0].name).toBe('vue')
  })
})
```

- [ ] **Step 2: Implement the store**

```typescript
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { DependencyMatrix, PackageRow, ChangelogEntry } from '@/types/package'
import {
  getDependencyMatrix,
  getPackageChangelog,
  exportMatrixCsv,
} from '@/services/packages'

export const usePackagesStore = defineStore('packages', () => {
  const matrix = ref<DependencyMatrix | null>(null)
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const selectedEcosystem = ref<string | null>(null)
  const selectedRepoListId = ref<string | null>(null)
  const changelog = ref<ChangelogEntry[]>([])
  const changelogLoading = ref(false)
  const searchQuery = ref('')

  const filteredPackages = computed<PackageRow[]>(() => {
    if (!matrix.value) return []
    let pkgs = matrix.value.packages
    if (selectedEcosystem.value) {
      pkgs = pkgs.filter((p) => p.ecosystem === selectedEcosystem.value)
    }
    if (searchQuery.value) {
      const q = searchQuery.value.toLowerCase()
      pkgs = pkgs.filter((p) => p.name.toLowerCase().includes(q))
    }
    return pkgs
  })

  const ecosystems = computed(() => {
    if (!matrix.value) return []
    const set = new Set(matrix.value.packages.map((p) => p.ecosystem))
    return [...set].sort()
  })

  const driftCount = computed(() => {
    if (!matrix.value) return 0
    return matrix.value.packages.filter((p) => p.hasDrift).length
  })

  async function loadMatrix(repoListId?: string, ecosystem?: string) {
    error.value = null
    isLoading.value = true
    try {
      matrix.value = await getDependencyMatrix(repoListId, ecosystem)
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  async function fetchChangelog(
    packageName: string,
    ecosystem: string,
    fromVersion: string,
    toVersion: string,
  ) {
    changelogLoading.value = true
    try {
      changelog.value = await getPackageChangelog(packageName, ecosystem, fromVersion, toVersion)
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      changelogLoading.value = false
    }
  }

  async function exportCsv(repoListId?: string) {
    try {
      return await exportMatrixCsv(repoListId)
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  return {
    matrix,
    isLoading,
    error,
    selectedEcosystem,
    selectedRepoListId,
    changelog,
    changelogLoading,
    searchQuery,
    filteredPackages,
    ecosystems,
    driftCount,
    loadMatrix,
    fetchChangelog,
    exportCsv,
  }
})
```

- [ ] **Step 3: Run tests**

Run: `pnpm vitest run --reporter verbose`

- [ ] **Step 4: Commit**

```bash
git add src/stores/packages.ts src/stores/__tests__/packages.spec.ts
git commit -m "feat(packages): implement packages Pinia store with matrix and changelog actions"
```

---

## Task 5: Packages.vue — Full Matrix UI

Build the Packages view with: ecosystem filter tabs, search, sortable matrix table, drift highlighting, expandable changelog, and CSV export.

**Files:**
- Modify: `src/views/Packages.vue`

- [ ] **Step 1: Implement the full view**

The view should have:
1. **Header** with title and Export CSV button
2. **Filter bar**: ecosystem pills/tabs, search input, repo list selector
3. **Summary stats**: total packages, packages with drift, ecosystems represented
4. **Matrix table**:
   - Columns: Package name, Ecosystem, Repos count, Drift indicator, Dev-only badge
   - Sortable by name, repo count, drift
   - Rows with drift get an amber left-border accent
   - Click row to expand and show per-repo version breakdown
5. **Expanded row**:
   - Shows version per repo (with colour-coding for different versions)
   - "View Changelog" button → loads changelog entries inline
6. **Empty state** when no data

Design: dark theme matching Scanner.vue — `bg-[#0F1117]`, `bg-[#1A1D27]`, `border-[#2A2D3A]`, font-mono for versions/package names.

- [ ] **Step 2: Run typecheck**

Run: `pnpm typecheck`

- [ ] **Step 3: Run all tests**

Run: `pnpm vitest run`

- [ ] **Step 4: Commit**

```bash
git add src/views/Packages.vue
git commit -m "feat(ui): implement Packages view with dependency matrix and changelog"
```

---

## Task 6: Update PLANNING.md

**Files:**
- Modify: `PLANNING.md`

- [ ] **Step 1: Mark Phase 4.1 items**

Mark as `[implemented]`:
- Cross-repo package table
- Filter by ecosystem
- Filter by repo list
- Highlight version drift
- Sort by package name, number of repos, highest drift
- Identify orphan packages (unique to one repo) — via `repoCount == 1`
- Export matrix as CSV

Mark as deferred with notes:
- Show latest available version — "deferred: requires registry API integration"
- Show outdated indicator — "deferred: requires latest version lookup"
- Identify superseded packages — "deferred: requires configurable supersession list"
- Export matrix as JSON — "deferred: CSV export implemented, JSON trivial to add"

- [ ] **Step 2: Mark Phase 4.2 items**

Mark as `[implemented]`:
- Fetch and display changelog entries between versions
- Pull from GitHub Releases API
- Highlight breaking changes

Mark as deferred:
- `CHANGELOG.md` fallback — "deferred: GitHub Releases API is primary source"
- Show per-repo current→target with changelog — "partial: changelog fetched per-package, not per-repo"
- Cache changelogs in SQLite — "deferred"

- [ ] **Step 3: Commit**

```bash
git add PLANNING.md
git commit -m "docs(planning): mark Phase 4 package intelligence items as implemented"
```

---

## Execution Summary

| Task | Description | Est. Complexity |
|------|-------------|-----------------|
| 1 | Dependency matrix command + CSV export | Standard |
| 2 | Frontend type updates | Simple |
| 3 | Changelog fetching (GitHub Releases + npm registry) | Standard |
| 4 | Packages Pinia store | Standard |
| 5 | Packages.vue matrix UI | Standard |
| 6 | Update PLANNING.md | Simple |

**Total: 6 tasks**. Simpler than Phase 3 since the scan data already exists in SQLite. Main complexity is in the changelog fetching (npm→GitHub mapping) and the matrix table UI.
