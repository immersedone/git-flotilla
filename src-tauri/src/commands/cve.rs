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
