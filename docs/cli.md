# CLI Reference

Git Flotilla includes a CLI companion binary (`git-flotilla-cli`) for querying scan results, CVE alerts, and health reports from the terminal. It is read-only -- all data modifications happen through the GUI.

---

## Installation

The CLI binary is built alongside the GUI when you build from source:

```bash
git clone https://github.com/immersedone/git-flotilla.git
cd git-flotilla
cargo build --release --manifest-path src-tauri/Cargo.toml --bin git-flotilla-cli
```

The binary will be at `src-tauri/target/release/git-flotilla-cli`.

Alternatively, download a pre-built binary from the [Releases page](https://github.com/immersedone/git-flotilla/releases) (when available).

### Prerequisites

The CLI requires the GUI to have been launched at least once to create and populate the SQLite database. The CLI opens the database in **read-only mode** and never modifies data.

---

## Database Location

The CLI automatically finds the database at the same location the GUI uses:

| Platform | Default Path |
|----------|-------------|
| Linux | `~/.local/share/com.gitflotilla.desktop/flotilla.db` |
| macOS | `~/Library/Application Support/com.gitflotilla.desktop/flotilla.db` |
| Windows | `%APPDATA%\com.gitflotilla.desktop\flotilla.db` |

### Custom Database Path

Override the database path with the `FLOTILLA_DB_PATH` environment variable:

```bash
FLOTILLA_DB_PATH=/path/to/flotilla.db git-flotilla-cli report
```

This is useful for:
- CI pipelines that copy the database to a known location
- Running the CLI against a database on a network share
- Testing with a specific database snapshot

---

## Global Flags

| Flag | Description |
|------|-------------|
| `--json` | Output in JSON format instead of human-readable tables. Works on all commands. |
| `--help` | Show help for the command |
| `--version` | Show the CLI version |

The `--json` flag can be placed before or after the subcommand:

```bash
git-flotilla-cli --json report
git-flotilla-cli report --json
```

---

## Commands

### `repo list`

List all known repositories.

```bash
git-flotilla-cli repo list
```

**Output (human-readable):**

```
REPO                                               PROVIDER   BRANCH       LAST SCANNED
------------------------------------------------------------------------------------------
acme-org/api-gateway                               github     main         2026-04-08T14:30:00Z
acme-org/auth-service                              github     main         2026-04-08T14:30:02Z
acme-org/customer-portal                           github     main         2026-04-08T14:30:05Z
acme-group/admin-dashboard                         gitlab     main         2026-04-08T14:30:08Z

4 repo(s) total
```

**Output (JSON):**

```bash
git-flotilla-cli repo list --json
```

```json
[
  {
    "id": "github:acme-org/api-gateway",
    "fullName": "acme-org/api-gateway",
    "provider": "github",
    "defaultBranch": "main",
    "isPrivate": false,
    "lastScannedAt": "2026-04-08T14:30:00Z"
  }
]
```

---

### `scan --repo <ID>`

Show the latest scan result for a single repository.

```bash
git-flotilla-cli scan --repo "github:acme-org/api-gateway"
```

**Output:**

```
Scan result for github:acme-org/api-gateway
--------------------------------------------------
Scanned at:       2026-04-08T14:30:00Z
Health score:     85/100
Node version:     20.11.0
PHP version:      ^8.2
Package manager:  pnpm 9.15.0
Has develop:      yes
```

**JSON output:**

```bash
git-flotilla-cli scan --repo "github:acme-org/api-gateway" --json
```

```json
{
  "repoId": "github:acme-org/api-gateway",
  "scannedAt": "2026-04-08T14:30:00Z",
  "healthScore": 85,
  "nodeVersion": "20.11.0",
  "phpVersion": "^8.2",
  "packageManager": "pnpm",
  "packageManagerVersion": "9.15.0",
  "hasDevelop": true,
  "excluded": false,
  "excludeReason": null,
  "manifestPaths": "[\"package.json\",\"packages/api/package.json\"]",
  "workflowFiles": "[\"ci.yml\",\"deploy.yml\"]"
}
```

---

### `scan --list <ID>`

Show scan summary for all repos in a repo list.

```bash
git-flotilla-cli scan --list "client-acme"
```

**Output:**

```
Repo list: client-acme
Total repos: 12
Scanned:     10
```

---

### `cve check`

Show a summary of CVE check status.

```bash
git-flotilla-cli cve check
```

**Output:**

```
CVE Status
----------------------------------------
Unique packages tracked: 312
Open CVE alerts:         3

Run `git-flotilla-cli cve list` for details.
```

**JSON output:**

```bash
git-flotilla-cli cve check --json
```

```json
{
  "uniquePackagesTracked": 312,
  "openCves": 3
}
```

---

### `cve list`

List current CVE alerts. Optionally filter by severity.

```bash
# All alerts
git-flotilla-cli cve list

# Only critical alerts
git-flotilla-cli cve list --severity critical

# Only high severity, as JSON
git-flotilla-cli cve list --severity high --json
```

**Output:**

```
CVE                  PACKAGE                        ECOSYSTEM    SEVERITY   STATUS
------------------------------------------------------------------------------------------
CVE-2026-12345       lodash                         npm          critical   new
CVE-2026-12400       express                        npm          high       acknowledged
CVE-2026-11999       symfony/http-kernel            composer     high       new

3 alert(s) total
```

**Severity filter values:** `critical`, `high`, `medium`, `low`

**JSON output:**

```json
[
  {
    "id": "CVE-2026-12345",
    "packageName": "lodash",
    "ecosystem": "npm",
    "severity": "critical",
    "summary": "Prototype Pollution in lodash",
    "status": "new"
  }
]
```

---

### `report`

Show a health summary across all repos.

```bash
git-flotilla-cli report
```

**Output:**

```
Git Flotilla -- Health Report
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Repos:              42
Scanned:            40
Avg Health Score:   78/100
Open CVEs:          3
  Critical:         1
  High:             2
Unique Packages:    312
```

**JSON output:**

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

---

## CI Pipeline Examples

### GitHub Actions -- Health Gate

Run a health check in CI and fail if critical CVEs exist:

```yaml
- name: Check for critical CVEs
  run: |
    CRITS=$(git-flotilla-cli cve list --severity critical --json | jq 'length')
    if [ "$CRITS" -gt 0 ]; then
      echo "CRITICAL: $CRITS critical CVE(s) found"
      git-flotilla-cli cve list --severity critical
      exit 1
    fi
  env:
    FLOTILLA_DB_PATH: ${{ secrets.FLOTILLA_DB_PATH }}
```

### Scheduled Report

Generate a JSON report and upload as an artifact:

```yaml
- name: Generate health report
  run: git-flotilla-cli report --json > health-report.json
  env:
    FLOTILLA_DB_PATH: /path/to/flotilla.db

- name: Upload report
  uses: actions/upload-artifact@v4
  with:
    name: health-report
    path: health-report.json
```

---

## Sharing the Database with the GUI

The CLI and GUI share the same SQLite database. This means:

- Any repos discovered or scanned in the GUI are immediately visible to the CLI
- The CLI never writes to the database (read-only mode)
- You can run the CLI while the GUI is open -- SQLite handles concurrent readers

If you want to use the CLI on a different machine (e.g. a CI server), copy the `flotilla.db` file and set `FLOTILLA_DB_PATH` to point to it. The database is a single file and is portable across platforms.

---

## Error Handling

The CLI handles common error cases gracefully:

| Scenario | Behaviour |
|----------|-----------|
| Database not found | Prints error with path and suggests launching the GUI first or setting `FLOTILLA_DB_PATH` |
| No repos discovered | Prints "No repos found" message |
| No scan results | Prints "No scan results found" for the specified repo |
| Invalid severity filter | Shows available options |
| Empty results with `--json` | Returns `[]` (empty array) or `null` |

All errors are printed to stderr with a non-zero exit code.
