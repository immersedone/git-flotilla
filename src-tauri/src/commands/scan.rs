use crate::error::{AppError, AppResult};
use crate::models::ScanResult;

#[tauri::command]
pub async fn scan_repo(repo_id: String) -> AppResult<ScanResult> {
    let _ = repo_id;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn scan_repo_list(list_id: String) -> AppResult<String> {
    let _ = list_id;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn get_scan_result(repo_id: String) -> AppResult<ScanResult> {
    let _ = repo_id;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn list_scan_results(repo_list_id: Option<String>) -> AppResult<Vec<ScanResult>> {
    let _ = repo_list_id;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn abort_scan(operation_id: String) -> AppResult<()> {
    let _ = operation_id;
    Err(AppError::Operation("not implemented".into()))
}
