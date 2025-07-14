use crate::machines::{
    buffer1::BufferV1, extruder1::ExtruderV2, laser::LaserMachine, mock::MockMachine, mock2::Mock2Machine, winder2::Winder2, MACHINE_BUFFER_V1, MACHINE_EXTRUDER_V1, MACHINE_LASER_V1, MACHINE_MOCK, MACHINE_MOCK2, MACHINE_WINDER_V2, VENDOR_QITECH
};
use control_core::machines::{identification::MachineIdentification, registry::MachineRegistry};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref MACHINE_REGISTRY: MachineRegistry = {
        let mut mc = MachineRegistry::new();
        mc.register::<Winder2>(Winder2::MACHINE_IDENTIFICATION);
        mc.register::<LaserMachine>(MachineIdentification {
            vendor: VENDOR_QITECH,
            machine: MACHINE_LASER_V1,
        });
        mc.register::<ExtruderV2>(MachineIdentification {
            vendor: VENDOR_QITECH,
            machine: MACHINE_EXTRUDER_V1,
        });
        mc.register::<MockMachine>(MockMachine::MACHINE_IDENTIFICATION);
        mc.register::<BufferV1>(MockMachine::MACHINE_IDENTIFICATION);
        mc.register::<Mock2Machine>(Mock2Machine::MACHINE_IDENTIFICATION);
        mc
    };
}
