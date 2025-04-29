use crate::machines::{
    extruder1::ExtruderV2, winder2::Winder2, MACHINE_EXTRUDER_V1, MACHINE_WINDER_V1, VENDOR_QITECH,
};
use control_core::{identification::MachineIdentification, machines::registry::MachineRegistry};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref MACHINE_REGISTRY: MachineRegistry = {
        let mut mc = MachineRegistry::new();
        mc.register::<Winder2>(MachineIdentification::new(VENDOR_QITECH, MACHINE_WINDER_V1));
        mc.register::<ExtruderV2>(MachineIdentification::new(
            VENDOR_QITECH,
            MACHINE_EXTRUDER_V1,
        ));
        mc
    };
}
