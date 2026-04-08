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
