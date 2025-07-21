use crate::machines::{
    buffer1::BufferV1, extruder1::ExtruderV2, laser::LaserMachine, mock::MockMachine,
    mock2::Mock2Machine, winder2::Winder2,
};
use control_core::machines::registry::MachineRegistry;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref MACHINE_REGISTRY: MachineRegistry = {
        let mut mc = MachineRegistry::new();
        mc.register::<Winder2>(Winder2::MACHINE_IDENTIFICATION);
        mc.register::<LaserMachine>(LaserMachine::MACHINE_IDENTIFICATION);
        mc.register::<ExtruderV2>(ExtruderV2::MACHINE_IDENTIFICATION);
        mc.register::<MockMachine>(MockMachine::MACHINE_IDENTIFICATION);
        mc.register::<Mock2Machine>(Mock2Machine::MACHINE_IDENTIFICATION);
        mc.register::<BufferV1>(BufferV1::MACHINE_IDENTIFICATION);
        mc
    };
}
