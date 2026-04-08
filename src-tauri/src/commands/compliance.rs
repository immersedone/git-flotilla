use crate::error::{AppError, AppResult};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SecretFinding {
    pub repo_id: String,
    pub file_path: String,
    pub line_number: u32,
    pub secret_type: String,
    pub matched_pattern: String,
}

#[derive(Debug, Serialize)]
pub struct LicenceFinding {
    pub repo_id: String,
    pub package_name: String,
    pub ecosystem: String,
    pub licence: String,
    pub is_flagged: bool,
    pub flag_reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BranchProtectionStatus {
    pub repo_id: String,
    pub branch: String,
    pub requires_reviews: bool,
    pub requires_status_checks: bool,
    pub restricts_pushes: bool,
    pub is_compliant: bool,
    pub issues: Vec<String>,
}

#[tauri::command]
pub async fn scan_secrets(repo_ids: Vec<String>) -> AppResult<Vec<SecretFinding>> {
    let _ = repo_ids;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn scan_licences(
    repo_ids: Vec<String>,
    blocked_licences: Vec<String>,
) -> AppResult<Vec<LicenceFinding>> {
    let _ = (repo_ids, blocked_licences);
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn audit_branch_protection(
    repo_ids: Vec<String>,
) -> AppResult<Vec<BranchProtectionStatus>> {
    let _ = repo_ids;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn archive_repos(repo_ids: Vec<String>) -> AppResult<Vec<String>> {
    let _ = repo_ids;
    Err(AppError::Operation("not implemented".into()))
}
