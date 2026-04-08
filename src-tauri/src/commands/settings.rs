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
#[serde(rename_all = "camelCase")]
pub struct RateLimitStatus {
    pub github: Option<RateLimitInfo>,
    pub gitlab: Option<RateLimitInfo>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitInfo {
    pub remaining: u32,
    pub limit: u32,
    pub reset_epoch: u64,
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
    Ok(RateLimitStatus {
        github: crate::services::rate_limiter::get_github(),
        gitlab: None,
    })
}
