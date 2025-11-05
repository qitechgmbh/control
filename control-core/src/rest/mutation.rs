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

/// Request body for querying machine events (read-only)
#[derive(Debug, serde::Deserialize)]
pub struct MachineEventBody {
    pub machine_identification_unique: MachineIdentificationUnique,
    /// Event types to retrieve as an array of event names.
    /// - None: returns all available events (LiveValues and State)
    /// - Some(["LiveValues"]): returns only LiveValues with all fields
    /// - Some(["State"]): returns only State with all fields
    /// - Some(["LiveValues", "State"]): returns both events with all fields
    /// - Some([]): returns no events (empty data object)
    ///
    /// Example JSON: { "events": ["LiveValues"] } - returns only LiveValues
    pub events: Option<Vec<String>>,
}

/// Response for read-only machine event queries
#[derive(Debug, Serialize, Clone)]
pub struct MachineEventResponse {
    pub success: bool,
    pub error: Option<String>,
    pub data: Option<serde_json::Value>,
}

impl MachineEventResponse {
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
