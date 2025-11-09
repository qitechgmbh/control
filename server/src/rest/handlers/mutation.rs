use std::fmt::Debug;

use serde::Serialize;

use machines::machine_identification::MachineIdentificationUnique;

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
