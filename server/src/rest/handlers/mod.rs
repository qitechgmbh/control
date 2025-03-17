use axum::body::Body;
use serde::Serialize;

pub mod machine_mutation;
pub mod write_machine_device_identification;

#[derive(Debug, Serialize, Clone)]
pub struct MutationResponse {
    pub success: bool,
    pub error: Option<String>,
}

impl MutationResponse {
    pub fn success() -> Self {
        Self {
            success: true,
            error: None,
        }
    }
    pub fn error(error: String) -> Self {
        Self {
            success: false,
            error: Some(error),
        }
    }
}

impl From<MutationResponse> for Body {
    fn from(mutation_response: MutationResponse) -> Self {
        let body = serde_json::to_string(&mutation_response).unwrap();
        Body::from(body)
    }
}
