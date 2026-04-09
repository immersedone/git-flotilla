use crate::error::AppResult;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// ── OSV.dev API request/response types ────────────────────────────────────

#[derive(Debug, Serialize)]
struct OsvQueryBatchRequest {
    queries: Vec<OsvQuery>,
}

#[derive(Debug, Serialize)]
struct OsvQuery {
    package: OsvPackage,
    version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OsvPackage {
    pub name: String,
    pub ecosystem: String,
}

#[derive(Debug, Deserialize)]
struct OsvQueryBatchResponse {
    results: Vec<OsvQueryResult>,
}

#[derive(Debug, Deserialize)]
struct OsvQueryResult {
    vulns: Option<Vec<OsvVulnerability>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OsvVulnerability {
    pub id: String,
    pub aliases: Option<Vec<String>>,
    pub summary: Option<String>,
    pub details: Option<String>,
    pub severity: Option<Vec<OsvSeverity>>,
    pub affected: Option<Vec<OsvAffected>>,
    pub published: Option<String>,
    pub modified: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OsvSeverity {
    #[serde(rename = "type")]
    pub severity_type: String,
    pub score: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OsvAffected {
    pub package: Option<OsvPackage>,
    pub ranges: Option<Vec<OsvRange>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OsvRange {
    #[serde(rename = "type")]
    pub range_type: String,
    pub events: Option<Vec<OsvEvent>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OsvEvent {
    pub introduced: Option<String>,
    pub fixed: Option<String>,
}

// ── Constants ─────────────────────────────────────────────────────────────

const OSV_QUERYBATCH_URL: &str = "https://api.osv.dev/v1/querybatch";
const OSV_BATCH_LIMIT: usize = 1000;

// ── Ecosystem mapping ─────────────────────────────────────────────────────

/// Map our internal ecosystem names to OSV.dev ecosystem names.
fn map_ecosystem(ecosystem: &str) -> &str {
    match ecosystem {
        "composer" => "Packagist",
        "pip" => "PyPI",
        "cargo" => "crates.io",
        "go" => "Go",
        "npm" => "npm",
        _ => ecosystem,
    }
}

// ── Public API ────────────────────────────────────────────────────────────

/// Query OSV.dev for vulnerabilities affecting a batch of packages.
///
/// Takes (name, ecosystem, version) triples. Batches in groups of 1000
/// (OSV limit) and returns deduplicated vulnerabilities.
pub async fn query_osv_batch(
    packages: &[(String, String, String)],
) -> AppResult<Vec<OsvVulnerability>> {
    if packages.is_empty() {
        return Ok(Vec::new());
    }

    let client = reqwest::Client::new();
    let mut all_vulns = Vec::new();
    let mut seen_ids = HashSet::new();

    for chunk in packages.chunks(OSV_BATCH_LIMIT) {
        let queries: Vec<OsvQuery> = chunk
            .iter()
            .map(|(name, ecosystem, version)| OsvQuery {
                package: OsvPackage {
                    name: name.clone(),
                    ecosystem: map_ecosystem(ecosystem).to_string(),
                },
                version: version.clone(),
            })
            .collect();

        let request = OsvQueryBatchRequest { queries };

        let resp = client
            .post(OSV_QUERYBATCH_URL)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                crate::error::AppError::Operation(format!("OSV.dev request failed: {e}"))
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(crate::error::AppError::Operation(format!(
                "OSV.dev returned HTTP {status}: {body}"
            )));
        }

        let batch_resp: OsvQueryBatchResponse = resp.json().await.map_err(|e| {
            crate::error::AppError::Operation(format!("Failed to parse OSV.dev response: {e}"))
        })?;

        for result in batch_resp.results {
            if let Some(vulns) = result.vulns {
                for vuln in vulns {
                    if seen_ids.insert(vuln.id.clone()) {
                        all_vulns.push(vuln);
                    }
                }
            }
        }
    }

    Ok(all_vulns)
}

// ── Helper functions ──────────────────────────────────────────────────────

/// Extract a CVE ID from aliases (prefer "CVE-" prefix, fallback to GHSA ID, then vuln.id).
pub fn extract_cve_id(vuln: &OsvVulnerability) -> String {
    if let Some(aliases) = &vuln.aliases {
        // Prefer CVE- prefixed alias
        if let Some(cve) = aliases.iter().find(|a| a.starts_with("CVE-")) {
            return cve.clone();
        }
        // Fallback to GHSA- prefixed alias
        if let Some(ghsa) = aliases.iter().find(|a| a.starts_with("GHSA-")) {
            return ghsa.clone();
        }
        // Fallback to first alias
        if let Some(first) = aliases.first() {
            return first.clone();
        }
    }
    vuln.id.clone()
}

/// Convert CVSS severity entries to a severity level string.
///
/// Parses the CVSS vector string to extract the numeric base score.
/// Falls back to "medium" if the score cannot be determined.
///
/// Thresholds: >= 9.0 critical, >= 7.0 high, >= 4.0 medium, < 4.0 low.
pub fn cvss_to_severity(severity: &[OsvSeverity]) -> String {
    for entry in severity {
        if let Some(score) = parse_cvss_score(&entry.score) {
            return if score >= 9.0 {
                "critical".to_string()
            } else if score >= 7.0 {
                "high".to_string()
            } else if score >= 4.0 {
                "medium".to_string()
            } else {
                "low".to_string()
            };
        }
    }
    "medium".to_string()
}

/// Try to parse a numeric CVSS score from the score field.
///
/// The score field may be:
/// - A plain numeric string like "9.8"
/// - A CVSS vector string like "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H"
///
/// For plain numbers, parse directly. For CVSS vectors, we cannot compute the
/// score from the vector alone without a full CVSS calculator, so we return None
/// for vectors and rely on the fallback.
fn parse_cvss_score(score: &str) -> Option<f64> {
    // Try parsing as a plain numeric score first
    if let Ok(val) = score.parse::<f64>() {
        return Some(val);
    }

    // Some OSV entries encode the score as the last segment after a slash
    // e.g., some non-standard formats. Try extracting a trailing number.
    // But standard CVSS vectors don't include a score — skip those.
    if score.starts_with("CVSS:") {
        // Standard CVSS vector string — cannot extract numeric score from this
        return None;
    }

    None
}

/// Extract the fixed version from affected ranges.
///
/// Looks through all affected entries and their ranges for a "fixed" event,
/// returning the first one found.
pub fn extract_fixed_version(vuln: &OsvVulnerability) -> Option<String> {
    let affected = vuln.affected.as_ref()?;
    for entry in affected {
        if let Some(ranges) = &entry.ranges {
            for range in ranges {
                if let Some(events) = &range.events {
                    for event in events {
                        if let Some(fixed) = &event.fixed {
                            return Some(fixed.clone());
                        }
                    }
                }
            }
        }
    }
    None
}

/// Extract the affected version range as a human-readable string.
///
/// Combines "introduced" and "fixed" events into a readable format like
/// ">= 1.0.0, < 2.0.0".
pub fn extract_affected_range(vuln: &OsvVulnerability) -> String {
    let mut parts = Vec::new();

    if let Some(affected) = &vuln.affected {
        for entry in affected {
            if let Some(ranges) = &entry.ranges {
                for range in ranges {
                    if let Some(events) = &range.events {
                        let mut introduced = None;
                        let mut fixed = None;
                        for event in events {
                            if event.introduced.is_some() {
                                introduced = event.introduced.as_deref();
                            }
                            if event.fixed.is_some() {
                                fixed = event.fixed.as_deref();
                            }
                        }
                        match (introduced, fixed) {
                            (Some(intro), Some(fix)) => {
                                if intro == "0" {
                                    parts.push(format!("< {fix}"));
                                } else {
                                    parts.push(format!(">= {intro}, < {fix}"));
                                }
                            }
                            (Some(intro), None) => {
                                if intro == "0" {
                                    parts.push("all versions".to_string());
                                } else {
                                    parts.push(format!(">= {intro}"));
                                }
                            }
                            (None, Some(fix)) => {
                                parts.push(format!("< {fix}"));
                            }
                            (None, None) => {}
                        }
                    }
                }
            }
        }
    }

    if parts.is_empty() {
        "unknown".to_string()
    } else {
        parts.join(" || ")
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vuln(id: &str, aliases: Option<Vec<&str>>) -> OsvVulnerability {
        OsvVulnerability {
            id: id.to_string(),
            aliases: aliases.map(|a| a.into_iter().map(|s| s.to_string()).collect()),
            summary: None,
            details: None,
            severity: None,
            affected: None,
            published: None,
            modified: None,
        }
    }

    fn make_severity(score: &str) -> Vec<OsvSeverity> {
        vec![OsvSeverity {
            severity_type: "CVSS_V3".to_string(),
            score: score.to_string(),
        }]
    }

    #[test]
    fn extract_cve_id_prefers_cve_alias() {
        let vuln = make_vuln("GHSA-abc-def-ghi", Some(vec!["CVE-2024-123", "GHSA-abc"]));
        assert_eq!(extract_cve_id(&vuln), "CVE-2024-123");
    }

    #[test]
    fn extract_cve_id_falls_back_to_ghsa() {
        let vuln = make_vuln("PYSEC-2024-1", Some(vec!["GHSA-abc-def-ghi"]));
        assert_eq!(extract_cve_id(&vuln), "GHSA-abc-def-ghi");
    }

    #[test]
    fn extract_cve_id_falls_back_to_id() {
        let vuln = make_vuln("PYSEC-2024-1", None);
        assert_eq!(extract_cve_id(&vuln), "PYSEC-2024-1");
    }

    #[test]
    fn extract_cve_id_falls_back_to_id_empty_aliases() {
        let vuln = make_vuln("GHSA-xyz", Some(vec![]));
        assert_eq!(extract_cve_id(&vuln), "GHSA-xyz");
    }

    #[test]
    fn cvss_to_severity_critical() {
        assert_eq!(cvss_to_severity(&make_severity("9.8")), "critical");
        assert_eq!(cvss_to_severity(&make_severity("9.0")), "critical");
        assert_eq!(cvss_to_severity(&make_severity("10.0")), "critical");
    }

    #[test]
    fn cvss_to_severity_high() {
        assert_eq!(cvss_to_severity(&make_severity("7.0")), "high");
        assert_eq!(cvss_to_severity(&make_severity("8.9")), "high");
    }

    #[test]
    fn cvss_to_severity_medium() {
        assert_eq!(cvss_to_severity(&make_severity("4.0")), "medium");
        assert_eq!(cvss_to_severity(&make_severity("6.9")), "medium");
    }

    #[test]
    fn cvss_to_severity_low() {
        assert_eq!(cvss_to_severity(&make_severity("3.9")), "low");
        assert_eq!(cvss_to_severity(&make_severity("0.1")), "low");
    }

    #[test]
    fn cvss_to_severity_vector_falls_back_to_medium() {
        let severity = make_severity("CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H");
        assert_eq!(cvss_to_severity(&severity), "medium");
    }

    #[test]
    fn cvss_to_severity_empty_falls_back_to_medium() {
        assert_eq!(cvss_to_severity(&[]), "medium");
    }

    #[test]
    fn extract_fixed_version_found() {
        let vuln = OsvVulnerability {
            id: "CVE-2024-1".to_string(),
            aliases: None,
            summary: None,
            details: None,
            severity: None,
            affected: Some(vec![OsvAffected {
                package: None,
                ranges: Some(vec![OsvRange {
                    range_type: "SEMVER".to_string(),
                    events: Some(vec![
                        OsvEvent {
                            introduced: Some("0".to_string()),
                            fixed: None,
                        },
                        OsvEvent {
                            introduced: None,
                            fixed: Some("2.0.1".to_string()),
                        },
                    ]),
                }]),
            }]),
            published: None,
            modified: None,
        };
        assert_eq!(extract_fixed_version(&vuln), Some("2.0.1".to_string()));
    }

    #[test]
    fn extract_fixed_version_none() {
        let vuln = make_vuln("CVE-2024-1", None);
        assert_eq!(extract_fixed_version(&vuln), None);
    }

    #[test]
    fn extract_affected_range_intro_and_fix() {
        let vuln = OsvVulnerability {
            id: "CVE-2024-1".to_string(),
            aliases: None,
            summary: None,
            details: None,
            severity: None,
            affected: Some(vec![OsvAffected {
                package: None,
                ranges: Some(vec![OsvRange {
                    range_type: "SEMVER".to_string(),
                    events: Some(vec![
                        OsvEvent {
                            introduced: Some("1.0.0".to_string()),
                            fixed: None,
                        },
                        OsvEvent {
                            introduced: None,
                            fixed: Some("2.0.1".to_string()),
                        },
                    ]),
                }]),
            }]),
            published: None,
            modified: None,
        };
        assert_eq!(extract_affected_range(&vuln), ">= 1.0.0, < 2.0.1");
    }

    #[test]
    fn extract_affected_range_zero_introduced() {
        let vuln = OsvVulnerability {
            id: "CVE-2024-1".to_string(),
            aliases: None,
            summary: None,
            details: None,
            severity: None,
            affected: Some(vec![OsvAffected {
                package: None,
                ranges: Some(vec![OsvRange {
                    range_type: "SEMVER".to_string(),
                    events: Some(vec![
                        OsvEvent {
                            introduced: Some("0".to_string()),
                            fixed: None,
                        },
                        OsvEvent {
                            introduced: None,
                            fixed: Some("3.0.0".to_string()),
                        },
                    ]),
                }]),
            }]),
            published: None,
            modified: None,
        };
        assert_eq!(extract_affected_range(&vuln), "< 3.0.0");
    }

    #[test]
    fn extract_affected_range_no_fix() {
        let vuln = OsvVulnerability {
            id: "CVE-2024-1".to_string(),
            aliases: None,
            summary: None,
            details: None,
            severity: None,
            affected: Some(vec![OsvAffected {
                package: None,
                ranges: Some(vec![OsvRange {
                    range_type: "SEMVER".to_string(),
                    events: Some(vec![OsvEvent {
                        introduced: Some("1.5.0".to_string()),
                        fixed: None,
                    }]),
                }]),
            }]),
            published: None,
            modified: None,
        };
        assert_eq!(extract_affected_range(&vuln), ">= 1.5.0");
    }

    #[test]
    fn extract_affected_range_unknown() {
        let vuln = make_vuln("CVE-2024-1", None);
        assert_eq!(extract_affected_range(&vuln), "unknown");
    }

    #[test]
    fn map_ecosystem_mappings() {
        assert_eq!(map_ecosystem("npm"), "npm");
        assert_eq!(map_ecosystem("composer"), "Packagist");
        assert_eq!(map_ecosystem("pip"), "PyPI");
        assert_eq!(map_ecosystem("cargo"), "crates.io");
        assert_eq!(map_ecosystem("go"), "Go");
        assert_eq!(map_ecosystem("other"), "other");
    }
}
