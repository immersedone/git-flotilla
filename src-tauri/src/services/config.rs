use crate::commands::settings::{AppSettings, HealthScoreWeights};
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};

// ── Config Schema ─────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FlotillaConfig {
    pub scan_interval: String,
    pub cve_poll_interval: String,
    pub inter_request_delay_ms: u32,
    pub concurrent_workers: u32,
    pub divergence_threshold: u32,
    pub health_score_weights: ConfigHealthScoreWeights,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConfigHealthScoreWeights {
    pub has_codeowners: u32,
    pub has_security_md: u32,
    pub has_env_example: u32,
    pub has_editorconfig: u32,
    pub no_floating_action_tags: u32,
    pub deps_up_to_date: u32,
    pub no_known_cves: u32,
    pub runtime_not_eol: u32,
}

impl Default for ConfigHealthScoreWeights {
    fn default() -> Self {
        Self {
            has_codeowners: 10,
            has_security_md: 10,
            has_env_example: 5,
            has_editorconfig: 5,
            no_floating_action_tags: 15,
            deps_up_to_date: 20,
            no_known_cves: 20,
            runtime_not_eol: 15,
        }
    }
}

impl Default for FlotillaConfig {
    fn default() -> Self {
        Self {
            scan_interval: "daily".to_string(),
            cve_poll_interval: "1hr".to_string(),
            inter_request_delay_ms: 200,
            concurrent_workers: 5,
            divergence_threshold: 50,
            health_score_weights: ConfigHealthScoreWeights::default(),
        }
    }
}

// ── Conversion helpers ────────────────────────────────────────────────────

impl FlotillaConfig {
    /// Build a `FlotillaConfig` from `AppSettings`.
    pub fn from_app_settings(settings: &AppSettings) -> Self {
        let scan_interval = match settings.scan_interval_minutes {
            None | Some(0) => "manual".to_string(),
            Some(m) if m <= 1440 => "daily".to_string(),
            Some(_) => "weekly".to_string(),
        };

        let cve_poll_interval = match settings.cve_poll_interval_minutes {
            None | Some(0) => "off".to_string(),
            Some(m) if m <= 15 => "15min".to_string(),
            Some(m) if m <= 30 => "30min".to_string(),
            Some(m) if m <= 60 => "1hr".to_string(),
            Some(m) if m <= 360 => "6hr".to_string(),
            Some(_) => "daily".to_string(),
        };

        FlotillaConfig {
            scan_interval,
            cve_poll_interval,
            inter_request_delay_ms: settings.request_delay_ms,
            concurrent_workers: settings.parallel_workers,
            divergence_threshold: 50,
            health_score_weights: ConfigHealthScoreWeights {
                has_codeowners: settings.health_score_weights.has_codeowners,
                has_security_md: settings.health_score_weights.has_security_md,
                has_env_example: settings.health_score_weights.has_env_example,
                has_editorconfig: settings.health_score_weights.has_editorconfig,
                no_floating_action_tags: settings.health_score_weights.no_floating_action_tags,
                deps_up_to_date: settings.health_score_weights.deps_up_to_date,
                no_known_cves: settings.health_score_weights.no_known_cves,
                runtime_not_eol: settings.health_score_weights.runtime_not_eol,
            },
        }
    }

    /// Convert back to `AppSettings` for persistence.
    pub fn to_app_settings(&self) -> AppSettings {
        let scan_interval_minutes = match self.scan_interval.as_str() {
            "manual" => None,
            "daily" => Some(1440),
            "weekly" => Some(10080),
            _ => Some(1440),
        };

        let cve_poll_interval_minutes = match self.cve_poll_interval.as_str() {
            "off" => None,
            "15min" => Some(15),
            "30min" => Some(30),
            "1hr" => Some(60),
            "6hr" => Some(360),
            "daily" => Some(1440),
            _ => Some(60),
        };

        AppSettings {
            scan_interval_minutes,
            cve_poll_interval_minutes,
            parallel_workers: self.concurrent_workers,
            request_delay_ms: self.inter_request_delay_ms,
            health_score_weights: HealthScoreWeights {
                has_codeowners: self.health_score_weights.has_codeowners,
                has_security_md: self.health_score_weights.has_security_md,
                has_env_example: self.health_score_weights.has_env_example,
                has_editorconfig: self.health_score_weights.has_editorconfig,
                no_floating_action_tags: self.health_score_weights.no_floating_action_tags,
                deps_up_to_date: self.health_score_weights.deps_up_to_date,
                no_known_cves: self.health_score_weights.no_known_cves,
                runtime_not_eol: self.health_score_weights.runtime_not_eol,
            },
            webhook_url: None,
            webhook_events: Vec::new(),
            dark_mode: true,
        }
    }
}

// ── Public API ────────────────────────────────────────────────────────────

/// Serialize a `FlotillaConfig` to YAML.
pub fn export_config(config: &FlotillaConfig) -> AppResult<String> {
    serde_yaml_ng::to_string(config)
        .map_err(|e| AppError::Operation(format!("YAML serialisation failed: {e}")))
}

/// Parse a YAML string into a `FlotillaConfig`.
pub fn import_config(yaml: &str) -> AppResult<FlotillaConfig> {
    serde_yaml_ng::from_str(yaml)
        .map_err(|e| AppError::InvalidInput(format!("Invalid config YAML: {e}")))
}

/// Validate a `FlotillaConfig` and return a list of human-readable errors.
/// Returns an empty vec if valid.
pub fn validate_config(config: &FlotillaConfig) -> Vec<String> {
    let mut errors = Vec::new();

    let valid_scan_intervals = ["manual", "daily", "weekly"];
    if !valid_scan_intervals.contains(&config.scan_interval.as_str()) {
        errors.push(format!(
            "Invalid scan_interval '{}': expected one of {:?}",
            config.scan_interval, valid_scan_intervals
        ));
    }

    let valid_cve_intervals = ["off", "15min", "30min", "1hr", "6hr", "daily"];
    if !valid_cve_intervals.contains(&config.cve_poll_interval.as_str()) {
        errors.push(format!(
            "Invalid cve_poll_interval '{}': expected one of {:?}",
            config.cve_poll_interval, valid_cve_intervals
        ));
    }

    if config.inter_request_delay_ms > 10000 {
        errors.push(format!(
            "inter_request_delay_ms {} exceeds maximum of 10000",
            config.inter_request_delay_ms
        ));
    }

    if config.concurrent_workers == 0 || config.concurrent_workers > 50 {
        errors.push(format!(
            "concurrent_workers {} out of range 1..50",
            config.concurrent_workers
        ));
    }

    if config.divergence_threshold == 0 {
        errors.push("divergence_threshold must be greater than 0".to_string());
    }

    let w = &config.health_score_weights;
    let total = w.has_codeowners
        + w.has_security_md
        + w.has_env_example
        + w.has_editorconfig
        + w.no_floating_action_tags
        + w.deps_up_to_date
        + w.no_known_cves
        + w.runtime_not_eol;
    if total != 100 {
        errors.push(format!(
            "Health score weights sum to {} but must sum to 100",
            total
        ));
    }

    errors
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_default_values() {
        let config = FlotillaConfig::default();
        assert_eq!(config.scan_interval, "daily");
        assert_eq!(config.cve_poll_interval, "1hr");
        assert_eq!(config.inter_request_delay_ms, 200);
        assert_eq!(config.concurrent_workers, 5);
        assert_eq!(config.divergence_threshold, 50);
        assert_eq!(config.health_score_weights.has_codeowners, 10);
    }

    #[test]
    fn config_yaml_round_trip() {
        let config = FlotillaConfig::default();
        let yaml = export_config(&config).expect("export");
        let restored = import_config(&yaml).expect("import");
        assert_eq!(restored.scan_interval, config.scan_interval);
        assert_eq!(restored.cve_poll_interval, config.cve_poll_interval);
        assert_eq!(
            restored.inter_request_delay_ms,
            config.inter_request_delay_ms
        );
        assert_eq!(restored.concurrent_workers, config.concurrent_workers);
        assert_eq!(restored.divergence_threshold, config.divergence_threshold);
        assert_eq!(
            restored.health_score_weights.no_known_cves,
            config.health_score_weights.no_known_cves
        );
    }

    #[test]
    fn config_validate_valid() {
        let config = FlotillaConfig::default();
        let errors = validate_config(&config);
        assert!(errors.is_empty(), "Expected no errors: {:?}", errors);
    }

    #[test]
    fn config_validate_invalid_scan_interval() {
        let mut config = FlotillaConfig::default();
        config.scan_interval = "every_second".to_string();
        let errors = validate_config(&config);
        assert!(errors.iter().any(|e| e.contains("scan_interval")));
    }

    #[test]
    fn config_validate_invalid_cve_interval() {
        let mut config = FlotillaConfig::default();
        config.cve_poll_interval = "5min".to_string();
        let errors = validate_config(&config);
        assert!(errors.iter().any(|e| e.contains("cve_poll_interval")));
    }

    #[test]
    fn config_validate_workers_zero() {
        let mut config = FlotillaConfig::default();
        config.concurrent_workers = 0;
        let errors = validate_config(&config);
        assert!(errors.iter().any(|e| e.contains("concurrent_workers")));
    }

    #[test]
    fn config_validate_weights_not_100() {
        let mut config = FlotillaConfig::default();
        config.health_score_weights.has_codeowners = 99;
        let errors = validate_config(&config);
        assert!(errors.iter().any(|e| e.contains("sum to")));
    }

    #[test]
    fn config_from_app_settings_round_trip() {
        let original = AppSettings::default();
        let config = FlotillaConfig::from_app_settings(&original);
        let restored = config.to_app_settings();
        assert_eq!(restored.parallel_workers, original.parallel_workers);
        assert_eq!(restored.request_delay_ms, original.request_delay_ms);
        assert_eq!(
            restored.health_score_weights.has_codeowners,
            original.health_score_weights.has_codeowners
        );
    }

    #[test]
    fn config_serialises_camel_case() {
        let config = FlotillaConfig::default();
        let json = serde_json::to_string(&config).expect("serialize");
        assert!(
            json.contains("\"scanInterval\""),
            "expected camelCase: {json}"
        );
        assert!(
            json.contains("\"cvePollInterval\""),
            "expected camelCase: {json}"
        );
        assert!(
            json.contains("\"interRequestDelayMs\""),
            "expected camelCase: {json}"
        );
        assert!(
            json.contains("\"concurrentWorkers\""),
            "expected camelCase: {json}"
        );
        assert!(
            json.contains("\"divergenceThreshold\""),
            "expected camelCase: {json}"
        );
    }

    #[test]
    fn config_import_rejects_invalid_yaml() {
        let result = import_config(": bad yaml [[[");
        assert!(result.is_err());
    }
}
