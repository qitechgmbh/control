use crate::model::{Affect, Analysis, Component, Rating, Source, Vulnerability};

#[derive(Debug, serde::Deserialize)]
struct TrivyReport {
    #[serde(rename = "Results", default)]
    results: Vec<TrivyResult>,
}

#[derive(Debug, serde::Deserialize)]
struct TrivyResult {
    #[serde(rename = "Vulnerabilities", default)]
    vulnerabilities: Vec<TrivyVulnerability>,
}

#[derive(Debug, serde::Deserialize)]
struct TrivyVulnerability {
    #[serde(rename = "VulnerabilityID")]
    vulnerability_id: String,
    #[serde(rename = "PkgName")]
    _pkg_name: String,
    #[serde(rename = "InstalledVersion")]
    _installed_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "Severity")]
    severity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "Title")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "PrimaryURL")]
    primary_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "PkgIdentifier")]
    pkg_identifier: Option<PkgIdentifier>,
}

#[derive(Debug, serde::Deserialize)]
struct PkgIdentifier {
    #[serde(rename = "PURL")]
    purl: String,
}

pub fn read_trivy_report(path: &str) -> anyhow::Result<Vec<Vulnerability>> {
    let content = std::fs::read_to_string(path)?;
    let report: TrivyReport = serde_json::from_str(&content)?;

    let mut seen = std::collections::HashSet::new();
    let vulnerabilities: Vec<Vulnerability> = report
        .results
        .iter()
        .flat_map(|r| &r.vulnerabilities)
        .filter_map(|tv| {
            if !seen.insert(&tv.vulnerability_id) {
                return None;
            }
            let severity = map_severity(&tv.severity);
            Some(Vulnerability {
                id: tv.vulnerability_id.clone(),
                source: Some(Source {
                    name: Some("NVD".into()),
                    url: tv.primary_url.clone(),
                }),
                ratings: vec![Rating {
                    source: None,
                    score: None,
                    severity,
                    method: Some("CVSSv31".into()),
                    vector: None,
                }],
                analysis: Some(Analysis {
                    state: String::new(),
                    justification: None,
                    response: Vec::new(),
                    detail: tv.title.clone(),
                    first_issued: None,
                    last_updated: None,
                }),
                affects: tv
                    .pkg_identifier
                    .as_ref()
                    .map(|pi| {
                        vec![Affect {
                            ref_: pi.purl.clone(),
                            versions: Vec::new(),
                        }]
                    })
                    .unwrap_or_default(),
            })
        })
        .collect();

    Ok(vulnerabilities)
}

pub fn match_to_components(
    vulns: &[Vulnerability],
    components: &[Component],
) -> anyhow::Result<Vec<Vulnerability>> {
    let component_purls: std::collections::HashSet<String> = components
        .iter()
        .filter_map(|c| c.purl.clone())
        .collect();
    let component_refs: std::collections::HashMap<String, String> = components
        .iter()
        .filter_map(|c| Some((c.purl.clone()?, c.bom_ref.clone()?)))
        .collect();

    let mut result = Vec::new();
    for vuln in vulns {
        let mut v = vuln.clone();
        let mut mapped = false;
        if !vuln.affects.is_empty() {
            for affect in &vuln.affects {
                if component_purls.contains(&affect.ref_) {
                    if let Some(bom_ref) = component_refs.get(&affect.ref_) {
                        v.affects = vec![Affect {
                            ref_: bom_ref.clone(),
                            versions: Vec::new(),
                        }];
                        mapped = true;
                        break;
                    }
                }
            }
        }
        if !mapped {
            v.affects = Vec::new();
        }
        result.push(v);
    }

    Ok(result)
}

fn map_severity(trivy_severity: &Option<String>) -> Option<String> {
    match trivy_severity.as_deref() {
        Some("CRITICAL") => Some("critical".into()),
        Some("HIGH") => Some("high".into()),
        Some("MEDIUM") => Some("medium".into()),
        Some("LOW") => Some("low".into()),
        _ => None,
    }
}
