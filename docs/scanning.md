# Scanning

Flotilla's scanner inspects repositories via the GitHub/GitLab API to extract dependency information, runtime versions, workflow configurations, and health indicators. Scans run in parallel with rate limit awareness, and results are stored locally for CVE matching and drift analysis.

---

## What Gets Scanned

### Dependency Manifest Files

Flotilla discovers and parses all manifest files in a repository, not just the root-level ones (monorepo-aware).

| File | Ecosystem | What is extracted |
|------|-----------|-------------------|
| `package.json` | npm | `dependencies`, `devDependencies`, `engines.node`, `packageManager` field |
| `composer.json` | composer | `require`, `require-dev`, `require.php` version constraint |
| `requirements.txt` | pip | Package names and version specifiers |
| `Cargo.toml` | cargo | `[dependencies]`, `[dev-dependencies]` |
| `go.mod` | go | `require` directives with versions |

### Runtime Version Files

Flotilla detects Node.js version from multiple sources, in priority order:

| Source | Priority | Example |
|--------|----------|---------|
| `.nvmrc` | 1 (highest) | `20.11.0` |
| `.node-version` | 2 | `20.11.0` |
| `.tool-versions` | 3 | `nodejs 20.11.0` |
| `package.json` `engines.node` | 4 | `">=20.0.0"` |
| CI workflow files | 5 (lowest) | `node-version: '20'` in a GitHub Actions workflow |

The detected source is stored as `nodeVersionSource` so you can see where the version definition lives.

PHP version is detected from `composer.json`'s `require.php` field (e.g. `"^8.2"`).

### Package Manager Detection

Flotilla detects the package manager by checking for lockfiles:

| Lockfile | Package Manager |
|----------|----------------|
| `pnpm-lock.yaml` | pnpm |
| `yarn.lock` | yarn |
| `bun.lockb` | bun |
| `package-lock.json` | npm |

The package manager version is detected from the `packageManager` field in `package.json` (e.g. `"pnpm@9.15.0"`).

### Workflow Files

All files matching `.github/workflows/*.yml` are inventoried. Flotilla also detects **floating action tags** -- actions referenced with a version tag like `@v4` instead of a pinned commit SHA. Floating tags are a supply chain risk because the tag can be moved to point to different code.

Example of a floating tag (flagged):
```yaml
- uses: actions/checkout@v4     # Floating tag -- can be moved
```

Example of a pinned SHA (not flagged):
```yaml
- uses: actions/checkout@b4ffde65f46336ab88eb53be808477a3936bae11  # v4.1.1
```

### Health Indicator Files

| File | What it means |
|------|---------------|
| `.env.example` | Environment variable documentation exists |
| `CODEOWNERS` | Code ownership rules defined |
| `SECURITY.md` | Security policy documented |
| `.editorconfig` | Editor configuration standardised |

### Branch Detection

Flotilla checks whether a `develop` branch exists (stored as `hasDevelop`). This is used for PR targeting -- when creating PRs, Flotilla can target `develop` instead of the default branch if it exists.

---

## Ecosystems Supported

| Ecosystem | Manifest File | Registry Lookup |
|-----------|--------------|-----------------|
| npm | `package.json` | npm registry (latest version) |
| composer | `composer.json` | Planned (packagist.org) |
| pip | `requirements.txt` | Planned (PyPI) |
| cargo | `Cargo.toml` | Planned (crates.io) |
| go | `go.mod` | Planned (proxy.golang.org) |

npm has full registry lookup support for latest version checking. Other ecosystems extract versions from manifest files but do not yet query their registries for latest versions.

---

## Monorepo Detection

Flotilla searches the entire repository tree for manifest files, excluding common non-source directories:

**Excluded directories:**
- `node_modules/`
- `vendor/`
- `dist/`
- `build/`
- `.next/`
- `.nuxt/`
- `.cache/`

All discovered manifest paths are stored in the `manifestPaths[]` array on the scan result. For example, a monorepo might produce:

```
manifestPaths: [
  "package.json",
  "packages/api/package.json",
  "packages/web/package.json",
  "packages/shared/package.json"
]
```

Dependencies from all manifests are extracted and stored, so the full dependency picture is captured even for complex monorepos.

---

## Health Score Breakdown

Each repo receives a score from 0 to 100 based on the following rules. Weights are configurable in `.flotilla/config.yaml`.

| Rule | Default Points | Condition |
|------|---------------|-----------|
| Has `CODEOWNERS` | +10 | `CODEOWNERS` file exists in the repo |
| Has `SECURITY.md` | +10 | `SECURITY.md` file exists |
| Has `.env.example` | +5 | `.env.example` file exists |
| Has `.editorconfig` | +5 | `.editorconfig` file exists |
| No floating action tags | +15 | All GitHub Actions in workflows use pinned SHAs |
| Dependencies up to date | +20 | All dependencies are within 1 major version of latest |
| No known CVEs | +20 | No open CVE alerts match any dependency |
| Runtime not EOL | +15 | Detected Node.js / PHP version is not end-of-life |

**Total possible: 100 points**

A score of 80+ generally indicates a well-maintained repo. Below 50 suggests significant technical debt or security concerns.

### Customising Weights

Edit your `.flotilla/config.yaml` to adjust weights:

```yaml
health_score:
  has_codeowners: 15       # Increased -- code ownership is critical for us
  has_security_md: 5       # Decreased -- less important for internal repos
  no_known_cves: 30        # Increased -- security is top priority
  # ... other rules
```

Weights must sum to 100 or less. See [Configuration](./configuration.md) for the full reference.

---

## Auto-Exclude Logic

Repos without any relevant manifest files are automatically marked as **excluded** with a reason. Excluded repos are:

- Skipped during batch scans (saving API quota)
- Skipped during batch operations
- Still visible in the UI with an "excluded" badge

Common exclude reasons:
- "No manifest files found" -- the repo has no `package.json`, `composer.json`, `requirements.txt`, `Cargo.toml`, or `go.mod`

You can manually un-exclude a repo or add exclusion patterns to repo lists:

```yaml
# In a repo list definition
exclude_patterns:
  - "acme-org/docs-*"        # Exclude all docs repos
  - "acme-org/archived-*"    # Exclude archived repos
```

---

## Batch Scanning

### Running a Batch Scan

1. Go to the **Scanner** view
2. Select a repo list
3. Click **Scan All**

Flotilla scans repos in parallel (default: 5 concurrent) with progress tracking:

- **Queued** -- waiting for a worker
- **Scanning** -- actively being scanned
- **Done** -- scan complete
- **Failed** -- scan failed (error shown)

You can **abort** a running scan at any time. Progress is preserved -- repos already scanned keep their results.

### Parallelism

Configure the number of concurrent scan workers in Settings or `config.yaml`:

```yaml
scan:
  parallelism: 5    # 1-20 recommended
```

Higher values scan faster but consume API rate limit faster.

### Inter-Request Delay

To avoid hammering the API, Flotilla inserts a configurable delay between API requests:

```yaml
scan:
  inter_request_delay_ms: 200    # Default: 200ms
```

Set to `0` for maximum speed (at the cost of rate limit consumption), or increase for very rate-limited environments.

---

## Rate Limits

### GitHub

- **Authenticated:** 5,000 requests/hour
- **Display:** The top bar shows remaining requests and reset time
- **Behaviour:** When remaining requests drop below 100, Flotilla warns you. Below 50, it auto-pauses.

### GitLab

- **GitLab.com:** 2,000 requests/minute
- **Self-hosted:** Varies by instance configuration

### Optimisation

Flotilla caches API responses in the local SQLite database. File contents are considered fresh for 1 hour, and PR status for 15 minutes. This reduces redundant API calls significantly when re-scanning.

---

## Scheduled Scans

Flotilla can run scans automatically in the background:

| Interval | Description |
|----------|-------------|
| `manual` | Only scan when you click the button |
| `daily` | Scan all repo lists once per day |
| `weekly` | Scan all repo lists once per week |

Configure in Settings or `config.yaml`:

```yaml
scan:
  schedule: daily
```

When a scheduled scan completes, an in-app notification is shown. If CVE checking is enabled (`cve.check_after_scan: true`), a CVE check runs immediately after the scan.
