use crate::error::{AppError, AppResult};
use crate::models::BatchOperation;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOperationInput {
    pub operation_type: String,
    pub mode: Option<String>,
    pub target_repo_ids: Vec<String>,
    pub package_name: Option<String>,
    pub target_version: Option<String>,
    pub version_map: Option<std::collections::HashMap<String, String>>,
    pub file_path: Option<String>,
    pub file_content: Option<String>,
    pub pr_title_template: Option<String>,
    pub pr_body_template: Option<String>,
    pub branch_prefix: Option<String>,
    pub label: Option<String>,
    pub is_dry_run: bool,
    pub skip_ci: bool,
    pub also_target_branches: Vec<String>,
    pub divergence_check: bool,
    pub divergence_threshold: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct ValidateResult {
    pub repo_id: String,
    pub is_applied: bool,
    pub current_version: Option<String>,
    pub has_overrides: bool,
}

#[tauri::command]
pub async fn create_operation(input: CreateOperationInput) -> AppResult<BatchOperation> {
    let _ = input;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn run_operation(id: String) -> AppResult<()> {
    let _ = id;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn abort_operation(id: String) -> AppResult<()> {
    let _ = id;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn list_operations() -> AppResult<Vec<BatchOperation>> {
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn get_operation(id: String) -> AppResult<BatchOperation> {
    let _ = id;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn validate_operation(
    package_name: String,
    target_version: String,
    repo_ids: Vec<String>,
) -> AppResult<Vec<ValidateResult>> {
    let _ = (package_name, target_version, repo_ids);
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn rollback_operation(id: String) -> AppResult<()> {
    let _ = id;
    Err(AppError::Operation("not implemented".into()))
}
