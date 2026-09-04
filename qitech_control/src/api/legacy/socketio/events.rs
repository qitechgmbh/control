use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use serde::Deserialize;
use serde::Serialize;

use crate::api::legacy;

#[derive(Debug, Clone, Serialize)]
pub struct SocketIOEvent {
    pub name: &'static str,
    pub data: serde_json::Value,
    pub ts: u64,
}

impl SocketIOEvent {
    pub fn new(name: &'static str, data: impl Serialize) -> Self {
        Self {
            name,
            data: serde_json::to_value(data).unwrap(),
            ts: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }
}

// --- ethercat state event ---
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum EthercatDevicesEvent {
    State(String),
    Done(EthercatSetupDone),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EthercatSetupDone {
    pub devices: Vec<legacy::EtherCATDeviceMetadata>,
}

// --- machines event ---
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MachinesEvent {
    pub machines: Vec<MachineObj>,
}

// --- modbus rtu devices event ---
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModbusDevicesEvent {
    pub devices: Vec<legacy::ModbusDeviceMetadata>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MachineObj {
    pub machine_identification_unique: legacy::MachineIdentificationUnique,
    pub error: Option<String>,
}
