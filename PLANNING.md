# PLANNING.md — Git Flotilla

> Tracks all planned features, their scope, and implementation status.
> Claude Code: update status to `[implemented]` when a feature is complete.
> Status legend: `[ ]` planned · `[wip]` in progress · `[implemented]` done · `[blocked]` blocked

---

## Phase 1 — Foundation

### 1.1 Project Scaffold
- [ ] Tauri v2 + Vite + Vue 3 + TypeScript project initialisation
- [ ] Tailwind CSS v4 configured with design tokens
- [ ] Pinia stores skeleton (one file per domain, empty state)
- [ ] Vue Router configured with all route stubs
- [ ] ESLint + Prettier + TypeScript strict mode
- [ ] Rust workspace configured with all crate dependencies declared
- [ ] SQLite DB initialised via sqlx in Tauri setup hook

### 1.2 Database Schema
- [ ] `repos` table
- [ ] `repo_lists` table (with `parent_id` for nesting)
- [ ] `repo_list_members` join table
- [ ] `scan_results` table
- [ ] `repo_packages` table (ecosystem, name, version, is_dev)
- [ ] `cve_alerts` table
- [ ] `cve_affected_repos` join table
- [ ] `cve_watchlist` table (user-subscribed packages)
- [ ] `batch_operations` table
- [ ] `operation_results` table
- [ ] `audit_log` table
- [ ] `settings` table (key/value)
- [ ] Migration runner on app start

### 1.3 App Shell UI
- [ ] Sidebar navigation with icon + label
- [ ] CVE badge counter on sidebar CVE item
- [ ] Top bar: logo, global search trigger, notification bell, API rate limit indicator, auth status
- [ ] Main content area with router `<RouterView>`
- [ ] Dark theme applied globally
- [ ] Light theme toggle stored in settings
- [ ] Responsive layout (min width: 1024px)

---

## Phase 2 — Authentication & Repo Management

### 2.1 Authentication
- [implemented] GitHub Personal Access Token input + validation
- [ ] GitHub OAuth App flow (optional, for teams)
- [ ] GitLab Personal Access Token input + validation
- [implemented] OS keychain storage via `keyring` crate
- [implemented] Token scope validation on save (warn on missing scopes)
- [ ] Multi-account support (multiple GitHub/GitLab accounts) — note: model supports it, UI shows first account
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
- [ ] Nested lists (parent → child hierarchy, max 3 levels) — note: data model supports it, UI shows root lists only
- [implemented] Tag repos with arbitrary labels
- [ ] Filter repo discovery table by tags — note: text search covers name/tags but no dedicated tag filter
- [implemented] **Exclusion rules**: support org-level (`ORG/*`) and repo-level exclusion patterns on repo lists (data model)
- [ ] **Auto-exclude**: automatically mark repos without relevant manifests as excluded (with reason), respected by scanner and batch operations — requires scanner
- [implemented] Export repo list as YAML
- [implemented] Import repo list from YAML
- [ ] Store repo lists in `.flotilla/repo-lists/*.yaml` — currently SQLite only; YAML is manual export
- [ ] Repo list sidebar tree with expand/collapse

---

## Phase 3 — Scanning

### 3.1 Single Repo Scanner
- [ ] Fetch and parse `package.json` (dependencies + devDependencies)
- [ ] Fetch and parse `composer.json` (require + require-dev)
- [ ] Fetch and parse `requirements.txt`
- [ ] Fetch and parse `Cargo.toml`
- [ ] Fetch and parse `go.mod`
- [ ] Monorepo-aware manifest discovery: find **all** `package.json` / `composer.json` files (excluding `node_modules/`, `vendor/`, `dist/`, `build/`, `.next/`, `.nuxt/`, `.cache/`), store as `manifestPaths[]` array per repo
- [ ] Detect Node version from `.nvmrc`, `.node-version`, `.tool-versions`, CI workflow files, `package.json#engines.node` (in priority order)
- [ ] Store `nodeVersionSource` — where the version was detected (e.g. `.nvmrc`, `.node-version`, `engines.node`)
- [ ] Detect PHP version from `composer.json#require.php`
- [ ] Detect package manager: check for `pnpm-lock.yaml`, `yarn.lock`, `bun.lockb`, `package-lock.json`
- [ ] Detect package manager version from lockfile headers, `packageManager` field, `engines.pnpm`/`engines.npm`
- [ ] Detect `develop` branch existence (`hasDevelop` flag) — used for PR targeting
- [ ] Store `lastPushed` timestamp — useful for staleness detection and incremental scan updates
- [ ] Auto-exclude logic: mark repos without relevant manifests as `exclude: true` with `excludeReason`
- [ ] Inventory `.github/workflows/*.yml` files
- [ ] Detect floating Action tags in workflow files (e.g. `@v4` instead of pinned SHA)
- [ ] Detect presence of: `.env.example`, `CODEOWNERS`, `SECURITY.md`, `.editorconfig`
- [ ] Compute health score (0–100) based on configurable rules
- [ ] Store scan result in SQLite with timestamp
- [ ] Scan diff: compare to previous scan, surface what changed

### 3.2 Batch Scanner
- [ ] Scan entire repo list in parallel (configurable worker count, default: 5)
- [ ] Progress indicator: X / N repos scanned
- [ ] Per-repo status: queued / scanning / done / failed
- [ ] Abort running scan
- [ ] Scan summary on completion: health score distribution, top issues
- [ ] Rate limit awareness: display remaining GitHub/GitLab API quota, auto-pause when low (<100 remaining)
- [ ] Configurable inter-request delay (default: 200ms) to avoid hammering the API
- [ ] Incremental scan (update mode): only re-check repos pushed since last scan (`lastPushed` comparison)

### 3.3 Scheduled Scans
- [ ] Background scheduler (tokio interval)
- [ ] Configurable scan interval: manual / daily / weekly
- [ ] On scan completion: automatically trigger CVE check (see Phase 4)
- [ ] In-app notification on scan completion

### 3.4 Fingerprint Profiles
- [ ] Define a named "healthy repo" profile with expected values
  - Expected Node version (semver range)
  - Expected PHP version (semver range)
  - Expected package manager
  - Required files list
  - Required workflow files list
- [ ] Apply a profile to a repo list
- [ ] Flag repos deviating from their assigned profile
- [ ] Show profile compliance badge on repo cards

---

## Phase 4 — Package Intelligence

### 4.1 Dependency Matrix
- [ ] Cross-repo package table: rows = packages, columns = repos, cells = version used
- [ ] Filter by ecosystem (npm / composer / pip / cargo / go)
- [ ] Filter by repo list
- [ ] Highlight version drift (same package, different versions across repos)
- [ ] Sort by: package name, number of repos using it, highest drift
- [ ] Show latest available version from registry (npm, packagist, pypi, crates.io)
- [ ] Show outdated indicator (current vs latest)
- [ ] Identify packages unique to one repo ("orphan packages")
- [ ] Identify superseded packages (configurable list, e.g. `node-fetch` → native)
- [ ] Export matrix as CSV
- [ ] Export matrix as JSON

### 4.2 Changelog Aggregation
- [ ] When proposing a package bump, fetch and display changelog entries between current version and target version
- [ ] Pull from GitHub Releases API or `CHANGELOG.md` in the package repo
- [ ] Highlight breaking changes, deprecations, and security fixes
- [ ] Show per-repo: current version → target version with relevant changelog section
- [ ] Cache changelogs in SQLite to avoid repeated API calls

### 4.3 Package Standardisation
- [ ] Select a package + target version from matrix
- [ ] Preview which repos would be affected
- [ ] Dry run: show diff for each affected repo
- [ ] Create batch operation: bump package to target version across selected repos
- [ ] Open PRs or direct commit per user preference

---

## Phase 5 — CVE Monitoring

### 5.1 CVE Data Ingestion
- [ ] Query OSV.dev API for all packages in latest scans (primary source)
- [ ] Query GitHub Advisory Database (secondary source)
- [ ] Query NVD NIST API (tertiary source)
- [ ] Deduplicate CVEs across sources by CVE ID
- [ ] Store CVEs in SQLite with full metadata
- [ ] Match CVEs against `repo_packages` table → populate `cve_affected_repos`
- [ ] Run CVE check automatically after every scan completes
- [ ] Run CVE check on app start (if last check > configured interval)

### 5.2 CVE Scheduler
- [ ] Hourly background polling by default
- [ ] User-configurable interval: off / 15min / 30min / 1hr / 6hr / daily
- [ ] Setting persisted in `settings` table
- [ ] "Check now" manual trigger button
- [ ] Last checked timestamp displayed in CVE view
- [ ] In-app notification when new CVEs are found

### 5.3 CVE Alert UI
- [ ] CVE list view: filterable by severity, ecosystem, status, repo
- [ ] Severity colour coding: critical (red), high (orange), medium (amber), low (blue)
- [ ] CVE detail panel: full description, affected version range, fixed version, affected repos
- [ ] Badge counter on sidebar CVE nav item (count of unacknowledged critical + high)
- [ ] Per-repo CVE summary on repo cards
- [ ] Mark CVE as: acknowledged / dismissed
- [ ] Snooze CVE (re-alert after N days)
- [ ] "Patch affected repos" CTA → pre-fill batch operation with fixedVersion
- [ ] **Incident Timeline View**: unified timeline per CVE showing when vulnerability was published, when Flotilla detected it, which repos were scanned, which PRs were created/merged/still open — one screen to answer "where are we?" during an active incident
- [ ] **Blast Radius Analysis**: before patching, show the full dependency graph of an affected package — which repos use it directly, which get it transitively, and which transitive paths exist; helps prioritise patching order (e.g. public-facing apps before internal tools)

### 5.4 CVE Watchlist
- [ ] User can subscribe to arbitrary packages (not just ones in current scans)
- [ ] Watchlist management UI: add / remove packages per ecosystem
- [ ] Watchlist packages included in every CVE poll
- [ ] Notify when a watched package receives a new CVE

---

## Phase 6 — Batch Operations

### 6.1 File Update Operations
- [ ] Select source file from local filesystem or from a repo
- [ ] Select target repos / repo list
- [ ] Variable injection in file content: `{{repo}}`, `{{owner}}`, `{{branch}}`, `{{date}}`
- [ ] Dry run: show diff per repo before any write
- [ ] Diff viewer component (side-by-side or unified)
- [ ] Execute: commit to default branch or open PR
- [ ] Configurable commit message template
- [ ] Parallelism: configurable worker count
- [ ] **Workflow Sync mode**: dedicated UI for pushing GitHub Actions workflows across repos
  - [ ] Built-in template library for common workflows (lockfile regen, hotfix-back-to-develop)
  - [ ] `.nvmrc` / `.node-version` sync (update Node version across repos in one operation)
  - [ ] `package.json` field updates (e.g. `engines.node=>=20`, repeatable)
  - [ ] Force overwrite toggle (overwrite even if target file matches)
- [ ] **Lockfile CI workflow template**: optionally install `update-security-fix-lockfile.yml` alongside security PRs
  - [ ] Auto-detects Node/pnpm version from `.nvmrc` / `.node-version` / `.tool-versions` / CI workflows / `engines.node`
  - [ ] Regenerates lockfile using the correct package manager (npm/pnpm/yarn/bun)

### 6.2 Package Bump / Pin Operations
- [ ] Select package name + ecosystem
- [ ] Select target version (or "latest" / "pin to current")
- [ ] Select target repos
- [ ] Dry run diff per repo
- [ ] Execute via commit or PR
- [ ] Support bumping multiple packages in one operation
- [ ] **Pin mode** (`pin`): Set exact version + add `overrides`, `resolutions`, `pnpm.overrides` to lock entire dep tree — used during active incidents
- [ ] **Bump mode** (`bump`): Set version range + **remove** overrides/resolutions — used after upstream fix lands
- [ ] **Pin-then-bump lifecycle tracking**: track which repos are still pinned so users know when to bump back; CVE "Patch" CTA defaults to pin mode
- [ ] **Version map**: target different safe versions per major version (e.g. major 0 → 0.30.3, major 1 → 1.13.6) for repos on different major versions of the same package
- [ ] **Monorepo-aware patching**: update all matching manifest files within a repo, overrides/resolutions only at root `package.json`
- [ ] **Fresh lockfile option**: delete and regenerate lockfile from scratch (edge case but sometimes needed)
- [ ] **Validate mode**: audit whether a fix is already applied across all repos without making changes (distinct from dry-run)

### 6.3 PR Workflow
- [ ] PR title template with variable injection (`{{PACKAGE}}`, `{{VERSION}}`, `{{REPO}}`, `{{CVE}}`, `{{SEVERITY}}`, `{{DATE}}`, `{{MODE}}`)
- [ ] PR body template (Markdown) with variable injection
- [ ] **Conditional template sections**: `{{#FIELD}}content{{/FIELD}}` blocks removed entirely when field is empty (avoids ugly blank sections in PR bodies)
- [ ] Separate default templates for pin vs bump operations (editable in Settings)
- [ ] Draft PR toggle
- [ ] **Skip CI toggle**: append `[skip ci]` to commit messages (important when pushing to hundreds of repos simultaneously)
- [ ] Auto-assign reviewers from CODEOWNERS or configured default
- [ ] Target branch override per operation
- [ ] **Multi-branch targeting**: create PRs against additional branches (e.g. `develop`, `staging`) in the same operation
- [ ] **Divergence detection**: auto-detect when `develop` has diverged >N commits from main, offer separate PR directly against develop (configurable threshold, default: 50)
- [ ] **Hotfix-back-to-develop workflow**: optionally install GitHub Actions workflow that auto-creates develop PRs when hotfix/security-fix PRs are opened against main
- [ ] PR labels (configurable, e.g. `flotilla`, `security`, `dependencies`)
- [ ] Link all PRs from a batch to a tracking issue (optional)
- [ ] **Idempotent PR creation**: detect existing PR from same branch → close with comment → delete stale remote branch → create fresh PR (prevents duplicates on re-run)

### 6.4 Operation Tracking
- [ ] Operation list view: all past and in-progress operations
- [ ] Per-operation detail: status per repo, diff, PR links
- [ ] Live progress for running operations
- [ ] Abort running operation
- [ ] **Resumability**: save per-repo progress to SQLite during batch operations; resume from where it left off after crash/abort/network failure
- [ ] Rollback: for Flotilla commits, store pre-change SHA and offer revert PR
- [ ] PR status tracker: open / merged / closed / checks-failing across all Flotilla PRs
- [ ] **Batch-level PR status summary**: aggregate view showing open/merged/conflicting/missing counts across all repos in an operation
- [ ] **Downloadable operation logs**: detailed per-operation log export for debugging (beyond audit log)

---

## Phase 7 — UX & Quality of Life

### 7.1 Dashboard
- [ ] Repo health score distribution chart (bar chart: score buckets)
- [ ] CVE summary: count by severity across all repos
- [ ] Recent Flotilla activity feed (last 10 operations)
- [ ] Pinned repo lists (user-configurable, shown as quick-access cards)
- [ ] "Last scanned" freshness indicator per repo list
- [ ] **Drift Dashboard widget**: summary of repos that have diverged from baselines (Node version drift, CI workflow drift, config file drift across repos in a list)

### 7.2 Command Palette
- [ ] Global trigger: `Ctrl+K` / `Cmd+K`
- [ ] Search: repos, repo lists, views, actions
- [ ] Recent items surfaced first
- [ ] Keyboard navigation: arrow keys + Enter to execute
- [ ] Fuzzy search

### 7.3 Notifications
- [ ] In-app notification centre (bell icon in top bar)
- [ ] Notification types: scan complete, CVE found, operation complete, PR merged/failed
- [ ] **Automated rollback detection**: monitor merged Flotilla PRs for reverts; alert immediately if someone reverts a security pin or batch operation commit
- [ ] Mark as read / clear all
- [ ] Webhook delivery: Slack / Teams / Discord (configurable URL + event filter)
- [ ] Webhook test button
- [ ] Weekly digest: exportable JSON/CSV summary of all repo health

### 7.4 PR Merge Queue
- [ ] Dedicated view for all open Flotilla-created PRs across repos
- [ ] Per-PR: CI status, merge conflicts, review status, age
- [ ] One-click merge for individual PRs
- [ ] **Batch merge**: "merge all green" button — merge all PRs where CI passes and no conflicts
- [ ] Conflict detection with link to resolve in GitHub/GitLab
- [ ] Filter by operation, repo list, label, or status
- [ ] Sort by: age, repo name, CI status

### 7.5 Repo Similarity Clustering
- [ ] Auto-group repos by tech stack fingerprint (framework, package manager, shared dependency sets)
- [ ] Cluster visualisation: repos grouped by similarity score
- [ ] "Apply to all repos like this one" — select a cluster as the target for a batch operation
- [ ] Surface patterns: "42 repos use Laravel + Vue", "18 repos use Next.js"
- [ ] Tag clusters with user-defined labels for reuse

### 7.6 Audit Log
- [ ] Every Flotilla action logged: timestamp, action type, repos affected, outcome
- [ ] Audit log view: filterable, searchable
- [ ] Audit log is append-only (no deletions)
- [ ] Export audit log as CSV

---

## Phase 8 — Advanced Features

### 8.1 Repo Health Fingerprinting
- [ ] Action SHA pinning enforcement: scan for floating tags, bulk-pin via PR
- [ ] `.env.example` drift detection (compare keys to codebase usage)
- [ ] Duplicate/conflicting workflow detection
- [ ] Stale branch detection (branches with no activity > N days)
- [ ] Missing file enforcement (CODEOWNERS, SECURITY.md, etc.) with bulk-add operation
- [ ] **Branch protection audit**: scan branch protection rules across all repos, flag inconsistencies (e.g. repo A requires reviews, repo B doesn't), offer batch enforcement of a standard ruleset
- [ ] **Drift Dashboard**: dedicated view showing where repos diverge from each other or a baseline — Node version drift, CI workflow version drift, config file drift (`.editorconfig`, `.nvmrc` differences); answer "which repos are snowflakes?"
- [ ] **Repo archival assistant**: identify stale repos (no pushes in N months, no open PRs, no recent CI runs), surface them in a list, offer batch archival via GitHub/GitLab API

### 8.2 Security & Compliance
- [ ] **Secret exposure scanner**: scan repos for accidentally committed secrets (`.env` files, API keys in code, hardcoded credentials) using pattern-based detection (regex for common key formats: AWS keys, GitHub tokens, Stripe keys, etc.)
- [ ] Optional integration with `trufflehog` / `gitleaks` for deeper scanning
- [ ] **License compliance matrix**: scan all transitive dependencies, flag non-permissive licences (GPL, AGPL in commercial projects), generate compliance report per repo or repo list
- [ ] Licence allowlist / blocklist configurable per repo list
- [ ] Export licence report as CSV / PDF

### 8.3 Reporting
- [ ] Repo health report: per repo or per repo list, exportable PDF/CSV
- [ ] Dependency age report: how many packages are N major versions behind
- [ ] CVE history report: CVEs found, patched, dismissed over time
- [ ] Operation history report: all Flotilla actions, success/failure rates

### 8.4 Custom Script Runner
- [ ] Run an arbitrary shell command across N repos (clone → run command → collect output)
- [ ] GUI interface: select target repos/list, enter command, configure parallelism
- [ ] Live output streaming per repo as commands complete
- [ ] Aggregate results view: stdout/stderr per repo, exit codes, pass/fail summary
- [ ] Preset command library (e.g. `npx depcheck`, `npm outdated --json`, `composer outdated`)
- [ ] Save custom commands as reusable presets
- [ ] Dry run: show which repos would be targeted without executing

### 8.5 Team / Config Portability
- [ ] `.flotilla/config.yaml` schema defined and documented
- [ ] `.flotilla/repo-lists/*.yaml` schema defined and documented
- [ ] **Config hierarchy**: `.flotilla/config.yaml` (global defaults) → per-repo-list overrides → per-operation UI form values (always win)
- [ ] **Per-repo operation overrides**: skip lockfile regen, use different branch prefix, etc. for specific repos within a batch
- [ ] Import config from URL (for team onboarding: "pull config from this URL")
- [ ] Config validation on import with error reporting
- [ ] Configurable inter-request delay (default: 200ms) persisted in settings

---

## Phase 9 — CLI Companion

- [ ] `git-flotilla` CLI binary (separate Rust binary in workspace)
- [ ] `git-flotilla scan [repo|list]` — run scan
- [ ] `git-flotilla cve check` — check CVEs for current scans
- [ ] `git-flotilla op apply [operation-file]` — apply a batch operation from YAML definition
- [ ] `git-flotilla repo list` — list known repos
- [ ] `git-flotilla report` — output health summary
- [ ] CLI shares config and DB with the GUI (same app data directory)
- [ ] Machine-readable output: `--json` flag on all commands

---

## Phase 10 — GitLab Support

> GitLab mirrors GitHub implementation throughout. Implement GitHub first, then add GitLab.

- [ ] GitLab PAT auth + scope validation
- [ ] GitLab repo/group discovery
- [ ] GitLab API client in `services/gitlab.rs`
- [ ] GitLab MR (Merge Request) support in batch operations (mirrors PR workflow)
- [ ] GitLab CI file scanning (`.gitlab-ci.yml`)
- [ ] GitLab-specific health checks

---

## Architecture Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Desktop framework | Tauri v2 | Rust backend, small binary, cross-platform, better than Electron for security |
| Frontend framework | Vue 3 + Composition API | Team preference, TypeScript-first, excellent Tauri ecosystem |
| State management | Pinia | Official Vue state library, TypeScript native |
| Styling | Tailwind CSS v4 | Utility-first, design tokens, dark mode trivial |
| Local DB | SQLite via sqlx | Embedded, no daemon, migrations, async |
| Secret storage | OS keychain (keyring) | Never store tokens in files |
| HTTP client | reqwest (Rust) | Async, feature-rich, well maintained |
| Git operations | git2 (libgit2) | Native bindings, no shelling out |
| CVE source | OSV.dev (primary) | Free, comprehensive, covers all ecosystems |
| Async runtime | tokio | Standard for Rust async |
| Parallelism | tokio::spawn + semaphore | Configurable concurrency for batch ops |
| Scheduling | tokio interval tasks | Lightweight, no external scheduler needed |

---

## Known Constraints & Limitations

- GitHub API rate limit: 5,000 requests/hour (authenticated). Batch scans of large orgs must respect this.
  - Strategy: cache API responses in SQLite, only re-fetch if stale (>1hr for file contents, >15min for PR status)
- GitLab API rate limit: varies by instance (GitLab.com: 2,000 req/min). Same caching strategy.
- OSV.dev: no rate limit documented, but batch all package queries using the `/v1/querybatch` endpoint
- NVD NIST API: 50 requests/30 seconds without API key; apply for key if heavy usage
- libgit2 / git2-rs does not support all Git operations — shell out to `git` for anything unsupported
- Tauri v2 on Linux requires `webkit2gtk` — document in README

---

## Open Questions

- [ ] Should Flotilla support self-hosted GitHub Enterprise / GitLab instances? (likely yes — add base URL config)
- [ ] Should there be a cloud sync option for team config? (out of scope for v1)
- [ ] Should the CLI be distributed separately from the GUI? (TBD — likely same binary with `--headless` flag)
- [ ] Support for Bitbucket? (out of scope for v1, design with extensibility in mind)
- [ ] Support for Gitea / Forgejo? (out of scope for v1)
