use crate::models::RateLimitInfo;
use crate::services::github::RateLimitSnapshot;
use crate::services::gitlab::GitLabRateLimitSnapshot;
use std::sync::{LazyLock, RwLock};

static GITHUB: LazyLock<RwLock<Option<RateLimitInfo>>> = LazyLock::new(|| RwLock::new(None));
static GITLAB: LazyLock<RwLock<Option<RateLimitInfo>>> = LazyLock::new(|| RwLock::new(None));

/// Update the in-memory GitHub rate limit snapshot.
pub fn update_github(snapshot: RateLimitSnapshot) {
    if let Ok(mut guard) = GITHUB.write() {
        *guard = Some(RateLimitInfo {
            remaining: snapshot.remaining,
            limit: snapshot.limit,
            reset_epoch: snapshot.reset_epoch,
        });
    }
    // silently drop on poison — stale data is better than a panic
}

/// Retrieve the current GitHub rate limit snapshot, if set.
pub fn get_github() -> Option<RateLimitInfo> {
    GITHUB.read().ok()?.clone()
}

/// Update the in-memory GitLab rate limit snapshot.
pub fn update_gitlab(snapshot: GitLabRateLimitSnapshot) {
    if let Ok(mut guard) = GITLAB.write() {
        *guard = Some(RateLimitInfo {
            remaining: snapshot.remaining,
            limit: snapshot.limit,
            reset_epoch: snapshot.reset_epoch,
        });
    }
}

/// Retrieve the current GitLab rate limit snapshot, if set.
pub fn get_gitlab() -> Option<RateLimitInfo> {
    GITLAB.read().ok()?.clone()
}
