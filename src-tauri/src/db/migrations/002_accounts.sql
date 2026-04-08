-- Stores connected account metadata (tokens live in OS keychain, not here)
CREATE TABLE IF NOT EXISTS accounts (
    id         TEXT PRIMARY KEY,   -- "github:{username}"
    provider   TEXT NOT NULL,      -- "github" | "gitlab"
    username   TEXT NOT NULL,
    avatar_url TEXT,
    added_at   TEXT NOT NULL DEFAULT (datetime('now'))
);
