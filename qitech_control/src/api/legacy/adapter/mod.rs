use qitech_framework::MachineIdentification;
use qitech_framework::MachineInstanceIdentification;
use qitech_framework::RuntimeRequestKind;

use crate::api::types::MachineInstance;

mod laser_v1;

pub fn get(ident: MachineIdentification) -> Option<MachineLegacyDataAdapter> {
    const IDENT_LASER: MachineIdentification = MachineIdentification {
        vendor_id: 1,
        machine_id: 6,
    };

    match ident {
        IDENT_LASER => Some(laser_v1::ADAPTER),
        _ => None,
    }
}

#[derive(Clone)]
pub struct MachineLegacyDataAdapter {
    pub convert_request: fn(
        MachineInstanceIdentification,
        serde_json::Value,
    ) -> Result<RuntimeRequestKind, serde_json::Error>,

    pub init_state_event: fn(&MachineInstance, is_default_state: bool) -> Option<serde_json::Value>,
    pub init_measurements_event: fn(&MachineInstance) -> Option<serde_json::Value>,
}
