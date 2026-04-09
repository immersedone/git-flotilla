# PLANNING.md — Git Flotilla

> Tracks all planned features, their scope, and implementation status.
> Claude Code: update status to `[implemented]` when a feature is complete.
> Status legend: `[ ]` planned · `[wip]` in progress · `[implemented]` done · `[blocked]` blocked

---

## Phase 1 — Foundation

### 1.1 Project Scaffold
- [implemented] Tauri v2 + Vite + Vue 3 + TypeScript project initialisation
- [implemented] Tailwind CSS v4 configured with design tokens
- [implemented] Pinia stores skeleton (one file per domain, empty state)
- [implemented] Vue Router configured with all route stubs
- [implemented] ESLint + Prettier + TypeScript strict mode
- [implemented] Rust workspace configured with all crate dependencies declared
- [implemented] SQLite DB initialised via sqlx in Tauri setup hook

### 1.2 Database Schema
- [implemented] `repos` table
- [implemented] `repo_lists` table (with `parent_id` for nesting)
- [implemented] `repo_list_members` join table
- [implemented] `scan_results` table
- [implemented] `repo_packages` table (ecosystem, name, version, is_dev)
- [implemented] `cve_alerts` table
- [implemented] `cve_affected_repos` join table
- [implemented] `cve_watchlist` table (user-subscribed packages)
- [implemented] `batch_operations` table
- [implemented] `operation_results` table
- [implemented] `audit_log` table
- [implemented] `settings` table (key/value)
- [implemented] Migration runner on app start

### 1.3 App Shell UI
- [implemented] Sidebar navigation with icon + label
- [implemented] CVE badge counter on sidebar CVE item
- [implemented] Top bar: logo, global search trigger, notification bell, API rate limit indicator, auth status
- [implemented] Main content area with router `<RouterView>`
- [implemented] Dark theme applied globally
- [implemented] Light theme toggle stored in settings — CSS custom properties with localStorage persistence, toggle in Settings view
- [implemented] Responsive layout (min width: 1024px)

---

## Phase 2 — Authentication & Repo Management

### 2.1 Authentication

- [implemented] GitHub Personal Access Token input + validation
- [ ] GitHub OAuth App flow (optional, for teams)
- [implemented] GitLab Personal Access Token input + validation
- [implemented] OS keychain storage via `keyring` crate
- [implemented] Token scope validation on save (warn on missing scopes)
- [ ] Multi-account support (multiple GitHub/GitLab accounts) — note: model supports it, UI shows first account
- [implemented] Auth status indicator in top bar
- [implemented] Token revocation / removal

### 2.2 Repo Discovery
- [implemented] List all repos accessible to authenticated GitHub user
- [implemented] List all repos in authenticated GitHub orgs
- [implemented] List all repos accessible to authenticated GitLab user
- [implemented] List all repos in authenticated GitLab groups
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
- [implemented] **Auto-exclude**: automatically mark repos without relevant manifests as excluded (with reason), respected by scanner and batch operations
- [implemented] Export repo list as YAML
- [implemented] Import repo list from YAML
- [ ] Store repo lists in `.flotilla/repo-lists/*.yaml` — currently SQLite only; YAML is manual export
- [ ] Repo list sidebar tree with expand/collapse

---

## Phase 3 — Scanning

### 3.1 Single Repo Scanner
- [implemented] Fetch and parse `package.json` (dependencies + devDependencies)
- [implemented] Fetch and parse `composer.json` (require + require-dev)
- [implemented] Fetch and parse `requirements.txt` — parser in services/scanner.rs, wired into scan_repo
- [implemented] Fetch and parse `Cargo.toml` — parser in services/scanner.rs, wired into scan_repo
- [implemented] Fetch and parse `go.mod` — parser in services/scanner.rs, wired into scan_repo
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
- [implemented] Background scheduler (tokio interval) — services/scheduler.rs wired into main.rs setup
- [ ] Configurable scan interval: manual / daily / weekly — note: scheduler runs hourly CVE polls; scan scheduling not yet configurable
- [ ] On scan completion: automatically trigger CVE check (see Phase 4)
- [implemented] In-app notification on scan completion — in-memory notification store with push/list/mark-read/clear commands

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
- [implemented] Show latest available version from registry (npm) — npm registry lookups implemented; packagist/pypi/crates.io deferred
- [implemented] Show outdated indicator (current vs latest) — for npm ecosystem; other ecosystems deferred
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
- [implemented] Hourly background polling by default — tokio interval in services/scheduler.rs, wired into main.rs setup
- [ ] User-configurable interval: off / 15min / 30min / 1hr / 6hr / daily — scheduler runs at fixed hourly interval; user config not yet wired
- [ ] Setting persisted in `settings` table — deferred
- [implemented] "Check now" manual trigger button
- [implemented] Last checked timestamp displayed in CVE view
- [implemented] In-app notification when new CVEs are found — in-memory notification store with topbar bell dropdown

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
- [implemented] Variable injection in file content: `{{repo}}`, `{{owner}}`, `{{branch}}`, `{{date}}` — mustache-style template engine in services/template.rs with `{{VAR}}` and `{{#FIELD}}content{{/FIELD}}` conditional sections
- [implemented] Dry run: show diff per repo before any write
- [ ] Diff viewer component (side-by-side or unified) — deferred: basic diff string shown
- [implemented] Execute: commit to default branch or open PR — GitHub push/PR API methods implemented (create_or_update_file, create_branch, get_branch_sha, create_pull_request, close_pull_request, delete_branch, list_pull_requests)
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
- [implemented] **Conditional template sections** — `{{#FIELD}}content{{/FIELD}}` syntax in services/template.rs
- [ ] Separate default templates for pin vs bump — deferred to Settings
- [ ] Draft PR toggle — deferred
- [implemented] **Skip CI toggle** — in creation form and stored on operation
- [ ] Auto-assign reviewers — deferred
- [ ] Target branch override — deferred
- [ ] **Multi-branch targeting** — deferred
- [ ] **Divergence detection** — deferred
- [implemented] **Idempotent PR creation** — list_pull_requests, close_pull_request, delete_branch methods on GitHubClient support detect-close-delete-recreate flow

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
- [implemented] Global trigger: `Ctrl+K` / `Cmd+K`
- [implemented] Search: repos, repo lists, views, actions — 15 commands registered
- [implemented] Recent items surfaced first — stored in localStorage
- [implemented] Keyboard navigation: arrow keys + Enter to execute
- [implemented] Fuzzy search

### 7.3 Notifications
- [implemented] In-app notification centre — in-memory store with push/list/mark-read/clear commands, topbar bell with dropdown
- [implemented] Notification types — scan completion, CVE alerts, operation status
- [ ] **Automated rollback detection** — deferred
- [implemented] Mark as read / clear all
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
- [implemented] Cluster repos by tech stack fingerprint (package_manager, node_version) with top-3 shared packages — get_repo_clusters command

### 7.6 Audit Log
- [implemented] Every Flotilla action logged: timestamp, action type, repos affected, outcome
- [implemented] Audit log view: filterable by action_type, searchable
- [implemented] Audit log is append-only (no deletions)
- [implemented] Export audit log as CSV — export_audit_log_csv command

---

## Phase 8 — Advanced Features

### 8.1 Repo Health Fingerprinting
- [ ] Action SHA pinning enforcement — deferred: floating tags detected in scan, bulk-pin via PR requires patcher
- [ ] `.env.example` drift detection — deferred
- [ ] Duplicate/conflicting workflow detection — deferred
- [ ] Stale branch detection — deferred
- [ ] Missing file enforcement — deferred
- [implemented] **Branch protection audit**: scan branch protection rules — note: placeholder returning default_branch status, GitHub API integration deferred
- [implemented] **Drift Dashboard**: dedicated view showing Node version, package manager, and PM version drift across scanned repos
- [implemented] **Repo archival assistant**: archive_repos command implemented — note: returns count, actual GitHub API archival deferred

### 8.2 Security & Compliance
- [implemented] **Secret exposure scanner**: pattern-based detection of committed secrets (.env files, API keys, credentials) via manifest and workflow file path analysis
- [ ] Optional integration with `trufflehog` / `gitleaks` — deferred
- [implemented] **License compliance matrix**: lists all packages per repo with licence status — note: actual licence lookup from registries deferred
- [ ] Licence allowlist / blocklist — deferred
- [ ] Export licence report as CSV / PDF — deferred

### 8.3 Reporting
- [implemented] CSV reporting: export_audit_log_csv, export_health_report_csv, export_cve_report_csv commands
- [ ] Webhook event delivery — deferred
- [ ] Weekly digest export — deferred
- [ ] Downloadable operation logs — deferred

### 8.4 Custom Script Runner
- [implemented] Run an arbitrary shell command across N repos (via temp dir with env vars)
- [implemented] GUI interface: select target repos, enter command, run/abort
- [ ] Live output streaming per repo — deferred: results shown after completion
- [implemented] Aggregate results view: stdout/stderr per repo, exit codes, pass/fail
- [implemented] Preset command library: CRUD for script presets
- [implemented] Save custom commands as reusable presets
- [ ] Dry run — deferred

### 8.5 Team / Config Portability
- [implemented] `.flotilla/config.yaml` schema — FlotillaConfig YAML schema in services/config.rs
- [ ] `.flotilla/repo-lists/*.yaml` schema — deferred: repo lists currently SQLite-only with manual YAML export
- [ ] Config hierarchy — deferred
- [ ] Per-repo operation overrides — deferred
- [ ] Import config from URL — deferred
- [implemented] Config validation — export/import/validate commands in services/config.rs
- [implemented] Configurable inter-request delay persisted in settings

---

## Phase 9 — CLI Companion

- [implemented] `git-flotilla-cli` CLI binary (second binary target in src-tauri)
- [implemented] `git-flotilla-cli scan --repo <ID>` — show latest scan result for a repo
- [implemented] `git-flotilla-cli scan --list <ID>` — show scan summary for a repo list
- [implemented] `git-flotilla-cli cve check` — show CVE check status summary
- [implemented] `git-flotilla-cli cve list [--severity <LEVEL>]` — list CVE alerts with optional severity filter
- [ ] `git-flotilla-cli op apply [operation-file]` — apply a batch operation from YAML definition
- [implemented] `git-flotilla-cli repo list` — list known repos
- [implemented] `git-flotilla-cli report` — output health summary (repos, scanned, avg health, CVEs, packages)
- [implemented] CLI shares DB with the GUI (same Tauri app data directory, overridable via FLOTILLA_DB_PATH)
- [implemented] Machine-readable output: `--json` flag on all commands
- [implemented] Read-only DB access (CLI opens DB in read-only mode)
- [implemented] Graceful error handling: missing DB, invalid args, empty results

---

## Phase 10 — GitLab Support

> GitLab mirrors GitHub implementation throughout. Implement GitHub first, then add GitLab.

- [implemented] GitLab PAT auth + scope validation — note: GitLab PATs don't expose scopes via API, so validation confirms token works (GET /user succeeds)
- [implemented] GitLab repo/group discovery — paginated project + group listing, deduplicated, rate limit tracked
- [implemented] GitLab API client in `services/gitlab.rs` — supports self-hosted instances via configurable base_url
- [implemented] GitLab MR (Merge Request) support — create_merge_request, list_merge_requests on GitLabClient
- [implemented] GitLab CI file scanning (`.gitlab-ci.yml`) — discover_gitlab_ci function in scanner.rs
- [ ] GitLab-specific health checks — deferred

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

## Remaining Items (v0.2+)

Low-priority or requiring external integrations. Not blocking v0.1.0 release.

| Category | Item | Notes |
|----------|------|-------|
| UI polish | Nested repo list tree UI | Data model supports it; UI shows flat list only |
| UI polish | Dedicated tag filter dropdown | Text search covers tags, but no tag-specific filter |
| UI polish | Scan diff (compare two scans) | Requires at least two scan results for same repo |
| UI polish | Per-repo CVE summary badges on repo cards | CVE alerts view has full detail |
| Scanning | Fingerprint profiles (user-defined "healthy repo" templates) | Health scoring works; user-configurable profiles deferred |
| Scanning | Configurable scan interval UI | Scheduler runs hourly; Settings UI for interval not wired |
| Scanning | Populate `lastPushed` from GitHub API `pushed_at` | Field exists but not filled from API response |
| Operations | Actual `git commit` + `git push` via GitHub Contents API | API methods exist; wiring into operation engine pending |
| Operations | Monorepo-aware package.json patching (patcher service) | Manifests detected; modification logic not implemented |
| Operations | Pin-then-bump lifecycle tracking UI | Data model supports; no dedicated tracking UI |
| PR workflow | Idempotent PR creation (detect→close→recreate) | API methods exist; orchestration in operation engine pending |
| PR workflow | Divergence detection + multi-branch targeting | PR creation supports base branch; auto-detection not wired |
| PR workflow | CODEOWNERS-based reviewer assignment | Requires parsing CODEOWNERS file |
| Compliance | Licence lookup from npm/packagist registries | Lists packages with "unknown" licence; real lookup deferred |
| Compliance | Licence allowlist/blocklist per repo list | |
| Compliance | Trufflehog/gitleaks integration | Pattern-based scanner implemented; deep tool integration deferred |
| Notifications | Webhook delivery (Slack/Teams/Discord) | In-app notifications work; external webhook delivery not implemented |
| Notifications | Weekly digest export (JSON/CSV) | CSV report exports exist; automated weekly digest not scheduled |
| Registry | Packagist/PyPI/crates.io latest version lookups | npm lookups implemented; other registries deferred |
| Config | `.flotilla/repo-lists/*.yaml` auto-sync from directory | Manual YAML import/export works; directory watch not implemented |
| Config | Per-repo operation overrides | |
| GitLab | MR support in batch operations engine | API methods exist; not wired into operation execution |
| GitLab | `.gitlab-ci.yml` scanning in scan_repo | Detection function exists; not called during scan (GitHub tree API only) |
| Reporting | PDF export for health/compliance reports | CSV exports implemented; PDF deferred |
| Superseded | Configurable superseded package list (e.g. `node-fetch` → native) | |
| Matrix | JSON export of dependency matrix | CSV export works; JSON trivial to add |

---

## Design & UI

### Current State
The UI uses Tailwind CSS v4 with a dark-first design (light theme toggle available). All views are functional with data tables, forms, and expandable detail panels. The visual design is utilitarian — information-dense, monospace accents for code values, status communicated through colour.

### Planned: Figma Design System
A comprehensive design system will be created in Figma via Claude Code's MCP Figma integration at a later date. This will include:
- Component library (buttons, inputs, badges, cards, tables, modals)
- Layout templates for each major view
- Colour token definitions (dark + light themes)
- Typography scale
- Icon set selection
- Responsive breakpoint rules

**Known issues to address in the design phase:**
- Many views use hardcoded hex colour values (`bg-[#0F1117]`) instead of Tailwind design tokens — will need migration to token-based classes once the design system defines them
- Light theme only swaps CSS custom properties on `<html>` — individual views don't yet use `dark:`/`light:` variant classes
- No consistent spacing/sizing scale across views — each view was built independently
- Component library (`src/components/ui/`) has shell components but views inline most UI patterns
- No loading skeleton / shimmer states — views show plain "Loading..." text
- Mobile/tablet responsive behaviour untested (min-width 1024px assumed)
- No empty state illustrations — text-only empty states throughout

---

## Open Questions

- [implemented] Should Flotilla support self-hosted GitHub Enterprise / GitLab instances? — Yes: GitLab client accepts configurable `base_url`; GitHub Enterprise support deferred
- [ ] Should there be a cloud sync option for team config? (out of scope for v1)
- [ ] Should the CLI be distributed separately from the GUI? (TBD — currently a second binary target in src-tauri)
- [ ] Support for Bitbucket? (out of scope for v1, design with extensibility in mind)
- [ ] Support for Gitea / Forgejo? (out of scope for v1)
