use crate::registry::MachineRegistry;

pub mod ff01;

pub fn register_machines(registry: &mut MachineRegistry)
{
    #[cfg(not(feature = "mock-machine"))]
    registry.register::<ff01::FF01>(ff01::FF01::MACHINE_IDENTIFICATION);
}