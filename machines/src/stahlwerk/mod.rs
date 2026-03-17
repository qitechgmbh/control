use crate::registry::MachineRegistry;

mod ff01;
use ff01::FF01;

#[cfg(feature = "mock-machine")]
use ff01::FF01;

pub mod machine_registry
{
    // 0x01 = Stahlwerk, 0x00 = First machine
    pub const FF01: u16 = 0x01_00;
}

pub fn register_machines(registry: &mut MachineRegistry)
{
    #[cfg(not(feature = "mock-machine"))]
    registry.register::<FF01>(FF01::MACHINE_IDENTIFICATION);
}