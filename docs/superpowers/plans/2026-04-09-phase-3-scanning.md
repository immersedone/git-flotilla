# Phase 3: Scanning — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the repo scanning engine — single repo scanning with monorepo-aware manifest discovery, dependency extraction, version detection, health scoring, and batch scanning of repo lists with progress tracking.

**Architecture:** The GitHub API Contents/Trees endpoints fetch repo file structures remotely (no cloning). A `services/scanner.rs` module orchestrates scanning by calling `GitHubClient` methods, parsing manifests, computing health scores, and persisting results to SQLite. Batch scanning uses a tokio semaphore for concurrency control. A Tauri event channel streams progress to the frontend. The frontend `Scanner.vue` provides scan triggers, live progress, and results display.

**Tech Stack:** Rust (tokio, reqwest, serde_json, sqlx, base64), TypeScript (Vue 3, Pinia, Tauri events)

**Scope:** Phase 3.1 (Single Repo Scanner) + Phase 3.2 (Batch Scanner). Deferred: 3.3 (Scheduled Scans), 3.4 (Fingerprint Profiles).

---

## File Structure

### New files to create:
- `src-tauri/src/services/scanner.rs` — Core scanning business logic (manifest discovery, parsing, version detection, health scoring)
- `src-tauri/src/services/content_fetcher.rs` — GitHub/GitLab content fetching abstraction (get tree, get file contents, list branches)
- `src/components/scan/ScanProgress.vue` — Progress bar component for batch scans
- `src/components/scan/ScanResultCard.vue` — Individual scan result display card
- `src/components/scan/ScanResultsTable.vue` — Table of scan results with sorting/filtering

### Files to modify:
- `src-tauri/src/services/github.rs` — Add `get_repo_tree()`, `get_file_content()`, `list_branches()` methods
- `src-tauri/src/services/mod.rs` — Register new modules
- `src-tauri/src/commands/scan.rs` — Replace stubs with real implementations
- `src-tauri/src/commands/repos.rs` — Add `update_last_scanned_at()` helper (called after scan)
- `src/stores/scans.ts` — Add action methods (scanRepo, scanRepoList, loadResults, etc.)
- `src/services/scan.ts` — Add scan progress event listener type
- `src/types/scan.ts` — Add `ScanProgress`, `BatchScanStatus` types
- `src/views/Scanner.vue` — Full scanning UI

---

## Task 1: Extend GitHubClient with Content Fetching Methods

Add three new methods to `GitHubClient` for fetching repo contents remotely via the GitHub API. These are the foundation for all scanning — no git clone needed.

**Files:**
- Modify: `src-tauri/src/services/github.rs`

- [ ] **Step 1: Write tests for the new response structs**

Add to the `#[cfg(test)]` module at the bottom of `github.rs`:

```rust
#[test]
fn deserialize_tree_entry() {
    let json = r#"{"path":"src/main.rs","mode":"100644","type":"blob","sha":"abc123","size":1234}"#;
    let entry: GitHubTreeEntry = serde_json::from_str(json).expect("parse");
    assert_eq!(entry.path, "src/main.rs");
    assert_eq!(entry.entry_type, "blob");
    assert_eq!(entry.size, Some(1234));
}

#[test]
fn deserialize_tree_response() {
    let json = r#"{"sha":"abc","url":"https://example.com","tree":[{"path":"README.md","mode":"100644","type":"blob","sha":"def","size":100}],"truncated":false}"#;
    let resp: GitHubTreeResponse = serde_json::from_str(json).expect("parse");
    assert_eq!(resp.tree.len(), 1);
    assert!(!resp.truncated);
}

#[test]
fn deserialize_content_response() {
    let json = r#"{"name":"package.json","path":"package.json","sha":"abc","size":200,"encoding":"base64","content":"eyJuYW1lIjoidGVzdCJ9"}"#;
    let resp: GitHubContentResponse = serde_json::from_str(json).expect("parse");
    assert_eq!(resp.encoding, "base64");
    assert_eq!(resp.content, "eyJuYW1lIjoidGVzdCJ9");
}

#[test]
fn deserialize_branch_response() {
    let json = r#"[{"name":"main","protected":true},{"name":"develop","protected":false}]"#;
    let branches: Vec<GitHubBranch> = serde_json::from_str(json).expect("parse");
    assert_eq!(branches.len(), 2);
    assert_eq!(branches[1].name, "develop");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --manifest-path src-tauri/Cargo.toml -- github`
Expected: FAIL — `GitHubTreeEntry`, `GitHubTreeResponse`, `GitHubContentResponse`, `GitHubBranch` not defined

- [ ] **Step 3: Add response structs**

Add after the existing response models section (after `GitHubOrg`):

```rust
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
```

- [ ] **Step 4: Add `get_repo_tree` method to `GitHubClient`**

```rust
/// Fetch the full recursive tree for a repo at a given ref (branch/SHA).
/// Returns all file paths in the repo without downloading content.
pub async fn get_repo_tree(
    &self,
    owner: &str,
    repo: &str,
    tree_sha: &str,
) -> AppResult<(GitHubTreeResponse, Option<RateLimitSnapshot>)> {
    let path = format!("/repos/{owner}/{repo}/git/trees/{tree_sha}?recursive=1");
    let (body, headers) = self.get::<GitHubTreeResponse>(&path).await?;
    let rl = extract_rate_limit(&headers);
    Ok((body, rl))
}
```

- [ ] **Step 5: Add `get_file_content` method to `GitHubClient`**

```rust
/// Fetch a single file's content (base64-encoded) from a repo.
/// The `path` is relative to the repo root, e.g. "package.json" or "frontend/package.json".
pub async fn get_file_content(
    &self,
    owner: &str,
    repo: &str,
    file_path: &str,
    git_ref: &str,
) -> AppResult<(GitHubContentResponse, Option<RateLimitSnapshot>)> {
    let path = format!(
        "/repos/{owner}/{repo}/contents/{file_path}?ref={git_ref}"
    );
    let (body, headers) = self.get::<GitHubContentResponse>(&path).await?;
    let rl = extract_rate_limit(&headers);
    Ok((body, rl))
}
```

- [ ] **Step 6: Add `list_branches` method to `GitHubClient`**

```rust
/// Fetch branch list for a repo (paginated, returns all).
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
        if done { break; }
        page += 1;
    }
    Ok((all, last_rl))
}
```

- [ ] **Step 7: Add `decode_base64_content` helper**

Add as a standalone function (not a method) near the pure helpers section:

```rust
/// Decode base64 content from GitHub API responses.
/// GitHub returns base64 with embedded newlines that must be stripped first.
pub fn decode_base64_content(encoded: &str) -> AppResult<String> {
    use base64::Engine;
    let cleaned = encoded.replace('\n', "");
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&cleaned)
        .map_err(|e| AppError::GitHub(format!("Base64 decode failed: {e}")))?;
    String::from_utf8(bytes)
        .map_err(|e| AppError::GitHub(format!("UTF-8 decode failed: {e}")))
}
```

- [ ] **Step 8: Add `base64` crate dependency**

Add to `src-tauri/Cargo.toml` under `[dependencies]`:
```toml
base64 = "0.22"
```

- [ ] **Step 9: Add test for `decode_base64_content`**

```rust
#[test]
fn decode_base64_valid() {
    let encoded = base64::engine::general_purpose::STANDARD.encode(b"{\"name\":\"test\"}");
    let decoded = decode_base64_content(&encoded).expect("decode");
    assert_eq!(decoded, "{\"name\":\"test\"}");
}

#[test]
fn decode_base64_with_newlines() {
    // GitHub inserts newlines every 60 chars
    let encoded = "eyJuYW1l\nIjoidGVz\ndCJ9";
    let decoded = decode_base64_content(encoded).expect("decode");
    assert_eq!(decoded, "{\"name\":\"test\"}");
}
```

- [ ] **Step 10: Run all tests to verify they pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml -- github`
Expected: All tests pass (existing 4 + new 6 = 10 tests)

- [ ] **Step 11: Commit**

```bash
git add src-tauri/src/services/github.rs src-tauri/Cargo.toml
git commit -m "feat(scan): add GitHub tree, content, and branch fetching methods"
```

---

## Task 2: Create Scanner Service — Manifest Discovery

The scanner service discovers all manifest files in a repo by fetching the recursive tree and filtering for known filenames. This is the foundation — later tasks add parsing logic.

**Files:**
- Create: `src-tauri/src/services/scanner.rs`
- Modify: `src-tauri/src/services/mod.rs`

- [ ] **Step 1: Write tests for manifest discovery logic**

Create `src-tauri/src/services/scanner.rs`:

```rust
use crate::error::{AppError, AppResult};
use crate::services::github::{GitHubTreeEntry, GitHubTreeResponse};

/// Directories to exclude when searching for manifests.
const EXCLUDED_DIRS: &[&str] = &[
    "node_modules/",
    "vendor/",
    "dist/",
    "build/",
    ".next/",
    ".nuxt/",
    ".cache/",
    ".git/",
    "__pycache__/",
    "target/",  // Rust build output
];

/// Manifest filenames we look for.
const MANIFEST_FILES: &[&str] = &[
    "package.json",
    "composer.json",
    "requirements.txt",
    "Cargo.toml",
    "go.mod",
];

/// Filter a tree response to find all manifest file paths, excluding vendor directories.
pub fn discover_manifests(tree: &GitHubTreeResponse) -> Vec<String> {
    tree.tree
        .iter()
        .filter(|entry| entry.entry_type == "blob")
        .filter(|entry| {
            let filename = entry.path.rsplit('/').next().unwrap_or(&entry.path);
            MANIFEST_FILES.contains(&filename)
        })
        .filter(|entry| {
            !EXCLUDED_DIRS.iter().any(|dir| entry.path.contains(dir))
        })
        .map(|entry| entry.path.clone())
        .collect()
}

/// Find workflow files (.github/workflows/*.yml and *.yaml).
pub fn discover_workflows(tree: &GitHubTreeResponse) -> Vec<String> {
    tree.tree
        .iter()
        .filter(|entry| entry.entry_type == "blob")
        .filter(|entry| {
            entry.path.starts_with(".github/workflows/")
                && (entry.path.ends_with(".yml") || entry.path.ends_with(".yaml"))
        })
        .map(|entry| entry.path.clone())
        .collect()
}

/// Check whether a specific file exists in the tree.
pub fn file_exists(tree: &GitHubTreeResponse, path: &str) -> bool {
    tree.tree.iter().any(|e| e.entry_type == "blob" && e.path == path)
}

/// Detect which lockfile exists to determine the package manager.
/// Returns (package_manager_name, lockfile_path).
pub fn detect_package_manager(tree: &GitHubTreeResponse) -> Option<(&'static str, String)> {
    // Check in priority order: pnpm > yarn > bun > npm
    let lockfiles: &[(&str, &str)] = &[
        ("pnpm",  "pnpm-lock.yaml"),
        ("yarn",  "yarn.lock"),
        ("bun",   "bun.lockb"),
        ("npm",   "package-lock.json"),
    ];
    for (pm, lockfile) in lockfiles {
        if let Some(entry) = tree.tree.iter().find(|e| {
            e.entry_type == "blob"
                && e.path.rsplit('/').next().unwrap_or(&e.path) == *lockfile
                // Only match root-level lockfile (no '/' in path, or just one level)
                && !e.path.contains('/')
        }) {
            return Some((pm, entry.path.clone()));
        }
    }
    None
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::github::{GitHubTreeEntry, GitHubTreeResponse};

    fn make_tree(paths: &[&str]) -> GitHubTreeResponse {
        GitHubTreeResponse {
            sha: "abc".to_string(),
            url: "https://example.com".to_string(),
            tree: paths
                .iter()
                .map(|p| GitHubTreeEntry {
                    path: p.to_string(),
                    mode: "100644".to_string(),
                    entry_type: "blob".to_string(),
                    sha: "def".to_string(),
                    size: Some(100),
                })
                .collect(),
            truncated: false,
        }
    }

    #[test]
    fn discover_manifests_finds_root_and_nested() {
        let tree = make_tree(&[
            "package.json",
            "frontend/package.json",
            "apps/web/package.json",
            "README.md",
        ]);
        let manifests = discover_manifests(&tree);
        assert_eq!(manifests, vec![
            "package.json",
            "frontend/package.json",
            "apps/web/package.json",
        ]);
    }

    #[test]
    fn discover_manifests_excludes_vendor_dirs() {
        let tree = make_tree(&[
            "package.json",
            "node_modules/lodash/package.json",
            "vendor/autoload/composer.json",
            "dist/package.json",
            "frontend/package.json",
        ]);
        let manifests = discover_manifests(&tree);
        assert_eq!(manifests, vec!["package.json", "frontend/package.json"]);
    }

    #[test]
    fn discover_manifests_finds_all_ecosystems() {
        let tree = make_tree(&[
            "package.json",
            "backend/composer.json",
            "ml/requirements.txt",
            "tools/Cargo.toml",
            "proxy/go.mod",
        ]);
        let manifests = discover_manifests(&tree);
        assert_eq!(manifests.len(), 5);
    }

    #[test]
    fn discover_workflows_finds_yml_and_yaml() {
        let tree = make_tree(&[
            ".github/workflows/ci.yml",
            ".github/workflows/deploy.yaml",
            ".github/CODEOWNERS",
            "scripts/build.yml",
        ]);
        let wf = discover_workflows(&tree);
        assert_eq!(wf, vec![
            ".github/workflows/ci.yml",
            ".github/workflows/deploy.yaml",
        ]);
    }

    #[test]
    fn file_exists_works() {
        let tree = make_tree(&[".env.example", "CODEOWNERS", "src/main.rs"]);
        assert!(file_exists(&tree, ".env.example"));
        assert!(file_exists(&tree, "CODEOWNERS"));
        assert!(!file_exists(&tree, "SECURITY.md"));
    }

    #[test]
    fn detect_package_manager_pnpm() {
        let tree = make_tree(&["package.json", "pnpm-lock.yaml"]);
        let (pm, _) = detect_package_manager(&tree).expect("found");
        assert_eq!(pm, "pnpm");
    }

    #[test]
    fn detect_package_manager_npm_fallback() {
        let tree = make_tree(&["package.json", "package-lock.json"]);
        let (pm, _) = detect_package_manager(&tree).expect("found");
        assert_eq!(pm, "npm");
    }

    #[test]
    fn detect_package_manager_none_without_lockfile() {
        let tree = make_tree(&["package.json", "src/index.ts"]);
        assert!(detect_package_manager(&tree).is_none());
    }

    #[test]
    fn detect_package_manager_ignores_nested_lockfiles() {
        let tree = make_tree(&["package.json", "apps/web/pnpm-lock.yaml"]);
        assert!(detect_package_manager(&tree).is_none());
    }
}
```

- [ ] **Step 2: Register the scanner module**

In `src-tauri/src/services/mod.rs`, add:
```rust
pub mod scanner;
```

- [ ] **Step 3: Run tests to verify they pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml -- scanner`
Expected: All 9 tests pass

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/services/scanner.rs src-tauri/src/services/mod.rs
git commit -m "feat(scan): add manifest discovery and tree analysis functions"
```

---

## Task 3: Scanner Service — Manifest Parsing

Parse `package.json` and `composer.json` to extract dependencies. Other ecosystems (`requirements.txt`, `Cargo.toml`, `go.mod`) use simpler line-based parsing.

**Files:**
- Modify: `src-tauri/src/services/scanner.rs`

- [ ] **Step 1: Write tests for package.json parsing**

Add to scanner.rs tests module:

```rust
#[test]
fn parse_package_json_extracts_deps() {
    let content = r#"{
        "name": "my-app",
        "dependencies": {
            "vue": "^3.4.0",
            "axios": "1.7.2"
        },
        "devDependencies": {
            "vite": "^5.0.0",
            "typescript": "~5.3.0"
        }
    }"#;
    let packages = parse_package_json(content, "github:org/repo");
    assert_eq!(packages.len(), 4);
    let vue = packages.iter().find(|p| p.name == "vue").unwrap();
    assert_eq!(vue.version, "^3.4.0");
    assert!(!vue.is_dev);
    assert_eq!(vue.ecosystem, "npm");
    let vite = packages.iter().find(|p| p.name == "vite").unwrap();
    assert!(vite.is_dev);
}

#[test]
fn parse_package_json_handles_no_deps() {
    let content = r#"{"name": "empty-project"}"#;
    let packages = parse_package_json(content, "github:org/repo");
    assert!(packages.is_empty());
}

#[test]
fn parse_package_json_extracts_engines_node() {
    let content = r#"{"name":"app","engines":{"node":">=20.0.0"}}"#;
    let node_ver = extract_engines_node(content);
    assert_eq!(node_ver, Some(">=20.0.0".to_string()));
}

#[test]
fn parse_package_json_extracts_package_manager_field() {
    let content = r#"{"name":"app","packageManager":"pnpm@9.1.0"}"#;
    let pm = extract_package_manager_field(content);
    assert_eq!(pm, Some(("pnpm".to_string(), "9.1.0".to_string())));
}

#[test]
fn parse_composer_json_extracts_deps() {
    let content = r#"{
        "require": {
            "php": "^8.2",
            "laravel/framework": "^11.0"
        },
        "require-dev": {
            "phpunit/phpunit": "^11.0"
        }
    }"#;
    let packages = parse_composer_json(content, "github:org/repo");
    // "php" is not a package — it's a version constraint
    assert_eq!(packages.len(), 2);
    let laravel = packages.iter().find(|p| p.name == "laravel/framework").unwrap();
    assert_eq!(laravel.ecosystem, "composer");
    assert!(!laravel.is_dev);
}

#[test]
fn parse_composer_json_extracts_php_version() {
    let content = r#"{"require":{"php":"^8.2","ext-mbstring":"*"}}"#;
    let php_ver = extract_php_version(content);
    assert_eq!(php_ver, Some("^8.2".to_string()));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --manifest-path src-tauri/Cargo.toml -- scanner`
Expected: Compilation fails — `parse_package_json`, `parse_composer_json`, etc. not defined

- [ ] **Step 3: Implement package.json parsing**

Add to `scanner.rs` (above the tests module):

```rust
use crate::models::RepoPackage;

/// Parse a package.json string and extract all dependencies.
pub fn parse_package_json(content: &str, repo_id: &str) -> Vec<RepoPackage> {
    let now = chrono::Utc::now().to_rfc3339();
    let parsed: serde_json::Value = match serde_json::from_str(content) {
        Ok(v) => v,
        Err(_) => return vec![],
    };

    let mut packages = Vec::new();

    if let Some(deps) = parsed.get("dependencies").and_then(|d| d.as_object()) {
        for (name, version) in deps {
            if let Some(ver) = version.as_str() {
                packages.push(RepoPackage {
                    repo_id: repo_id.to_string(),
                    ecosystem: "npm".to_string(),
                    name: name.clone(),
                    version: ver.to_string(),
                    is_dev: false,
                    scanned_at: now.clone(),
                });
            }
        }
    }

    if let Some(deps) = parsed.get("devDependencies").and_then(|d| d.as_object()) {
        for (name, version) in deps {
            if let Some(ver) = version.as_str() {
                packages.push(RepoPackage {
                    repo_id: repo_id.to_string(),
                    ecosystem: "npm".to_string(),
                    name: name.clone(),
                    version: ver.to_string(),
                    is_dev: true,
                    scanned_at: now.clone(),
                });
            }
        }
    }

    packages
}

/// Extract `engines.node` from package.json content.
pub fn extract_engines_node(content: &str) -> Option<String> {
    let parsed: serde_json::Value = serde_json::from_str(content).ok()?;
    parsed
        .get("engines")?
        .get("node")?
        .as_str()
        .map(|s| s.to_string())
}

/// Extract `packageManager` field (e.g. "pnpm@9.1.0") → ("pnpm", "9.1.0").
pub fn extract_package_manager_field(content: &str) -> Option<(String, String)> {
    let parsed: serde_json::Value = serde_json::from_str(content).ok()?;
    let field = parsed.get("packageManager")?.as_str()?;
    let (name, version) = field.split_once('@')?;
    Some((name.to_string(), version.to_string()))
}

/// Parse a composer.json string and extract all dependencies (excluding "php" and "ext-*").
pub fn parse_composer_json(content: &str, repo_id: &str) -> Vec<RepoPackage> {
    let now = chrono::Utc::now().to_rfc3339();
    let parsed: serde_json::Value = match serde_json::from_str(content) {
        Ok(v) => v,
        Err(_) => return vec![],
    };

    let mut packages = Vec::new();

    let is_package = |name: &str| -> bool {
        name != "php" && !name.starts_with("ext-") && name.contains('/')
    };

    if let Some(deps) = parsed.get("require").and_then(|d| d.as_object()) {
        for (name, version) in deps {
            if is_package(name) {
                if let Some(ver) = version.as_str() {
                    packages.push(RepoPackage {
                        repo_id: repo_id.to_string(),
                        ecosystem: "composer".to_string(),
                        name: name.clone(),
                        version: ver.to_string(),
                        is_dev: false,
                        scanned_at: now.clone(),
                    });
                }
            }
        }
    }

    if let Some(deps) = parsed.get("require-dev").and_then(|d| d.as_object()) {
        for (name, version) in deps {
            if is_package(name) {
                if let Some(ver) = version.as_str() {
                    packages.push(RepoPackage {
                        repo_id: repo_id.to_string(),
                        ecosystem: "composer".to_string(),
                        name: name.clone(),
                        version: ver.to_string(),
                        is_dev: true,
                        scanned_at: now.clone(),
                    });
                }
            }
        }
    }

    packages
}

/// Extract PHP version constraint from composer.json `require.php`.
pub fn extract_php_version(content: &str) -> Option<String> {
    let parsed: serde_json::Value = serde_json::from_str(content).ok()?;
    parsed
        .get("require")?
        .get("php")?
        .as_str()
        .map(|s| s.to_string())
}
```

- [ ] **Step 4: Add `chrono` dependency to Cargo.toml**

Add to `src-tauri/Cargo.toml` under `[dependencies]`:
```toml
chrono = { version = "0.4", features = ["serde"] }
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml -- scanner`
Expected: All scanner tests pass

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/services/scanner.rs src-tauri/Cargo.toml
git commit -m "feat(scan): add package.json and composer.json parsing"
```

---

## Task 4: Scanner Service — Node Version Detection

Detect Node.js version from multiple sources in priority order: `.nvmrc` > `.node-version` > `.tool-versions` > `package.json#engines.node`. Record the source for display.

**Files:**
- Modify: `src-tauri/src/services/scanner.rs`

- [ ] **Step 1: Write tests for Node version detection**

Add to scanner.rs tests:

```rust
#[test]
fn detect_node_from_nvmrc() {
    let files = vec![
        (".nvmrc", "20.11.0\n"),
        ("package.json", r#"{"engines":{"node":">=18"}}"#),
    ];
    let (ver, source) = detect_node_version(&files).unwrap();
    assert_eq!(ver, "20.11.0");
    assert_eq!(source, ".nvmrc");
}

#[test]
fn detect_node_from_node_version_file() {
    let files = vec![
        (".node-version", "v22.1.0"),
    ];
    let (ver, source) = detect_node_version(&files).unwrap();
    assert_eq!(ver, "22.1.0");
    assert_eq!(source, ".node-version");
}

#[test]
fn detect_node_from_tool_versions() {
    let files = vec![
        (".tool-versions", "nodejs 20.11.0\npnpm 9.0.0\n"),
    ];
    let (ver, source) = detect_node_version(&files).unwrap();
    assert_eq!(ver, "20.11.0");
    assert_eq!(source, ".tool-versions");
}

#[test]
fn detect_node_from_engines_fallback() {
    let files = vec![
        ("package.json", r#"{"engines":{"node":">=20.0.0"}}"#),
    ];
    let (ver, source) = detect_node_version(&files).unwrap();
    assert_eq!(ver, ">=20.0.0");
    assert_eq!(source, "engines.node");
}

#[test]
fn detect_node_none_when_missing() {
    let files: Vec<(&str, &str)> = vec![
        ("package.json", r#"{"name":"test"}"#),
    ];
    assert!(detect_node_version(&files).is_none());
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --manifest-path src-tauri/Cargo.toml -- scanner::tests::detect_node`
Expected: FAIL — `detect_node_version` not defined

- [ ] **Step 3: Implement `detect_node_version`**

Add to `scanner.rs`:

```rust
/// Detect Node.js version from available file contents.
/// Accepts a slice of (filename, content) pairs.
/// Returns (version, source) where source is the filename or "engines.node".
/// Priority: .nvmrc > .node-version > .tool-versions > package.json engines.node
pub fn detect_node_version(files: &[(&str, &str)]) -> Option<(String, String)> {
    // 1. .nvmrc
    if let Some((_, content)) = files.iter().find(|(name, _)| *name == ".nvmrc") {
        let ver = content.trim().strip_prefix('v').unwrap_or(content.trim());
        if !ver.is_empty() {
            return Some((ver.to_string(), ".nvmrc".to_string()));
        }
    }

    // 2. .node-version
    if let Some((_, content)) = files.iter().find(|(name, _)| *name == ".node-version") {
        let ver = content.trim().strip_prefix('v').unwrap_or(content.trim());
        if !ver.is_empty() {
            return Some((ver.to_string(), ".node-version".to_string()));
        }
    }

    // 3. .tool-versions
    if let Some((_, content)) = files.iter().find(|(name, _)| *name == ".tool-versions") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 && (parts[0] == "nodejs" || parts[0] == "node") {
                let ver = parts[1].strip_prefix('v').unwrap_or(parts[1]);
                return Some((ver.to_string(), ".tool-versions".to_string()));
            }
        }
    }

    // 4. package.json engines.node
    if let Some((_, content)) = files.iter().find(|(name, _)| *name == "package.json") {
        if let Some(ver) = extract_engines_node(content) {
            return Some((ver, "engines.node".to_string()));
        }
    }

    None
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml -- scanner`
Expected: All tests pass

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/services/scanner.rs
git commit -m "feat(scan): add Node.js version detection with priority chain"
```

---

## Task 5: Scanner Service — Health Score Computation

Compute a 0–100 health score based on the configurable rules from PLANNING.md. Also detect floating Action tags in workflow files.

**Files:**
- Modify: `src-tauri/src/services/scanner.rs`

- [ ] **Step 1: Write tests for floating action tag detection**

Add to scanner.rs tests:

```rust
#[test]
fn detect_floating_action_tags_finds_unpinned() {
    let workflow = r#"
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
      - uses: company/custom-action@abc123def456
    "#;
    let floating = detect_floating_action_tags(workflow);
    assert_eq!(floating.len(), 2);
    assert!(floating.contains(&"actions/checkout@v4".to_string()));
    assert!(floating.contains(&"actions/setup-node@v4".to_string()));
}

#[test]
fn detect_floating_action_tags_allows_pinned_sha() {
    let workflow = r#"
    steps:
      - uses: actions/checkout@a5ac7e51b41094c92402da3b24376905380afc29
    "#;
    let floating = detect_floating_action_tags(workflow);
    assert!(floating.is_empty());
}

#[test]
fn health_score_perfect_repo() {
    let input = HealthScoreInput {
        has_codeowners: true,
        has_security_md: true,
        has_dot_env_example: true,
        has_editorconfig: true,
        floating_action_count: 0,
        has_known_cves: false,
        node_version_current: true,
    };
    let (score, _flags) = compute_health_score(&input);
    assert_eq!(score, 100);
}

#[test]
fn health_score_missing_everything() {
    let input = HealthScoreInput {
        has_codeowners: false,
        has_security_md: false,
        has_dot_env_example: false,
        has_editorconfig: false,
        floating_action_count: 3,
        has_known_cves: true,
        node_version_current: false,
    };
    let (score, flags) = compute_health_score(&input);
    assert_eq!(score, 0);
    assert!(!flags.is_empty());
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --manifest-path src-tauri/Cargo.toml -- scanner::tests::detect_floating`
Expected: FAIL

- [ ] **Step 3: Implement floating action tag detection**

```rust
/// Detect GitHub Actions `uses:` lines with floating version tags (e.g. @v4, @main)
/// instead of pinned SHA commits. Returns the action references that are floating.
pub fn detect_floating_action_tags(workflow_content: &str) -> Vec<String> {
    let mut floating = Vec::new();
    for line in workflow_content.lines() {
        let trimmed = line.trim();
        if let Some(uses_val) = trimmed.strip_prefix("- uses:").or_else(|| trimmed.strip_prefix("uses:")) {
            let action = uses_val.trim().trim_matches('"').trim_matches('\'');
            if let Some((_action_name, ref_part)) = action.split_once('@') {
                // A pinned SHA is 40 hex chars
                let is_pinned_sha = ref_part.len() == 40 && ref_part.chars().all(|c| c.is_ascii_hexdigit());
                if !is_pinned_sha {
                    floating.push(action.to_string());
                }
            }
        }
    }
    floating
}
```

- [ ] **Step 4: Implement health score computation**

```rust
use crate::models::ScanFlag;

/// Input for health score calculation. Each field maps to a scoring rule.
pub struct HealthScoreInput {
    pub has_codeowners: bool,
    pub has_security_md: bool,
    pub has_dot_env_example: bool,
    pub has_editorconfig: bool,
    pub floating_action_count: usize,
    pub has_known_cves: bool,
    pub node_version_current: bool,
}

/// Compute health score (0–100) and generate flags for issues found.
pub fn compute_health_score(input: &HealthScoreInput) -> (u32, Vec<ScanFlag>) {
    let mut score: i32 = 0;
    let mut flags = Vec::new();

    // Has CODEOWNERS (+10)
    if input.has_codeowners {
        score += 10;
    } else {
        flags.push(ScanFlag {
            flag_type: "missing_codeowners".to_string(),
            message: "Missing CODEOWNERS file".to_string(),
            severity: "low".to_string(),
        });
    }

    // Has SECURITY.md (+10)
    if input.has_security_md {
        score += 10;
    } else {
        flags.push(ScanFlag {
            flag_type: "missing_security_md".to_string(),
            message: "Missing SECURITY.md file".to_string(),
            severity: "low".to_string(),
        });
    }

    // Has .env.example (+5)
    if input.has_dot_env_example {
        score += 5;
    } else {
        flags.push(ScanFlag {
            flag_type: "missing_env_example".to_string(),
            message: "Missing .env.example file".to_string(),
            severity: "info".to_string(),
        });
    }

    // Has .editorconfig (+5)
    if input.has_editorconfig {
        score += 5;
    } else {
        flags.push(ScanFlag {
            flag_type: "missing_editorconfig".to_string(),
            message: "Missing .editorconfig file".to_string(),
            severity: "info".to_string(),
        });
    }

    // No floating action tags (+15)
    if input.floating_action_count == 0 {
        score += 15;
    } else {
        flags.push(ScanFlag {
            flag_type: "floating_action_tags".to_string(),
            message: format!("{} GitHub Action(s) using floating version tags instead of pinned SHAs", input.floating_action_count),
            severity: "medium".to_string(),
        });
    }

    // No known CVEs (+20)
    if !input.has_known_cves {
        score += 20;
    } else {
        flags.push(ScanFlag {
            flag_type: "known_cves".to_string(),
            message: "Repository has known CVE vulnerabilities".to_string(),
            severity: "high".to_string(),
        });
    }

    // Node version not EOL (+15)
    if input.node_version_current {
        score += 15;
    } else {
        flags.push(ScanFlag {
            flag_type: "node_eol".to_string(),
            message: "Node.js version may be end-of-life".to_string(),
            severity: "medium".to_string(),
        });
    }

    // Dependencies up to date (+20) — awarded by default for now; Phase 4 will check properly
    score += 20;

    let clamped = score.clamp(0, 100) as u32;
    (clamped, flags)
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml -- scanner`
Expected: All tests pass

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/services/scanner.rs
git commit -m "feat(scan): add health score computation and floating action tag detection"
```

---

## Task 6: Implement `scan_repo` Command — Full Single Repo Scan

Wire up the scanner service functions into a complete `scan_repo` Tauri command that fetches the tree, discovers manifests, fetches and parses each manifest, detects versions, computes health score, and persists everything to SQLite.

**Files:**
- Modify: `src-tauri/src/commands/scan.rs`

- [ ] **Step 1: Replace the stub scan_repo with the real implementation**

Replace the entire `src-tauri/src/commands/scan.rs` file:

```rust
use crate::{
    db,
    error::{AppError, AppResult},
    models::{RepoPackage, ScanResult},
    services::{
        github::{decode_base64_content, GitHubClient},
        scanner::{
            self, compute_health_score, detect_floating_action_tags, detect_node_version,
            detect_package_manager, discover_manifests, discover_workflows,
            extract_package_manager_field, extract_php_version, file_exists,
            parse_composer_json, parse_package_json, HealthScoreInput,
        },
    },
};
use crate::commands::auth::KEYCHAIN_SERVICE;
use keyring::Entry;
use sqlx::Row;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

// ── Helpers ────────────────────────────────────────────────────────────────

fn get_github_token(repo_id: &str) -> AppResult<String> {
    // repo_id format: "github:{owner}/{name}" — extract provider
    let provider = repo_id.split(':').next().unwrap_or("github");
    if provider != "github" {
        return Err(AppError::Operation(format!("Unsupported provider: {provider}")));
    }

    let pool_result = db::pool();
    // Try to find a GitHub account in the accounts table
    let token = tauri::async_runtime::block_on(async {
        let pool = pool_result?;
        let row = sqlx::query("SELECT id FROM accounts WHERE provider = 'github' LIMIT 1")
            .fetch_optional(pool)
            .await?;
        match row {
            Some(row) => {
                let account_id: String = row.get("id");
                Entry::new(KEYCHAIN_SERVICE, &account_id)
                    .map_err(AppError::from)?
                    .get_password()
                    .map_err(AppError::from)
            }
            None => Err(AppError::Auth("No GitHub account configured".into())),
        }
    })?;
    Ok(token)
}

/// Fetch a file's decoded content from GitHub. Returns None if file doesn't exist (404).
async fn fetch_file_content(
    client: &GitHubClient,
    owner: &str,
    repo: &str,
    file_path: &str,
    git_ref: &str,
) -> AppResult<Option<String>> {
    match client.get_file_content(owner, repo, file_path, git_ref).await {
        Ok((resp, rl)) => {
            if let Some(snapshot) = rl {
                crate::services::rate_limiter::update_github(snapshot);
            }
            let content = decode_base64_content(&resp.content)?;
            Ok(Some(content))
        }
        Err(AppError::NotFound(_)) => Ok(None),
        Err(e) => Err(e),
    }
}

// ── Commands ───────────────────────────────────────────────────────────────

/// Scan a single repo: fetch tree, discover manifests, parse dependencies,
/// detect versions, compute health score, persist to SQLite.
#[tauri::command]
pub async fn scan_repo(repo_id: String) -> AppResult<ScanResult> {
    let pool = db::pool()?;

    // Fetch repo metadata from DB
    let repo_row = sqlx::query(
        "SELECT provider, owner, name, default_branch FROM repos WHERE id = ?",
    )
    .bind(&repo_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Repo not found: {repo_id}")))?;

    let provider: String = repo_row.get("provider");
    let owner: String = repo_row.get("owner");
    let name: String = repo_row.get("name");
    let default_branch: String = repo_row.get("default_branch");

    if provider != "github" {
        return Err(AppError::Operation(format!("Unsupported provider: {provider}. Only GitHub is supported.")));
    }

    // Get token for the provider
    let account_row = sqlx::query("SELECT id FROM accounts WHERE provider = ? LIMIT 1")
        .bind(&provider)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::Auth(format!("No {provider} account configured")))?;
    let account_id: String = account_row.get("id");
    let token = Entry::new(KEYCHAIN_SERVICE, &account_id)
        .map_err(AppError::from)?
        .get_password()
        .map_err(AppError::from)?;

    let client = GitHubClient::new(&token);

    // 1. Fetch repo tree
    let (tree, tree_rl) = client.get_repo_tree(&owner, &name, &default_branch).await?;
    if let Some(rl) = tree_rl {
        crate::services::rate_limiter::update_github(rl);
    }

    // 2. Discover manifests and workflows
    let manifest_paths = discover_manifests(&tree);
    let workflow_files = discover_workflows(&tree);

    // 3. Detect package manager from lockfile
    let pm_from_lockfile = detect_package_manager(&tree);

    // 4. Check for develop branch
    let (branches, branch_rl) = client.list_branches(&owner, &name).await?;
    if let Some(rl) = branch_rl {
        crate::services::rate_limiter::update_github(rl);
    }
    let has_develop = branches.iter().any(|b| b.name == "develop");

    // 5. Get repo pushed_at from GitHub API
    let last_pushed: Option<String> = sqlx::query("SELECT last_scanned_at FROM repos WHERE id = ?")
        .bind(&repo_id)
        .fetch_optional(pool)
        .await?
        .and_then(|r| r.get("last_scanned_at"));

    // 6. Fetch and parse manifest files + version detection files
    let mut all_packages: Vec<RepoPackage> = Vec::new();
    let mut version_files: Vec<(String, String)> = Vec::new();

    // Fetch version detection files
    let version_file_paths = [".nvmrc", ".node-version", ".tool-versions"];
    for vf_path in &version_file_paths {
        if file_exists(&tree, vf_path) {
            if let Some(content) = fetch_file_content(&client, &owner, &name, vf_path, &default_branch).await? {
                version_files.push((vf_path.to_string(), content));
            }
        }
    }

    // Fetch and parse root package.json (needed for version detection + packages)
    let mut root_pkg_json: Option<String> = None;
    for manifest in &manifest_paths {
        if let Some(content) = fetch_file_content(&client, &owner, &name, manifest, &default_branch).await? {
            let filename = manifest.rsplit('/').next().unwrap_or(manifest);
            match filename {
                "package.json" => {
                    let pkgs = parse_package_json(&content, &repo_id);
                    all_packages.extend(pkgs);
                    // Keep root package.json for version detection
                    if !manifest.contains('/') {
                        root_pkg_json = Some(content.clone());
                    }
                }
                "composer.json" => {
                    let pkgs = parse_composer_json(&content, &repo_id);
                    all_packages.extend(pkgs);
                }
                // TODO: requirements.txt, Cargo.toml, go.mod parsing in future
                _ => {}
            }
        }
    }

    // 7. Detect Node version
    let mut detect_files: Vec<(&str, &str)> = version_files
        .iter()
        .map(|(name, content)| (name.as_str(), content.as_str()))
        .collect();
    if let Some(ref pkg_json) = root_pkg_json {
        detect_files.push(("package.json", pkg_json));
    }
    let node_detection = detect_node_version(&detect_files);
    let node_version = node_detection.as_ref().map(|(v, _)| v.clone());
    let node_version_source = node_detection.map(|(_, s)| s);

    // 8. Detect PHP version from root composer.json
    let php_version = manifest_paths
        .iter()
        .find(|p| !p.contains('/') && p.ends_with("composer.json"))
        .and_then(|_| {
            // Re-fetch if we didn't already parse it above — but we did
            // Check if any composer.json was at root
            None::<String>
        });
    // Actually extract from fetched content
    let php_version = {
        let root_composer = manifest_paths.iter().find(|p| *p == "composer.json");
        match root_composer {
            Some(_) => {
                match fetch_file_content(&client, &owner, &name, "composer.json", &default_branch).await? {
                    Some(content) => extract_php_version(&content),
                    None => None,
                }
            }
            None => None,
        }
    };

    // 9. Determine package manager + version
    let (package_manager, package_manager_version) = {
        let pm_name = pm_from_lockfile.as_ref().map(|(pm, _)| pm.to_string());
        let pm_version = root_pkg_json
            .as_deref()
            .and_then(extract_package_manager_field)
            .map(|(_, v)| v);
        (pm_name, pm_version)
    };

    // 10. Detect file presence
    let has_dot_env_example = file_exists(&tree, ".env.example");
    let has_codeowners = file_exists(&tree, "CODEOWNERS") || file_exists(&tree, ".github/CODEOWNERS");
    let has_security_md = file_exists(&tree, "SECURITY.md");
    let has_editorconfig = file_exists(&tree, ".editorconfig");

    // 11. Detect floating action tags in workflows
    let mut floating_action_count = 0usize;
    for wf in &workflow_files {
        if let Some(content) = fetch_file_content(&client, &owner, &name, wf, &default_branch).await? {
            floating_action_count += detect_floating_action_tags(&content).len();
        }
    }

    // 12. Auto-exclude repos without manifests
    let excluded = manifest_paths.is_empty();
    let exclude_reason = if excluded {
        Some("No package manifests found".to_string())
    } else {
        None
    };

    // 13. Compute health score
    let (health_score, flags) = compute_health_score(&HealthScoreInput {
        has_codeowners,
        has_security_md,
        has_dot_env_example,
        has_editorconfig,
        floating_action_count,
        has_known_cves: false, // Phase 5 will populate
        node_version_current: true, // TODO: check against EOL list
    });

    let now = chrono::Utc::now().to_rfc3339();

    // 14. Persist scan result
    let manifest_paths_json = serde_json::to_string(&manifest_paths).unwrap_or_else(|_| "[]".to_string());
    let workflow_files_json = serde_json::to_string(&workflow_files).unwrap_or_else(|_| "[]".to_string());
    let flags_json = serde_json::to_string(&flags).unwrap_or_else(|_| "[]".to_string());

    sqlx::query(
        r#"INSERT INTO scan_results
           (repo_id, scanned_at, manifest_paths, node_version, node_version_source,
            php_version, package_manager, package_manager_version, has_develop,
            last_pushed, has_dot_env_example, workflow_files, health_score, flags,
            excluded, exclude_reason)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
    )
    .bind(&repo_id)
    .bind(&now)
    .bind(&manifest_paths_json)
    .bind(&node_version)
    .bind(&node_version_source)
    .bind(&php_version)
    .bind(&package_manager)
    .bind(&package_manager_version)
    .bind(has_develop as i64)
    .bind(&last_pushed)
    .bind(has_dot_env_example as i64)
    .bind(&workflow_files_json)
    .bind(health_score as i64)
    .bind(&flags_json)
    .bind(excluded as i64)
    .bind(&exclude_reason)
    .execute(pool)
    .await?;

    // 15. Persist packages (delete old + insert new)
    sqlx::query("DELETE FROM repo_packages WHERE repo_id = ?")
        .bind(&repo_id)
        .execute(pool)
        .await?;

    for pkg in &all_packages {
        sqlx::query(
            "INSERT INTO repo_packages (repo_id, ecosystem, name, version, is_dev, scanned_at)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&pkg.repo_id)
        .bind(&pkg.ecosystem)
        .bind(&pkg.name)
        .bind(&pkg.version)
        .bind(pkg.is_dev as i64)
        .bind(&now)
        .execute(pool)
        .await?;
    }

    // 16. Update repos.last_scanned_at
    sqlx::query("UPDATE repos SET last_scanned_at = ?, updated_at = datetime('now') WHERE id = ?")
        .bind(&now)
        .bind(&repo_id)
        .execute(pool)
        .await?;

    tracing::info!("Scanned {} — {} manifests, {} packages, health={}", repo_id, manifest_paths.len(), all_packages.len(), health_score);

    Ok(ScanResult {
        repo_id,
        scanned_at: now,
        manifest_paths,
        node_version,
        node_version_source,
        php_version,
        package_manager,
        package_manager_version,
        has_develop,
        last_pushed,
        has_dot_env_example,
        workflow_files,
        health_score,
        flags,
        excluded,
        exclude_reason,
    })
}

/// Fetch latest scan result for a repo from SQLite.
#[tauri::command]
pub async fn get_scan_result(repo_id: String) -> AppResult<ScanResult> {
    let pool = db::pool()?;
    let row = sqlx::query(
        r#"SELECT repo_id, scanned_at, manifest_paths, node_version, node_version_source,
                  php_version, package_manager, package_manager_version, has_develop,
                  last_pushed, has_dot_env_example, workflow_files, health_score, flags,
                  excluded, exclude_reason
           FROM scan_results WHERE repo_id = ?
           ORDER BY scanned_at DESC LIMIT 1"#,
    )
    .bind(&repo_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("No scan results for repo: {repo_id}")))?;

    Ok(scan_row_to_model(&row))
}

/// List scan results, optionally filtered by repo list membership.
#[tauri::command]
pub async fn list_scan_results(repo_list_id: Option<String>) -> AppResult<Vec<ScanResult>> {
    let pool = db::pool()?;

    let rows = if let Some(list_id) = repo_list_id {
        sqlx::query(
            r#"SELECT sr.repo_id, sr.scanned_at, sr.manifest_paths, sr.node_version,
                      sr.node_version_source, sr.php_version, sr.package_manager,
                      sr.package_manager_version, sr.has_develop, sr.last_pushed,
                      sr.has_dot_env_example, sr.workflow_files, sr.health_score, sr.flags,
                      sr.excluded, sr.exclude_reason
               FROM scan_results sr
               INNER JOIN (
                   SELECT repo_id, MAX(scanned_at) as latest
                   FROM scan_results GROUP BY repo_id
               ) latest ON sr.repo_id = latest.repo_id AND sr.scanned_at = latest.latest
               INNER JOIN repo_list_members m ON m.repo_id = sr.repo_id
               WHERE m.list_id = ?
               ORDER BY sr.repo_id"#,
        )
        .bind(list_id)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query(
            r#"SELECT sr.repo_id, sr.scanned_at, sr.manifest_paths, sr.node_version,
                      sr.node_version_source, sr.php_version, sr.package_manager,
                      sr.package_manager_version, sr.has_develop, sr.last_pushed,
                      sr.has_dot_env_example, sr.workflow_files, sr.health_score, sr.flags,
                      sr.excluded, sr.exclude_reason
               FROM scan_results sr
               INNER JOIN (
                   SELECT repo_id, MAX(scanned_at) as latest
                   FROM scan_results GROUP BY repo_id
               ) latest ON sr.repo_id = latest.repo_id AND sr.scanned_at = latest.latest
               ORDER BY sr.repo_id"#,
        )
        .fetch_all(pool)
        .await?
    };

    Ok(rows.iter().map(scan_row_to_model).collect())
}

fn scan_row_to_model(row: &sqlx::sqlite::SqliteRow) -> ScanResult {
    ScanResult {
        repo_id: row.get("repo_id"),
        scanned_at: row.get("scanned_at"),
        manifest_paths: serde_json::from_str(row.get::<String, _>("manifest_paths").as_str()).unwrap_or_default(),
        node_version: row.get("node_version"),
        node_version_source: row.get("node_version_source"),
        php_version: row.get("php_version"),
        package_manager: row.get("package_manager"),
        package_manager_version: row.get("package_manager_version"),
        has_develop: row.get::<i64, _>("has_develop") != 0,
        last_pushed: row.get("last_pushed"),
        has_dot_env_example: row.get::<i64, _>("has_dot_env_example") != 0,
        workflow_files: serde_json::from_str(row.get::<String, _>("workflow_files").as_str()).unwrap_or_default(),
        health_score: row.get::<i64, _>("health_score") as u32,
        flags: serde_json::from_str(row.get::<String, _>("flags").as_str()).unwrap_or_default(),
        excluded: row.get::<i64, _>("excluded") != 0,
        exclude_reason: row.get("exclude_reason"),
    }
}

/// Placeholder for batch scan — implemented in Task 7.
#[tauri::command]
pub async fn scan_repo_list(list_id: String) -> AppResult<String> {
    let _ = list_id;
    Err(AppError::Operation("not implemented".into()))
}

/// Placeholder for abort — implemented in Task 7.
#[tauri::command]
pub async fn abort_scan(operation_id: String) -> AppResult<()> {
    let _ = operation_id;
    Err(AppError::Operation("not implemented".into()))
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: Compiles without errors

- [ ] **Step 3: Run all Rust tests to ensure no regressions**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: All existing + new tests pass

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/scan.rs
git commit -m "feat(scan): implement scan_repo, get_scan_result, list_scan_results commands"
```

---

## Task 7: Implement Batch Scanner with Progress Events

Scan an entire repo list in parallel using a tokio semaphore for concurrency control. Emit Tauri events for real-time progress tracking in the frontend.

**Files:**
- Modify: `src-tauri/src/commands/scan.rs`
- Modify: `src-tauri/src/main.rs` (add `tauri::AppHandle` state)

- [ ] **Step 1: Add scan progress event types**

Add to the top of `commands/scan.rs`:

```rust
use serde::Serialize;
use tauri::Emitter;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgressEvent {
    pub repo_id: String,
    pub status: String,     // "scanning" | "done" | "failed"
    pub current: usize,
    pub total: usize,
    pub error: Option<String>,
}
```

- [ ] **Step 2: Add global abort flag**

Add a module-level abort mechanism using `lazy_static` or `std::sync`:

```rust
use std::sync::atomic::{AtomicBool, Ordering};

static SCAN_ABORT: AtomicBool = AtomicBool::new(false);
```

- [ ] **Step 3: Implement `scan_repo_list` command**

Replace the placeholder:

```rust
/// Scan all repos in a list. Emits `scan-progress` events.
/// Returns the list_id as a "scan operation ID" for abort tracking.
#[tauri::command]
pub async fn scan_repo_list(list_id: String, app: tauri::AppHandle) -> AppResult<String> {
    let pool = db::pool()?;

    // Reset abort flag
    SCAN_ABORT.store(false, Ordering::SeqCst);

    // Fetch repo IDs for this list
    let member_rows = sqlx::query("SELECT repo_id FROM repo_list_members WHERE list_id = ? ORDER BY added_at")
        .bind(&list_id)
        .fetch_all(pool)
        .await?;

    let repo_ids: Vec<String> = member_rows.iter().map(|r| r.get::<String, _>("repo_id")).collect();
    let total = repo_ids.len();

    if total == 0 {
        return Err(AppError::InvalidInput("Repo list is empty".into()));
    }

    tracing::info!("Starting batch scan of {} repos in list {}", total, list_id);

    let semaphore = Arc::new(tokio::sync::Semaphore::new(5)); // 5 concurrent
    let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let results = Arc::new(Mutex::new(Vec::new()));

    let mut handles = Vec::new();

    for repo_id in repo_ids {
        let sem = semaphore.clone();
        let ctr = counter.clone();
        let res = results.clone();
        let app_handle = app.clone();
        let total = total;

        let handle = tokio::spawn(async move {
            // Check abort before acquiring permit
            if SCAN_ABORT.load(Ordering::SeqCst) {
                return;
            }

            let _permit = match sem.acquire().await {
                Ok(p) => p,
                Err(_) => return,
            };

            // Check abort after acquiring permit
            if SCAN_ABORT.load(Ordering::SeqCst) {
                return;
            }

            // Emit "scanning" event
            let current = ctr.fetch_add(1, Ordering::SeqCst) + 1;
            let _ = app_handle.emit("scan-progress", ScanProgressEvent {
                repo_id: repo_id.clone(),
                status: "scanning".to_string(),
                current,
                total,
                error: None,
            });

            // Scan the repo
            let result = scan_repo(repo_id.clone()).await;

            let event = match &result {
                Ok(_) => ScanProgressEvent {
                    repo_id: repo_id.clone(),
                    status: "done".to_string(),
                    current,
                    total,
                    error: None,
                },
                Err(e) => ScanProgressEvent {
                    repo_id: repo_id.clone(),
                    status: "failed".to_string(),
                    current,
                    total,
                    error: Some(e.to_string()),
                },
            };

            let _ = app_handle.emit("scan-progress", event);
            res.lock().await.push((repo_id, result));

            // Inter-request delay (200ms default)
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        });

        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        let _ = handle.await;
    }

    let final_results = results.lock().await;
    let succeeded = final_results.iter().filter(|(_, r)| r.is_ok()).count();
    let failed = final_results.iter().filter(|(_, r)| r.is_err()).count();

    tracing::info!("Batch scan complete: {} succeeded, {} failed", succeeded, failed);

    Ok(list_id)
}

/// Abort a running batch scan.
#[tauri::command]
pub async fn abort_scan(operation_id: String) -> AppResult<()> {
    let _ = operation_id;
    SCAN_ABORT.store(true, Ordering::SeqCst);
    tracing::info!("Scan abort requested");
    Ok(())
}
```

- [ ] **Step 4: Verify compilation**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: Compiles without errors

- [ ] **Step 5: Run all Rust tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: All tests pass

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/commands/scan.rs
git commit -m "feat(scan): implement batch scanner with progress events and abort support"
```

---

## Task 8: Frontend Types and Service Updates

Update the frontend types to include scan progress events, and ensure the service layer is ready.

**Files:**
- Modify: `src/types/scan.ts`
- Modify: `src/services/scan.ts`

- [ ] **Step 1: Add progress event types**

In `src/types/scan.ts`, add after the existing types:

```typescript
export interface ScanProgressEvent {
  repoId: string
  status: 'scanning' | 'done' | 'failed'
  current: number
  total: number
  error: string | null
}

export interface BatchScanSummary {
  total: number
  succeeded: number
  failed: number
  inProgress: number
}
```

- [ ] **Step 2: Update scan service with event listener**

Replace `src/services/scan.ts`:

```typescript
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { ScanResult, ScanProgressEvent } from '@/types/scan'

export function scanRepo(repoId: string): Promise<ScanResult> {
  return invoke('scan_repo', { repoId })
}

export function scanRepoList(listId: string): Promise<string> {
  return invoke('scan_repo_list', { listId })
}

export function getScanResult(repoId: string): Promise<ScanResult> {
  return invoke('get_scan_result', { repoId })
}

export function listScanResults(repoListId?: string): Promise<ScanResult[]> {
  return invoke('list_scan_results', { repoListId: repoListId ?? null })
}

export function abortScan(operationId: string): Promise<void> {
  return invoke('abort_scan', { operationId })
}

export function onScanProgress(
  callback: (event: ScanProgressEvent) => void,
): Promise<UnlistenFn> {
  return listen<ScanProgressEvent>('scan-progress', (event) => {
    callback(event.payload)
  })
}
```

- [ ] **Step 3: Commit**

```bash
git add src/types/scan.ts src/services/scan.ts
git commit -m "feat(scan): add scan progress event types and event listener service"
```

---

## Task 9: Frontend Scans Store — Actions

Implement the full Pinia store with scan actions, progress tracking, and error handling.

**Files:**
- Modify: `src/stores/scans.ts`
- Create: `src/stores/__tests__/scans.spec.ts`

- [ ] **Step 1: Write tests for the scans store**

Create `src/stores/__tests__/scans.spec.ts`:

```typescript
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useScansStore } from '../scans'

vi.mock('@/services/scan', () => ({
  scanRepo: vi.fn(),
  scanRepoList: vi.fn(),
  getScanResult: vi.fn(),
  listScanResults: vi.fn(),
  abortScan: vi.fn(),
  onScanProgress: vi.fn(),
}))

describe('useScansStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('initialises with empty state', () => {
    const store = useScansStore()
    expect(store.results).toEqual([])
    expect(store.isScanning).toBe(false)
    expect(store.scanProgress.current).toBe(0)
    expect(store.scanProgress.total).toBe(0)
    expect(store.error).toBeNull()
  })

  it('loads scan results', async () => {
    const { listScanResults } = await import('@/services/scan')
    const mockResults = [
      { repoId: 'github:org/repo', scannedAt: '2026-01-01', healthScore: 85 },
    ]
    vi.mocked(listScanResults).mockResolvedValueOnce(mockResults as any)

    const store = useScansStore()
    await store.loadResults()

    expect(store.results).toEqual(mockResults)
    expect(store.error).toBeNull()
  })

  it('handles loadResults error', async () => {
    const { listScanResults } = await import('@/services/scan')
    vi.mocked(listScanResults).mockRejectedValueOnce(new Error('DB error'))

    const store = useScansStore()
    await expect(store.loadResults()).rejects.toThrow('DB error')
    expect(store.error).toBe('Error: DB error')
  })

  it('computes averageHealthScore', () => {
    const store = useScansStore()
    store.results = [
      { healthScore: 80 },
      { healthScore: 60 },
      { healthScore: 100 },
    ] as any
    expect(store.averageHealthScore).toBe(80)
  })

  it('averageHealthScore is 0 when no results', () => {
    const store = useScansStore()
    expect(store.averageHealthScore).toBe(0)
  })
})
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `pnpm vitest run --reporter verbose src/stores/__tests__/scans.spec.ts`
Expected: Some tests fail (store doesn't have `loadResults` yet)

- [ ] **Step 3: Implement the full scans store**

Replace `src/stores/scans.ts`:

```typescript
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { ScanResult, ScanProgressEvent } from '@/types/scan'
import {
  scanRepo as scanRepoService,
  scanRepoList as scanRepoListService,
  listScanResults,
  abortScan as abortScanService,
  onScanProgress,
} from '@/services/scan'
import type { UnlistenFn } from '@tauri-apps/api/event'

export const useScansStore = defineStore('scans', () => {
  const results = ref<ScanResult[]>([])
  const isScanning = ref(false)
  const scanProgress = ref({ current: 0, total: 0 })
  const error = ref<string | null>(null)
  const repoStatuses = ref<Record<string, ScanProgressEvent>>({})

  let unlistenProgress: UnlistenFn | null = null

  const averageHealthScore = computed(() => {
    if (results.value.length === 0) return 0
    return Math.round(
      results.value.reduce((sum, r) => sum + r.healthScore, 0) / results.value.length,
    )
  })

  const scanSummary = computed(() => {
    const statuses = Object.values(repoStatuses.value)
    return {
      total: scanProgress.value.total,
      succeeded: statuses.filter((s) => s.status === 'done').length,
      failed: statuses.filter((s) => s.status === 'failed').length,
      inProgress: statuses.filter((s) => s.status === 'scanning').length,
    }
  })

  async function loadResults(repoListId?: string) {
    error.value = null
    try {
      results.value = await listScanResults(repoListId)
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function scanSingleRepo(repoId: string) {
    error.value = null
    isScanning.value = true
    try {
      const result = await scanRepoService(repoId)
      // Update or add the result
      const idx = results.value.findIndex((r) => r.repoId === repoId)
      if (idx >= 0) {
        results.value[idx] = result
      } else {
        results.value.push(result)
      }
      return result
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      isScanning.value = false
    }
  }

  async function scanList(listId: string) {
    error.value = null
    isScanning.value = true
    scanProgress.value = { current: 0, total: 0 }
    repoStatuses.value = {}

    // Listen for progress events
    unlistenProgress = await onScanProgress((event) => {
      scanProgress.value = { current: event.current, total: event.total }
      repoStatuses.value[event.repoId] = event
    })

    try {
      await scanRepoListService(listId)
      // Reload results after batch completes
      await loadResults(listId)
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      isScanning.value = false
      if (unlistenProgress) {
        unlistenProgress()
        unlistenProgress = null
      }
    }
  }

  async function abortCurrentScan() {
    try {
      await abortScanService('')
    } catch (e) {
      error.value = String(e)
    }
  }

  return {
    results,
    isScanning,
    scanProgress,
    error,
    repoStatuses,
    averageHealthScore,
    scanSummary,
    loadResults,
    scanSingleRepo,
    scanList,
    abortCurrentScan,
  }
})
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `pnpm vitest run --reporter verbose src/stores/__tests__/scans.spec.ts`
Expected: All 5 tests pass

- [ ] **Step 5: Commit**

```bash
git add src/stores/scans.ts src/stores/__tests__/scans.spec.ts
git commit -m "feat(scan): implement scans Pinia store with progress tracking"
```

---

## Task 10: Scanner.vue — Full Scanning UI

Build the Scanner view with: scan target selector, live progress bar, results table with health scores, expandable detail rows.

**Files:**
- Modify: `src/views/Scanner.vue`

- [ ] **Step 1: Implement the full Scanner view**

Replace `src/views/Scanner.vue`:

```vue
<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useScansStore } from '@/stores/scans'
import { useReposStore } from '@/stores/repos'
import { useRepoListsStore } from '@/stores/repoLists'
import type { ScanResult } from '@/types/scan'

const scansStore = useScansStore()
const reposStore = useReposStore()
const listsStore = useRepoListsStore()

const selectedListId = ref<string | null>(null)
const selectedRepoId = ref<string | null>(null)
const expandedRepoId = ref<string | null>(null)
const sortBy = ref<'repoId' | 'healthScore' | 'scannedAt'>('healthScore')
const sortAsc = ref(true)

const sortedResults = computed(() => {
  const sorted = [...scansStore.results]
  sorted.sort((a, b) => {
    const key = sortBy.value
    const aVal = a[key] ?? ''
    const bVal = b[key] ?? ''
    if (typeof aVal === 'number' && typeof bVal === 'number') {
      return sortAsc.value ? aVal - bVal : bVal - aVal
    }
    return sortAsc.value
      ? String(aVal).localeCompare(String(bVal))
      : String(bVal).localeCompare(String(aVal))
  })
  return sorted
})

const progressPercent = computed(() => {
  if (scansStore.scanProgress.total === 0) return 0
  return Math.round(
    (scansStore.scanProgress.current / scansStore.scanProgress.total) * 100,
  )
})

function repoName(repoId: string): string {
  return repoId.replace(/^github:/, '').replace(/^gitlab:/, '')
}

function healthColour(score: number): string {
  if (score >= 80) return 'text-green-500'
  if (score >= 50) return 'text-amber-500'
  return 'text-red-500'
}

function healthBg(score: number): string {
  if (score >= 80) return 'bg-green-500/20'
  if (score >= 50) return 'bg-amber-500/20'
  return 'bg-red-500/20'
}

function toggleSort(col: typeof sortBy.value) {
  if (sortBy.value === col) {
    sortAsc.value = !sortAsc.value
  } else {
    sortBy.value = col
    sortAsc.value = true
  }
}

function toggleExpand(repoId: string) {
  expandedRepoId.value = expandedRepoId.value === repoId ? null : repoId
}

async function handleScanRepo() {
  if (!selectedRepoId.value) return
  try {
    await scansStore.scanSingleRepo(selectedRepoId.value)
  } catch {
    // error already in store
  }
}

async function handleScanList() {
  if (!selectedListId.value) return
  try {
    await scansStore.scanList(selectedListId.value)
  } catch {
    // error already in store
  }
}

async function handleAbort() {
  await scansStore.abortCurrentScan()
}

onMounted(async () => {
  await Promise.all([
    reposStore.loadRepos(),
    listsStore.loadLists(),
    scansStore.loadResults(),
  ])
})
</script>

<template>
  <div class="p-6 max-w-[1400px]">
    <h1 class="text-2xl font-bold text-white mb-6">Scanner</h1>

    <!-- Scan Controls -->
    <div class="bg-[#1A1D27] rounded-lg border border-[#2A2D3A] p-4 mb-6">
      <div class="flex items-end gap-4 flex-wrap">
        <!-- Scan single repo -->
        <div class="flex-1 min-w-[250px]">
          <label class="block text-sm text-gray-400 mb-1">Scan Single Repo</label>
          <div class="flex gap-2">
            <select
              v-model="selectedRepoId"
              class="flex-1 bg-[#0F1117] border border-[#2A2D3A] rounded px-3 py-2 text-white text-sm"
            >
              <option :value="null" disabled>Select a repo...</option>
              <option v-for="repo in reposStore.repos" :key="repo.id" :value="repo.id">
                {{ repo.fullName }}
              </option>
            </select>
            <button
              class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm rounded disabled:opacity-50"
              :disabled="!selectedRepoId || scansStore.isScanning"
              @click="handleScanRepo"
            >
              Scan
            </button>
          </div>
        </div>

        <!-- Scan list -->
        <div class="flex-1 min-w-[250px]">
          <label class="block text-sm text-gray-400 mb-1">Scan Repo List</label>
          <div class="flex gap-2">
            <select
              v-model="selectedListId"
              class="flex-1 bg-[#0F1117] border border-[#2A2D3A] rounded px-3 py-2 text-white text-sm"
            >
              <option :value="null" disabled>Select a list...</option>
              <option v-for="list in listsStore.lists" :key="list.id" :value="list.id">
                {{ list.name }} ({{ list.repoIds.length }} repos)
              </option>
            </select>
            <button
              class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm rounded disabled:opacity-50"
              :disabled="!selectedListId || scansStore.isScanning"
              @click="handleScanList"
            >
              Scan All
            </button>
          </div>
        </div>
      </div>

      <!-- Progress bar -->
      <div v-if="scansStore.isScanning" class="mt-4">
        <div class="flex items-center justify-between mb-1">
          <span class="text-sm text-gray-400">
            Scanning {{ scansStore.scanProgress.current }} / {{ scansStore.scanProgress.total }} repos
          </span>
          <button
            class="text-sm text-red-400 hover:text-red-300"
            @click="handleAbort"
          >
            Abort
          </button>
        </div>
        <div class="w-full h-2 bg-[#2A2D3A] rounded-full overflow-hidden">
          <div
            class="h-full bg-blue-500 transition-all duration-300"
            :style="{ width: `${progressPercent}%` }"
          />
        </div>
        <!-- Per-repo status -->
        <div class="mt-2 flex gap-3 text-xs text-gray-500">
          <span class="text-green-500">{{ scansStore.scanSummary.succeeded }} done</span>
          <span v-if="scansStore.scanSummary.failed > 0" class="text-red-500">
            {{ scansStore.scanSummary.failed }} failed
          </span>
          <span>{{ scansStore.scanSummary.inProgress }} scanning</span>
        </div>
      </div>
    </div>

    <!-- Error -->
    <div v-if="scansStore.error" class="mb-4 p-3 bg-red-500/10 border border-red-500/30 rounded text-red-400 text-sm">
      {{ scansStore.error }}
    </div>

    <!-- Summary stats -->
    <div v-if="scansStore.results.length > 0" class="grid grid-cols-3 gap-4 mb-6">
      <div class="bg-[#1A1D27] rounded-lg border border-[#2A2D3A] p-4 text-center">
        <div class="text-2xl font-mono font-bold text-white">{{ scansStore.results.length }}</div>
        <div class="text-sm text-gray-400">Repos Scanned</div>
      </div>
      <div class="bg-[#1A1D27] rounded-lg border border-[#2A2D3A] p-4 text-center">
        <div class="text-2xl font-mono font-bold" :class="healthColour(scansStore.averageHealthScore)">
          {{ scansStore.averageHealthScore }}
        </div>
        <div class="text-sm text-gray-400">Avg Health Score</div>
      </div>
      <div class="bg-[#1A1D27] rounded-lg border border-[#2A2D3A] p-4 text-center">
        <div class="text-2xl font-mono font-bold text-amber-500">
          {{ scansStore.results.filter(r => r.flags.length > 0).length }}
        </div>
        <div class="text-sm text-gray-400">With Issues</div>
      </div>
    </div>

    <!-- Results table -->
    <div v-if="scansStore.results.length > 0" class="bg-[#1A1D27] rounded-lg border border-[#2A2D3A] overflow-hidden">
      <table class="w-full text-sm">
        <thead>
          <tr class="border-b border-[#2A2D3A] text-gray-400 text-left">
            <th class="px-4 py-3 cursor-pointer hover:text-white" @click="toggleSort('repoId')">
              Repository {{ sortBy === 'repoId' ? (sortAsc ? '\u25B2' : '\u25BC') : '' }}
            </th>
            <th class="px-4 py-3 cursor-pointer hover:text-white" @click="toggleSort('healthScore')">
              Health {{ sortBy === 'healthScore' ? (sortAsc ? '\u25B2' : '\u25BC') : '' }}
            </th>
            <th class="px-4 py-3">Package Manager</th>
            <th class="px-4 py-3">Node</th>
            <th class="px-4 py-3">Manifests</th>
            <th class="px-4 py-3">Issues</th>
            <th class="px-4 py-3 cursor-pointer hover:text-white" @click="toggleSort('scannedAt')">
              Scanned {{ sortBy === 'scannedAt' ? (sortAsc ? '\u25B2' : '\u25BC') : '' }}
            </th>
          </tr>
        </thead>
        <tbody>
          <template v-for="result in sortedResults" :key="result.repoId">
            <tr
              class="border-b border-[#2A2D3A] hover:bg-[#0F1117] cursor-pointer"
              :class="{ 'opacity-50': result.excluded }"
              @click="toggleExpand(result.repoId)"
            >
              <td class="px-4 py-3">
                <span class="text-white font-mono">{{ repoName(result.repoId) }}</span>
                <span v-if="result.excluded" class="ml-2 text-xs text-gray-500">(excluded)</span>
              </td>
              <td class="px-4 py-3">
                <span
                  class="inline-block px-2 py-0.5 rounded font-mono text-xs font-bold"
                  :class="[healthColour(result.healthScore), healthBg(result.healthScore)]"
                >
                  {{ result.healthScore }}
                </span>
              </td>
              <td class="px-4 py-3 text-gray-300 font-mono">
                {{ result.packageManager ?? '\u2014' }}
                <span v-if="result.packageManagerVersion" class="text-gray-500 text-xs">
                  {{ result.packageManagerVersion }}
                </span>
              </td>
              <td class="px-4 py-3 text-gray-300 font-mono">
                {{ result.nodeVersion ?? '\u2014' }}
                <span v-if="result.nodeVersionSource" class="text-gray-500 text-xs">
                  ({{ result.nodeVersionSource }})
                </span>
              </td>
              <td class="px-4 py-3 text-gray-300">{{ result.manifestPaths.length }}</td>
              <td class="px-4 py-3">
                <span v-if="result.flags.length === 0" class="text-green-500 text-xs">None</span>
                <span v-else class="text-amber-500 text-xs">{{ result.flags.length }}</span>
              </td>
              <td class="px-4 py-3 text-gray-500 text-xs">
                {{ new Date(result.scannedAt).toLocaleDateString() }}
              </td>
            </tr>
            <!-- Expanded detail row -->
            <tr v-if="expandedRepoId === result.repoId">
              <td colspan="7" class="px-6 py-4 bg-[#0F1117]">
                <div class="grid grid-cols-2 gap-4 text-sm">
                  <div>
                    <h4 class="text-gray-400 font-medium mb-2">Manifest Paths</h4>
                    <ul class="space-y-1">
                      <li v-for="mp in result.manifestPaths" :key="mp" class="font-mono text-gray-300 text-xs">
                        {{ mp }}
                      </li>
                      <li v-if="result.manifestPaths.length === 0" class="text-gray-500 text-xs">None found</li>
                    </ul>
                  </div>
                  <div>
                    <h4 class="text-gray-400 font-medium mb-2">Workflow Files</h4>
                    <ul class="space-y-1">
                      <li v-for="wf in result.workflowFiles" :key="wf" class="font-mono text-gray-300 text-xs">
                        {{ wf }}
                      </li>
                      <li v-if="result.workflowFiles.length === 0" class="text-gray-500 text-xs">None found</li>
                    </ul>
                  </div>
                  <div v-if="result.flags.length > 0" class="col-span-2">
                    <h4 class="text-gray-400 font-medium mb-2">Issues</h4>
                    <ul class="space-y-1">
                      <li v-for="flag in result.flags" :key="flag.flagType" class="text-xs flex items-center gap-2">
                        <span
                          class="inline-block w-2 h-2 rounded-full"
                          :class="{
                            'bg-red-500': flag.severity === 'high' || flag.severity === 'critical',
                            'bg-amber-500': flag.severity === 'medium',
                            'bg-blue-500': flag.severity === 'low' || flag.severity === 'info',
                          }"
                        />
                        <span class="text-gray-300">{{ flag.message }}</span>
                      </li>
                    </ul>
                  </div>
                  <div>
                    <h4 class="text-gray-400 font-medium mb-2">Details</h4>
                    <dl class="space-y-1 text-xs">
                      <div class="flex gap-2">
                        <dt class="text-gray-500">PHP:</dt>
                        <dd class="text-gray-300 font-mono">{{ result.phpVersion ?? 'N/A' }}</dd>
                      </div>
                      <div class="flex gap-2">
                        <dt class="text-gray-500">Develop branch:</dt>
                        <dd class="text-gray-300">{{ result.hasDevelop ? 'Yes' : 'No' }}</dd>
                      </div>
                      <div class="flex gap-2">
                        <dt class="text-gray-500">.env.example:</dt>
                        <dd class="text-gray-300">{{ result.hasDotEnvExample ? 'Yes' : 'No' }}</dd>
                      </div>
                    </dl>
                  </div>
                </div>
              </td>
            </tr>
          </template>
        </tbody>
      </table>
    </div>

    <!-- Empty state -->
    <div v-else-if="!scansStore.isScanning" class="text-center py-12 text-gray-500">
      <p class="text-lg mb-2">No scan results yet</p>
      <p class="text-sm">Select a repo or repo list above to start scanning.</p>
    </div>
  </div>
</template>
```

- [ ] **Step 2: Verify frontend compiles**

Run: `pnpm typecheck`
Expected: No type errors

- [ ] **Step 3: Run all frontend tests**

Run: `pnpm vitest run --reporter verbose`
Expected: All tests pass

- [ ] **Step 4: Commit**

```bash
git add src/views/Scanner.vue
git commit -m "feat(ui): implement Scanner view with progress tracking and results table"
```

---

## Task 11: Update PLANNING.md with Implementation Status

Mark all completed Phase 3 items as `[implemented]` in PLANNING.md.

**Files:**
- Modify: `PLANNING.md`

- [ ] **Step 1: Update Phase 3.1 items**

Mark the following as `[implemented]`:
- Fetch and parse `package.json`
- Fetch and parse `composer.json`
- Monorepo-aware manifest discovery
- Detect Node version (with priority chain)
- Store `nodeVersionSource`
- Detect PHP version from `composer.json#require.php`
- Detect package manager from lockfile
- Detect package manager version from `packageManager` field
- Detect `develop` branch existence
- Store `lastPushed`
- Auto-exclude logic
- Inventory `.github/workflows/*.yml` files
- Detect floating Action tags in workflow files
- Detect presence of `.env.example`, `CODEOWNERS`, `SECURITY.md`, `.editorconfig`
- Compute health score
- Store scan result in SQLite with timestamp

Mark as deferred with note:
- `requirements.txt`, `Cargo.toml`, `go.mod` parsing — "deferred: npm/composer ecosystems prioritised"
- Scan diff — "deferred: requires two scan results for comparison"

- [ ] **Step 2: Update Phase 3.2 items**

Mark the following as `[implemented]`:
- Scan entire repo list in parallel
- Progress indicator: X / N repos scanned
- Per-repo status: queued / scanning / done / failed
- Abort running scan
- Configurable inter-request delay (200ms default)

Mark as deferred with note:
- Scan summary on completion — "deferred: basic progress events implemented"
- Rate limit awareness — "partial: rate limit updated per API call, auto-pause not yet implemented"
- Incremental scan — "deferred: requires `lastPushed` comparison logic"

- [ ] **Step 3: Commit**

```bash
git add PLANNING.md
git commit -m "docs(planning): mark Phase 3 scanning items as implemented"
```

---

## Execution Summary

| Task | Description | Est. Complexity |
|------|-------------|-----------------|
| 1 | GitHubClient content fetching methods | Standard |
| 2 | Scanner service — manifest discovery | Simple |
| 3 | Scanner service — manifest parsing | Standard |
| 4 | Scanner service — Node version detection | Simple |
| 5 | Scanner service — health score computation | Simple |
| 6 | `scan_repo` command (full implementation) | Complex |
| 7 | Batch scanner with progress events | Complex |
| 8 | Frontend types and service updates | Simple |
| 9 | Frontend scans store with actions | Standard |
| 10 | Scanner.vue full UI | Standard |
| 11 | Update PLANNING.md | Simple |

**Total: 11 tasks**. Tasks 1–5 build up scanner primitives (tested individually). Task 6 wires them into the command. Task 7 adds batch parallelism. Tasks 8–10 build the frontend. Task 11 documents status.
