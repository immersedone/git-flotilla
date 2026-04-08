use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccountInfo {
    pub id: String,
    pub provider: String,
    pub username: String,
    pub scopes: Vec<String>,
}

#[tauri::command]
pub async fn add_account(provider: String, token: String) -> AppResult<AccountInfo> {
    let _ = (provider, token);
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn remove_account(id: String) -> AppResult<()> {
    let _ = id;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn list_accounts() -> AppResult<Vec<AccountInfo>> {
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn validate_token(provider: String, token: String) -> AppResult<AccountInfo> {
    let _ = (provider, token);
    Err(AppError::Operation("not implemented".into()))
}
