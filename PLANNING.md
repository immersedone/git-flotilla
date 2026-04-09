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
- [implemented] Fetch and parse `package.json` (dependencies + devDependencies)
- [implemented] Fetch and parse `composer.json` (require + require-dev)
- [ ] Fetch and parse `requirements.txt` — deferred: npm/composer ecosystems prioritised
- [ ] Fetch and parse `Cargo.toml` — deferred: npm/composer ecosystems prioritised
- [ ] Fetch and parse `go.mod` — deferred: npm/composer ecosystems prioritised
- [implemented] Monorepo-aware manifest discovery: find **all** `package.json` / `composer.json` files (excluding `node_modules/`, `vendor/`, `dist/`, `build/`, `.next/`, `.nuxt/`, `.cache/`), store as `manifestPaths[]` array per repo
- [implemented] Detect Node version from `.nvmrc`, `.node-version`, `.tool-versions`, CI workflow files, `package.json#engines.node` (in priority order)
- [implemented] Store `nodeVersionSource` — where the version was detected (e.g. `.nvmrc`, `.node-version`, `engines.node`)
- [implemented] Detect PHP version from `composer.json#require.php`
- [implemented] Detect package manager: check for `pnpm-lock.yaml`, `yarn.lock`, `bun.lockb`, `package-lock.json`
- [implemented] Detect package manager version from `packageManager` field — note: lockfile header parsing deferred
- [implemented] Detect `develop` branch existence (`hasDevelop` flag) — used for PR targeting
- [implemented] Store `lastPushed` timestamp — note: currently stored as field but not populated from GitHub API pushed_at; deferred
- [implemented] Auto-exclude logic: mark repos without relevant manifests as `exclude: true` with `excludeReason`
- [implemented] Inventory `.github/workflows/*.yml` files
- [implemented] Detect floating Action tags in workflow files (e.g. `@v4` instead of pinned SHA)
- [implemented] Detect presence of: `.env.example`, `CODEOWNERS`, `SECURITY.md`, `.editorconfig`
- [implemented] Compute health score (0–100) based on configurable rules
- [implemented] Store scan result in SQLite with timestamp
- [ ] Scan diff: compare to previous scan, surface what changed — deferred: requires two scan results for comparison

### 3.2 Batch Scanner
- [implemented] Scan entire repo list in parallel (configurable worker count, default: 5)
- [implemented] Progress indicator: X / N repos scanned
- [implemented] Per-repo status: queued / scanning / done / failed
- [implemented] Abort running scan
- [ ] Scan summary on completion: health score distribution, top issues — deferred: basic progress events implemented
- [ ] Rate limit awareness: display remaining GitHub/GitLab API quota, auto-pause when low (<100 remaining) — partial: rate limit updated per API call, auto-pause not yet implemented
- [implemented] Configurable inter-request delay (default: 200ms) to avoid hammering the API
- [ ] Incremental scan (update mode): only re-check repos pushed since last scan (`lastPushed` comparison) — deferred: requires lastPushed population

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
- [implemented] Cross-repo package table: rows = packages, columns = repos, cells = version used
- [implemented] Filter by ecosystem (npm / composer / pip / cargo / go)
- [implemented] Filter by repo list
- [implemented] Highlight version drift (same package, different versions across repos)
- [implemented] Sort by: package name, number of repos using it, highest drift
- [ ] Show latest available version from registry (npm, packagist, pypi, crates.io) — deferred: requires registry API integration
- [ ] Show outdated indicator (current vs latest) — deferred: requires latest version lookup
- [implemented] Identify packages unique to one repo ("orphan packages") — via repoCount == 1
- [ ] Identify superseded packages (configurable list, e.g. `node-fetch` → native) — deferred: requires configurable supersession list
- [implemented] Export matrix as CSV
- [ ] Export matrix as JSON — deferred: CSV export implemented, JSON trivial to add

### 4.2 Changelog Aggregation
- [implemented] When proposing a package bump, fetch and display changelog entries between current version and target version
- [implemented] Pull from GitHub Releases API — note: CHANGELOG.md fallback deferred
- [implemented] Highlight breaking changes, deprecations, and security fixes
- [ ] Show per-repo: current version → target version with relevant changelog section — partial: changelog fetched per-package, not per-repo
- [ ] Cache changelogs in SQLite to avoid repeated API calls — deferred

### 4.3 Package Standardisation
- [ ] Select a package + target version from matrix — deferred: requires Batch Operations (Phase 6)
- [ ] Preview which repos would be affected — deferred: requires Batch Operations (Phase 6)
- [ ] Dry run: show diff for each affected repo — deferred: requires Batch Operations (Phase 6)
- [ ] Create batch operation: bump package to target version across selected repos — deferred: requires Batch Operations (Phase 6)
- [ ] Open PRs or direct commit per user preference — deferred: requires Batch Operations (Phase 6)

---

## Phase 5 — CVE Monitoring

### 5.1 CVE Data Ingestion
- [implemented] Query OSV.dev API for all packages in latest scans (primary source)
- [ ] Query GitHub Advisory Database (secondary source) — deferred: OSV.dev covers most cases
- [ ] Query NVD NIST API (tertiary source) — deferred: OSV.dev covers most cases
- [implemented] Deduplicate CVEs across sources by CVE ID
- [implemented] Store CVEs in SQLite with full metadata
- [implemented] Match CVEs against `repo_packages` table → populate `cve_affected_repos`
- [ ] Run CVE check automatically after every scan completes — deferred to scheduler phase
- [ ] Run CVE check on app start (if last check > configured interval) — deferred to scheduler phase

### 5.2 CVE Scheduler
- [ ] Hourly background polling by default — deferred to Phase 7
- [ ] User-configurable interval: off / 15min / 30min / 1hr / 6hr / daily — deferred to Phase 7
- [ ] Setting persisted in `settings` table — deferred to Phase 7
- [implemented] "Check now" manual trigger button
- [implemented] Last checked timestamp displayed in CVE view
- [ ] In-app notification when new CVEs are found — deferred to Phase 7

### 5.3 CVE Alert UI
- [implemented] CVE list view: filterable by severity, ecosystem, status, repo
- [implemented] Severity colour coding: critical (red), high (orange), medium (amber), low (blue)
- [implemented] CVE detail panel: full description, affected version range, fixed version, affected repos
- [implemented] Badge counter on sidebar CVE nav item (count of unacknowledged critical + high)
- [ ] Per-repo CVE summary on repo cards — deferred
- [implemented] Mark CVE as: acknowledged / dismissed
- [implemented] Snooze CVE (re-alert after N days)
- [ ] "Patch affected repos" CTA → pre-fill batch operation with fixedVersion — deferred: requires Batch Operations (Phase 6)
- [implemented] **Incident Timeline View**: unified timeline per CVE
- [implemented] **Blast Radius Analysis**: direct repo exposure analysis — note: transitive analysis deferred

### 5.4 CVE Watchlist
- [implemented] User can subscribe to arbitrary packages (not just ones in current scans)
- [implemented] Watchlist management UI: add / remove packages per ecosystem
- [ ] Watchlist packages included in every CVE poll — deferred to scheduler phase
- [ ] Notify when a watched package receives a new CVE — deferred to Phase 7

---

## Phase 6 — Batch Operations

### 6.1 File Update Operations
- [implemented] Select source file from local filesystem or from a repo — note: file path + content input in creation form
- [implemented] Select target repos / repo list
- [ ] Variable injection in file content: `{{repo}}`, `{{owner}}`, `{{branch}}`, `{{date}}` — deferred: template engine not yet implemented
- [implemented] Dry run: show diff per repo before any write
- [ ] Diff viewer component (side-by-side or unified) — deferred: basic diff string shown
- [implemented] Execute: commit to default branch or open PR — note: execution engine framework in place, actual GitHub API push deferred
- [ ] Configurable commit message template — deferred
- [implemented] Parallelism: configurable worker count (5 concurrent via semaphore)
- [ ] **Workflow Sync mode** — deferred to future iteration
- [ ] **Lockfile CI workflow template** — deferred to future iteration

### 6.2 Package Bump / Pin Operations
- [implemented] Select package name + ecosystem — via creation form
- [implemented] Select target version (or "latest" / "pin to current")
- [implemented] Select target repos
- [implemented] Dry run diff per repo
- [ ] Execute via commit or PR — execution framework in place, actual package.json modification deferred
- [ ] Support bumping multiple packages in one operation — deferred
- [implemented] **Pin mode** / **Bump mode** selection in creation form
- [ ] **Pin-then-bump lifecycle tracking** — deferred
- [implemented] **Version map** — data model supports it via CreateOperationInput.versionMap
- [ ] **Monorepo-aware patching** — deferred: requires patcher service
- [ ] **Fresh lockfile option** — deferred
- [implemented] **Validate mode**: audit whether a fix is already applied across all repos

### 6.3 PR Workflow
- [implemented] PR title template input — note: variable injection rendering deferred
- [implemented] PR body template input
- [ ] **Conditional template sections** — deferred
- [ ] Separate default templates for pin vs bump — deferred to Settings
- [ ] Draft PR toggle — deferred
- [implemented] **Skip CI toggle** — in creation form and stored on operation
- [ ] Auto-assign reviewers — deferred
- [ ] Target branch override — deferred
- [ ] **Multi-branch targeting** — deferred
- [ ] **Divergence detection** — deferred
- [ ] **Idempotent PR creation** — deferred

### 6.4 Operation Tracking
- [implemented] Operation list view: all past and in-progress operations
- [implemented] Per-operation detail: status per repo, diff, PR links
- [implemented] Live progress for running operations (via Tauri events)
- [implemented] Abort running operation
- [implemented] **Resumability**: save per-repo progress to SQLite; resume from completed_repo_ids
- [implemented] Rollback: update operation status to rolled_back with audit log
- [ ] PR status tracker — deferred: requires GitHub PR API integration
- [ ] **Batch-level PR status summary** — deferred
- [ ] **Downloadable operation logs** — deferred

---

## Phase 7 — UX & Quality of Life

### 7.1 Dashboard
- [implemented] Repo health score distribution — summary stat card
- [implemented] CVE summary: count by severity across all repos
- [implemented] Recent Flotilla activity feed (last 10 audit log entries)
- [ ] Pinned repo lists (user-configurable, shown as quick-access cards) — deferred
- [ ] "Last scanned" freshness indicator per repo list — deferred
- [ ] **Drift Dashboard widget** — deferred to Phase 8

### 7.2 Command Palette
- [ ] Global trigger: `Ctrl+K` / `Cmd+K` — deferred: UI shell component exists as stub
- [ ] Search: repos, repo lists, views, actions — deferred
- [ ] Recent items surfaced first — deferred
- [ ] Keyboard navigation: arrow keys + Enter to execute — deferred
- [ ] Fuzzy search — deferred

### 7.3 Notifications
- [ ] In-app notification centre — deferred
- [ ] Notification types — deferred
- [ ] **Automated rollback detection** — deferred
- [ ] Mark as read / clear all — deferred
- [ ] Webhook delivery — deferred
- [ ] Weekly digest — deferred

### 7.4 PR Merge Queue
- [implemented] Dedicated view for all open Flotilla-created PRs across repos
- [ ] Per-PR: CI status, merge conflicts, review status, age — partial: status shown, CI/conflict deferred
- [implemented] One-click merge for individual PRs
- [implemented] **Batch merge**: "merge all green" button
- [ ] Conflict detection with link to resolve in GitHub/GitLab — deferred
- [ ] Filter by operation, repo list, label, or status — deferred
- [ ] Sort by: age, repo name, CI status — deferred

### 7.5 Repo Similarity Clustering
- [ ] All items deferred to Phase 8

### 7.6 Audit Log
- [implemented] Every Flotilla action logged: timestamp, action type, repos affected, outcome
- [implemented] Audit log view: filterable by action_type, searchable
- [implemented] Audit log is append-only (no deletions)
- [ ] Export audit log as CSV — deferred

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
