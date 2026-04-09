use crate::models::{RepoPackage, ScanFlag};
use crate::services::github::GitHubTreeResponse;
use chrono::Utc;

// ── Constants ─────────────────────────────────────────────────────────────

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
    "target/",
];

const MANIFEST_FILES: &[&str] = &[
    "package.json",
    "composer.json",
    "requirements.txt",
    "Cargo.toml",
    "go.mod",
];

// ── Task 2: Manifest Discovery ────────────────────────────────────────────

/// Check whether a path falls inside one of the excluded directories.
fn is_excluded_path(path: &str) -> bool {
    EXCLUDED_DIRS.iter().any(|dir| path.contains(dir))
}

/// Discover all manifest files from a GitHub tree, excluding vendor directories.
pub fn discover_manifests(tree: &GitHubTreeResponse) -> Vec<String> {
    tree.tree
        .iter()
        .filter(|entry| entry.entry_type == "blob")
        .filter(|entry| !is_excluded_path(&entry.path))
        .filter(|entry| {
            MANIFEST_FILES
                .iter()
                .any(|m| entry.path == *m || entry.path.ends_with(&format!("/{m}")))
        })
        .map(|entry| entry.path.clone())
        .collect()
}

/// Discover GitHub Actions workflow files from a tree.
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

/// Check if a specific file exists in the tree.
pub fn file_exists(tree: &GitHubTreeResponse, path: &str) -> bool {
    tree.tree.iter().any(|entry| entry.path == path)
}

/// Detect the package manager from root-level lockfiles.
/// Priority: pnpm > yarn > bun > npm.
/// Returns (manager_name, lockfile_path) or None.
pub fn detect_package_manager(tree: &GitHubTreeResponse) -> Option<(&'static str, String)> {
    let root_blobs: Vec<&str> = tree
        .tree
        .iter()
        .filter(|e| e.entry_type == "blob" && !e.path.contains('/'))
        .map(|e| e.path.as_str())
        .collect();

    if root_blobs.contains(&"pnpm-lock.yaml") {
        Some(("pnpm", "pnpm-lock.yaml".to_string()))
    } else if root_blobs.contains(&"yarn.lock") {
        Some(("yarn", "yarn.lock".to_string()))
    } else if root_blobs.contains(&"bun.lockb") || root_blobs.contains(&"bun.lock") {
        let lockfile = if root_blobs.contains(&"bun.lockb") {
            "bun.lockb"
        } else {
            "bun.lock"
        };
        Some(("bun", lockfile.to_string()))
    } else if root_blobs.contains(&"package-lock.json") {
        Some(("npm", "package-lock.json".to_string()))
    } else {
        None
    }
}

// ── Task 3: Manifest Parsing ──────────────────────────────────────────────

/// Parse a `package.json` and extract dependencies and devDependencies.
pub fn parse_package_json(content: &str, repo_id: &str) -> Vec<RepoPackage> {
    let mut packages = Vec::new();
    let now = Utc::now().to_rfc3339();

    let val: serde_json::Value = match serde_json::from_str(content) {
        Ok(v) => v,
        Err(_) => return packages,
    };

    if let Some(deps) = val.get("dependencies").and_then(|d| d.as_object()) {
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

    if let Some(deps) = val.get("devDependencies").and_then(|d| d.as_object()) {
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

/// Extract `engines.node` from a `package.json` string.
pub fn extract_engines_node(content: &str) -> Option<String> {
    let val: serde_json::Value = serde_json::from_str(content).ok()?;
    val.get("engines")?
        .get("node")?
        .as_str()
        .map(|s| s.to_string())
}

/// Extract the `packageManager` field from `package.json`.
/// Parses values like "pnpm@9.1.0" into ("pnpm", "9.1.0").
pub fn extract_package_manager_field(content: &str) -> Option<(String, String)> {
    let val: serde_json::Value = serde_json::from_str(content).ok()?;
    let field = val.get("packageManager")?.as_str()?;
    let mut parts = field.splitn(2, '@');
    let name = parts.next()?.to_string();
    let version = parts.next()?.to_string();
    if name.is_empty() || version.is_empty() {
        return None;
    }
    Some((name, version))
}

/// Parse a `composer.json` and extract require + require-dev packages.
/// Excludes "php" and "ext-*" entries, and only includes entries where
/// the name contains '/' (real Composer packages).
pub fn parse_composer_json(content: &str, repo_id: &str) -> Vec<RepoPackage> {
    let mut packages = Vec::new();
    let now = Utc::now().to_rfc3339();

    let val: serde_json::Value = match serde_json::from_str(content) {
        Ok(v) => v,
        Err(_) => return packages,
    };

    let should_include =
        |name: &str| -> bool { name != "php" && !name.starts_with("ext-") && name.contains('/') };

    if let Some(deps) = val.get("require").and_then(|d| d.as_object()) {
        for (name, version) in deps {
            if should_include(name) {
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

    if let Some(deps) = val.get("require-dev").and_then(|d| d.as_object()) {
        for (name, version) in deps {
            if should_include(name) {
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

/// Extract the PHP version constraint from `require.php` in `composer.json`.
pub fn extract_php_version(content: &str) -> Option<String> {
    let val: serde_json::Value = serde_json::from_str(content).ok()?;
    val.get("require")?
        .get("php")?
        .as_str()
        .map(|s| s.to_string())
}

// ── Task 4: Node Version Detection ───────────────────────────────────────

/// Detect the Node.js version from a set of file contents.
/// Priority: `.nvmrc` > `.node-version` > `.tool-versions` > `engines.node`.
/// Returns (version, source) or None.
pub fn detect_node_version(files: &[(&str, &str)]) -> Option<(String, String)> {
    let find_file = |name: &str| -> Option<&str> {
        files
            .iter()
            .find(|(path, _)| *path == name)
            .map(|(_, content)| *content)
    };

    // .nvmrc
    if let Some(content) = find_file(".nvmrc") {
        let version = content.trim().strip_prefix('v').unwrap_or(content.trim());
        if !version.is_empty() {
            return Some((version.to_string(), ".nvmrc".to_string()));
        }
    }

    // .node-version
    if let Some(content) = find_file(".node-version") {
        let version = content.trim().strip_prefix('v').unwrap_or(content.trim());
        if !version.is_empty() {
            return Some((version.to_string(), ".node-version".to_string()));
        }
    }

    // .tool-versions
    if let Some(content) = find_file(".tool-versions") {
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("nodejs ") {
                let version = rest.trim().strip_prefix('v').unwrap_or(rest.trim());
                if !version.is_empty() {
                    return Some((version.to_string(), ".tool-versions".to_string()));
                }
            } else if let Some(rest) = trimmed.strip_prefix("node ") {
                let version = rest.trim().strip_prefix('v').unwrap_or(rest.trim());
                if !version.is_empty() {
                    return Some((version.to_string(), ".tool-versions".to_string()));
                }
            }
        }
    }

    // engines.node from package.json
    if let Some(content) = find_file("package.json") {
        if let Some(version) = extract_engines_node(content) {
            return Some((version, "engines.node".to_string()));
        }
    }

    None
}

// ── Task 5: Health Score & Floating Action Tags ──────────────────────────

/// Input for computing a repository health score.
pub struct HealthScoreInput {
    pub has_codeowners: bool,
    pub has_security_md: bool,
    pub has_dot_env_example: bool,
    pub has_editorconfig: bool,
    pub floating_action_count: usize,
    pub has_known_cves: bool,
    pub node_version_current: bool,
}

/// Detect floating (unpinned) action tags in a GitHub Actions workflow file.
/// Returns a list of action references that use tags instead of full SHA pins.
pub fn detect_floating_action_tags(workflow_content: &str) -> Vec<String> {
    let mut floating = Vec::new();

    for line in workflow_content.lines() {
        let trimmed = line.trim();

        // Match lines like "- uses: actions/checkout@v4" or "uses: actions/checkout@v4"
        let uses_value = trimmed
            .strip_prefix("- uses:")
            .or_else(|| trimmed.strip_prefix("uses:"))
            .map(|rest| rest.trim());

        if let Some(action_ref) = uses_value {
            // Remove surrounding quotes if present
            let action_ref = action_ref.trim_matches('"').trim_matches('\'');

            if let Some((_action, tag)) = action_ref.rsplit_once('@') {
                // A pinned SHA is exactly 40 hex characters
                let is_sha = tag.len() == 40 && tag.chars().all(|c| c.is_ascii_hexdigit());
                if !is_sha {
                    floating.push(action_ref.to_string());
                }
            }
        }
    }

    floating
}

/// Compute a health score (0-100) and associated flags for a repository.
pub fn compute_health_score(input: &HealthScoreInput) -> (u32, Vec<ScanFlag>) {
    let mut score: u32 = 0;
    let mut flags = Vec::new();

    // CODEOWNERS: +10
    if input.has_codeowners {
        score += 10;
    } else {
        flags.push(ScanFlag {
            flag_type: "missing_codeowners".to_string(),
            message: "Repository is missing CODEOWNERS file".to_string(),
            severity: "warning".to_string(),
        });
    }

    // SECURITY.md: +10
    if input.has_security_md {
        score += 10;
    } else {
        flags.push(ScanFlag {
            flag_type: "missing_security_md".to_string(),
            message: "Repository is missing SECURITY.md file".to_string(),
            severity: "warning".to_string(),
        });
    }

    // .env.example: +5
    if input.has_dot_env_example {
        score += 5;
    } else {
        flags.push(ScanFlag {
            flag_type: "missing_env_example".to_string(),
            message: "Repository is missing .env.example file".to_string(),
            severity: "info".to_string(),
        });
    }

    // .editorconfig: +5
    if input.has_editorconfig {
        score += 5;
    } else {
        flags.push(ScanFlag {
            flag_type: "missing_editorconfig".to_string(),
            message: "Repository is missing .editorconfig file".to_string(),
            severity: "info".to_string(),
        });
    }

    // No floating action tags: +15
    if input.floating_action_count == 0 {
        score += 15;
    } else {
        flags.push(ScanFlag {
            flag_type: "floating_action_tags".to_string(),
            message: format!(
                "{} workflow action(s) use floating tags instead of pinned SHAs",
                input.floating_action_count
            ),
            severity: "warning".to_string(),
        });
    }

    // No known CVEs: +20
    if !input.has_known_cves {
        score += 20;
    } else {
        flags.push(ScanFlag {
            flag_type: "known_cves".to_string(),
            message: "Repository has dependencies with known CVEs".to_string(),
            severity: "critical".to_string(),
        });
    }

    // Node version current: +15
    if input.node_version_current {
        score += 15;
    } else {
        flags.push(ScanFlag {
            flag_type: "node_version_eol".to_string(),
            message: "Node.js version may be end-of-life or not specified".to_string(),
            severity: "warning".to_string(),
        });
    }

    // Dependencies up to date: +20 (awarded unconditionally for now)
    score += 20;

    (score, flags)
}

// ── Tests ─────────────────────────────────────────────────────────────────

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
                    sha: "aaa".to_string(),
                    size: Some(100),
                })
                .collect(),
            truncated: false,
        }
    }

    // ── Task 2 Tests ──────────────────────────────────────────────────────

    #[test]
    fn discover_manifests_finds_root_and_nested() {
        let tree = make_tree(&["package.json", "apps/web/package.json", "README.md"]);
        let manifests = discover_manifests(&tree);
        assert_eq!(manifests, vec!["package.json", "apps/web/package.json"]);
    }

    #[test]
    fn discover_manifests_excludes_vendor_dirs() {
        let tree = make_tree(&[
            "package.json",
            "node_modules/foo/package.json",
            "vendor/laravel/composer.json",
            "dist/package.json",
            "build/package.json",
            ".next/package.json",
            ".nuxt/package.json",
            ".cache/package.json",
        ]);
        let manifests = discover_manifests(&tree);
        assert_eq!(manifests, vec!["package.json"]);
    }

    #[test]
    fn discover_manifests_finds_all_ecosystems() {
        let tree = make_tree(&[
            "package.json",
            "composer.json",
            "requirements.txt",
            "Cargo.toml",
            "go.mod",
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
            "src/main.rs",
        ]);
        let workflows = discover_workflows(&tree);
        assert_eq!(
            workflows,
            vec![".github/workflows/ci.yml", ".github/workflows/deploy.yaml"]
        );
    }

    #[test]
    fn file_exists_works() {
        let tree = make_tree(&["CODEOWNERS", "SECURITY.md", "src/main.rs"]);
        assert!(file_exists(&tree, "CODEOWNERS"));
        assert!(file_exists(&tree, "SECURITY.md"));
        assert!(!file_exists(&tree, ".env.example"));
    }

    #[test]
    fn detect_package_manager_pnpm() {
        let tree = make_tree(&["package.json", "pnpm-lock.yaml", "package-lock.json"]);
        let (pm, _) = detect_package_manager(&tree).unwrap();
        assert_eq!(pm, "pnpm");
    }

    #[test]
    fn detect_package_manager_npm_fallback() {
        let tree = make_tree(&["package.json", "package-lock.json"]);
        let (pm, _) = detect_package_manager(&tree).unwrap();
        assert_eq!(pm, "npm");
    }

    #[test]
    fn detect_package_manager_none_without_lockfile() {
        let tree = make_tree(&["package.json"]);
        assert!(detect_package_manager(&tree).is_none());
    }

    #[test]
    fn detect_package_manager_ignores_nested_lockfiles() {
        let tree = make_tree(&["package.json", "apps/web/pnpm-lock.yaml"]);
        assert!(detect_package_manager(&tree).is_none());
    }

    // ── Task 3 Tests ──────────────────────────────────────────────────────

    #[test]
    fn parse_package_json_extracts_deps() {
        let content = r#"{
            "dependencies": {
                "vue": "^3.4.0",
                "pinia": "^2.1.0"
            },
            "devDependencies": {
                "vite": "^5.0.0",
                "typescript": "^5.3.0"
            }
        }"#;
        let packages = parse_package_json(content, "github:test/repo");
        assert_eq!(packages.len(), 4);

        let regular: Vec<_> = packages.iter().filter(|p| !p.is_dev).collect();
        let dev: Vec<_> = packages.iter().filter(|p| p.is_dev).collect();
        assert_eq!(regular.len(), 2);
        assert_eq!(dev.len(), 2);
        assert!(packages.iter().all(|p| p.ecosystem == "npm"));
        assert!(packages.iter().all(|p| p.repo_id == "github:test/repo"));
    }

    #[test]
    fn parse_package_json_handles_no_deps() {
        let content = r#"{ "name": "empty-project", "version": "1.0.0" }"#;
        let packages = parse_package_json(content, "github:test/empty");
        assert!(packages.is_empty());
    }

    #[test]
    fn parse_package_json_extracts_engines_node() {
        let content = r#"{ "engines": { "node": ">=20.0.0" } }"#;
        let version = extract_engines_node(content);
        assert_eq!(version, Some(">=20.0.0".to_string()));
    }

    #[test]
    fn parse_package_json_extracts_package_manager_field() {
        let content = r#"{ "packageManager": "pnpm@9.1.0" }"#;
        let (name, version) = extract_package_manager_field(content).unwrap();
        assert_eq!(name, "pnpm");
        assert_eq!(version, "9.1.0");
    }

    #[test]
    fn parse_composer_json_extracts_deps() {
        let content = r#"{
            "require": {
                "php": "^8.2",
                "ext-json": "*",
                "laravel/framework": "^11.0"
            },
            "require-dev": {
                "phpunit/phpunit": "^10.5"
            }
        }"#;
        let packages = parse_composer_json(content, "github:test/laravel");
        assert_eq!(packages.len(), 2);
        assert!(packages
            .iter()
            .any(|p| p.name == "laravel/framework" && !p.is_dev));
        assert!(packages
            .iter()
            .any(|p| p.name == "phpunit/phpunit" && p.is_dev));
        assert!(packages.iter().all(|p| p.ecosystem == "composer"));
    }

    #[test]
    fn parse_composer_json_extracts_php_version() {
        let content = r#"{ "require": { "php": "^8.2" } }"#;
        let version = extract_php_version(content);
        assert_eq!(version, Some("^8.2".to_string()));
    }

    // ── Task 4 Tests ──────────────────────────────────────────────────────

    #[test]
    fn detect_node_from_nvmrc() {
        let files: Vec<(&str, &str)> = vec![
            (".nvmrc", "20.11.0"),
            ("package.json", r#"{ "engines": { "node": ">=18" } }"#),
        ];
        let (version, source) = detect_node_version(&files).unwrap();
        assert_eq!(version, "20.11.0");
        assert_eq!(source, ".nvmrc");
    }

    #[test]
    fn detect_node_from_node_version_file() {
        let files: Vec<(&str, &str)> = vec![(".node-version", "v20.11.0\n")];
        let (version, source) = detect_node_version(&files).unwrap();
        assert_eq!(version, "20.11.0");
        assert_eq!(source, ".node-version");
    }

    #[test]
    fn detect_node_from_tool_versions() {
        let files: Vec<(&str, &str)> = vec![(
            ".tool-versions",
            "ruby 3.2.0\nnodejs 20.11.0\npython 3.12.0\n",
        )];
        let (version, source) = detect_node_version(&files).unwrap();
        assert_eq!(version, "20.11.0");
        assert_eq!(source, ".tool-versions");
    }

    #[test]
    fn detect_node_from_engines_fallback() {
        let files: Vec<(&str, &str)> =
            vec![("package.json", r#"{ "engines": { "node": ">=18.0.0" } }"#)];
        let (version, source) = detect_node_version(&files).unwrap();
        assert_eq!(version, ">=18.0.0");
        assert_eq!(source, "engines.node");
    }

    #[test]
    fn detect_node_none_when_missing() {
        let files: Vec<(&str, &str)> = vec![("package.json", r#"{ "name": "no-engines" }"#)];
        assert!(detect_node_version(&files).is_none());
    }

    // ── Task 5 Tests ──────────────────────────────────────────────────────

    #[test]
    fn detect_floating_action_tags_finds_unpinned() {
        let workflow = r#"
jobs:
  build:
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
      - uses: actions/checkout@a5ac7e51b41094c92402da3b24376905380afc29
"#;
        let floating = detect_floating_action_tags(workflow);
        assert_eq!(floating.len(), 2);
        assert!(floating.contains(&"actions/checkout@v4".to_string()));
        assert!(floating.contains(&"actions/setup-node@v4".to_string()));
    }

    #[test]
    fn detect_floating_action_tags_allows_pinned_sha() {
        let workflow = r#"
jobs:
  build:
    steps:
      - uses: actions/checkout@a5ac7e51b41094c92402da3b24376905380afc29
      - uses: actions/setup-node@b39b52d1213e96004bfcb1c61a8a6fa8ab84f3e8
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
        let (score, flags) = compute_health_score(&input);
        assert_eq!(score, 100);
        assert!(flags.is_empty());
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
        // Only the unconditional +20 for dependencies
        assert_eq!(score, 20);
        assert_eq!(flags.len(), 7);
    }
}
