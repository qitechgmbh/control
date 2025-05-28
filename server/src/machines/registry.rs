use crate::machines::{
    MACHINE_DRE, MACHINE_EXTRUDER_V1, MACHINE_MOCK, MACHINE_WINDER_V1, VENDOR_QITECH, 
    dre::DreMachine, extruder1::ExtruderV2, mock::MockMachine, winder2::Winder2,
};
use control_core::machines::{identification::MachineIdentification, registry::MachineRegistry};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref MACHINE_REGISTRY: MachineRegistry = {
        let mut mc = MachineRegistry::new();
        mc.register::<Winder2>(MachineIdentification {
            vendor: VENDOR_QITECH,
            machine: MACHINE_WINDER_V1,
        });
        mc.register::<DreMachine>(MachineIdentification {
            vendor: VENDOR_QITECH,
            machine: MACHINE_DRE,
        });
        mc.register::<ExtruderV2>(MachineIdentification {
            vendor: VENDOR_QITECH,
            machine: MACHINE_EXTRUDER_V1,
        });
        mc.register::<MockMachine>(MachineIdentification {
            vendor: VENDOR_QITECH,
            machine: MACHINE_MOCK,
        });
        mc
    };
}
