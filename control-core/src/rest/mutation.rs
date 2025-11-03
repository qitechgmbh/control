use std::fmt::Debug;

use serde::Serialize;

use crate::machines::identification::MachineIdentificationUnique;

#[derive(Debug, Serialize, Clone)]
pub struct MutationResponse {
    pub success: bool,
    pub error: Option<String>,
}

impl MutationResponse {
    pub const fn success() -> Self {
        Self {
            success: true,
            error: None,
        }
    }
    pub const fn error(error: String) -> Self {
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

#[derive(Debug, serde::Deserialize)]
pub struct MachineVideoStreamListBody {
    pub machine_identification_unique: MachineIdentificationUnique,
}

#[derive(Debug, serde::Deserialize)]
pub struct MachineVideoStreamBody {
    pub machine_identification_unique: MachineIdentificationUnique,
    pub stream_id: String,
}

#[derive(Debug, Serialize)]
pub struct VideoStreamListResponse {
    pub streams: Vec<String>,
}

/// Request body for querying machine state (read-only)
#[derive(Debug, serde::Deserialize)]
pub struct MachineQueryBody {
    pub machine_identification_unique: MachineIdentificationUnique,
    /// Fields to include in the response. Must not be empty.
    /// Use dot notation for nested fields (e.g., "live_values.temperature")
    pub fields: Vec<String>,
}

/// Response for read-only machine queries
#[derive(Debug, Serialize, Clone)]
pub struct MachineQueryResponse {
    pub success: bool,
    pub error: Option<String>,
    pub data: Option<serde_json::Value>,
}

impl MachineQueryResponse {
    pub fn success(data: serde_json::Value) -> Self {
        Self {
            success: true,
            error: None,
            data: Some(data),
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            success: false,
            error: Some(error),
            data: None,
        }
    }
}
