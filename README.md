# Git Flotilla

> Cross-platform GUI for managing, scanning, and batch-updating multiple GitHub and GitLab repositories at scale.

![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue)
![Stack](https://img.shields.io/badge/stack-Tauri%20%7C%20Rust%20%7C%20Vue%203%20%7C%20TypeScript-orange)
![License](https://img.shields.io/badge/license-MIT-green)

---

## What is Git Flotilla?

Git Flotilla is a desktop application for DevOps engineers and agencies who manage many repositories and need a single interface to:

- **Scan** repos for runtime versions, package managers, dependency files, and workflow health — monorepo-aware, incremental, rate-limit-safe
- **Monitor CVEs** automatically — with incident timelines, blast radius analysis, and one-click patching
- **Patch at scale** — pin/bump packages with version maps, sync workflows, push dotfiles across hundreds of repos in one resumable operation
- **Audit dependencies** — cross-repo package matrix with changelog aggregation, version drift, licence compliance, and secret exposure scanning
- **Manage PRs** — merge queue with batch merge, divergence detection, multi-branch targeting, and rollback alerts
- **Run scripts** — execute arbitrary commands across repos with live output and preset library
- **Track operations** — dry run everything, validate fixes, download operation logs, roll back commits

Built with Tauri, Rust, Vue 3, TypeScript, and Tailwind CSS.

---

## Features

### Repository Management
- Connect GitHub and GitLab accounts (PAT or OAuth)
- Auto-discover repos from orgs/groups
- Organise repos into nested lists (Client → Project → Repos)
- Tag repos with arbitrary labels for dynamic filtering
- Import/export repo lists as YAML for team sharing

### Scanning
- Detect: Node version (with source tracking — `.nvmrc`, `.node-version`, `.tool-versions`, `engines.node`), PHP version, package manager + version
- **Monorepo-aware**: discovers all manifest files in a repo, not just root
- Parse: `package.json`, `composer.json`, `requirements.txt`, `Cargo.toml`, `go.mod`
- Inventory workflow files, detect floating Action tags
- Check for required files: `.env.example`, `CODEOWNERS`, `SECURITY.md`, `.editorconfig`
- Health score (0–100) per repo based on configurable rules
- Scan diff: see what changed since your last scan
- **Incremental scan**: only re-check repos pushed since last scan
- **Rate limit awareness**: shows remaining API quota, auto-pauses when low
- **Auto-exclude**: skips repos without relevant manifests (with reason)
- Scheduled background scans

### CVE Monitoring
- Automatically checks CVEs after every scan
- Hourly background polling (configurable: off / 15min / 1hr / 6hr / daily)
- Sources: OSV.dev (primary), GitHub Advisory Database, NVD NIST
- Matches CVEs against all detected packages across all repos
- One-click "Patch affected repos" — creates a **pin** operation to lock the vulnerable package immediately
- **Incident timeline**: unified view per CVE showing detection, PRs created, merge status across all repos
- **Blast radius analysis**: dependency graph showing direct + transitive exposure to prioritise patching
- **Rollback detection**: alerts if someone reverts a Flotilla security PR
- CVE watchlist: subscribe to packages not yet in your repos
- Severity badges: critical / high / medium / low

### Dependency Intelligence
- Cross-repo package matrix: see every package used across selected repos with versions side by side
- Highlight version drift across repos
- Show latest available version from registries
- Identify orphan packages (used in only one repo)
- **Changelog aggregation**: when bumping a package, see the changelog entries between current and target version with breaking changes highlighted
- **Repo similarity clustering**: auto-group repos by tech stack fingerprint (e.g. "42 repos use Laravel + Vue")
- "Standardise to version" — bump a package to a target version across all selected repos via PR

### Batch Operations
- **Package pin**: lock to exact version + add `overrides`/`resolutions`/`pnpm.overrides` — for active security incidents
- **Package bump**: update to version range + remove overrides — for after upstream fix lands
- **Pin-then-bump lifecycle**: tracks which repos are still pinned so you know when to bump back
- **Version map**: target different safe versions per major version (e.g. major 0 → 0.30.3, major 1 → 1.13.6)
- **Monorepo-aware**: patches all matching manifest files within a repo, overrides only at root
- **File update**: push any file (with variable injection) to N repos via commit or PR
- **Workflow sync**: dedicated mode for pushing GitHub Actions workflows, `.nvmrc` updates, and `package.json` field updates across repos
- **Validate mode**: audit whether a fix is already applied across all repos without making changes
- **Dry run mode**: always preview diffs before writing anything
- **Configurable parallelism**: default 5 concurrent repos
- **Resumable**: saves per-repo progress; resumes from where it left off after crash or abort
- **PR builder**: title/body templates with `{{PACKAGE}}`, `{{VERSION}}`, `{{CVE}}`, `{{SEVERITY}}` variables and conditional sections
- **Multi-branch PRs**: target develop, staging, or other branches; auto-detect diverged branches
- **Idempotent**: re-running an operation cleanly replaces stale PRs instead of creating duplicates
- **Skip CI**: toggle to append `[skip ci]` to commits when pushing to many repos at once
- **Rollback**: revert any Flotilla-initiated commit via a PR

### PR Merge Queue
- Dedicated view for all open Flotilla-created PRs across repos
- Per-PR: CI status, merge conflicts, review status
- **Batch merge**: "merge all green" — merge all PRs where CI passes and no conflicts
- One-click merge for individual PRs

### Custom Script Runner
- Run arbitrary shell commands across N repos (clone → execute → collect output)
- Live output streaming per repo
- Preset command library (e.g. `npx depcheck`, `npm outdated --json`)
- Save custom commands as reusable presets

### Compliance & Security
- **Secret exposure scanner**: detect accidentally committed secrets (API keys, tokens, credentials) using pattern-based detection
- **Licence compliance matrix**: scan transitive dependencies, flag non-permissive licences (GPL, AGPL), generate compliance reports
- **Branch protection audit**: scan protection rules across all repos, flag inconsistencies, batch-enforce a standard ruleset
- **Drift dashboard**: see where repos diverge from each other — Node versions, CI workflows, config files — spot snowflake repos instantly
- **Repo archival assistant**: identify stale repos (no pushes, no PRs, no CI runs), batch-archive via API

### Developer Experience
- Command palette (`Ctrl+K` / `Cmd+K`) for instant navigation
- API rate limit indicator in top bar (remaining requests, reset time)
- Full audit log of every action + downloadable per-operation logs
- Webhook support: Slack, Teams, Discord
- Weekly digest export (JSON/CSV)
- CLI companion: `git-flotilla` binary for CI/scripting use

---

## Installation

### Prerequisites
- macOS 10.15+, Windows 10+, or Ubuntu 20.04+ / Debian 11+
- On Linux: `webkit2gtk-4.1` (`sudo apt install libwebkit2gtk-4.1-dev`)

### Download

Download the latest release from the [Releases page](https://github.com/your-org/git-flotilla/releases).

| Platform | Format |
|----------|--------|
| macOS | `.dmg` |
| Windows | `.msi` |
| Linux | `.AppImage` / `.deb` |

### Build from Source

**Prerequisites:** Rust 1.78+, Node.js 20+, pnpm 9+

```bash
git clone https://github.com/your-org/git-flotilla.git
cd git-flotilla

# Install frontend dependencies
pnpm install

# Run in development
pnpm tauri dev

# Build for production
pnpm tauri build
```

---

## Getting Started

1. **Connect an account** — go to Settings → Accounts, add a GitHub or GitLab Personal Access Token
2. **Discover repos** — Flotilla will list all repos accessible to your token
3. **Create a repo list** — group repos by client, project, or whatever makes sense for you
4. **Run a scan** — select your repo list and click Scan. Flotilla will parse all dependency files and check for CVEs immediately
5. **Review CVE alerts** — the CVE panel shows any vulnerabilities found, matched to the exact repos and versions affected
6. **Patch** — click "Patch affected repos" on any CVE to open a pre-filled batch operation ready to bump the package and open PRs

---

## Configuration

Flotilla stores configuration in `.flotilla/` in your home directory (or a location you configure in Settings).

```
~/.flotilla/
├── config.yaml           # App settings (scan intervals, health score weights, webhook URLs)
└── repo-lists/
    ├── client-acme.yaml  # Repo list definitions (committable, shareable)
    └── internal.yaml
```

Auth tokens are stored in your OS keychain — never in config files.

### Team Sharing

Commit your `.flotilla/repo-lists/` directory (and `config.yaml` if appropriate) to a shared repository. Team members pull it to sync repo lists and settings. Each person provides their own auth tokens.

---

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop shell | [Tauri v2](https://tauri.app) |
| Backend / logic | [Rust](https://rust-lang.org) |
| Async runtime | [Tokio](https://tokio.rs) |
| HTTP client | [reqwest](https://docs.rs/reqwest) |
| Git operations | [git2](https://docs.rs/git2) (libgit2) |
| Local database | SQLite via [sqlx](https://docs.rs/sqlx) |
| Secret storage | OS keychain via [keyring](https://docs.rs/keyring) |
| Frontend | [Vue 3](https://vuejs.org) + TypeScript |
| State | [Pinia](https://pinia.vuejs.org) |
| Routing | [Vue Router](https://router.vuejs.org) |
| Styling | [Tailwind CSS v4](https://tailwindcss.com) |
| Build tool | [Vite](https://vitejs.dev) |

---

## CVE Data Sources

| Source | Usage |
|--------|-------|
| [OSV.dev](https://osv.dev) | Primary — covers npm, composer, pip, cargo, go, and more |
| [GitHub Advisory Database](https://github.com/advisories) | Secondary — additional coverage for GitHub-hosted packages |
| [NVD NIST](https://nvd.nist.gov) | Tertiary — comprehensive CVE database |

---

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md). Please read [CLAUDE.md](./CLAUDE.md) if using AI assistance.

---

## License

MIT — see [LICENSE](./LICENSE)
