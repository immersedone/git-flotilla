use crate::commands::settings::RateLimitInfo;
use std::sync::{LazyLock, RwLock};

static GITHUB: LazyLock<RwLock<Option<RateLimitInfo>>> =
    LazyLock::new(|| RwLock::new(None));

/// Update the in-memory GitHub rate limit snapshot.
pub fn update_github(remaining: u32, limit: u32, reset_epoch: u64) {
    *GITHUB.write().unwrap() = Some(RateLimitInfo {
        remaining,
        limit,
        reset_epoch,
    });
}

/// Retrieve the current GitHub rate limit snapshot, if set.
pub fn get_github() -> Option<RateLimitInfo> {
    GITHUB.read().unwrap().clone()
}
