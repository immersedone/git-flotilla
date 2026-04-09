# Configuration

Git Flotilla stores its configuration in two places:

- **`.flotilla/config.yaml`** -- application settings (safe to commit and share)
- **`.flotilla/repo-lists/*.yaml`** -- repo list definitions (committable, shareable)
- **OS keychain** -- auth tokens (never in files)

---

## Settings Overview

These settings are available in the GUI under **Settings**, and also in `.flotilla/config.yaml`.

| Setting | Default | Description |
|---------|---------|-------------|
| `scan.schedule` | `daily` | Background scan frequency: `manual`, `daily`, `weekly` |
| `scan.parallelism` | `5` | Number of repos to scan concurrently |
| `scan.stale_after_hours` | `24` | Hours before a scan result is considered stale |
| `cve.poll_interval` | `1hr` | CVE polling frequency: `off`, `15min`, `30min`, `1hr`, `6hr`, `daily` |
| `cve.check_after_scan` | `true` | Automatically run CVE check after every scan |
| `cve.notify_on` | `[critical, high]` | Severity levels that trigger notifications |
| `operations.default_dry_run` | `true` | Default to dry run mode for batch operations |
| `operations.parallelism` | `5` | Number of repos to process concurrently in batch operations |

---

## Health Score Weights

The health score (0-100) is computed per repo based on configurable rules. Adjust the weights to match your team's priorities.

| Rule | Default Weight | What it checks |
|------|---------------|----------------|
| `has_codeowners` | 10 | Repo has a `CODEOWNERS` file |
| `has_security_md` | 10 | Repo has a `SECURITY.md` file |
| `has_env_example` | 5 | Repo has a `.env.example` file |
| `has_editorconfig` | 5 | Repo has a `.editorconfig` file |
| `no_floating_action_tags` | 15 | No workflow files use floating action tags (e.g. `@v4` instead of pinned SHA) |
| `dependencies_not_majorly_outdated` | 20 | All dependencies within 1 major version of latest |
| `no_known_cves` | 20 | No open CVEs against any dependency |
| `runtime_not_eol` | 15 | Node.js / PHP version is not end-of-life |

Configure in `.flotilla/config.yaml`:

```yaml
health_score:
  has_codeowners: 10
  has_security_md: 10
  has_env_example: 5
  has_editorconfig: 5
  no_floating_action_tags: 15
  dependencies_not_majorly_outdated: 20
  no_known_cves: 20
  runtime_not_eol: 15
```

---

## Config Export and Import

### Export

In the GUI, go to **Settings** and click **Export Config** to download the current configuration as YAML.

### Import

Click **Import Config** and select a `.yaml` file. Flotilla validates the config before applying it.

### Validation

The config is validated on import and on app start. Invalid values are rejected with a descriptive error message. You can also validate manually:

```yaml
# This will be rejected -- invalid poll_interval
cve:
  poll_interval: 2hr  # Must be: off | 15min | 30min | 1hr | 6hr | daily
```

---

## `.flotilla/config.yaml` Full Example

```yaml
# .flotilla/config.yaml
# Git Flotilla configuration file.
# This file is safe to commit to a shared repository.
# Auth tokens are NEVER stored here -- they live in your OS keychain.

# --- Scanning ---------------------------------------------------------------

scan:
  # How often to run background scans
  # Options: manual | daily | weekly
  schedule: daily

  # Number of repos to scan in parallel
  parallelism: 5

  # How old a scan can be before it's considered stale (hours)
  stale_after_hours: 24

# --- CVE Monitoring ---------------------------------------------------------

cve:
  # How often to poll CVE sources
  # Options: off | 15min | 30min | 1hr | 6hr | daily
  poll_interval: 1hr

  # Automatically run CVE check after every scan completes
  check_after_scan: true

  # Severity levels to show notifications for
  notify_on:
    - critical
    - high

  # CVE data sources (ordered by priority)
  sources:
    - osv_dev
    - github_advisory
    - nvd_nist

# --- Health Score Weights ----------------------------------------------------

health_score:
  has_codeowners: 10
  has_security_md: 10
  has_env_example: 5
  has_editorconfig: 5
  no_floating_action_tags: 15
  dependencies_not_majorly_outdated: 20
  no_known_cves: 20
  runtime_not_eol: 15

# --- Batch Operations --------------------------------------------------------

operations:
  # Default to dry run mode (always preview before applying)
  default_dry_run: true

  # Number of repos to process in parallel during batch operations
  parallelism: 5

  # Default PR settings
  pr:
    draft: false
    labels:
      - flotilla
    title_template: "chore(deps): {{action}} {{package}} to {{version}} [flotilla]"
    body_template: |
      ## Automated update by Git Flotilla

      {{#CVE}}
      ### Security Fix
      This PR addresses **{{CVE}}** ({{SEVERITY}}).
      Affected package: `{{PACKAGE}}` -- fixed in `{{VERSION}}`.
      {{/CVE}}

      **Package:** `{{PACKAGE}}`
      **Ecosystem:** {{ECOSYSTEM}}
      **Previous version:** `{{PREVIOUS_VERSION}}`
      **New version:** `{{VERSION}}`

      ---
      *Generated by [Git Flotilla](https://github.com/immersedone/git-flotilla)*

# --- Notifications -----------------------------------------------------------

notifications:
  webhooks:
    slack:
      url: ""
      events:
        - cve_critical
        - cve_high
        - operation_complete
        - operation_failed

    teams:
      url: ""
      events:
        - cve_critical
        - operation_failed

    discord:
      url: ""
      events: []

# --- Fingerprint Profiles ----------------------------------------------------
# Define what a "healthy" repo looks like for each project type.
# Assign profiles to repo lists to flag deviating repos.

profiles:
  laravel-standard:
    description: "Standard Laravel application"
    expected_php_version: "^8.2"
    expected_package_manager: pnpm
    expected_node_version: "^20"
    required_files:
      - .env.example
      - CODEOWNERS
      - .editorconfig
    required_workflows:
      - .github/workflows/ci.yml

  vue-spa:
    description: "Vue 3 SPA"
    expected_node_version: "^20"
    expected_package_manager: pnpm
    required_files:
      - .env.example
      - .editorconfig
    required_workflows:
      - .github/workflows/ci.yml

# --- Superseded Packages -----------------------------------------------------
# Packages that have been replaced by better alternatives.
# Flotilla will flag repos still using these.

superseded_packages:
  npm:
    - old: node-fetch
      replacement: "native fetch (Node 18+)"
    - old: request
      replacement: got or native fetch
    - old: moment
      replacement: date-fns or dayjs
  composer: []
```

---

## `.flotilla/repo-lists/*.yaml` Format

Repo list YAML files define groups of repositories. They can be committed to a shared repository so your team uses the same repo groupings.

```yaml
# .flotilla/repo-lists/client-acme.yaml

id: client-acme
name: "Client: Acme Corp"
description: "All repositories for Acme Corp engagement"
created_at: "2026-01-01T00:00:00Z"

# Assign a fingerprint profile (defined in config.yaml)
profile: laravel-standard

# Tags applied to all repos in this list
tags:
  - client:acme
  - laravel

# Nested children for sub-grouping (up to 3 levels)
children:
  - id: acme-backend
    name: "Backend Services"
    repos:
      - provider: github
        full_name: acme-org/api-gateway
        tags:
          - laravel
          - api
      - provider: github
        full_name: acme-org/auth-service
        tags:
          - laravel
          - auth

  - id: acme-frontend
    name: "Frontend Applications"
    profile: vue-spa    # Override profile for this child group
    repos:
      - provider: github
        full_name: acme-org/customer-portal
        tags:
          - vue3
          - spa
      - provider: gitlab
        full_name: acme-group/admin-dashboard
        tags:
          - vue3
          - spa
```

### Repo list fields

| Field | Required | Description |
|-------|----------|-------------|
| `id` | Yes | Unique identifier for the list |
| `name` | Yes | Display name |
| `description` | No | Description of the list |
| `created_at` | No | ISO 8601 timestamp |
| `profile` | No | Fingerprint profile name (from `config.yaml`) |
| `tags` | No | Tags applied to all repos in this list |
| `children` | No | Nested sub-lists |
| `children[].repos` | No | Repos in each sub-list |
| `children[].repos[].provider` | Yes | `github` or `gitlab` |
| `children[].repos[].full_name` | Yes | `owner/repo` format |
| `children[].repos[].tags` | No | Per-repo tags |

---

## Environment Variables

| Variable | Description |
|----------|-------------|
| `FLOTILLA_DB_PATH` | Override the path to the SQLite database. By default, the CLI and GUI use the Tauri app data directory (`~/.local/share/com.gitflotilla.desktop/flotilla.db` on Linux, `~/Library/Application Support/com.gitflotilla.desktop/flotilla.db` on macOS, `%APPDATA%/com.gitflotilla.desktop/flotilla.db` on Windows). |

This is useful when running the CLI in CI or when you want to point to a specific database:

```bash
FLOTILLA_DB_PATH=/path/to/flotilla.db git-flotilla-cli report --json
```

---

## Database Location

Both the GUI and CLI share the same SQLite database. The default locations are:

| Platform | Path |
|----------|------|
| Linux | `~/.local/share/com.gitflotilla.desktop/flotilla.db` |
| macOS | `~/Library/Application Support/com.gitflotilla.desktop/flotilla.db` |
| Windows | `%APPDATA%\com.gitflotilla.desktop\flotilla.db` |

The GUI creates and migrates the database on first launch. The CLI opens it in read-only mode.

---

## Team Sharing Workflow

1. Export your `config.yaml` and repo list YAMLs
2. Commit them to a shared repository:
   ```
   shared-config/
   ├── .flotilla/
   │   ├── config.yaml
   │   └── repo-lists/
   │       ├── client-acme.yaml
   │       └── internal-tools.yaml
   ```
3. Team members clone the shared repo and import the config and repo lists into their local Flotilla instance
4. Each person provides their own auth tokens via the GUI -- tokens are stored in each person's OS keychain and never appear in config files
