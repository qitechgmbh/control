use crate::model::{Bom, Component, Vulnerability};

pub fn read_sbom(path: &str) -> anyhow::Result<Bom> {
    let content = std::fs::read_to_string(path)?;
    let bom: Bom = serde_json::from_str(&content)?;
    Ok(bom)
}

pub fn extract_root_component(bom: &Bom) -> anyhow::Result<Component> {
    bom.metadata
        .as_ref()
        .and_then(|m| m.component.clone())
        .ok_or_else(|| anyhow::anyhow!("SBOM has no metadata.component"))
}

pub fn stub_vulnerabilities(bom: &Bom) -> Vec<Vulnerability> {
    bom.vulnerabilities
        .iter()
        .map(|v| {
            let mut vuln = v.clone();
            vuln.analysis = Some(crate::model::Analysis {
                state: String::new(),
                justification: None,
                response: Vec::new(),
                detail: None,
                first_issued: None,
                last_updated: None,
            });
            vuln
        })
        .collect()
}
