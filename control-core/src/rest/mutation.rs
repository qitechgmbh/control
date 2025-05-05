use std::fmt::Debug;

use serde::Serialize;

use crate::machines::identification::MachineIdentificationUnique;

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

#[derive(Debug, serde::Deserialize)]
pub struct MachineMutationBody<T>
where
    T: Debug,
{
    pub machine_identification_unique: MachineIdentificationUnique,
    pub data: T,
}
