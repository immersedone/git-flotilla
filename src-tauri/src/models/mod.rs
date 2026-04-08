use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Repo {
    pub id: String,
    pub provider: String,
    pub owner: String,
    pub name: String,
    pub full_name: String,
    pub url: String,
    pub default_branch: String,
    pub is_private: bool,
    pub last_scanned_at: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RepoList {
    pub id: String,
    pub name: String,
    pub description: String,
    pub repo_ids: Vec<String>,
    pub parent_id: Option<String>,
    pub exclude_patterns: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub repo_id: String,
    pub scanned_at: String,
    pub manifest_paths: Vec<String>,
    pub node_version: Option<String>,
    pub node_version_source: Option<String>,
    pub php_version: Option<String>,
    pub package_manager: Option<String>,
    pub package_manager_version: Option<String>,
    pub has_develop: bool,
    pub last_pushed: Option<String>,
    pub has_dot_env_example: bool,
    pub workflow_files: Vec<String>,
    pub health_score: u32,
    pub flags: Vec<ScanFlag>,
    pub excluded: bool,
    pub exclude_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScanFlag {
    pub flag_type: String,
    pub message: String,
    pub severity: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RepoPackage {
    pub repo_id: String,
    pub ecosystem: String,
    pub name: String,
    pub version: String,
    pub is_dev: bool,
    pub scanned_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CveAlert {
    pub id: String,
    pub package_name: String,
    pub ecosystem: String,
    pub severity: String,
    pub summary: String,
    pub affected_version_range: String,
    pub fixed_version: Option<String>,
    pub published_at: String,
    pub detected_at: String,
    pub affected_repos: Vec<String>,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BatchOperation {
    pub id: String,
    #[serde(rename = "type")]
    pub operation_type: String,
    pub mode: Option<String>,
    pub status: String,
    pub target_repo_ids: Vec<String>,
    pub completed_repo_ids: Vec<String>,
    pub version_map: Option<HashMap<String, String>>,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub results: Vec<OperationResult>,
    pub is_dry_run: bool,
    pub skip_ci: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OperationResult {
    pub repo_id: String,
    pub status: String,
    pub pr_url: Option<String>,
    pub error: Option<String>,
    pub diff: Option<String>,
}
