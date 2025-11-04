use std::{sync::Arc, time::Instant};

use control_core::{machines::api::MachineApi, socketio::namespace::Namespace};
use serde_json::Value;
use smol::lock::Mutex;

use crate::machines::winder2::api::Mutation;

use super::Winder2;

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

    fn api_event_namespace(&mut self) -> Arc<Mutex<Namespace>> {
        self.namespace.namespace.clone()
    }

    fn api_event(&mut self, events: Option<&control_core::rest::mutation::EventFields>) -> Result<Value, anyhow::Error> {
        let live_values = super::super::api::LiveValuesEvent {
            traverse_position: Some(0.0),
            puller_speed: 0.0,
            spool_rpm: 0.0,
            tension_arm_angle: 0.0,
            spool_progress: 0.0,
        };

        let state = self.build_state_event();

        // Build response with requested events and fields
        let mut result = serde_json::Map::new();

        // Determine which events to include
        let (include_live_values, live_values_fields) = match events {
            None => (true, None),
            Some(ef) => (ef.live_values.is_some(), ef.live_values.as_ref()),
        };

        let (include_state, state_fields) = match events {
            None => (true, None),
            Some(ef) => (ef.state.is_some(), ef.state.as_ref()),
        };

        // Add LiveValues if requested
        if include_live_values {
            let live_values_json = serde_json::to_value(live_values)?;
            let filtered = crate::rest::event_filter::filter_event_fields(live_values_json, live_values_fields)?;
            if !filtered.is_null() {
                result.insert("LiveValues".to_string(), filtered);
            }
        }

        // Add State if requested
        if include_state {
            let state_json = serde_json::to_value(state)?;
            let filtered = crate::rest::event_filter::filter_event_fields(state_json, state_fields)?;
            if !filtered.is_null() {
                result.insert("State".to_string(), filtered);
            }
        }

        Ok(Value::Object(result))
    }
}