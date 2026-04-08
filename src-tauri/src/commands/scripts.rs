use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ScriptPreset {
    pub id: String,
    pub name: String,
    pub command: String,
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct ScriptRepoResult {
    pub repo_id: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
}

#[tauri::command]
pub async fn run_script(
    command: String,
    repo_ids: Vec<String>,
    parallel: u32,
) -> AppResult<String> {
    let _ = (command, repo_ids, parallel);
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn abort_script(run_id: String) -> AppResult<()> {
    let _ = run_id;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn list_presets() -> AppResult<Vec<ScriptPreset>> {
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn save_preset(preset: ScriptPreset) -> AppResult<ScriptPreset> {
    let _ = preset;
    Err(AppError::Operation("not implemented".into()))
}

#[tauri::command]
pub async fn delete_preset(id: String) -> AppResult<()> {
    let _ = id;
    Err(AppError::Operation("not implemented".into()))
}
