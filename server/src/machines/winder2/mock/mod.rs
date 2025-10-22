use std::time::Instant;

use control_core::machines::{
    connection::MachineCrossConnectionState, identification::MachineIdentificationUnique,
};
use control_core_derive::Machine;
pub mod act;
pub mod api;
pub mod mock_emit;
pub mod new;
use crate::machines::winder2::api::Winder2Namespace;

use super::api::{
    ModeState, PullerState, SpoolAutomaticActionState, SpoolSpeedControllerState, TensionArmState,
    TraverseState,
};

#[derive(Debug, Machine)]
pub struct Winder2 {
    machine_identification_unique: MachineIdentificationUnique,
    namespace: Winder2Namespace,
    last_measurement_emit: Instant,
    pub is_default_state: bool,
    /// traverse state
    pub traverse_state: TraverseState,
    /// puller state
    pub puller_state: PullerState,
    /// spool automatic action state and progress
    pub spool_automatic_action_state: SpoolAutomaticActionState,
    /// mode state
    pub mode_state: ModeState,
    /// tension arm state
    pub tension_arm_state: TensionArmState,
    /// spool speed controller state
    pub spool_speed_controller_state: SpoolSpeedControllerState,
    /// connected machine state
    pub connected_machine_state: MachineCrossConnectionState,
}

impl std::fmt::Display for Winder2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Winder2")
    }
}
