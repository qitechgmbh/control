use qitech_framework::MachineIdentification;
use qitech_framework::MachineInstanceIdentification;
use qitech_framework::RuntimeRequestKind;

use crate::api::types::MachineInstance;

pub mod aquapath_v1;
mod extruder_v1;
pub mod laser_v1;
mod mixer_v1;

pub fn get(ident: MachineIdentification) -> Option<MachineLegacyDataAdapter> {
    const IDENT_LASER: MachineIdentification = MachineIdentification {
        vendor_id: 1,
        machine_id: 6,
    };

    const IDENT_AQUAPATH: MachineIdentification = MachineIdentification {
        vendor_id: 1,
        machine_id: 9,
    };

    // The frontend calls these "extruder2" and "extruder3"; they share one schema and one adapter.
    const IDENT_EXTRUDER_V1: MachineIdentification = MachineIdentification {
        vendor_id: 1,
        machine_id: 4,
    };

    const IDENT_EXTRUDER_V2: MachineIdentification = MachineIdentification {
        vendor_id: 1,
        machine_id: 22,
    };

    const IDENT_MIXER: MachineIdentification = MachineIdentification {
        vendor_id: 1,
        machine_id: 18,
    };

    match ident {
        IDENT_LASER => Some(laser_v1::ADAPTER),
        IDENT_AQUAPATH => Some(aquapath_v1::ADAPTER),
        IDENT_EXTRUDER_V1 | IDENT_EXTRUDER_V2 => Some(extruder_v1::ADAPTER),
        IDENT_MIXER => Some(mixer_v1::ADAPTER),
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
