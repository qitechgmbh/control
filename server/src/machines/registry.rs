#[cfg(feature = "mock-machine")]
use crate::machines::{extruder1::mock::ExtruderV2, mock::MockMachine, winder2::mock::Winder2};

#[cfg(not(feature = "mock-machine"))]
use crate::machines::{
    aquapath1::AquaPathV1, buffer1::BufferV1, extruder1::ExtruderV2, laser::LaserMachine,
    winder2::Winder2,
};

use control_core::machines::registry::MachineRegistry;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref MACHINE_REGISTRY: MachineRegistry = {
        let mut mc = MachineRegistry::new();
        mc.register::<Winder2>(Winder2::MACHINE_IDENTIFICATION);
        mc.register::<ExtruderV2>(ExtruderV2::MACHINE_IDENTIFICATION);

        #[cfg(feature = "mock-machine")]
        mc.register::<MockMachine>(MockMachine::MACHINE_IDENTIFICATION);

        #[cfg(not(feature = "mock-machine"))]
        mc.register::<LaserMachine>(LaserMachine::MACHINE_IDENTIFICATION);

        #[cfg(not(feature = "mock-machine"))]
        mc.register::<BufferV1>(BufferV1::MACHINE_IDENTIFICATION);

        #[cfg(not(feature = "mock-machine"))]
        mc.register::<AquaPathV1>(AquaPathV1::MACHINE_IDENTIFICATION);
        mc
    };
}
