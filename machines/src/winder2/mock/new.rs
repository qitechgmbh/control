use super::{
    ModeState, PullerState, SpoolAutomaticActionState, SpoolSpeedControllerState, TensionArmState,
    TraverseState, Winder2,
};
use crate::winder2::api::Winder2Namespace;
use crate::{MachineCrossConnectionState, MachineNewHardware, MachineNewParams, MachineNewTrait};

impl MachineNewTrait for Winder2 {
    fn new(params: &MachineNewParams<'_, '_, '_, '_, '_, '_, '_>) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        // Mock machine can work with either Serial or Ethercat hardware
        // For the mock machine, we don't need to actually use the hardware
        // We just validate that we have the expected hardware type
        match params.hardware {
            MachineNewHardware::Serial(_) => {
                // For serial mode, we could potentially use the serial device if needed
                // but for a mock machine, we'll just note it and proceed
            }
            MachineNewHardware::Ethercat(_) => {
                // For ethercat mode, we could potentially use the ethercat devices
                // but for a mock machine, we'll just note it and proceed
            }
        }

        let now = std::time::Instant::now();

        let mut extruder_mock_machine = Self {
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
            connected_machine_state: MachineCrossConnectionState::default(),
        };

        extruder_mock_machine.emit_state();

        Ok(extruder_mock_machine)
    }
}
