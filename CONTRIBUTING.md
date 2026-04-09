# Contributing to Git Flotilla

## Before You Start

1. Read `CLAUDE.md` fully — it defines all architectural rules, naming conventions, and what not to do
2. Check `PLANNING.md` for the feature you want to work on and its current status
3. Open an issue or comment on an existing one before starting significant work

## Development Setup

```bash
# Prerequisites: Rust 1.78+, Node.js 20+, pnpm 9+

# On Linux only
sudo apt install libwebkit2gtk-4.1-dev libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev

# Clone and install
git clone https://github.com/your-org/git-flotilla.git
cd git-flotilla
pnpm install

# Run dev server (Tauri + Vite HMR)
pnpm tauri dev
```

## Branch Naming

```
feat/cve-scheduler
fix/scan-missing-package-json
chore/bump-reqwest
docs/update-planning
```

## Commit Format

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(cve): add hourly OSV.dev polling scheduler
fix(scan): handle missing package.json gracefully
chore(deps): bump reqwest to 0.12
docs(planning): mark scanner as [implemented]
```

Valid scopes: `auth`, `repos`, `scan`, `packages`, `cve`, `ops`, `ui`, `db`, `cli`, `settings`

## Before Submitting a PR

**Frontend:**
```bash
pnpm typecheck    # Must pass with zero errors
pnpm lint         # Must pass with zero errors
```

**Backend:**
```bash
cargo fmt --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

## Rules (from CLAUDE.md)

- No `unwrap()` or `expect()` in Rust production paths
- No `any` in TypeScript
- No GitHub/GitLab API calls from Vue components — only through Rust commands
- No tokens in config files — OS keychain only
- No UI changes that require recompiling Rust
- Always compute a dry run result before executing a batch operation
- Update `PLANNING.md` status when implementing a feature
