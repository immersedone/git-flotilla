# Getting Started with Git Flotilla

This guide walks you through installing Git Flotilla, connecting your first account, and running your first scan.

---

## 1. Download and Install

Download the latest release for your platform from the [Releases page](https://github.com/immersedone/git-flotilla/releases).

| Platform | Format | Notes |
|----------|--------|-------|
| macOS | `.dmg` | Universal binary (Intel + Apple Silicon). Unsigned builds will show a Gatekeeper warning — right-click and choose "Open" to bypass. |
| Windows | `.msi` or `.exe` (NSIS) | Standard installer. No special requirements. |
| Linux | `.AppImage` / `.deb` | Requires `webkit2gtk-4.1`: `sudo apt install libwebkit2gtk-4.1-dev` |

### Build from Source

Prerequisites: Rust 1.78+, Node.js 20+, pnpm 9+

```bash
git clone https://github.com/immersedone/git-flotilla.git
cd git-flotilla
pnpm install
pnpm tauri build
```

The built artifacts will be in `src-tauri/target/release/bundle/`.

---

## 2. Connect a GitHub Account

1. Open Git Flotilla and navigate to **Settings** (bottom of the sidebar)
2. Under **Accounts**, click **Add Account**
3. Select **GitHub** and enter a Personal Access Token

### Creating a GitHub PAT

Go to [GitHub Settings > Developer Settings > Personal Access Tokens > Fine-grained tokens](https://github.com/settings/tokens?type=beta) (or use classic tokens).

**Required scopes (classic token):**

| Scope | Why |
|-------|-----|
| `repo` | Read repository contents, create branches and PRs |
| `workflow` | Read and update GitHub Actions workflow files |
| `read:org` | Discover repos in organisations you belong to |

Flotilla validates token scopes on save and warns if any required scope is missing.

**GitLab users:** create a PAT at your GitLab instance under User Settings > Access Tokens with `api`, `read_repository`, and `write_repository` scopes.

> Tokens are stored in your OS keychain (macOS Keychain, Windows Credential Manager, or Linux Secret Service). They are never written to config files or the local database.

---

## 3. Discover Repos

After connecting an account:

1. Go to the **Repos** view in the sidebar
2. Flotilla automatically discovers all repositories accessible to your token, including repos in your organisations and groups
3. Repos appear in a searchable, filterable table
4. Use the search bar to find repos by name or tag

For large organisations (100+ repos), discovery paginates automatically. The API rate limit indicator in the top bar shows remaining quota.

---

## 4. Create a Repo List

Repo lists let you group repositories for batch scanning and operations.

1. Go to the **Repo Lists** view
2. Click **Create List**
3. Give it a name (e.g. "Client: Acme Corp") and optional description
4. Select repos from the discovery table to add them
5. Optionally add tags to all repos in the list

You can create multiple lists to organise by client, project, team, or technology.

### Export and Share

Click **Export YAML** on any repo list to save it as a `.yaml` file. Share this with your team by committing it to a shared repository. See [Configuration](./configuration.md) for the YAML format.

---

## 5. Run Your First Scan

1. Go to the **Scanner** view
2. Select a repo list (or individual repos)
3. Click **Scan**

Flotilla scans each repository for:

- **Dependency files:** `package.json`, `composer.json`, `requirements.txt`, `Cargo.toml`, `go.mod`
- **Runtime versions:** Node.js (from `.nvmrc`, `.node-version`, `.tool-versions`, `engines.node`), PHP
- **Package manager:** npm, pnpm, yarn, bun, or Composer (with version detection)
- **Workflow files:** `.github/workflows/*.yml` with floating action tag detection
- **Health indicators:** `CODEOWNERS`, `SECURITY.md`, `.env.example`, `.editorconfig`

Scanning runs in parallel (default: 5 repos concurrently) with a configurable inter-request delay to respect API rate limits. Repos without relevant manifest files are automatically excluded.

Each repo receives a **health score** (0-100) based on what was found. See [Scanning](./scanning.md) for the full breakdown.

---

## 6. Check CVE Alerts

After scanning completes, navigate to the **CVE Alerts** view in the sidebar. The badge on the sidebar item shows the count of unacknowledged critical and high severity alerts.

Flotilla queries [OSV.dev](https://osv.dev) to match all detected packages against known vulnerabilities.

- **Critical** (red) and **High** (orange) alerts appear at the top
- Click any CVE to see the full detail: affected version range, fixed version, and which of your repos are exposed
- Use **Incident Timeline** to see the full lifecycle of a CVE across your repos
- Use **Blast Radius** to understand which repos are directly affected

CVE monitoring also runs in the background on an hourly schedule (configurable in Settings).

For each CVE with a known fix, click **Patch affected repos** to pre-fill a batch operation that pins the package to the safe version.

See [CVE Monitoring](./cve-monitoring.md) for the full guide.

---

## 7. Export the Dependency Matrix

1. Go to the **Packages** view
2. Select an ecosystem filter (npm, composer, pip, cargo, go) or view all
3. The matrix shows every package used across your scanned repos with version columns

Use this view to:

- Spot **version drift** (same package, different versions across repos)
- Find the **latest available version** from package registries
- Identify **orphan packages** used in only one repo
- View **changelog entries** between your current version and the latest

Click **Export CSV** to download the full matrix for reporting or further analysis.

---

## CLI Usage

Git Flotilla includes a CLI companion (`git-flotilla-cli`) that reads from the same database as the GUI. It is useful for CI pipelines, scripting, and quick checks from the terminal.

The CLI opens the database in read-only mode and never modifies data.

### List repos

```bash
git-flotilla-cli repo list
```

```
REPO                                               PROVIDER   BRANCH       LAST SCANNED
------------------------------------------------------------------------------------------
acme-org/api-gateway                               github     main         2026-04-08T14:30:00Z
acme-org/customer-portal                           github     main         2026-04-08T14:30:05Z

2 repo(s) total
```

### Show health report

```bash
git-flotilla-cli report --json
```

```json
{
  "repos": 42,
  "scannedRepos": 40,
  "averageHealthScore": 78,
  "openCves": 3,
  "criticalCves": 1,
  "highCves": 2,
  "uniquePackages": 312
}
```

### List CVE alerts

```bash
git-flotilla-cli cve list --severity critical
```

```
CVE                  PACKAGE                        ECOSYSTEM    SEVERITY   STATUS
------------------------------------------------------------------------------------------
CVE-2026-12345       lodash                         npm          critical   new

1 alert(s) total
```

### Show scan result for a repo

```bash
git-flotilla-cli scan --repo "github:acme-org/api-gateway"
```

### Show scan summary for a repo list

```bash
git-flotilla-cli scan --list "client-acme"
```

### CVE status summary

```bash
git-flotilla-cli cve check
```

All commands support the `--json` flag for machine-readable output.

See [CLI Reference](./cli.md) for the full command reference.

---

## Next Steps

- [Configuration](./configuration.md) -- customise scan intervals, health score weights, and PR templates
- [Scanning](./scanning.md) -- understand what gets scanned and how health scores work
- [CVE Monitoring](./cve-monitoring.md) -- set up automated CVE polling and incident response
- [Batch Operations](./batch-operations.md) -- patch packages, sync files, and open PRs across repos
- [CLI Reference](./cli.md) -- full CLI command documentation
