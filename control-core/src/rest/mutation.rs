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

/// Event field specification for querying specific fields from an event type
#[derive(Debug, serde::Deserialize, Clone)]
pub struct EventFields {
    /// Fields to retrieve from LiveValues event
    /// None = all fields, Some([]) = no fields, Some(["field1", "field2"]) = specific fields
    #[serde(rename = "LiveValues")]
    pub live_values: Option<Vec<String>>,
    /// Fields to retrieve from State event
    /// None = all fields, Some([]) = no fields, Some(["field1", "field2"]) = specific fields
    #[serde(rename = "State")]
    pub state: Option<Vec<String>>,
}

/// Request body for querying machine events (read-only)
#[derive(Debug, serde::Deserialize)]
pub struct MachineEventBody {
    pub machine_identification_unique: MachineIdentificationUnique,
    /// Event types and their requested fields.
    /// - None: returns all available events with all fields
    /// - Some(EventFields { live_values: Some([...]), state: None }): returns LiveValues with specific fields, no State
    /// - Some(EventFields { live_values: None, state: None }): returns both events with all fields
    pub events: Option<EventFields>,
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
