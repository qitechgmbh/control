use std::time::Instant;

pub mod act;
pub mod api;
pub mod mock_emit;
pub mod new;

use super::api::{
    ModeState, PullerState, SpoolAutomaticActionState, SpoolSpeedControllerState, TensionArmState,
    TraverseState, Winder2Namespace,
};
use crate::{
    AsyncThreadMessage, Machine, MachineConnection, MachineMessage,
    machine_identification::MachineIdentificationUnique,
};
use smol::channel::{Receiver, Sender};

#[derive(Debug)]
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

    /// Receive from Api or MainThread
    api_receiver: Receiver<MachineMessage>,
    api_sender: Sender<MachineMessage>,

    /// Communicate with main thread
    main_sender: Option<Sender<AsyncThreadMessage>>,

    /// All currently "connected" Machines
    connected_machines: Vec<MachineConnection>,
    /// Defaults to limit of 2
    max_connected_machines: usize,
}

impl std::fmt::Display for Winder2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Winder2")
    }
}

impl Machine for Winder2 {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}
