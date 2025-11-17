use crate::{
    MachineNewParams, MachineNewTrait,
    winder2::api::{
        ModeState, PullerState, SpoolAutomaticActionState, SpoolSpeedControllerState,
        TensionArmState, TraverseState, Winder2Namespace,
    },
};

use super::Winder2;

impl MachineNewTrait for Winder2 {
    fn new(params: &MachineNewParams<'_, '_, '_, '_, '_, '_, '_>) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        // Mock machine can work with either Serial or Ethercat hardware
        // For the mock machine, we don't need to actually use the hardware
        // We just validate that we have the expected hardware type
        match params.hardware {
            crate::MachineNewHardware::Serial(_) => {
                // For serial mode, we could potentially use the serial device if needed
                // but for a mock machine, we'll just note it and proceed
            }
            crate::MachineNewHardware::Ethercat(_) => {
                // For ethercat mode, we could potentially use the ethercat devices
                // but for a mock machine, we'll just note it and proceed
            }
        }

        let now = std::time::Instant::now();

        let (sender, receiver) = smol::channel::unbounded();
        let mut winder_mock_machine = Self {
            main_sender: params.main_thread_channel.clone(),
            max_connected_machines: 2,
            api_receiver: receiver,
            api_sender: sender,
            machine_identification_unique: params.get_machine_identification_unique(),
            namespace: Winder2Namespace {
                namespace: params.namespace.clone(),
            },
            last_measurement_emit: now,
            is_default_state: true,
            traverse_state: TraverseState::default(),
            puller_state: PullerState::default(),
            spool_automatic_action_state: SpoolAutomaticActionState::default(),
            mode_state: ModeState::default(),
            tension_arm_state: TensionArmState::default(),
            spool_speed_controller_state: SpoolSpeedControllerState::default(),
            connected_machines: vec![],
        };

        winder_mock_machine.emit_state();

        Ok(winder_mock_machine)
    }
}
