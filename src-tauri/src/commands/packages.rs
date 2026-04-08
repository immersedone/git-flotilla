use crate::error::{AppError, AppResult};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DependencyMatrix {
    pub packages: Vec<PackageRow>,
    pub repo_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct PackageRow {
    pub name: String,
    pub ecosystem: String,
    pub versions_by_repo: std::collections::HashMap<String, String>,
    pub latest_version: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChangelogEntry {
    pub version: String,
    pub body: String,
    pub published_at: String,
    pub is_breaking: bool,
}

#[tauri::command]
pub async fn get_dependency_matrix(
    repo_list_id: Option<String>,
    ecosystem: Option<String>,
) -> AppResult<DependencyMatrix> {
    let _ = (repo_list_id, ecosystem);
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn get_package_changelog(
    package_name: String,
    ecosystem: String,
    from_version: String,
    to_version: String,
) -> AppResult<Vec<ChangelogEntry>> {
    let _ = (package_name, ecosystem, from_version, to_version);
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn export_matrix_csv(repo_list_id: Option<String>) -> AppResult<String> {
    let _ = repo_list_id;
    Err(AppError::Operation("not implemented".into()))
}
