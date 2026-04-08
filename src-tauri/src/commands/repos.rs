use crate::error::{AppError, AppResult};
use crate::models::{Repo, RepoList};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRepoListInput {
    pub name: String,
    pub description: String,
    pub parent_id: Option<String>,
}

#[tauri::command]
pub async fn discover_repos(account_id: String) -> AppResult<Vec<Repo>> {
    let _ = account_id;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn list_repos(repo_list_id: Option<String>) -> AppResult<Vec<Repo>> {
    let _ = repo_list_id;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn get_repo(id: String) -> AppResult<Repo> {
    let _ = id;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn create_repo_list(input: CreateRepoListInput) -> AppResult<RepoList> {
    let _ = input;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn update_repo_list(id: String, input: CreateRepoListInput) -> AppResult<RepoList> {
    let _ = (id, input);
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn delete_repo_list(id: String) -> AppResult<()> {
    let _ = id;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn list_repo_lists() -> AppResult<Vec<RepoList>> {
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn add_repos_to_list(list_id: String, repo_ids: Vec<String>) -> AppResult<()> {
    let _ = (list_id, repo_ids);
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn remove_repos_from_list(list_id: String, repo_ids: Vec<String>) -> AppResult<()> {
    let _ = (list_id, repo_ids);
    Err(AppError::Operation("not implemented".into()))
}
