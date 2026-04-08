use crate::error::{AppError, AppResult};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct FlotillaPr {
    pub repo_id: String,
    pub pr_number: u64,
    pub title: String,
    pub state: String,
    pub mergeable: Option<String>,
    pub ci_status: Option<String>,
    pub operation_id: String,
    pub created_at: String,
    pub html_url: String,
}

#[tauri::command]
pub async fn list_flotilla_prs(operation_id: Option<String>) -> AppResult<Vec<FlotillaPr>> {
    let _ = operation_id;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn merge_pr(repo_id: String, pr_number: u64) -> AppResult<()> {
    let _ = (repo_id, pr_number);
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn merge_all_green(operation_id: Option<String>) -> AppResult<u32> {
    let _ = operation_id;
    Err(AppError::Operation("not implemented".into()))
}
