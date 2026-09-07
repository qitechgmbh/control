use qitech_framework::MachineIdentification;
use qitech_framework::MachineInstanceIdentification;
use qitech_framework::RuntimeRequestKind;
use qitech_framework::machine::MachineDescriptor;

use crate::api::types::MachineInstance;
use crate::machines::ExtruderV1;
use crate::machines::ExtruderV2;
use crate::machines::LaserV1;
use crate::machines::WinderV1_7031_Spool;
use crate::machines::WinderV1_Regular;
use crate::machines::aquapath::AquaPathV1;

pub mod aquapath_v1;
mod extruder_v1;
pub mod laser_v1;
pub mod winder_v1;

pub fn get(ident: MachineIdentification) -> Option<MachineLegacyDataAdapter> {
    match ident {
        LaserV1::IDENTIFICATION => Some(laser_v1::ADAPTER),
        AquaPathV1::IDENTIFICATION => Some(aquapath_v1::ADAPTER),
        ExtruderV1::IDENTIFICATION | ExtruderV2::IDENTIFICATION => Some(extruder_v1::ADAPTER),
        WinderV1_Regular::IDENTIFICATION | WinderV1_7031_Spool::IDENTIFICATION => {
            Some(winder_v1::ADAPTER)
        }
        _ => None,
    }
}

#[derive(Clone)]
pub struct MachineLegacyDataAdapter {
    pub convert_request: fn(
        MachineInstanceIdentification,
        serde_json::Value,
    ) -> Result<Vec<RuntimeRequestKind>, serde_json::Error>,

    pub init_state_event: fn(&MachineInstance, is_default_state: bool) -> Option<serde_json::Value>,
    pub init_measurements_event: fn(&MachineInstance) -> Option<serde_json::Value>,
}
