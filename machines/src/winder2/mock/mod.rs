use super::api::{
    ModeState, PullerState, SpoolAutomaticActionState, SpoolSpeedControllerState, TensionArmState,
    TraverseState,
};
use crate::winder2::api::Winder2Namespace;
use crate::{MachineCrossConnectionState, machine_identification::MachineIdentificationUnique};
use std::time::Instant;

pub mod act;
pub mod api;
pub mod mock_emit;
pub mod new;

#[derive(Debug)]
pub struct Winder2 {
    pub machine_identification_unique: MachineIdentificationUnique,
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
