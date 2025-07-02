use crate::machines::{
    buffer1::Buffer1, extruder1::ExtruderV2, laser::LaserMachine, mock::MockMachine, winder2::Winder2, MACHINE_EXTRUDER_V1, MACHINE_LASER_V1, MACHINE_MOCK, MACHINE_WINDER_V1, VENDOR_QITECH
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
        mc.register::<LaserMachine>(MachineIdentification {
            vendor: VENDOR_QITECH,
            machine: MACHINE_LASER_V1,
        });
        mc.register::<ExtruderV2>(MachineIdentification {
            vendor: VENDOR_QITECH,
            machine: MACHINE_EXTRUDER_V1,
        });
        mc.register::<MockMachine>(MachineIdentification {
            vendor: VENDOR_QITECH,
            machine: MACHINE_MOCK,
        });
        mc.register::<Buffer1>(MachineIdentification { vendor: (VENDOR_QITECH), machine: (8) });
        mc
    };
}
