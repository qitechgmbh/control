use api::ExtruderV2Namespace;
use chrono::DateTime;
use control_core::{
    actors::mitsubishi_inverter_rs485::MitsubishiInverterRS485Actor, machines::Machine,
};
pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct ExtruderV2 {
    inverter: MitsubishiInverterRS485Actor,
    namespace: ExtruderV2Namespace,
    last_response_emit: DateTime<chrono::Utc>,
}

impl std::fmt::Display for ExtruderV2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ExtruderV1")
    }
}

impl Machine for ExtruderV2 {}

impl ExtruderV2 {}
