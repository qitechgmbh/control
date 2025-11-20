use super::Winder2;
use crate::{MachineApi, winder2::api::Mutation};
use serde_json::Value;
use std::time::Instant;

impl MachineApi for Winder2 {
    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(request_body)?;
        match mutation {
            Mutation::EnableTraverseLaserpointer(enable) => self.set_laser(enable),
            Mutation::SetMode(mode) => self.set_mode(&mode.into()),
            Mutation::SetTraverseLimitOuter(limit) => self.traverse_set_limit_outer(limit),
            Mutation::SetTraverseLimitInner(limit) => self.traverse_set_limit_inner(limit),
            Mutation::SetTraverseStepSize(size) => self.traverse_set_step_size(size),
            Mutation::SetTraversePadding(padding) => self.traverse_set_padding(padding),
            Mutation::GotoTraverseLimitOuter => self.traverse_goto_limit_outer(),
            Mutation::GotoTraverseLimitInner => self.traverse_goto_limit_inner(),
            Mutation::GotoTraverseHome => self.traverse_goto_home(),
            Mutation::SetPullerRegulationMode(regulation) => self.puller_set_regulation(regulation),
            Mutation::SetPullerTargetSpeed(value) => self.puller_set_target_speed(value),
            Mutation::SetPullerTargetDiameter(_) => todo!(),
            Mutation::SetPullerForward(value) => self.puller_set_forward(value),
            Mutation::SetPullerGearRatio(gear_ratio) => self.puller_set_gear_ratio(gear_ratio),
            Mutation::SetSpoolRegulationMode(mode) => self.spool_set_regulation_mode(mode),
            Mutation::SetSpoolMinMaxMinSpeed(speed) => self.spool_set_minmax_min_speed(speed),
            Mutation::SetSpoolMinMaxMaxSpeed(speed) => self.spool_set_minmax_max_speed(speed),
            Mutation::SetSpoolForward(value) => self.spool_set_forward(value),
            Mutation::SetSpoolAdaptiveTensionTarget(value) => {
                self.spool_set_adaptive_tension_target(value)
            }
            Mutation::SetSpoolAdaptiveRadiusLearningRate(value) => {
                self.spool_set_adaptive_radius_learning_rate(value)
            }
            Mutation::SetSpoolAdaptiveMaxSpeedMultiplier(value) => {
                self.spool_set_adaptive_max_speed_multiplier(value)
            }
            Mutation::SetSpoolAdaptiveAccelerationFactor(value) => {
                self.spool_set_adaptive_acceleration_factor(value)
            }
            Mutation::SetSpoolAdaptiveDeaccelerationUrgencyMultiplier(value) => {
                self.spool_set_adaptive_deacceleration_urgency_multiplier(value)
            }
            Mutation::SetSpoolAutomaticRequiredMeters(meters) => {
                self.set_spool_automatic_required_meters(meters)
            }
            Mutation::SetSpoolAutomaticAction(mode) => self.set_spool_automatic_mode(mode),
            Mutation::ResetSpoolProgress => self.stop_or_pull_spool_reset(Instant::now()),
            Mutation::ZeroTensionArmAngle => self.tension_arm_zero(),
            Mutation::SetConnectedMachine(machine_identification_unique) => {
                self.set_connected_buffer(machine_identification_unique)
            }
            Mutation::DisconnectMachine(machine_identification_unique) => {
                self.disconnect_buffer(machine_identification_unique)
            }
        }
        Ok(())
    }

    fn api_event_namespace(
        &mut self,
    ) -> std::option::Option<control_core::socketio::namespace::Namespace> {
        self.namespace.namespace.clone()
    }

    fn api_get_sender(&self) -> smol::channel::Sender<crate::MachineMessage> {
        self.api_sender.clone()
    }
}
