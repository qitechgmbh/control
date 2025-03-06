use serde::Serialize;

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
