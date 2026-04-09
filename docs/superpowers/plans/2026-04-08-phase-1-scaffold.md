# Phase 1 — Project Scaffold Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Scaffold the complete Git Flotilla project — Tauri v2 + Vue 3 + TypeScript + Tailwind CSS v4 + Pinia + Vue Router — with all domain skeleton files, SQLite migrations, app shell UI, and a passing build.

**Architecture:** Tauri v2 desktop app with a Rust backend handling all I/O and a Vue 3 frontend. All inter-process communication goes through typed `tauri::command` handlers. SQLite (via sqlx) stores all persistent state in the Tauri app data directory.

**Tech Stack:** Rust 1.94 · Tauri v2 · tokio · sqlx + SQLite · git2 · keyring · reqwest · Vue 3 · TypeScript (strict) · Tailwind CSS v4 · Pinia · Vue Router 4 · @tanstack/vue-query · vee-validate + zod · pnpm 10

---

## Environment

- Working directory: `/var/www/vhosts/git-flotilla`
- Rust: 1.94.1 (`~/.cargo/bin/cargo`)
- Node: v25.2.1 (`pnpm` as package manager)
- Tauri CLI: 2.10.1 (available as `cargo tauri`)

---

## File Map

### Created — Frontend
| File | Responsibility |
|------|---------------|
| `package.json` | Frontend dependencies and scripts |
| `vite.config.ts` | Vite build config with `@tailwindcss/vite` plugin |
| `tsconfig.json` | TypeScript config (strict mode) |
| `tsconfig.node.json` | TS config for Vite/Node tooling |
| `eslint.config.ts` | ESLint flat config for Vue + TS |
| `src/env.d.ts` | Vite client type declarations |
| `src/main.ts` | Vue app entry — mount with router + pinia + vue-query |
| `src/App.vue` | Root component — shell layout |
| `src/styles/main.css` | Tailwind v4 import + `@theme` design tokens |
| `src/router/index.ts` | Vue Router with all route stubs |
| `src/stores/auth.ts` | Pinia store — auth state skeleton |
| `src/stores/repos.ts` | Pinia store — repos state skeleton |
| `src/stores/repoLists.ts` | Pinia store — repo lists skeleton |
| `src/stores/scans.ts` | Pinia store — scan results skeleton |
| `src/stores/packages.ts` | Pinia store — packages/matrix skeleton |
| `src/stores/cve.ts` | Pinia store — CVE alerts skeleton |
| `src/stores/operations.ts` | Pinia store — batch operations skeleton |
| `src/stores/mergeQueue.ts` | Pinia store — PR merge queue skeleton |
| `src/stores/scripts.ts` | Pinia store — script runner skeleton |
| `src/stores/compliance.ts` | Pinia store — compliance/security skeleton |
| `src/stores/settings.ts` | Pinia store — app settings skeleton |
| `src/services/auth.ts` | Typed Tauri invoke wrappers — auth |
| `src/services/repos.ts` | Typed Tauri invoke wrappers — repos |
| `src/services/scan.ts` | Typed Tauri invoke wrappers — scanning |
| `src/services/packages.ts` | Typed Tauri invoke wrappers — packages |
| `src/services/cve.ts` | Typed Tauri invoke wrappers — CVE |
| `src/services/operations.ts` | Typed Tauri invoke wrappers — operations |
| `src/services/mergeQueue.ts` | Typed Tauri invoke wrappers — merge queue |
| `src/services/scripts.ts` | Typed Tauri invoke wrappers — scripts |
| `src/services/compliance.ts` | Typed Tauri invoke wrappers — compliance |
| `src/types/repo.ts` | `Repo`, `RepoList` TypeScript interfaces |
| `src/types/scan.ts` | `ScanResult`, `ScanFlag` interfaces |
| `src/types/package.ts` | `RepoPackage` interface |
| `src/types/cve.ts` | `CveAlert` interface |
| `src/types/operation.ts` | `BatchOperation`, `OperationResult` interfaces |
| `src/types/mergeQueue.ts` | `MergeablePr` interface |
| `src/types/script.ts` | `ScriptPreset`, `ScriptRun` interfaces |
| `src/types/compliance.ts` | `SecretFinding`, `LicenceFinding`, `BranchProtection` interfaces |
| `src/components/layout/AppSidebar.vue` | Sidebar nav with all route links |
| `src/components/layout/AppTopbar.vue` | Top bar — search, rate limit, notif, auth |
| `src/components/ui/Button.vue` | Base button component |
| `src/components/ui/Badge.vue` | Severity / status badge component |
| `src/components/ui/Card.vue` | Surface card component |
| `src/components/ui/Input.vue` | Text input component |
| `src/components/ui/Modal.vue` | Dialog/modal component |
| `src/components/ui/Table.vue` | Data table component |
| `src/components/ui/Tooltip.vue` | Tooltip wrapper component |
| `src/components/ui/CommandPalette.vue` | Ctrl+K command palette stub |
| `src/views/Dashboard.vue` | Dashboard stub view |
| `src/views/RepoLists.vue` | Repo lists stub view |
| `src/views/Scanner.vue` | Scanner stub view |
| `src/views/Packages.vue` | Package matrix stub view |
| `src/views/CVEAlerts.vue` | CVE alerts stub view |
| `src/views/CVEIncident.vue` | CVE incident timeline stub view |
| `src/views/Operations.vue` | Batch operations stub view |
| `src/views/MergeQueue.vue` | PR merge queue stub view |
| `src/views/ScriptRunner.vue` | Custom script runner stub view |
| `src/views/DriftDashboard.vue` | Drift dashboard stub view |
| `src/views/Compliance.vue` | Compliance stub view |
| `src/views/Settings.vue` | Settings stub view |
| `src/views/Auth.vue` | Auth/account setup stub view |

### Created — Rust / Tauri Backend
| File | Responsibility |
|------|---------------|
| `src-tauri/Cargo.toml` | All Rust dependencies |
| `src-tauri/build.rs` | Tauri build script |
| `src-tauri/tauri.conf.json` | Tauri app configuration |
| `src-tauri/capabilities/default.json` | Tauri capability permissions |
| `src-tauri/src/main.rs` | App entry point — Tauri builder + command registration |
| `src-tauri/src/lib.rs` | Library root — module declarations |
| `src-tauri/src/error.rs` | Unified `AppError` type with Tauri serialisation |
| `src-tauri/src/commands/mod.rs` | Command module re-exports |
| `src-tauri/src/commands/auth.rs` | Auth command stubs |
| `src-tauri/src/commands/repos.rs` | Repo command stubs |
| `src-tauri/src/commands/scan.rs` | Scan command stubs |
| `src-tauri/src/commands/packages.rs` | Package command stubs |
| `src-tauri/src/commands/cve.rs` | CVE command stubs |
| `src-tauri/src/commands/operations.rs` | Operations command stubs |
| `src-tauri/src/commands/merge_queue.rs` | Merge queue command stubs |
| `src-tauri/src/commands/scripts.rs` | Script runner command stubs |
| `src-tauri/src/commands/compliance.rs` | Compliance command stubs |
| `src-tauri/src/commands/settings.rs` | Settings command stubs |
| `src-tauri/src/models/mod.rs` | Serde model structs (mirrors TS types) |
| `src-tauri/src/db/mod.rs` | DB pool initialisation |
| `src-tauri/src/db/migrations/001_initial.sql` | Full SQLite schema — all tables |
| `src-tauri/src/services/mod.rs` | Services module (empty, populated later) |

### Modified
| File | Change |
|------|--------|
| `.gitignore` | Add `node_modules/`, `dist/`, `src-tauri/target/`, `.flotilla/cache/` |

---

## Task 1: Project root files

**Files:**
- Create: `package.json`
- Create: `tsconfig.json`
- Create: `tsconfig.node.json`
- Create: `eslint.config.ts`
- Create: `.gitignore`

- [ ] **Step 1.1: Create `package.json`**

```json
{
  "name": "git-flotilla",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vue-tsc --noEmit && vite build",
    "preview": "vite preview",
    "tauri": "tauri",
    "typecheck": "vue-tsc --noEmit",
    "lint": "eslint ."
  },
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "@tanstack/vue-query": "^5.0.0",
    "lucide-vue-next": "^0.460.0",
    "pinia": "^2.2.0",
    "vee-validate": "^4.14.0",
    "@vee-validate/zod": "^4.14.0",
    "vue": "^3.5.0",
    "vue-router": "^4.4.0",
    "zod": "^3.23.0"
  },
  "devDependencies": {
    "@eslint/js": "^9.0.0",
    "@tauri-apps/cli": "^2.0.0",
    "@vitejs/plugin-vue": "^5.2.0",
    "@tailwindcss/vite": "^4.0.0",
    "eslint": "^9.0.0",
    "eslint-plugin-vue": "^9.0.0",
    "tailwindcss": "^4.0.0",
    "typescript": "^5.6.0",
    "typescript-eslint": "^8.0.0",
    "vite": "^6.0.0",
    "vue-tsc": "^2.2.0"
  }
}
```

- [ ] **Step 1.2: Create `tsconfig.json`**

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "useDefineForClassFields": true,
    "module": "ESNext",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "preserve",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "baseUrl": ".",
    "paths": {
      "@/*": ["./src/*"]
    }
  },
  "include": ["src/**/*.ts", "src/**/*.d.ts", "src/**/*.tsx", "src/**/*.vue"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
```

- [ ] **Step 1.3: Create `tsconfig.node.json`**

```json
{
  "compilerOptions": {
    "composite": true,
    "skipLibCheck": true,
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowSyntheticDefaultImports": true,
    "strict": true
  },
  "include": ["vite.config.ts", "eslint.config.ts"]
}
```

- [ ] **Step 1.4: Create `eslint.config.ts`**

```typescript
import js from '@eslint/js'
import eslintPluginVue from 'eslint-plugin-vue'
import tseslint from 'typescript-eslint'

export default tseslint.config(
  js.configs.recommended,
  ...tseslint.configs.recommended,
  ...eslintPluginVue.configs['flat/recommended'],
  {
    rules: {
      '@typescript-eslint/no-explicit-any': 'error',
      '@typescript-eslint/no-unused-vars': 'error',
      'vue/component-api-style': ['error', ['script-setup']],
      'vue/define-macros-order': ['error', {
        order: ['defineOptions', 'defineProps', 'defineEmits', 'defineSlots'],
      }],
    },
  },
  {
    ignores: ['dist/', 'src-tauri/target/', 'node_modules/'],
  },
)
```

- [ ] **Step 1.5: Create `.gitignore`**

```
# Dependencies
node_modules/

# Build output
dist/

# Tauri build artifacts
src-tauri/target/

# Flotilla local cache (tokens never committed)
.flotilla/cache/

# Logs
logs/
*.log

# OS
.DS_Store
Thumbs.db

# Environment
.env
.env.local

# IDE
.idea/
.vscode/settings.json
```

---

## Task 2: Vite config and Tailwind v4

**Files:**
- Create: `vite.config.ts`
- Create: `src/styles/main.css`
- Create: `src/env.d.ts`

- [ ] **Step 2.1: Create `vite.config.ts`**

```typescript
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import tailwindcss from '@tailwindcss/vite'
import path from 'path'

const host = process.env.TAURI_DEV_HOST

export default defineConfig({
  plugins: [
    tailwindcss(),
    vue(),
  ],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? { protocol: 'ws', host, port: 1421 }
      : undefined,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
})
```

- [ ] **Step 2.2: Create `src/styles/main.css`**

Design tokens use Tailwind v4's `@theme` directive. All colours are available as `bg-primary`, `text-success`, etc.

```css
@import "tailwindcss";

@theme {
  /* Brand colours */
  --color-primary: #3B82F6;
  --color-success: #22C55E;
  --color-warning: #F59E0B;
  --color-danger: #EF4444;

  /* Surface colours */
  --color-surface: #0F1117;
  --color-surface-alt: #1A1D27;
  --color-border: #2A2D3A;
  --color-muted: #6B7280;

  /* Typography */
  --font-family-mono: ui-monospace, 'Cascadia Code', 'Fira Code', monospace;
}

/* Global base styles */
html, body {
  background-color: var(--color-surface);
  color: #F9FAFB;
  font-family: system-ui, sans-serif;
  height: 100%;
  overflow: hidden;
}

#app {
  height: 100%;
}

/* Monospace accent for code values */
.font-mono {
  font-family: var(--font-family-mono);
}
```

- [ ] **Step 2.3: Create `src/env.d.ts`**

```typescript
/// <reference types="vite/client" />
```

---

## Task 3: Install frontend dependencies

- [ ] **Step 3.1: Install all packages**

Run from `/var/www/vhosts/git-flotilla`:
```bash
cd /var/www/vhosts/git-flotilla && pnpm install
```

Expected: pnpm resolves and installs all packages. No errors. A `node_modules/` directory and `pnpm-lock.yaml` are created.

---

## Task 4: Tauri backend — Cargo.toml and build files

**Files:**
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/build.rs`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/capabilities/default.json`

- [ ] **Step 4.1: Create `src-tauri/Cargo.toml`**

```toml
[package]
name    = "git-flotilla"
version = "0.1.0"
edition = "2021"

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri       = { version = "2", features = [] }
tauri-plugin-shell = "2"

# Async runtime
tokio = { version = "1", features = ["full"] }

# Serialisation
serde      = { version = "1", features = ["derive"] }
serde_json = "1"

# HTTP client (GitHub/GitLab APIs, CVE feeds)
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }

# SQLite persistence
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "sqlite", "migrate", "macros"] }

# Git operations
git2 = "0.19"

# OS keychain (token storage — never plaintext)
keyring = "3"

# Error handling
anyhow = "1"
thiserror = "2"

# Logging
tracing            = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Paths
dirs = "5"

[features]
default         = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]
```

- [ ] **Step 4.2: Create `src-tauri/build.rs`**

```rust
fn main() {
    tauri_build::build()
}
```

- [ ] **Step 4.3: Create `src-tauri/tauri.conf.json`**

```json
{
  "$schema": "https://raw.githubusercontent.com/tauri-apps/tauri/dev/crates/tauri-config-schema/schema.json",
  "productName": "Git Flotilla",
  "version": "0.1.0",
  "identifier": "com.gitflotilla.desktop",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:1420",
    "beforeDevCommand": "pnpm dev",
    "beforeBuildCommand": "pnpm build"
  },
  "app": {
    "withGlobalTauri": false,
    "windows": [
      {
        "title": "Git Flotilla",
        "width": 1400,
        "height": 900,
        "resizable": true,
        "minWidth": 1024,
        "minHeight": 700,
        "visible": true
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

- [ ] **Step 4.4: Create `src-tauri/capabilities/default.json`**

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "shell:allow-open"
  ]
}
```

- [ ] **Step 4.5: Copy placeholder icons from reference project**

```bash
mkdir -p /var/www/vhosts/git-flotilla/src-tauri/icons
cp /mnt/c/wsl-porthole-build/src-tauri/icons/*.png /var/www/vhosts/git-flotilla/src-tauri/icons/
cp /mnt/c/wsl-porthole-build/src-tauri/icons/*.ico /var/www/vhosts/git-flotilla/src-tauri/icons/
cp /mnt/c/wsl-porthole-build/src-tauri/icons/*.icns /var/www/vhosts/git-flotilla/src-tauri/icons/ 2>/dev/null || true
```

Expected: `src-tauri/icons/` populated with `.png`, `.ico`, `.icns` files.

---

## Task 5: Rust error type and project skeleton

**Files:**
- Create: `src-tauri/src/error.rs`
- Create: `src-tauri/src/lib.rs`
- Create: `src-tauri/src/main.rs`

- [ ] **Step 5.1: Create `src-tauri/src/error.rs`**

This is the unified error type. All commands return `Result<T, AppError>`. Tauri serialises `AppError` to the frontend automatically.

```rust
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
#[serde(tag = "type", content = "message")]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("GitHub API error: {0}")]
    GitHub(String),

    #[error("GitLab API error: {0}")]
    GitLab(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Keychain error: {0}")]
    Keychain(String),

    #[error("Git error: {0}")]
    Git(String),

    #[error("IO error: {0}")]
    Io(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    #[error("Operation error: {0}")]
    Operation(String),
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::Database(e.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        AppError::GitHub(e.to_string())
    }
}

impl From<git2::Error> for AppError {
    fn from(e: git2::Error) -> Self {
        AppError::Git(e.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e.to_string())
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Operation(e.to_string())
    }
}

// Required for Tauri command return type
impl From<AppError> for tauri::ipc::InvokeError {
    fn from(e: AppError) -> Self {
        tauri::ipc::InvokeError::from_anyhow(anyhow::anyhow!(e.to_string()))
    }
}

pub type AppResult<T> = Result<T, AppError>;
```

- [ ] **Step 5.2: Create `src-tauri/src/lib.rs`**

```rust
pub mod commands;
pub mod db;
pub mod error;
pub mod models;
pub mod services;

pub use error::{AppError, AppResult};
```

- [ ] **Step 5.3: Create `src-tauri/src/main.rs`**

```rust
// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

fn main() {
    // Initialise logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("git_flotilla=debug".parse().unwrap()),
        )
        .with_target(false)
        .init();

    tracing::info!("Git Flotilla starting");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Initialise DB on startup
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = git_flotilla::db::init(&app_handle).await {
                    tracing::error!("DB initialisation failed: {e}");
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Auth
            git_flotilla::commands::auth::add_account,
            git_flotilla::commands::auth::remove_account,
            git_flotilla::commands::auth::list_accounts,
            git_flotilla::commands::auth::validate_token,
            // Repos
            git_flotilla::commands::repos::discover_repos,
            git_flotilla::commands::repos::list_repos,
            git_flotilla::commands::repos::get_repo,
            // Repo lists
            git_flotilla::commands::repos::create_repo_list,
            git_flotilla::commands::repos::update_repo_list,
            git_flotilla::commands::repos::delete_repo_list,
            git_flotilla::commands::repos::list_repo_lists,
            git_flotilla::commands::repos::add_repos_to_list,
            git_flotilla::commands::repos::remove_repos_from_list,
            // Scanning
            git_flotilla::commands::scan::scan_repo,
            git_flotilla::commands::scan::scan_repo_list,
            git_flotilla::commands::scan::get_scan_result,
            git_flotilla::commands::scan::list_scan_results,
            git_flotilla::commands::scan::abort_scan,
            // Packages
            git_flotilla::commands::packages::get_dependency_matrix,
            git_flotilla::commands::packages::get_package_changelog,
            git_flotilla::commands::packages::export_matrix_csv,
            // CVE
            git_flotilla::commands::cve::check_cves,
            git_flotilla::commands::cve::list_cve_alerts,
            git_flotilla::commands::cve::acknowledge_cve,
            git_flotilla::commands::cve::dismiss_cve,
            git_flotilla::commands::cve::snooze_cve,
            git_flotilla::commands::cve::get_cve_incident,
            git_flotilla::commands::cve::get_blast_radius,
            git_flotilla::commands::cve::add_to_watchlist,
            git_flotilla::commands::cve::remove_from_watchlist,
            git_flotilla::commands::cve::list_watchlist,
            // Operations
            git_flotilla::commands::operations::create_operation,
            git_flotilla::commands::operations::run_operation,
            git_flotilla::commands::operations::abort_operation,
            git_flotilla::commands::operations::list_operations,
            git_flotilla::commands::operations::get_operation,
            git_flotilla::commands::operations::validate_operation,
            git_flotilla::commands::operations::rollback_operation,
            // Merge queue
            git_flotilla::commands::merge_queue::list_flotilla_prs,
            git_flotilla::commands::merge_queue::merge_pr,
            git_flotilla::commands::merge_queue::merge_all_green,
            // Scripts
            git_flotilla::commands::scripts::run_script,
            git_flotilla::commands::scripts::abort_script,
            git_flotilla::commands::scripts::list_presets,
            git_flotilla::commands::scripts::save_preset,
            git_flotilla::commands::scripts::delete_preset,
            // Compliance
            git_flotilla::commands::compliance::scan_secrets,
            git_flotilla::commands::compliance::scan_licences,
            git_flotilla::commands::compliance::audit_branch_protection,
            git_flotilla::commands::compliance::archive_repos,
            // Settings
            git_flotilla::commands::settings::get_settings,
            git_flotilla::commands::settings::save_settings,
            git_flotilla::commands::settings::get_rate_limit_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Git Flotilla");
}
```

---

## Task 6: Rust command stubs

Each command file contains stubbed `#[tauri::command]` functions that return `Err(AppError::Operation("not implemented".into()))`. This lets the project compile and allows frontend development to proceed.

**Files:**
- Create: `src-tauri/src/commands/mod.rs`
- Create: `src-tauri/src/commands/auth.rs`
- Create: `src-tauri/src/commands/repos.rs`
- Create: `src-tauri/src/commands/scan.rs`
- Create: `src-tauri/src/commands/packages.rs`
- Create: `src-tauri/src/commands/cve.rs`
- Create: `src-tauri/src/commands/operations.rs`
- Create: `src-tauri/src/commands/merge_queue.rs`
- Create: `src-tauri/src/commands/scripts.rs`
- Create: `src-tauri/src/commands/compliance.rs`
- Create: `src-tauri/src/commands/settings.rs`

- [ ] **Step 6.1: Create `src-tauri/src/commands/mod.rs`**

```rust
pub mod auth;
pub mod compliance;
pub mod cve;
pub mod merge_queue;
pub mod operations;
pub mod packages;
pub mod repos;
pub mod scan;
pub mod scripts;
pub mod settings;
```

- [ ] **Step 6.2: Create `src-tauri/src/commands/auth.rs`**

```rust
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccountInfo {
    pub id: String,
    pub provider: String,
    pub username: String,
    pub scopes: Vec<String>,
}

#[tauri::command]
pub async fn add_account(provider: String, token: String) -> AppResult<AccountInfo> {
    let _ = (provider, token);
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn remove_account(id: String) -> AppResult<()> {
    let _ = id;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn list_accounts() -> AppResult<Vec<AccountInfo>> {
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn validate_token(provider: String, token: String) -> AppResult<AccountInfo> {
    let _ = (provider, token);
    Err(AppError::Operation("not implemented".into()))
}
```

- [ ] **Step 6.3: Create `src-tauri/src/commands/repos.rs`**

```rust
use crate::error::{AppError, AppResult};
use crate::models::{Repo, RepoList};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRepoListInput {
    pub name: String,
    pub description: String,
    pub parent_id: Option<String>,
}

#[tauri::command]
pub async fn discover_repos(account_id: String) -> AppResult<Vec<Repo>> {
    let _ = account_id;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn list_repos(repo_list_id: Option<String>) -> AppResult<Vec<Repo>> {
    let _ = repo_list_id;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn get_repo(id: String) -> AppResult<Repo> {
    let _ = id;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn create_repo_list(input: CreateRepoListInput) -> AppResult<RepoList> {
    let _ = input;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn update_repo_list(id: String, input: CreateRepoListInput) -> AppResult<RepoList> {
    let _ = (id, input);
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn delete_repo_list(id: String) -> AppResult<()> {
    let _ = id;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn list_repo_lists() -> AppResult<Vec<RepoList>> {
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn add_repos_to_list(list_id: String, repo_ids: Vec<String>) -> AppResult<()> {
    let _ = (list_id, repo_ids);
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn remove_repos_from_list(list_id: String, repo_ids: Vec<String>) -> AppResult<()> {
    let _ = (list_id, repo_ids);
    Err(AppError::Operation("not implemented".into()))
}
```

- [ ] **Step 6.4: Create `src-tauri/src/commands/scan.rs`**

```rust
use crate::error::{AppError, AppResult};
use crate::models::ScanResult;

#[tauri::command]
pub async fn scan_repo(repo_id: String) -> AppResult<ScanResult> {
    let _ = repo_id;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn scan_repo_list(list_id: String) -> AppResult<String> {
    // Returns an operation ID for progress tracking
    let _ = list_id;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn get_scan_result(repo_id: String) -> AppResult<ScanResult> {
    let _ = repo_id;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn list_scan_results(repo_list_id: Option<String>) -> AppResult<Vec<ScanResult>> {
    let _ = repo_list_id;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn abort_scan(operation_id: String) -> AppResult<()> {
    let _ = operation_id;
    Err(AppError::Operation("not implemented".into()))
}
```

- [ ] **Step 6.5: Create `src-tauri/src/commands/packages.rs`**

```rust
use crate::error::{AppError, AppResult};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DependencyMatrix {
    pub packages: Vec<PackageRow>,
    pub repo_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct PackageRow {
    pub name: String,
    pub ecosystem: String,
    pub versions_by_repo: std::collections::HashMap<String, String>,
    pub latest_version: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChangelogEntry {
    pub version: String,
    pub body: String,
    pub published_at: String,
    pub is_breaking: bool,
}

#[tauri::command]
pub async fn get_dependency_matrix(
    repo_list_id: Option<String>,
    ecosystem: Option<String>,
) -> AppResult<DependencyMatrix> {
    let _ = (repo_list_id, ecosystem);
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn get_package_changelog(
    package_name: String,
    ecosystem: String,
    from_version: String,
    to_version: String,
) -> AppResult<Vec<ChangelogEntry>> {
    let _ = (package_name, ecosystem, from_version, to_version);
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn export_matrix_csv(repo_list_id: Option<String>) -> AppResult<String> {
    let _ = repo_list_id;
    Err(AppError::Operation("not implemented".into()))
}
```

- [ ] **Step 6.6: Create `src-tauri/src/commands/cve.rs`**

```rust
use crate::error::{AppError, AppResult};
use crate::models::CveAlert;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct BlastRadius {
    pub cve_id: String,
    pub direct_repos: Vec<String>,
    pub transitive_repos: Vec<String>,
    pub dependency_paths: Vec<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct IncidentTimeline {
    pub cve_id: String,
    pub published_at: String,
    pub detected_at: String,
    pub events: Vec<IncidentEvent>,
}

#[derive(Debug, Serialize)]
pub struct IncidentEvent {
    pub timestamp: String,
    pub event_type: String,
    pub repo_id: Option<String>,
    pub detail: String,
}

#[tauri::command]
pub async fn check_cves() -> AppResult<Vec<CveAlert>> {
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn list_cve_alerts(
    severity: Option<String>,
    status: Option<String>,
) -> AppResult<Vec<CveAlert>> {
    let _ = (severity, status);
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn acknowledge_cve(cve_id: String, repo_id: Option<String>) -> AppResult<()> {
    let _ = (cve_id, repo_id);
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn dismiss_cve(cve_id: String, repo_id: Option<String>) -> AppResult<()> {
    let _ = (cve_id, repo_id);
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn snooze_cve(cve_id: String, repo_id: Option<String>, days: u32) -> AppResult<()> {
    let _ = (cve_id, repo_id, days);
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn get_cve_incident(cve_id: String) -> AppResult<IncidentTimeline> {
    let _ = cve_id;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn get_blast_radius(cve_id: String) -> AppResult<BlastRadius> {
    let _ = cve_id;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn add_to_watchlist(package_name: String, ecosystem: String) -> AppResult<()> {
    let _ = (package_name, ecosystem);
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn remove_from_watchlist(package_name: String, ecosystem: String) -> AppResult<()> {
    let _ = (package_name, ecosystem);
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn list_watchlist() -> AppResult<Vec<serde_json::Value>> {
    Err(AppError::Operation("not implemented".into()))
}
```

- [ ] **Step 6.7: Create `src-tauri/src/commands/operations.rs`**

```rust
use crate::error::{AppError, AppResult};
use crate::models::BatchOperation;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOperationInput {
    pub operation_type: String,
    pub mode: Option<String>,
    pub target_repo_ids: Vec<String>,
    pub package_name: Option<String>,
    pub target_version: Option<String>,
    pub version_map: Option<std::collections::HashMap<String, String>>,
    pub file_path: Option<String>,
    pub file_content: Option<String>,
    pub pr_title_template: Option<String>,
    pub pr_body_template: Option<String>,
    pub branch_prefix: Option<String>,
    pub label: Option<String>,
    pub is_dry_run: bool,
    pub skip_ci: bool,
    pub also_target_branches: Vec<String>,
    pub divergence_check: bool,
    pub divergence_threshold: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct ValidateResult {
    pub repo_id: String,
    pub is_applied: bool,
    pub current_version: Option<String>,
    pub has_overrides: bool,
}

#[tauri::command]
pub async fn create_operation(input: CreateOperationInput) -> AppResult<BatchOperation> {
    let _ = input;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn run_operation(id: String) -> AppResult<()> {
    let _ = id;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn abort_operation(id: String) -> AppResult<()> {
    let _ = id;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn list_operations() -> AppResult<Vec<BatchOperation>> {
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn get_operation(id: String) -> AppResult<BatchOperation> {
    let _ = id;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn validate_operation(
    package_name: String,
    target_version: String,
    repo_ids: Vec<String>,
) -> AppResult<Vec<ValidateResult>> {
    let _ = (package_name, target_version, repo_ids);
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn rollback_operation(id: String) -> AppResult<()> {
    let _ = id;
    Err(AppError::Operation("not implemented".into()))
}
```

- [ ] **Step 6.8: Create `src-tauri/src/commands/merge_queue.rs`**

```rust
use crate::error::{AppError, AppResult};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct FlotillaPr {
    pub repo_id: String,
    pub pr_number: u64,
    pub title: String,
    pub state: String,
    pub mergeable: Option<String>,
    pub ci_status: Option<String>,
    pub operation_id: String,
    pub created_at: String,
    pub html_url: String,
}

#[tauri::command]
pub async fn list_flotilla_prs(operation_id: Option<String>) -> AppResult<Vec<FlotillaPr>> {
    let _ = operation_id;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn merge_pr(repo_id: String, pr_number: u64) -> AppResult<()> {
    let _ = (repo_id, pr_number);
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn merge_all_green(operation_id: Option<String>) -> AppResult<u32> {
    let _ = operation_id;
    Err(AppError::Operation("not implemented".into()))
}
```

- [ ] **Step 6.9: Create `src-tauri/src/commands/scripts.rs`**

```rust
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ScriptPreset {
    pub id: String,
    pub name: String,
    pub command: String,
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct ScriptRepoResult {
    pub repo_id: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
}

#[tauri::command]
pub async fn run_script(
    command: String,
    repo_ids: Vec<String>,
    parallel: u32,
) -> AppResult<String> {
    let _ = (command, repo_ids, parallel);
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn abort_script(run_id: String) -> AppResult<()> {
    let _ = run_id;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn list_presets() -> AppResult<Vec<ScriptPreset>> {
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn save_preset(preset: ScriptPreset) -> AppResult<ScriptPreset> {
    let _ = preset;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn delete_preset(id: String) -> AppResult<()> {
    let _ = id;
    Err(AppError::Operation("not implemented".into()))
}
```

- [ ] **Step 6.10: Create `src-tauri/src/commands/compliance.rs`**

```rust
use crate::error::{AppError, AppResult};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SecretFinding {
    pub repo_id: String,
    pub file_path: String,
    pub line_number: u32,
    pub secret_type: String,
    pub matched_pattern: String,
}

#[derive(Debug, Serialize)]
pub struct LicenceFinding {
    pub repo_id: String,
    pub package_name: String,
    pub ecosystem: String,
    pub licence: String,
    pub is_flagged: bool,
    pub flag_reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BranchProtectionStatus {
    pub repo_id: String,
    pub branch: String,
    pub requires_reviews: bool,
    pub requires_status_checks: bool,
    pub restricts_pushes: bool,
    pub is_compliant: bool,
    pub issues: Vec<String>,
}

#[tauri::command]
pub async fn scan_secrets(repo_ids: Vec<String>) -> AppResult<Vec<SecretFinding>> {
    let _ = repo_ids;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn scan_licences(
    repo_ids: Vec<String>,
    blocked_licences: Vec<String>,
) -> AppResult<Vec<LicenceFinding>> {
    let _ = (repo_ids, blocked_licences);
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn audit_branch_protection(
    repo_ids: Vec<String>,
) -> AppResult<Vec<BranchProtectionStatus>> {
    let _ = repo_ids;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn archive_repos(repo_ids: Vec<String>) -> AppResult<Vec<String>> {
    let _ = repo_ids;
    Err(AppError::Operation("not implemented".into()))
}
```

- [ ] **Step 6.11: Create `src-tauri/src/commands/settings.rs`**

```rust
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AppSettings {
    pub scan_interval_minutes: Option<u32>,
    pub cve_poll_interval_minutes: Option<u32>,
    pub parallel_workers: u32,
    pub request_delay_ms: u32,
    pub health_score_weights: HealthScoreWeights,
    pub webhook_url: Option<String>,
    pub webhook_events: Vec<String>,
    pub dark_mode: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthScoreWeights {
    pub has_codeowners: u32,
    pub has_security_md: u32,
    pub has_env_example: u32,
    pub has_editorconfig: u32,
    pub no_floating_action_tags: u32,
    pub deps_up_to_date: u32,
    pub no_known_cves: u32,
    pub runtime_not_eol: u32,
}

#[derive(Debug, Serialize)]
pub struct RateLimitStatus {
    pub github: Option<RateLimitInfo>,
    pub gitlab: Option<RateLimitInfo>,
}

#[derive(Debug, Serialize)]
pub struct RateLimitInfo {
    pub remaining: u32,
    pub limit: u32,
    pub reset_at: String,
}

#[tauri::command]
pub async fn get_settings() -> AppResult<AppSettings> {
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn save_settings(settings: AppSettings) -> AppResult<()> {
    let _ = settings;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn get_rate_limit_status() -> AppResult<RateLimitStatus> {
    Err(AppError::Operation("not implemented".into()))
}
```

---

## Task 7: Rust models and DB modules

**Files:**
- Create: `src-tauri/src/models/mod.rs`
- Create: `src-tauri/src/db/mod.rs`
- Create: `src-tauri/src/db/migrations/001_initial.sql`
- Create: `src-tauri/src/services/mod.rs`

- [ ] **Step 7.1: Create `src-tauri/src/models/mod.rs`**

All structs mirror TypeScript types exactly. Field names use `snake_case` in Rust and are serialised to `camelCase` for the frontend.

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Repo {
    pub id: String,
    pub provider: String,
    pub owner: String,
    pub name: String,
    pub full_name: String,
    pub url: String,
    pub default_branch: String,
    pub is_private: bool,
    pub last_scanned_at: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RepoList {
    pub id: String,
    pub name: String,
    pub description: String,
    pub repo_ids: Vec<String>,
    pub parent_id: Option<String>,
    pub exclude_patterns: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub repo_id: String,
    pub scanned_at: String,
    pub manifest_paths: Vec<String>,
    pub node_version: Option<String>,
    pub node_version_source: Option<String>,
    pub php_version: Option<String>,
    pub package_manager: Option<String>,
    pub package_manager_version: Option<String>,
    pub has_develop: bool,
    pub last_pushed: Option<String>,
    pub has_dot_env_example: bool,
    pub workflow_files: Vec<String>,
    pub health_score: u32,
    pub flags: Vec<ScanFlag>,
    pub excluded: bool,
    pub exclude_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScanFlag {
    pub flag_type: String,
    pub message: String,
    pub severity: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RepoPackage {
    pub repo_id: String,
    pub ecosystem: String,
    pub name: String,
    pub version: String,
    pub is_dev: bool,
    pub scanned_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CveAlert {
    pub id: String,
    pub package_name: String,
    pub ecosystem: String,
    pub severity: String,
    pub summary: String,
    pub affected_version_range: String,
    pub fixed_version: Option<String>,
    pub published_at: String,
    pub detected_at: String,
    pub affected_repos: Vec<String>,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BatchOperation {
    pub id: String,
    #[serde(rename = "type")]
    pub operation_type: String,
    pub mode: Option<String>,
    pub status: String,
    pub target_repo_ids: Vec<String>,
    pub completed_repo_ids: Vec<String>,
    pub version_map: Option<HashMap<String, String>>,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub results: Vec<OperationResult>,
    pub is_dry_run: bool,
    pub skip_ci: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OperationResult {
    pub repo_id: String,
    pub status: String,
    pub pr_url: Option<String>,
    pub error: Option<String>,
    pub diff: Option<String>,
}
```

- [ ] **Step 7.2: Create `src-tauri/src/db/mod.rs`**

```rust
use sqlx::SqlitePool;
use std::sync::OnceLock;
use tauri::AppHandle;

static DB_POOL: OnceLock<SqlitePool> = OnceLock::new();

pub fn pool() -> &'static SqlitePool {
    DB_POOL.get().expect("DB pool not initialised — call db::init() first")
}

pub async fn init(app: &AppHandle) -> Result<(), sqlx::Error> {
    let data_dir = app.path().app_data_dir()
        .expect("failed to resolve app data dir");

    std::fs::create_dir_all(&data_dir)
        .map_err(|e| sqlx::Error::Io(e))?;

    let db_path = data_dir.join("flotilla.db");
    let db_url = format!("sqlite://{}?mode=rwc", db_path.display());

    tracing::info!("Opening DB at {}", db_path.display());

    let pool = SqlitePool::connect(&db_url).await?;

    // Run embedded migrations
    sqlx::migrate!("./src/db/migrations").run(&pool).await?;

    DB_POOL.set(pool).map_err(|_| {
        sqlx::Error::PoolTimedOut
    })?;

    tracing::info!("DB ready — migrations applied");
    Ok(())
}
```

- [ ] **Step 7.3: Create `src-tauri/src/db/migrations/001_initial.sql`**

```sql
-- repos
CREATE TABLE IF NOT EXISTS repos (
    id              TEXT PRIMARY KEY,  -- "{provider}:{owner}/{name}"
    provider        TEXT NOT NULL,     -- "github" | "gitlab"
    owner           TEXT NOT NULL,
    name            TEXT NOT NULL,
    full_name       TEXT NOT NULL,
    url             TEXT NOT NULL,
    default_branch  TEXT NOT NULL DEFAULT 'main',
    is_private      INTEGER NOT NULL DEFAULT 0,
    last_scanned_at TEXT,
    tags            TEXT NOT NULL DEFAULT '[]',  -- JSON array
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- repo_lists
CREATE TABLE IF NOT EXISTS repo_lists (
    id               TEXT PRIMARY KEY,
    name             TEXT NOT NULL,
    description      TEXT NOT NULL DEFAULT '',
    parent_id        TEXT REFERENCES repo_lists(id) ON DELETE SET NULL,
    exclude_patterns TEXT NOT NULL DEFAULT '[]',  -- JSON array
    created_at       TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at       TEXT NOT NULL DEFAULT (datetime('now'))
);

-- repo_list_members (join table)
CREATE TABLE IF NOT EXISTS repo_list_members (
    list_id TEXT NOT NULL REFERENCES repo_lists(id) ON DELETE CASCADE,
    repo_id TEXT NOT NULL REFERENCES repos(id) ON DELETE CASCADE,
    added_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (list_id, repo_id)
);

-- scan_results
CREATE TABLE IF NOT EXISTS scan_results (
    id                      TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    repo_id                 TEXT NOT NULL REFERENCES repos(id) ON DELETE CASCADE,
    scanned_at              TEXT NOT NULL,
    manifest_paths          TEXT NOT NULL DEFAULT '[]',   -- JSON array
    node_version            TEXT,
    node_version_source     TEXT,
    php_version             TEXT,
    package_manager         TEXT,
    package_manager_version TEXT,
    has_develop             INTEGER NOT NULL DEFAULT 0,
    last_pushed             TEXT,
    has_dot_env_example     INTEGER NOT NULL DEFAULT 0,
    workflow_files          TEXT NOT NULL DEFAULT '[]',   -- JSON array
    health_score            INTEGER NOT NULL DEFAULT 0,
    flags                   TEXT NOT NULL DEFAULT '[]',   -- JSON array of ScanFlag
    excluded                INTEGER NOT NULL DEFAULT 0,
    exclude_reason          TEXT
);

CREATE INDEX IF NOT EXISTS idx_scan_results_repo_id ON scan_results(repo_id);
CREATE INDEX IF NOT EXISTS idx_scan_results_scanned_at ON scan_results(scanned_at DESC);

-- repo_packages (dependencies extracted from manifests)
CREATE TABLE IF NOT EXISTS repo_packages (
    id          TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    repo_id     TEXT NOT NULL REFERENCES repos(id) ON DELETE CASCADE,
    ecosystem   TEXT NOT NULL,
    name        TEXT NOT NULL,
    version     TEXT NOT NULL,
    is_dev      INTEGER NOT NULL DEFAULT 0,
    scanned_at  TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_repo_packages_repo_id ON repo_packages(repo_id);
CREATE INDEX IF NOT EXISTS idx_repo_packages_name ON repo_packages(name);
CREATE INDEX IF NOT EXISTS idx_repo_packages_ecosystem ON repo_packages(ecosystem);

-- cve_alerts
CREATE TABLE IF NOT EXISTS cve_alerts (
    id                     TEXT PRIMARY KEY,  -- CVE ID e.g. "CVE-2024-12345"
    package_name           TEXT NOT NULL,
    ecosystem              TEXT NOT NULL,
    severity               TEXT NOT NULL,
    summary                TEXT NOT NULL,
    affected_version_range TEXT NOT NULL,
    fixed_version          TEXT,
    published_at           TEXT NOT NULL,
    detected_at            TEXT NOT NULL,
    status                 TEXT NOT NULL DEFAULT 'new'
);

CREATE INDEX IF NOT EXISTS idx_cve_alerts_severity ON cve_alerts(severity);
CREATE INDEX IF NOT EXISTS idx_cve_alerts_status ON cve_alerts(status);

-- cve_affected_repos (join table)
CREATE TABLE IF NOT EXISTS cve_affected_repos (
    cve_id  TEXT NOT NULL REFERENCES cve_alerts(id) ON DELETE CASCADE,
    repo_id TEXT NOT NULL REFERENCES repos(id) ON DELETE CASCADE,
    status  TEXT NOT NULL DEFAULT 'new',  -- per-repo status override
    snoozed_until TEXT,
    PRIMARY KEY (cve_id, repo_id)
);

-- cve_watchlist (user-subscribed packages)
CREATE TABLE IF NOT EXISTS cve_watchlist (
    id           TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    package_name TEXT NOT NULL,
    ecosystem    TEXT NOT NULL,
    added_at     TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(package_name, ecosystem)
);

-- batch_operations
CREATE TABLE IF NOT EXISTS batch_operations (
    id                   TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    operation_type       TEXT NOT NULL,
    mode                 TEXT,  -- "pin" | "bump" | null
    status               TEXT NOT NULL DEFAULT 'pending',
    target_repo_ids      TEXT NOT NULL DEFAULT '[]',    -- JSON array
    completed_repo_ids   TEXT NOT NULL DEFAULT '[]',    -- JSON array (resumability)
    version_map          TEXT,  -- JSON object or null
    is_dry_run           INTEGER NOT NULL DEFAULT 0,
    skip_ci              INTEGER NOT NULL DEFAULT 0,
    created_at           TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at         TEXT
);

CREATE INDEX IF NOT EXISTS idx_batch_operations_status ON batch_operations(status);

-- operation_results (per-repo outcome)
CREATE TABLE IF NOT EXISTS operation_results (
    id           TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    operation_id TEXT NOT NULL REFERENCES batch_operations(id) ON DELETE CASCADE,
    repo_id      TEXT NOT NULL REFERENCES repos(id) ON DELETE CASCADE,
    status       TEXT NOT NULL,
    pr_url       TEXT,
    pre_change_sha TEXT,  -- for rollback
    error        TEXT,
    diff         TEXT,
    created_at   TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_operation_results_operation_id ON operation_results(operation_id);

-- audit_log (append-only — no deletes allowed)
CREATE TABLE IF NOT EXISTS audit_log (
    id           TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    timestamp    TEXT NOT NULL DEFAULT (datetime('now')),
    action_type  TEXT NOT NULL,
    repo_ids     TEXT NOT NULL DEFAULT '[]',  -- JSON array
    operation_id TEXT,
    outcome      TEXT NOT NULL,
    detail       TEXT
);

CREATE INDEX IF NOT EXISTS idx_audit_log_timestamp ON audit_log(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_audit_log_action_type ON audit_log(action_type);

-- settings (key/value)
CREATE TABLE IF NOT EXISTS settings (
    key        TEXT PRIMARY KEY,
    value      TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- script_presets (custom script runner)
CREATE TABLE IF NOT EXISTS script_presets (
    id          TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    name        TEXT NOT NULL,
    command     TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
```

- [ ] **Step 7.4: Create `src-tauri/src/services/mod.rs`**

```rust
// Services will be implemented in later phases.
// This module stub is required for lib.rs compilation.
```

---

## Task 8: TypeScript type definitions

**Files:** All in `src/types/`

- [ ] **Step 8.1: Create `src/types/repo.ts`**

```typescript
export interface Repo {
  id: string              // "{provider}:{owner}/{name}"
  provider: 'github' | 'gitlab'
  owner: string
  name: string
  fullName: string
  url: string
  defaultBranch: string
  isPrivate: boolean
  lastScannedAt: string | null
  tags: string[]
}

export interface RepoList {
  id: string
  name: string
  description: string
  repoIds: string[]
  parentId: string | null
  excludePatterns: string[]
  createdAt: string
  updatedAt: string
}
```

- [ ] **Step 8.2: Create `src/types/scan.ts`**

```typescript
export interface ScanFlag {
  flagType: string
  message: string
  severity: 'critical' | 'high' | 'medium' | 'low' | 'info'
}

export interface ScanResult {
  repoId: string
  scannedAt: string
  manifestPaths: string[]
  nodeVersion: string | null
  nodeVersionSource: string | null
  phpVersion: string | null
  packageManager: 'npm' | 'pnpm' | 'yarn' | 'bun' | 'composer' | null
  packageManagerVersion: string | null
  hasDevelop: boolean
  lastPushed: string | null
  hasDotEnvExample: boolean
  workflowFiles: string[]
  healthScore: number
  flags: ScanFlag[]
  excluded: boolean
  excludeReason: string | null
}
```

- [ ] **Step 8.3: Create `src/types/package.ts`**

```typescript
export type Ecosystem = 'npm' | 'composer' | 'pip' | 'cargo' | 'go'

export interface RepoPackage {
  repoId: string
  ecosystem: Ecosystem
  name: string
  version: string
  isDev: boolean
  scannedAt: string
}

export interface ChangelogEntry {
  version: string
  body: string
  publishedAt: string
  isBreaking: boolean
}

export interface PackageRow {
  name: string
  ecosystem: Ecosystem
  versionsByRepo: Record<string, string>
  latestVersion: string | null
}

export interface DependencyMatrix {
  packages: PackageRow[]
  repoIds: string[]
}
```

- [ ] **Step 8.4: Create `src/types/cve.ts`**

```typescript
export type CveSeverity = 'critical' | 'high' | 'medium' | 'low'
export type CveStatus = 'new' | 'acknowledged' | 'patched' | 'dismissed'

export interface CveAlert {
  id: string
  packageName: string
  ecosystem: string
  severity: CveSeverity
  summary: string
  affectedVersionRange: string
  fixedVersion: string | null
  publishedAt: string
  detectedAt: string
  affectedRepos: string[]
  status: CveStatus
}

export interface IncidentEvent {
  timestamp: string
  eventType: string
  repoId: string | null
  detail: string
}

export interface IncidentTimeline {
  cveId: string
  publishedAt: string
  detectedAt: string
  events: IncidentEvent[]
}

export interface BlastRadius {
  cveId: string
  directRepos: string[]
  transitiveRepos: string[]
  dependencyPaths: string[][]
}
```

- [ ] **Step 8.5: Create `src/types/operation.ts`**

```typescript
export type OperationType =
  | 'file_update'
  | 'package_pin'
  | 'package_bump'
  | 'workflow_sync'
  | 'script_run'
  | 'pr_create'
  | 'commit'

export type OperationMode = 'pin' | 'bump' | null

export type OperationStatus =
  | 'pending'
  | 'running'
  | 'completed'
  | 'failed'
  | 'rolled_back'
  | 'paused'

export interface OperationResult {
  repoId: string
  status: string
  prUrl: string | null
  error: string | null
  diff: string | null
}

export interface BatchOperation {
  id: string
  type: OperationType
  mode: OperationMode
  status: OperationStatus
  targetRepoIds: string[]
  completedRepoIds: string[]
  versionMap: Record<string, string> | null
  createdAt: string
  completedAt: string | null
  results: OperationResult[]
  isDryRun: boolean
  skipCi: boolean
}

export interface ValidateResult {
  repoId: string
  isApplied: boolean
  currentVersion: string | null
  hasOverrides: boolean
}

export interface CreateOperationInput {
  operationType: OperationType
  mode?: OperationMode
  targetRepoIds: string[]
  packageName?: string
  targetVersion?: string
  versionMap?: Record<string, string>
  filePath?: string
  fileContent?: string
  prTitleTemplate?: string
  prBodyTemplate?: string
  branchPrefix?: string
  label?: string
  isDryRun: boolean
  skipCi: boolean
  alsoTargetBranches: string[]
  divergenceCheck: boolean
  divergenceThreshold?: number
}
```

- [ ] **Step 8.6: Create `src/types/mergeQueue.ts`**

```typescript
export interface FlotillaPr {
  repoId: string
  prNumber: number
  title: string
  state: string
  mergeable: string | null
  ciStatus: string | null
  operationId: string
  createdAt: string
  htmlUrl: string
}
```

- [ ] **Step 8.7: Create `src/types/script.ts`**

```typescript
export interface ScriptPreset {
  id: string
  name: string
  command: string
  description: string
}

export interface ScriptRepoResult {
  repoId: string
  exitCode: number
  stdout: string
  stderr: string
  durationMs: number
}

export interface ScriptRun {
  id: string
  command: string
  repoIds: string[]
  results: ScriptRepoResult[]
  status: 'running' | 'completed' | 'aborted'
  startedAt: string
  completedAt: string | null
}
```

- [ ] **Step 8.8: Create `src/types/compliance.ts`**

```typescript
export interface SecretFinding {
  repoId: string
  filePath: string
  lineNumber: number
  secretType: string
  matchedPattern: string
}

export interface LicenceFinding {
  repoId: string
  packageName: string
  ecosystem: string
  licence: string
  isFlagged: boolean
  flagReason: string | null
}

export interface BranchProtectionStatus {
  repoId: string
  branch: string
  requiresReviews: boolean
  requiresStatusChecks: boolean
  restrictsPushes: boolean
  isCompliant: boolean
  issues: string[]
}
```

---

## Task 9: Typed service wrappers

Each service file exposes typed functions that call `invoke()` from `@tauri-apps/api/core`. Components must **never** call `invoke` directly — always use these wrappers.

**Files:** All in `src/services/`

- [ ] **Step 9.1: Create `src/services/auth.ts`**

```typescript
import { invoke } from '@tauri-apps/api/core'

export interface AccountInfo {
  id: string
  provider: string
  username: string
  scopes: string[]
}

export function addAccount(provider: string, token: string): Promise<AccountInfo> {
  return invoke('add_account', { provider, token })
}

export function removeAccount(id: string): Promise<void> {
  return invoke('remove_account', { id })
}

export function listAccounts(): Promise<AccountInfo[]> {
  return invoke('list_accounts')
}

export function validateToken(provider: string, token: string): Promise<AccountInfo> {
  return invoke('validate_token', { provider, token })
}
```

- [ ] **Step 9.2: Create `src/services/repos.ts`**

```typescript
import { invoke } from '@tauri-apps/api/core'
import type { Repo, RepoList } from '@/types/repo'

export interface CreateRepoListInput {
  name: string
  description: string
  parentId?: string
}

export function discoverRepos(accountId: string): Promise<Repo[]> {
  return invoke('discover_repos', { accountId })
}

export function listRepos(repoListId?: string): Promise<Repo[]> {
  return invoke('list_repos', { repoListId: repoListId ?? null })
}

export function getRepo(id: string): Promise<Repo> {
  return invoke('get_repo', { id })
}

export function createRepoList(input: CreateRepoListInput): Promise<RepoList> {
  return invoke('create_repo_list', { input })
}

export function updateRepoList(id: string, input: CreateRepoListInput): Promise<RepoList> {
  return invoke('update_repo_list', { id, input })
}

export function deleteRepoList(id: string): Promise<void> {
  return invoke('delete_repo_list', { id })
}

export function listRepoLists(): Promise<RepoList[]> {
  return invoke('list_repo_lists')
}

export function addReposToList(listId: string, repoIds: string[]): Promise<void> {
  return invoke('add_repos_to_list', { listId, repoIds })
}

export function removeReposFromList(listId: string, repoIds: string[]): Promise<void> {
  return invoke('remove_repos_from_list', { listId, repoIds })
}
```

- [ ] **Step 9.3: Create `src/services/scan.ts`**

```typescript
import { invoke } from '@tauri-apps/api/core'
import type { ScanResult } from '@/types/scan'

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
```

- [ ] **Step 9.4: Create `src/services/packages.ts`**

```typescript
import { invoke } from '@tauri-apps/api/core'
import type { DependencyMatrix, ChangelogEntry } from '@/types/package'

export function getDependencyMatrix(
  repoListId?: string,
  ecosystem?: string,
): Promise<DependencyMatrix> {
  return invoke('get_dependency_matrix', {
    repoListId: repoListId ?? null,
    ecosystem: ecosystem ?? null,
  })
}

export function getPackageChangelog(
  packageName: string,
  ecosystem: string,
  fromVersion: string,
  toVersion: string,
): Promise<ChangelogEntry[]> {
  return invoke('get_package_changelog', { packageName, ecosystem, fromVersion, toVersion })
}

export function exportMatrixCsv(repoListId?: string): Promise<string> {
  return invoke('export_matrix_csv', { repoListId: repoListId ?? null })
}
```

- [ ] **Step 9.5: Create `src/services/cve.ts`**

```typescript
import { invoke } from '@tauri-apps/api/core'
import type { CveAlert, IncidentTimeline, BlastRadius } from '@/types/cve'

export function checkCves(): Promise<CveAlert[]> {
  return invoke('check_cves')
}

export function listCveAlerts(severity?: string, status?: string): Promise<CveAlert[]> {
  return invoke('list_cve_alerts', {
    severity: severity ?? null,
    status: status ?? null,
  })
}

export function acknowledgeCve(cveId: string, repoId?: string): Promise<void> {
  return invoke('acknowledge_cve', { cveId, repoId: repoId ?? null })
}

export function dismissCve(cveId: string, repoId?: string): Promise<void> {
  return invoke('dismiss_cve', { cveId, repoId: repoId ?? null })
}

export function snoozeCve(cveId: string, repoId: string | undefined, days: number): Promise<void> {
  return invoke('snooze_cve', { cveId, repoId: repoId ?? null, days })
}

export function getCveIncident(cveId: string): Promise<IncidentTimeline> {
  return invoke('get_cve_incident', { cveId })
}

export function getBlastRadius(cveId: string): Promise<BlastRadius> {
  return invoke('get_blast_radius', { cveId })
}

export function addToWatchlist(packageName: string, ecosystem: string): Promise<void> {
  return invoke('add_to_watchlist', { packageName, ecosystem })
}

export function removeFromWatchlist(packageName: string, ecosystem: string): Promise<void> {
  return invoke('remove_from_watchlist', { packageName, ecosystem })
}
```

- [ ] **Step 9.6: Create `src/services/operations.ts`**

```typescript
import { invoke } from '@tauri-apps/api/core'
import type { BatchOperation, CreateOperationInput, ValidateResult } from '@/types/operation'

export function createOperation(input: CreateOperationInput): Promise<BatchOperation> {
  return invoke('create_operation', { input })
}

export function runOperation(id: string): Promise<void> {
  return invoke('run_operation', { id })
}

export function abortOperation(id: string): Promise<void> {
  return invoke('abort_operation', { id })
}

export function listOperations(): Promise<BatchOperation[]> {
  return invoke('list_operations')
}

export function getOperation(id: string): Promise<BatchOperation> {
  return invoke('get_operation', { id })
}

export function validateOperation(
  packageName: string,
  targetVersion: string,
  repoIds: string[],
): Promise<ValidateResult[]> {
  return invoke('validate_operation', { packageName, targetVersion, repoIds })
}

export function rollbackOperation(id: string): Promise<void> {
  return invoke('rollback_operation', { id })
}
```

- [ ] **Step 9.7: Create `src/services/mergeQueue.ts`**

```typescript
import { invoke } from '@tauri-apps/api/core'
import type { FlotillaPr } from '@/types/mergeQueue'

export function listFlotillaPrs(operationId?: string): Promise<FlotillaPr[]> {
  return invoke('list_flotilla_prs', { operationId: operationId ?? null })
}

export function mergePr(repoId: string, prNumber: number): Promise<void> {
  return invoke('merge_pr', { repoId, prNumber })
}

export function mergeAllGreen(operationId?: string): Promise<number> {
  return invoke('merge_all_green', { operationId: operationId ?? null })
}
```

- [ ] **Step 9.8: Create `src/services/scripts.ts`**

```typescript
import { invoke } from '@tauri-apps/api/core'
import type { ScriptPreset } from '@/types/script'

export function runScript(command: string, repoIds: string[], parallel: number): Promise<string> {
  return invoke('run_script', { command, repoIds, parallel })
}

export function abortScript(runId: string): Promise<void> {
  return invoke('abort_script', { runId })
}

export function listPresets(): Promise<ScriptPreset[]> {
  return invoke('list_presets')
}

export function savePreset(preset: ScriptPreset): Promise<ScriptPreset> {
  return invoke('save_preset', { preset })
}

export function deletePreset(id: string): Promise<void> {
  return invoke('delete_preset', { id })
}
```

- [ ] **Step 9.9: Create `src/services/compliance.ts`**

```typescript
import { invoke } from '@tauri-apps/api/core'
import type { SecretFinding, LicenceFinding, BranchProtectionStatus } from '@/types/compliance'

export function scanSecrets(repoIds: string[]): Promise<SecretFinding[]> {
  return invoke('scan_secrets', { repoIds })
}

export function scanLicences(repoIds: string[], blockedLicences: string[]): Promise<LicenceFinding[]> {
  return invoke('scan_licences', { repoIds, blockedLicences })
}

export function auditBranchProtection(repoIds: string[]): Promise<BranchProtectionStatus[]> {
  return invoke('audit_branch_protection', { repoIds })
}

export function archiveRepos(repoIds: string[]): Promise<string[]> {
  return invoke('archive_repos', { repoIds })
}
```

---

## Task 10: Pinia stores

Each store is a skeleton with typed state, no-op actions, and reactive getters. Actions will be implemented in later phases.

**Files:** All in `src/stores/`

- [ ] **Step 10.1: Create `src/stores/auth.ts`**

```typescript
import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { AccountInfo } from '@/services/auth'

export const useAuthStore = defineStore('auth', () => {
  const accounts = ref<AccountInfo[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const hasAccounts = computed(() => accounts.value.length > 0)
  const githubAccount = computed(() => accounts.value.find(a => a.provider === 'github'))
  const gitlabAccount = computed(() => accounts.value.find(a => a.provider === 'gitlab'))

  return { accounts, isLoading, error, hasAccounts, githubAccount, gitlabAccount }
})
```

Wait — the `computed` import is missing. Fix:

```typescript
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { AccountInfo } from '@/services/auth'

export const useAuthStore = defineStore('auth', () => {
  const accounts = ref<AccountInfo[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const hasAccounts = computed(() => accounts.value.length > 0)
  const githubAccount = computed(() => accounts.value.find(a => a.provider === 'github'))
  const gitlabAccount = computed(() => accounts.value.find(a => a.provider === 'gitlab'))

  return { accounts, isLoading, error, hasAccounts, githubAccount, gitlabAccount }
})
```

- [ ] **Step 10.2: Create `src/stores/repos.ts`**

```typescript
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Repo } from '@/types/repo'

export const useReposStore = defineStore('repos', () => {
  const repos = ref<Repo[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const searchQuery = ref('')

  const filteredRepos = computed(() => {
    if (!searchQuery.value) return repos.value
    const q = searchQuery.value.toLowerCase()
    return repos.value.filter(r =>
      r.fullName.toLowerCase().includes(q) ||
      r.tags.some(t => t.toLowerCase().includes(q)),
    )
  })

  return { repos, isLoading, error, searchQuery, filteredRepos }
})
```

- [ ] **Step 10.3: Create `src/stores/repoLists.ts`**

```typescript
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { RepoList } from '@/types/repo'

export const useRepoListsStore = defineStore('repoLists', () => {
  const lists = ref<RepoList[]>([])
  const selectedListId = ref<string | null>(null)
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const selectedList = computed(() =>
    lists.value.find(l => l.id === selectedListId.value) ?? null,
  )

  const rootLists = computed(() => lists.value.filter(l => l.parentId === null))

  return { lists, selectedListId, selectedList, rootLists, isLoading, error }
})
```

- [ ] **Step 10.4: Create remaining skeleton stores**

Create `src/stores/scans.ts`:
```typescript
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { ScanResult } from '@/types/scan'

export const useScansStore = defineStore('scans', () => {
  const results = ref<ScanResult[]>([])
  const isScanning = ref(false)
  const scanProgress = ref({ current: 0, total: 0 })
  const error = ref<string | null>(null)

  const averageHealthScore = computed(() => {
    if (results.value.length === 0) return 0
    return Math.round(results.value.reduce((sum, r) => sum + r.healthScore, 0) / results.value.length)
  })

  return { results, isScanning, scanProgress, error, averageHealthScore }
})
```

Create `src/stores/packages.ts`:
```typescript
import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { DependencyMatrix } from '@/types/package'

export const usePackagesStore = defineStore('packages', () => {
  const matrix = ref<DependencyMatrix | null>(null)
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const selectedEcosystem = ref<string | null>(null)

  return { matrix, isLoading, error, selectedEcosystem }
})
```

Create `src/stores/cve.ts`:
```typescript
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { CveAlert } from '@/types/cve'

export const useCveStore = defineStore('cve', () => {
  const alerts = ref<CveAlert[]>([])
  const isLoading = ref(false)
  const lastCheckedAt = ref<string | null>(null)
  const error = ref<string | null>(null)

  const criticalCount = computed(() =>
    alerts.value.filter(a => a.severity === 'critical' && a.status === 'new').length,
  )
  const highCount = computed(() =>
    alerts.value.filter(a => a.severity === 'high' && a.status === 'new').length,
  )
  const badgeCount = computed(() => criticalCount.value + highCount.value)

  return { alerts, isLoading, lastCheckedAt, error, criticalCount, highCount, badgeCount }
})
```

Create `src/stores/operations.ts`:
```typescript
import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { BatchOperation } from '@/types/operation'

export const useOperationsStore = defineStore('operations', () => {
  const operations = ref<BatchOperation[]>([])
  const activeOperationId = ref<string | null>(null)
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  return { operations, activeOperationId, isLoading, error }
})
```

Create `src/stores/mergeQueue.ts`:
```typescript
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { FlotillaPr } from '@/types/mergeQueue'

export const useMergeQueueStore = defineStore('mergeQueue', () => {
  const prs = ref<FlotillaPr[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const greenPrs = computed(() =>
    prs.value.filter(p => p.ciStatus === 'success' && p.mergeable === 'MERGEABLE'),
  )

  return { prs, isLoading, error, greenPrs }
})
```

Create `src/stores/scripts.ts`:
```typescript
import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { ScriptPreset, ScriptRun } from '@/types/script'

export const useScriptsStore = defineStore('scripts', () => {
  const presets = ref<ScriptPreset[]>([])
  const activeRun = ref<ScriptRun | null>(null)
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  return { presets, activeRun, isLoading, error }
})
```

Create `src/stores/compliance.ts`:
```typescript
import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { SecretFinding, LicenceFinding, BranchProtectionStatus } from '@/types/compliance'

export const useComplianceStore = defineStore('compliance', () => {
  const secretFindings = ref<SecretFinding[]>([])
  const licenceFindings = ref<LicenceFinding[]>([])
  const branchProtection = ref<BranchProtectionStatus[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  return { secretFindings, licenceFindings, branchProtection, isLoading, error }
})
```

Create `src/stores/settings.ts`:
```typescript
import { defineStore } from 'pinia'
import { ref } from 'vue'

export interface RateLimitInfo {
  remaining: number
  limit: number
  resetAt: string
}

export const useSettingsStore = defineStore('settings', () => {
  const scanIntervalMinutes = ref<number | null>(1440)  // daily
  const cvePollIntervalMinutes = ref<number | null>(60) // hourly
  const parallelWorkers = ref(5)
  const requestDelayMs = ref(200)
  const darkMode = ref(true)
  const rateLimitGithub = ref<RateLimitInfo | null>(null)
  const rateLimitGitlab = ref<RateLimitInfo | null>(null)

  return {
    scanIntervalMinutes,
    cvePollIntervalMinutes,
    parallelWorkers,
    requestDelayMs,
    darkMode,
    rateLimitGithub,
    rateLimitGitlab,
  }
})
```

---

## Task 11: Vue Router

**Files:**
- Create: `src/router/index.ts`

- [ ] **Step 11.1: Create `src/router/index.ts`**

```typescript
import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      redirect: '/dashboard',
    },
    {
      path: '/dashboard',
      name: 'dashboard',
      component: () => import('@/views/Dashboard.vue'),
      meta: { title: 'Dashboard' },
    },
    {
      path: '/repos',
      name: 'repos',
      component: () => import('@/views/RepoLists.vue'),
      meta: { title: 'Repositories' },
    },
    {
      path: '/scan',
      name: 'scan',
      component: () => import('@/views/Scanner.vue'),
      meta: { title: 'Scanner' },
    },
    {
      path: '/packages',
      name: 'packages',
      component: () => import('@/views/Packages.vue'),
      meta: { title: 'Packages' },
    },
    {
      path: '/cve',
      name: 'cve',
      component: () => import('@/views/CVEAlerts.vue'),
      meta: { title: 'CVE Alerts' },
    },
    {
      path: '/cve/:id',
      name: 'cve-incident',
      component: () => import('@/views/CVEIncident.vue'),
      meta: { title: 'CVE Incident' },
    },
    {
      path: '/ops',
      name: 'ops',
      component: () => import('@/views/Operations.vue'),
      meta: { title: 'Operations' },
    },
    {
      path: '/merge-queue',
      name: 'merge-queue',
      component: () => import('@/views/MergeQueue.vue'),
      meta: { title: 'PR Merge Queue' },
    },
    {
      path: '/scripts',
      name: 'scripts',
      component: () => import('@/views/ScriptRunner.vue'),
      meta: { title: 'Script Runner' },
    },
    {
      path: '/drift',
      name: 'drift',
      component: () => import('@/views/DriftDashboard.vue'),
      meta: { title: 'Drift Dashboard' },
    },
    {
      path: '/compliance',
      name: 'compliance',
      component: () => import('@/views/Compliance.vue'),
      meta: { title: 'Compliance' },
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('@/views/Settings.vue'),
      meta: { title: 'Settings' },
    },
    {
      path: '/auth',
      name: 'auth',
      component: () => import('@/views/Auth.vue'),
      meta: { title: 'Accounts' },
    },
  ],
})

router.afterEach((to) => {
  document.title = to.meta.title
    ? `${to.meta.title} — Git Flotilla`
    : 'Git Flotilla'
})

export default router
```

---

## Task 12: App entry point and root component

**Files:**
- Create: `src/main.ts`
- Create: `src/App.vue`

- [ ] **Step 12.1: Create `src/main.ts`**

```typescript
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { VueQueryPlugin } from '@tanstack/vue-query'
import App from './App.vue'
import router from './router'
import './styles/main.css'

const app = createApp(App)

app.use(createPinia())
app.use(router)
app.use(VueQueryPlugin)

app.mount('#app')
```

- [ ] **Step 12.2: Create `src/App.vue`**

```vue
<script setup lang="ts">
import AppSidebar from '@/components/layout/AppSidebar.vue'
import AppTopbar from '@/components/layout/AppTopbar.vue'
import CommandPalette from '@/components/ui/CommandPalette.vue'
import { ref, onMounted, onUnmounted } from 'vue'

const commandPaletteOpen = ref(false)

function handleKeydown(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
    e.preventDefault()
    commandPaletteOpen.value = !commandPaletteOpen.value
  }
  if (e.key === 'Escape') {
    commandPaletteOpen.value = false
  }
}

onMounted(() => window.addEventListener('keydown', handleKeydown))
onUnmounted(() => window.removeEventListener('keydown', handleKeydown))
</script>

<template>
  <div class="flex flex-col h-full bg-surface text-gray-100">
    <AppTopbar />
    <div class="flex flex-1 overflow-hidden">
      <AppSidebar />
      <main class="flex-1 overflow-auto p-6">
        <RouterView />
      </main>
    </div>
    <CommandPalette v-model:open="commandPaletteOpen" />
  </div>
</template>
```

---

## Task 13: Layout components

**Files:**
- Create: `src/components/layout/AppSidebar.vue`
- Create: `src/components/layout/AppTopbar.vue`

- [ ] **Step 13.1: Create `src/components/layout/AppSidebar.vue`**

```vue
<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { useCveStore } from '@/stores/cve'
import {
  LayoutDashboard,
  FolderGit2,
  ScanSearch,
  Package,
  ShieldAlert,
  Play,
  GitPullRequest,
  Terminal,
  TrendingDown,
  ShieldCheck,
  Settings,
  UserCircle,
} from 'lucide-vue-next'

const route = useRoute()
const cveStore = useCveStore()

const navItems = [
  { name: 'Dashboard',      to: '/dashboard',    icon: LayoutDashboard },
  { name: 'Repositories',   to: '/repos',         icon: FolderGit2 },
  { name: 'Scanner',        to: '/scan',          icon: ScanSearch },
  { name: 'Packages',       to: '/packages',      icon: Package },
  { name: 'CVE Alerts',     to: '/cve',           icon: ShieldAlert, badge: true },
  { name: 'Operations',     to: '/ops',           icon: Play },
  { name: 'PR Queue',       to: '/merge-queue',   icon: GitPullRequest },
  { name: 'Script Runner',  to: '/scripts',       icon: Terminal },
  { name: 'Drift',          to: '/drift',         icon: TrendingDown },
  { name: 'Compliance',     to: '/compliance',    icon: ShieldCheck },
]

const bottomItems = [
  { name: 'Settings', to: '/settings', icon: Settings },
  { name: 'Accounts', to: '/auth',     icon: UserCircle },
]

function isActive(to: string) {
  return route.path.startsWith(to)
}
</script>

<template>
  <nav class="w-52 flex-shrink-0 bg-surface-alt border-r border-border flex flex-col py-4">
    <!-- Logo -->
    <div class="px-4 mb-6">
      <span class="text-primary font-bold text-lg tracking-tight">Git Flotilla</span>
    </div>

    <!-- Main nav -->
    <div class="flex-1 flex flex-col gap-0.5 px-2">
      <RouterLink
        v-for="item in navItems"
        :key="item.to"
        :to="item.to"
        class="flex items-center gap-3 px-3 py-2 rounded-md text-sm transition-colors"
        :class="isActive(item.to)
          ? 'bg-primary/20 text-primary'
          : 'text-muted hover:text-gray-200 hover:bg-white/5'"
      >
        <component :is="item.icon" class="w-4 h-4 flex-shrink-0" />
        <span class="flex-1">{{ item.name }}</span>
        <!-- CVE badge -->
        <span
          v-if="item.badge && cveStore.badgeCount > 0"
          class="bg-danger text-white text-xs font-bold rounded-full px-1.5 py-0.5 min-w-[1.25rem] text-center"
        >
          {{ cveStore.badgeCount > 99 ? '99+' : cveStore.badgeCount }}
        </span>
      </RouterLink>
    </div>

    <!-- Bottom nav -->
    <div class="flex flex-col gap-0.5 px-2 pt-4 border-t border-border mt-4">
      <RouterLink
        v-for="item in bottomItems"
        :key="item.to"
        :to="item.to"
        class="flex items-center gap-3 px-3 py-2 rounded-md text-sm transition-colors"
        :class="isActive(item.to)
          ? 'bg-primary/20 text-primary'
          : 'text-muted hover:text-gray-200 hover:bg-white/5'"
      >
        <component :is="item.icon" class="w-4 h-4 flex-shrink-0" />
        <span>{{ item.name }}</span>
      </RouterLink>
    </div>
  </nav>
</template>
```

- [ ] **Step 13.2: Create `src/components/layout/AppTopbar.vue`**

```vue
<script setup lang="ts">
import { computed } from 'vue'
import { useSettingsStore } from '@/stores/settings'
import { useAuthStore } from '@/stores/auth'
import { Search, Bell, Zap } from 'lucide-vue-next'

const settingsStore = useSettingsStore()
const authStore = useAuthStore()

const emit = defineEmits<{
  'search': []
}>()

const rateLimitDisplay = computed(() => {
  const rl = settingsStore.rateLimitGithub
  if (!rl) return null
  const pct = Math.round((rl.remaining / rl.limit) * 100)
  const colour = pct < 20 ? 'text-danger' : pct < 50 ? 'text-warning' : 'text-success'
  return { text: rl.remaining.toLocaleString(), colour }
})
</script>

<template>
  <header class="h-12 bg-surface-alt border-b border-border flex items-center px-4 gap-4 flex-shrink-0">
    <!-- Left: Logo text (compact) -->
    <span class="text-muted text-xs font-mono w-44 flex-shrink-0">git-flotilla</span>

    <!-- Centre: Search trigger -->
    <button
      class="flex-1 flex items-center gap-2 bg-surface border border-border rounded-md px-3 py-1.5 text-sm text-muted hover:border-primary/50 transition-colors max-w-md"
      @click="emit('search')"
    >
      <Search class="w-3.5 h-3.5" />
      <span>Search repos, actions…</span>
      <kbd class="ml-auto text-xs bg-surface-alt px-1.5 py-0.5 rounded border border-border">⌘K</kbd>
    </button>

    <!-- Right: Rate limit, notif, auth -->
    <div class="flex items-center gap-3 ml-auto">
      <!-- GitHub API rate limit -->
      <div v-if="rateLimitDisplay" class="flex items-center gap-1.5 text-xs">
        <Zap class="w-3.5 h-3.5 text-muted" />
        <span :class="rateLimitDisplay.colour" class="font-mono">
          {{ rateLimitDisplay.text }}
        </span>
      </div>

      <!-- Notifications -->
      <button class="text-muted hover:text-gray-200 transition-colors">
        <Bell class="w-4 h-4" />
      </button>

      <!-- Auth status -->
      <div class="flex items-center gap-1.5 text-xs text-muted">
        <div
          class="w-2 h-2 rounded-full"
          :class="authStore.hasAccounts ? 'bg-success' : 'bg-danger'"
        />
        <span>{{ authStore.githubAccount?.username ?? 'Not connected' }}</span>
      </div>
    </div>
  </header>
</template>
```

---

## Task 14: UI base components

**Files:** All in `src/components/ui/`

- [ ] **Step 14.1: Create `src/components/ui/Button.vue`**

```vue
<script setup lang="ts">
defineProps<{
  variant?: 'primary' | 'secondary' | 'danger' | 'ghost'
  size?: 'sm' | 'md' | 'lg'
  disabled?: boolean
  loading?: boolean
}>()
</script>

<template>
  <button
    :disabled="disabled || loading"
    class="inline-flex items-center justify-center gap-2 font-medium rounded-md transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
    :class="{
      // Sizes
      'px-2.5 py-1 text-xs': size === 'sm' || !size && false,
      'px-3.5 py-1.5 text-sm': !size || size === 'md',
      'px-5 py-2 text-base': size === 'lg',
      // Variants
      'bg-primary text-white hover:bg-primary/80': !variant || variant === 'primary',
      'bg-surface-alt text-gray-200 border border-border hover:bg-white/10': variant === 'secondary',
      'bg-danger text-white hover:bg-danger/80': variant === 'danger',
      'text-muted hover:text-gray-200 hover:bg-white/5': variant === 'ghost',
    }"
  >
    <slot />
  </button>
</template>
```

- [ ] **Step 14.2: Create `src/components/ui/Badge.vue`**

```vue
<script setup lang="ts">
defineProps<{
  variant: 'critical' | 'high' | 'medium' | 'low' | 'info' | 'success' | 'muted'
}>()
</script>

<template>
  <span
    class="inline-flex items-center px-2 py-0.5 text-xs font-semibold rounded-full uppercase tracking-wide"
    :class="{
      'bg-danger/20 text-danger': variant === 'critical',
      'bg-orange-500/20 text-orange-400': variant === 'high',
      'bg-warning/20 text-warning': variant === 'medium',
      'bg-blue-500/20 text-blue-400': variant === 'low',
      'bg-primary/20 text-primary': variant === 'info',
      'bg-success/20 text-success': variant === 'success',
      'bg-surface-alt text-muted': variant === 'muted',
    }"
  >
    <slot />
  </span>
</template>
```

- [ ] **Step 14.3: Create `src/components/ui/Card.vue`**

```vue
<script setup lang="ts">
defineProps<{
  padding?: boolean
}>()
</script>

<template>
  <div
    class="bg-surface-alt border border-border rounded-lg"
    :class="{ 'p-4': padding !== false }"
  >
    <slot />
  </div>
</template>
```

- [ ] **Step 14.4: Create `src/components/ui/Input.vue`**

```vue
<script setup lang="ts">
defineProps<{
  modelValue?: string
  placeholder?: string
  disabled?: boolean
  type?: string
  error?: string
}>()

defineEmits<{
  'update:modelValue': [value: string]
}>()
</script>

<template>
  <div class="flex flex-col gap-1">
    <input
      :value="modelValue"
      :placeholder="placeholder"
      :disabled="disabled"
      :type="type ?? 'text'"
      class="bg-surface border rounded-md px-3 py-1.5 text-sm text-gray-100 placeholder:text-muted outline-none transition-colors disabled:opacity-50"
      :class="error
        ? 'border-danger focus:border-danger'
        : 'border-border focus:border-primary'"
      @input="$emit('update:modelValue', ($event.target as HTMLInputElement).value)"
    />
    <p v-if="error" class="text-danger text-xs">{{ error }}</p>
  </div>
</template>
```

- [ ] **Step 14.5: Create `src/components/ui/Modal.vue`**

```vue
<script setup lang="ts">
defineProps<{
  open: boolean
  title?: string
  size?: 'sm' | 'md' | 'lg' | 'xl'
}>()

defineEmits<{
  'update:open': [value: boolean]
}>()
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="fixed inset-0 z-50 flex items-center justify-center p-4"
    >
      <!-- Backdrop -->
      <div
        class="absolute inset-0 bg-black/60 backdrop-blur-sm"
        @click="$emit('update:open', false)"
      />

      <!-- Panel -->
      <div
        class="relative bg-surface-alt border border-border rounded-xl shadow-2xl w-full"
        :class="{
          'max-w-sm': size === 'sm',
          'max-w-lg': !size || size === 'md',
          'max-w-2xl': size === 'lg',
          'max-w-4xl': size === 'xl',
        }"
      >
        <div v-if="title" class="flex items-center justify-between px-6 py-4 border-b border-border">
          <h2 class="text-base font-semibold">{{ title }}</h2>
          <button
            class="text-muted hover:text-gray-200 transition-colors"
            @click="$emit('update:open', false)"
          >✕</button>
        </div>
        <div class="p-6">
          <slot />
        </div>
      </div>
    </div>
  </Teleport>
</template>
```

- [ ] **Step 14.6: Create `src/components/ui/Table.vue`**

```vue
<script setup lang="ts" generic="T">
defineProps<{
  items: T[]
  loading?: boolean
  emptyMessage?: string
}>()
</script>

<template>
  <div class="w-full overflow-x-auto">
    <table class="w-full text-sm border-collapse">
      <thead>
        <tr class="border-b border-border">
          <slot name="head" />
        </tr>
      </thead>
      <tbody>
        <tr v-if="loading">
          <td colspan="100%" class="py-12 text-center text-muted">Loading…</td>
        </tr>
        <tr v-else-if="items.length === 0">
          <td colspan="100%" class="py-12 text-center text-muted">
            {{ emptyMessage ?? 'No data' }}
          </td>
        </tr>
        <template v-else>
          <slot name="row" v-for="item in items" :item="item" />
        </template>
      </tbody>
    </table>
  </div>
</template>
```

- [ ] **Step 14.7: Create `src/components/ui/Tooltip.vue`**

```vue
<script setup lang="ts">
defineProps<{
  content: string
  position?: 'top' | 'bottom' | 'left' | 'right'
}>()
</script>

<template>
  <div class="relative inline-flex group">
    <slot />
    <div
      class="absolute z-50 px-2 py-1 text-xs text-white bg-gray-800 rounded whitespace-nowrap pointer-events-none opacity-0 group-hover:opacity-100 transition-opacity"
      :class="{
        'bottom-full left-1/2 -translate-x-1/2 mb-1': !position || position === 'top',
        'top-full left-1/2 -translate-x-1/2 mt-1': position === 'bottom',
        'right-full top-1/2 -translate-y-1/2 mr-1': position === 'left',
        'left-full top-1/2 -translate-y-1/2 ml-1': position === 'right',
      }"
    >
      {{ content }}
    </div>
  </div>
</template>
```

- [ ] **Step 14.8: Create `src/components/ui/CommandPalette.vue`**

```vue
<script setup lang="ts">
import { ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { Search } from 'lucide-vue-next'

defineProps<{
  open: boolean
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const router = useRouter()
const query = ref('')

const quickActions = [
  { label: 'Dashboard',     to: '/dashboard' },
  { label: 'Repositories',  to: '/repos' },
  { label: 'Scanner',       to: '/scan' },
  { label: 'CVE Alerts',    to: '/cve' },
  { label: 'Operations',    to: '/ops' },
  { label: 'PR Queue',      to: '/merge-queue' },
  { label: 'Script Runner', to: '/scripts' },
  { label: 'Compliance',    to: '/compliance' },
  { label: 'Settings',      to: '/settings' },
]

function navigate(to: string) {
  router.push(to)
  emit('update:open', false)
  query.value = ''
}

watch(() => false, () => {
  // Reset on close
  query.value = ''
})
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="fixed inset-0 z-50 flex items-start justify-center pt-24 px-4"
    >
      <div class="absolute inset-0 bg-black/60" @click="emit('update:open', false)" />

      <div class="relative w-full max-w-xl bg-surface-alt border border-border rounded-xl shadow-2xl overflow-hidden">
        <!-- Search input -->
        <div class="flex items-center gap-3 px-4 py-3 border-b border-border">
          <Search class="w-4 h-4 text-muted flex-shrink-0" />
          <input
            v-model="query"
            placeholder="Search repos, actions, views…"
            class="flex-1 bg-transparent text-sm text-gray-100 placeholder:text-muted outline-none"
            autofocus
          />
          <kbd class="text-xs text-muted bg-surface px-1.5 py-0.5 rounded border border-border">Esc</kbd>
        </div>

        <!-- Results -->
        <div class="max-h-80 overflow-y-auto py-2">
          <button
            v-for="action in quickActions"
            :key="action.to"
            class="w-full flex items-center px-4 py-2.5 text-sm text-gray-200 hover:bg-white/5 transition-colors text-left"
            @click="navigate(action.to)"
          >
            {{ action.label }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
```

---

## Task 15: Stub views

Each view is a minimal skeleton that renders a title and "coming soon" placeholder. This confirms routing works before real content is built.

**Files:** All in `src/views/`

- [ ] **Step 15.1: Create all stub views**

Create `src/views/Dashboard.vue`:
```vue
<script setup lang="ts">
</script>
<template>
  <div>
    <h1 class="text-2xl font-semibold mb-6">Dashboard</h1>
    <p class="text-muted">Repo health overview — coming in Phase 7.</p>
  </div>
</template>
```

Create `src/views/RepoLists.vue`:
```vue
<script setup lang="ts">
</script>
<template>
  <div>
    <h1 class="text-2xl font-semibold mb-6">Repositories</h1>
    <p class="text-muted">Repo discovery and list management — coming in Phase 2.</p>
  </div>
</template>
```

Create `src/views/Scanner.vue`:
```vue
<script setup lang="ts">
</script>
<template>
  <div>
    <h1 class="text-2xl font-semibold mb-6">Scanner</h1>
    <p class="text-muted">Repo scanning — coming in Phase 3.</p>
  </div>
</template>
```

Create `src/views/Packages.vue`:
```vue
<script setup lang="ts">
</script>
<template>
  <div>
    <h1 class="text-2xl font-semibold mb-6">Packages</h1>
    <p class="text-muted">Dependency matrix — coming in Phase 4.</p>
  </div>
</template>
```

Create `src/views/CVEAlerts.vue`:
```vue
<script setup lang="ts">
</script>
<template>
  <div>
    <h1 class="text-2xl font-semibold mb-6">CVE Alerts</h1>
    <p class="text-muted">CVE monitoring — coming in Phase 5.</p>
  </div>
</template>
```

Create `src/views/CVEIncident.vue`:
```vue
<script setup lang="ts">
import { useRoute } from 'vue-router'
const route = useRoute()
</script>
<template>
  <div>
    <h1 class="text-2xl font-semibold mb-6">CVE Incident: {{ route.params.id }}</h1>
    <p class="text-muted">Incident timeline — coming in Phase 5.</p>
  </div>
</template>
```

Create `src/views/Operations.vue`:
```vue
<script setup lang="ts">
</script>
<template>
  <div>
    <h1 class="text-2xl font-semibold mb-6">Operations</h1>
    <p class="text-muted">Batch operations — coming in Phase 6.</p>
  </div>
</template>
```

Create `src/views/MergeQueue.vue`:
```vue
<script setup lang="ts">
</script>
<template>
  <div>
    <h1 class="text-2xl font-semibold mb-6">PR Merge Queue</h1>
    <p class="text-muted">PR merge queue — coming in Phase 7.</p>
  </div>
</template>
```

Create `src/views/ScriptRunner.vue`:
```vue
<script setup lang="ts">
</script>
<template>
  <div>
    <h1 class="text-2xl font-semibold mb-6">Script Runner</h1>
    <p class="text-muted">Custom script runner — coming in Phase 8.</p>
  </div>
</template>
```

Create `src/views/DriftDashboard.vue`:
```vue
<script setup lang="ts">
</script>
<template>
  <div>
    <h1 class="text-2xl font-semibold mb-6">Drift Dashboard</h1>
    <p class="text-muted">Cross-repo drift analysis — coming in Phase 8.</p>
  </div>
</template>
```

Create `src/views/Compliance.vue`:
```vue
<script setup lang="ts">
</script>
<template>
  <div>
    <h1 class="text-2xl font-semibold mb-6">Compliance</h1>
    <p class="text-muted">Secret scanning, licence audit, branch protection — coming in Phase 8.</p>
  </div>
</template>
```

Create `src/views/Settings.vue`:
```vue
<script setup lang="ts">
</script>
<template>
  <div>
    <h1 class="text-2xl font-semibold mb-6">Settings</h1>
    <p class="text-muted">App settings — coming in Phase 8.</p>
  </div>
</template>
```

Create `src/views/Auth.vue`:
```vue
<script setup lang="ts">
</script>
<template>
  <div>
    <h1 class="text-2xl font-semibold mb-6">Accounts</h1>
    <p class="text-muted">GitHub / GitLab authentication — coming in Phase 2.</p>
  </div>
</template>
```

---

## Task 16: Verify frontend build

- [ ] **Step 16.1: Type-check the frontend**

```bash
cd /var/www/vhosts/git-flotilla && pnpm typecheck
```

Expected: exits 0 with no errors. If there are errors, fix them before continuing.

- [ ] **Step 16.2: Lint the frontend**

```bash
cd /var/www/vhosts/git-flotilla && pnpm lint
```

Expected: exits 0 with no errors.

- [ ] **Step 16.3: Build the frontend**

```bash
cd /var/www/vhosts/git-flotilla && pnpm build
```

Expected: `dist/` directory created with compiled assets. No TypeScript or Vite errors.

---

## Task 17: Verify Rust build

- [ ] **Step 17.1: Check Rust code compiles**

```bash
cd /var/www/vhosts/git-flotilla/src-tauri && cargo build 2>&1 | tail -20
```

Expected: `Finished dev [unoptimized + debuginfo] target(s)` — may take several minutes on first build to compile all dependencies.

If `git2` fails due to missing `libssl-dev` or `pkg-config`:
```bash
sudo apt-get install -y libssl-dev pkg-config libclang-dev
cd /var/www/vhosts/git-flotilla/src-tauri && cargo build 2>&1 | tail -20
```

- [ ] **Step 17.2: Run Clippy**

```bash
cd /var/www/vhosts/git-flotilla/src-tauri && cargo clippy -- -D warnings 2>&1 | tail -30
```

Expected: No warnings or errors. If `unused import` warnings appear in stub files, add `#[allow(unused_imports)]` to the relevant modules or remove the imports.

- [ ] **Step 17.3: Run Rust tests**

```bash
cd /var/www/vhosts/git-flotilla/src-tauri && cargo test 2>&1 | tail -10
```

Expected: `test result: ok. 0 passed; 0 failed` (no tests yet, but it should compile and run cleanly).

---

## Task 18: First commit

- [ ] **Step 18.1: Stage all files and commit**

```bash
cd /var/www/vhosts/git-flotilla
git add -A
git status
git commit -m "feat: scaffold Phase 1 — Tauri v2 + Vue 3 + TypeScript + Tailwind v4 project structure

- Tauri v2 app with pnpm, Vue 3, TypeScript strict, Tailwind CSS v4
- All Rust command domain stubs (auth, repos, scan, packages, cve, ops, merge_queue, scripts, compliance, settings)
- SQLite schema with 12 tables via sqlx migrations
- All Pinia stores, typed service wrappers, TypeScript types
- App shell: sidebar nav, topbar with rate limit indicator, command palette
- All 13 stub views wired to Vue Router
- Base UI component library (Button, Badge, Card, Input, Modal, Table, Tooltip, CommandPalette)"
```

Expected: commit created. Run `git log --oneline -1` to verify.

---

## Self-Review Notes

- All `AppError` variants are serialisable — Tauri can propagate them to the frontend
- `db::init()` uses `OnceLock` so it's safe to call once at startup and never again
- All command stubs use `let _ = param;` to suppress unused-variable warnings without `#[allow]` on every function
- The `Table.vue` component uses TypeScript generics (`generic="T"`) which requires Vue 3.3+
- Tailwind v4 uses `@theme` instead of `tailwind.config.js` — the old `tailwind.config.ts` is not needed; do not create it
- `AppTopbar` emits `'search'` but `App.vue` handles `commandPaletteOpen` itself — topbar doesn't need to wire the event, the Ctrl+K handler in App.vue covers it. The search button in topbar should emit to App.vue. Adjust `App.vue` to listen: `<AppTopbar @search="commandPaletteOpen = true" />`
