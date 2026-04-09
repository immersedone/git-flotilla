# Batch Operations

Batch operations are Flotilla's core capability for making changes across multiple repositories at once. Whether you are pinning a vulnerable package, bumping versions, or pushing a workflow file, operations follow the same pattern: select targets, preview with dry run, execute, and track progress.

---

## Operation Types

| Type | Description | Use case |
|------|-------------|----------|
| `file_update` | Push a file to N repos via commit or PR | Syncing `.nvmrc`, `.editorconfig`, workflow files |
| `package_pin` | Set exact version + add `overrides`/`resolutions` | Emergency security fix -- lock entire dep tree |
| `package_bump` | Set version range + remove overrides | After upstream fix confirmed -- return to normal |
| `workflow_sync` | Push GitHub Actions workflows across repos | Standardising CI pipelines |
| `script_run` | Execute a shell command across repos | Running audits, migrations, custom checks |
| `pr_create` | Open PRs across repos | Batch code changes via pull request |
| `commit` | Direct commit to default branch | Quick fixes that don't need review |

---

## Pin vs Bump Modes

These two modes handle the lifecycle of a security fix.

### Pin Mode (emergency response)

Use pin mode when a CVE is actively being exploited or is critical severity. Pin mode:

1. Sets the package to an **exact version** (e.g. `"1.13.6"` not `"^1.13.6"`)
2. Adds **override entries** to lock the entire dependency tree:
   - npm: `overrides` in `package.json`
   - pnpm: `pnpm.overrides` in `package.json`
   - yarn: `resolutions` in `package.json`
   - composer: conflict constraints in `composer.json`
3. This ensures no transitive dependency can pull in the vulnerable version

### Bump Mode (return to normal)

Use bump mode after the upstream package has been confirmed stable. Bump mode:

1. Sets the package to a **version range** (e.g. `"^1.13.6"`)
2. **Removes** override entries that were added during pin
3. Returns the repo to normal dependency resolution

### Pin-Then-Bump Lifecycle

Flotilla tracks which repos are still pinned. After a pin operation:

1. The operation is marked as "pin active"
2. Flotilla surfaces pinned repos in the operations view
3. When you are ready, create a bump operation targeting the same repos
4. The bump operation removes the overrides and updates the version range

---

## Dry Run

Every operation supports dry run mode. In dry run mode, Flotilla:

1. Evaluates each target repo
2. Generates a diff showing what would change
3. Reports the diff per repo without writing anything

Dry run is **on by default** (`operations.default_dry_run: true` in config). You must explicitly execute after reviewing the dry run results.

### Example Dry Run Output

```
Repo: acme-org/api-gateway
  File: package.json
  - "lodash": "^4.17.20"
  + "lodash": "4.17.21"
  
  File: package.json (overrides section)
  + "overrides": {
  +   "lodash": "4.17.21"
  + }

Repo: acme-org/customer-portal
  File: package.json
  - "lodash": "^4.17.19"
  + "lodash": "4.17.21"
  
  File: packages/shared/package.json
  - "lodash": "^4.17.19"
  + "lodash": "4.17.21"
```

---

## Validation Mode

Validation mode audits whether a fix is already applied across all target repos **without making any changes**. Use it to:

- Verify a previous patch operation was successful
- Check whether repos have already been updated manually
- Confirm compliance before closing a CVE

Validation mode reports per repo:
- **Already patched** -- the repo has the target version
- **Not patched** -- the repo still has a vulnerable version
- **Not applicable** -- the repo does not use the package

---

## Version Map

When a package has multiple active major versions, you can target different safe versions per major version using a version map:

```json
{
  "0": "0.30.3",
  "1": "1.13.6"
}
```

This means:
- Repos on major version 0.x will be updated to `0.30.3`
- Repos on major version 1.x will be updated to `1.13.6`

Version maps are useful when a security fix has been backported to multiple release lines.

---

## PR Templates

When an operation creates PRs, the title and body are generated from templates with variable injection.

### Available Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `{{PACKAGE}}` | Package name | `lodash` |
| `{{VERSION}}` | Target version | `4.17.21` |
| `{{REPO}}` | Repository full name | `acme-org/api-gateway` |
| `{{CVE}}` | CVE ID (if applicable) | `CVE-2024-12345` |
| `{{SEVERITY}}` | CVE severity (if applicable) | `critical` |
| `{{DATE}}` | Current date | `2026-04-09` |
| `{{MODE}}` | Operation mode | `pin` or `bump` |
| `{{ECOSYSTEM}}` | Package ecosystem | `npm` |

### Conditional Sections

Use `{{#FIELD}}content{{/FIELD}}` to include content only when a variable has a value:

```markdown
## Automated update by Git Flotilla

{{#CVE}}
### Security Fix
This PR addresses **{{CVE}}** ({{SEVERITY}}).
{{/CVE}}

**Package:** `{{PACKAGE}}`
**Version:** `{{VERSION}}`
```

If no CVE is associated with the operation, the entire security fix section is removed from the PR body.

### Default Templates

Configure default PR templates in `.flotilla/config.yaml`:

```yaml
operations:
  pr:
    draft: false
    labels:
      - flotilla
    title_template: "chore(deps): {{MODE}} {{PACKAGE}} to {{VERSION}} [flotilla]"
    body_template: |
      ## Automated update by Git Flotilla
      
      {{#CVE}}
      ### Security Fix
      This PR addresses **{{CVE}}** ({{SEVERITY}}).
      {{/CVE}}
      
      **Package:** `{{PACKAGE}}`
      **New version:** `{{VERSION}}`
```

---

## Parallelism and Resumability

### Parallel Execution

Operations run across repos in parallel with a configurable worker count:

```yaml
operations:
  parallelism: 5    # Default: 5 concurrent repos
```

Each repo is processed independently. If one repo fails, others continue.

### Resumability

Operation progress is saved to the database per repo. If the app crashes or you abort an operation:

1. The operation status is preserved (which repos are completed, which are pending)
2. When you resume, Flotilla picks up from the last incomplete repo
3. Already-completed repos are not re-processed

The `completedRepoIds` field on each operation tracks which repos have been successfully processed.

### Abort

You can abort a running operation at any time. Repos already processed keep their changes. Repos not yet started are skipped. You can resume later to complete the remaining repos.

---

## Skip CI

When pushing changes to many repos at once, you may want to avoid triggering CI on all of them simultaneously. Enable the **Skip CI** toggle to append `[skip ci]` to commit messages. This is respected by GitHub Actions, GitLab CI, and most CI systems.

```
chore(deps): pin lodash to 4.17.21 [skip ci]
```

---

## Idempotent PR Creation

If you re-run an operation that previously created PRs, Flotilla handles it cleanly:

1. Detects existing PRs from the same branch name
2. Closes the stale PR
3. Deletes the stale branch
4. Creates a fresh branch and PR

This prevents duplicate PRs from accumulating when you need to re-run an operation (e.g. after updating the target version).

---

## Rollback

For operations that committed changes directly (not via PR), Flotilla stores the pre-change commit SHA and offers a **rollback** option:

1. Go to the operation detail view
2. Click **Rollback**
3. Flotilla creates a revert commit (or PR) for each affected repo

Rollback is only available for Flotilla-initiated commits, not for merged PRs (which should be reverted through the normal Git workflow).

---

## Workflow Examples

### Emergency CVE Patch

1. CVE alert appears for `lodash` (critical severity)
2. Click **Patch affected repos** on the CVE
3. Review the pre-filled pin operation (all repos with vulnerable versions)
4. Run dry run -- review diffs
5. Execute -- PRs opened across all affected repos
6. Monitor PRs in the Merge Queue view
7. Merge all green PRs with one click
8. Later: create a bump operation to remove overrides

### Sync a Workflow File

1. Go to **Operations > New Operation**
2. Select type: **File Update**
3. Enter file path: `.github/workflows/ci.yml`
4. Paste the workflow file content (supports `{{REPO}}` and `{{BRANCH}}` variables)
5. Select target repos or repo list
6. Dry run to preview
7. Execute via PR

### Standardise Node Version

1. Go to **Operations > New Operation**
2. Select type: **File Update**
3. File path: `.nvmrc`
4. Content: `20.11.0`
5. Select all repos in your list
6. Execute
