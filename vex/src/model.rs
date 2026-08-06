use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Bom {
    #[serde(rename = "bomFormat")]
    pub bom_format: String,
    #[serde(rename = "specVersion")]
    pub spec_version: String,
    #[serde(rename = "serialNumber", skip_serializing_if = "Option::is_none")]
    pub serial_number: Option<String>,
    pub version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub components: Vec<Component>,
    #[serde(default)]
    pub vulnerabilities: Vec<Vulnerability>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component: Option<Component>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Component {
    #[serde(rename = "bom-ref", skip_serializing_if = "Option::is_none")]
    pub bom_ref: Option<String>,
    #[serde(rename = "type")]
    pub component_type: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purl: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manufacturer: Option<OrganizationalEntity>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrganizationalEntity {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Vulnerability {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ratings: Vec<Rating>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analysis: Option<Analysis>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affects: Vec<Affect>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Source {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Rating {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Analysis {
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub justification: Option<String>,
    #[serde(default)]
    pub response: Vec<String>,
    #[serde(default)]
    pub detail: Option<String>,
    #[serde(rename = "firstIssued", default)]
    pub first_issued: Option<String>,
    #[serde(rename = "lastUpdated", default)]
    pub last_updated: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Affect {
    #[serde(rename = "ref")]
    pub ref_: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub versions: Vec<AffectVersion>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AffectVersion {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

impl Bom {
    pub fn new_vex(
        component: Component,
        vulnerabilities: Vec<Vulnerability>,
    ) -> Self {
        Self {
            bom_format: "CycloneDX".into(),
            spec_version: "1.6".into(),
            serial_number: Some(format!("urn:uuid:{}", uuid::Uuid::new_v4())),
            version: 1,
            metadata: Some(Metadata {
                timestamp: Some(chrono::Utc::now().to_rfc3339()),
                component: Some(component),
            }),
            components: Vec::new(),
            vulnerabilities,
        }
    }
}
