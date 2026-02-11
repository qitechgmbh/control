use super::Gluetex;
use crate::gluetex::api::Mutation;
use crate::{MachineApi, MachineMessage};
use serde_json::Value;

impl MachineApi for Gluetex {
    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(request_body)?;
        match mutation {
            Mutation::EnableTraverseLaserpointer(enable) => {
                self.traverse_state.laserpointer = enable;
            }
            Mutation::SetMode(mode) => {
                self.mode_state.mode = mode;
                self.mode_state.can_wind = true;
            }
            Mutation::SetOperationMode(mode) => {
                self.mode_state.operation_mode = mode;
            }
            Mutation::SetTraverseLimitOuter(limit) => {
                self.traverse_state.limit_outer = limit;
            }
            Mutation::SetTraverseLimitInner(limit) => {
                self.traverse_state.limit_inner = limit;
            }
            Mutation::SetTraverseStepSize(size) => {
                self.traverse_state.step_size = size;
            }
            Mutation::SetTraversePadding(padding) => {
                self.traverse_state.padding = padding;
            }
            Mutation::GotoTraverseLimitOuter => {
                self.traverse_state.position_out = self.traverse_state.limit_outer;
                self.traverse_state.is_going_out = true;
            }
            Mutation::GotoTraverseLimitInner => {
                self.traverse_state.position_in = self.traverse_state.limit_inner;
                self.traverse_state.is_going_in = true;
            }
            Mutation::GotoTraverseHome => {
                self.traverse_state.position_in = 0.0;
                self.traverse_state.position_out = 0.0;
                self.traverse_state.is_homed = true;
                self.traverse_state.is_going_home = false;
            }
            Mutation::SetPullerRegulationMode(regulation) => {
                self.puller_state.regulation = regulation;
            }
            Mutation::SetPullerTargetSpeed(value) => {
                self.puller_state.target_speed = value;
                self.live_values.puller_speed = value;
            }
            Mutation::SetPullerTargetDiameter(value) => {
                self.puller_state.target_diameter = value;
            }
            Mutation::SetPullerForward(value) => {
                self.puller_state.forward = value;
            }
            Mutation::SetPullerGearRatio(gear_ratio) => {
                self.puller_state.gear_ratio = gear_ratio;
            }
            Mutation::SetSpoolRegulationMode(mode) => {
                self.spool_speed_controller_state.regulation_mode = mode;
            }
            Mutation::SetSpoolMinMaxMinSpeed(speed) => {
                self.spool_speed_controller_state.minmax_min_speed = speed;
            }
            Mutation::SetSpoolMinMaxMaxSpeed(speed) => {
                self.spool_speed_controller_state.minmax_max_speed = speed;
            }
            Mutation::SetSpoolForward(value) => {
                self.spool_speed_controller_state.forward = value;
            }
            Mutation::SetSpoolAdaptiveTensionTarget(value) => {
                self.spool_speed_controller_state.adaptive_tension_target = value;
            }
            Mutation::SetSpoolAdaptiveRadiusLearningRate(value) => {
                self.spool_speed_controller_state
                    .adaptive_radius_learning_rate = value;
            }
            Mutation::SetSpoolAdaptiveMaxSpeedMultiplier(value) => {
                self.spool_speed_controller_state
                    .adaptive_max_speed_multiplier = value;
            }
            Mutation::SetSpoolAdaptiveAccelerationFactor(value) => {
                self.spool_speed_controller_state
                    .adaptive_acceleration_factor = value;
            }
            Mutation::SetSpoolAdaptiveDeaccelerationUrgencyMultiplier(value) => {
                self.spool_speed_controller_state
                    .adaptive_deacceleration_urgency_multiplier = value;
            }
            Mutation::SetSpoolAutomaticRequiredMeters(meters) => {
                self.spool_automatic_action_state.spool_required_meters = meters;
            }
            Mutation::SetSpoolAutomaticAction(mode) => {
                self.set_spool_automatic_action(mode);
            }
            Mutation::ResetSpoolProgress => {
                self.reset_spool_progress();
            }
            Mutation::ZeroTensionArmAngle => {
                self.tension_arm_state.zeroed = true;
                self.live_values.tension_arm_angle = 0.0;
            }
            Mutation::SetHeatingEnabled(enabled) => {
                self.heating_states.enabled = enabled;
            }
            Mutation::SetHeatingTargetTemperature(zone, temperature) => {
                self.set_heating_target(zone, temperature);
            }
            Mutation::ConfigureHeatingPid(settings) => {
                self.set_heating_pid(settings);
            }
            Mutation::StartHeatingAutoTune(zone, _target_temperature) => {
                self.set_heating_autotune(zone, true);
            }
            Mutation::StopHeatingAutoTune(zone) => {
                self.set_heating_autotune(zone, false);
            }
            Mutation::SetConnectedMachine(ident) => {
                self.set_connected_machine(ident);
            }
            Mutation::DisconnectMachine(ident) => {
                self.disconnect_machine(ident);
            }
            Mutation::SetAddonMotor3Enabled(enabled) => {
                self.addon_motor_3_state.enabled = enabled;
            }
            Mutation::SetAddonMotor4Enabled(enabled) => {
                self.addon_motor_4_state.enabled = enabled;
            }
            Mutation::SetAddonMotor5Enabled(enabled) => {
                self.addon_motor_5_state.enabled = enabled;
            }
            Mutation::SetAddonMotor3Forward(forward) => {
                self.addon_motor_3_state.forward = forward;
            }
            Mutation::SetAddonMotor4Forward(forward) => {
                self.addon_motor_4_state.forward = forward;
            }
            Mutation::SetAddonMotor5Forward(forward) => {
                self.addon_motor_5_state.forward = forward;
            }
            Mutation::SetAddonMotor3MasterRatio(value) => {
                self.addon_motor_3_state.master_ratio = value;
            }
            Mutation::SetAddonMotor3SlaveRatio(value) => {
                self.addon_motor_3_state.slave_ratio = value;
            }
            Mutation::SetAddonMotor4MasterRatio(value) => {
                self.addon_motor_4_state.master_ratio = value;
            }
            Mutation::SetAddonMotor4SlaveRatio(value) => {
                self.addon_motor_4_state.slave_ratio = value;
            }
            Mutation::SetAddonMotor5MasterRatio(value) => {
                self.addon_motor_5_state.master_ratio = value;
            }
            Mutation::SetAddonMotor5SlaveRatio(value) => {
                self.addon_motor_5_state.slave_ratio = value;
            }
            Mutation::SetAddonMotor3Konturlaenge(value) => {
                self.addon_motor_3_state.konturlaenge_mm = value;
            }
            Mutation::SetAddonMotor3Pause(value) => {
                self.addon_motor_3_state.pause_mm = value;
            }
            Mutation::HomeAddonMotor3 => {
                self.addon_motor_3_state.pattern_state = "homed".to_string();
            }
            Mutation::SetAddonMotor5TensionEnabled(enabled) => {
                self.addon_motor_5_tension_control_state.enabled = enabled;
            }
            Mutation::SetAddonMotor5TensionTargetAngle(value) => {
                self.addon_motor_5_tension_control_state.target_angle = value;
            }
            Mutation::SetAddonMotor5TensionSensitivity(value) => {
                self.addon_motor_5_tension_control_state.sensitivity = value;
            }
            Mutation::SetAddonMotor5TensionMinSpeedFactor(value) => {
                self.addon_motor_5_tension_control_state.min_speed_factor = Some(value);
            }
            Mutation::SetAddonMotor5TensionMaxSpeedFactor(value) => {
                self.addon_motor_5_tension_control_state.max_speed_factor = Some(value);
            }
            Mutation::SetSlavePullerEnabled(enabled) => {
                self.slave_puller_state.enabled = enabled;
            }
            Mutation::SetSlavePullerForward(forward) => {
                self.slave_puller_state.forward = forward;
            }
            Mutation::SetSlavePullerTargetAngle(value) => {
                self.slave_puller_state.target_angle = value;
            }
            Mutation::SetSlavePullerSensitivity(value) => {
                self.slave_puller_state.sensitivity = value;
            }
            Mutation::SetSlavePullerMinSpeedFactor(value) => {
                self.slave_puller_state.min_speed_factor = Some(value);
            }
            Mutation::SetSlavePullerMaxSpeedFactor(value) => {
                self.slave_puller_state.max_speed_factor = Some(value);
            }
            Mutation::ZeroSlaveTensionArm => {
                self.slave_puller_state.tension_arm.zeroed = true;
                self.live_values.slave_tension_arm_angle = 0.0;
            }
            Mutation::ZeroAddonTensionArm => {
                self.addon_tension_arm_state.zeroed = true;
                self.live_values.addon_tension_arm_angle = 0.0;
            }
            Mutation::SetWinderTensionArmMonitorEnabled(enabled) => {
                self.winder_tension_arm_monitor_state.enabled = enabled;
            }
            Mutation::SetWinderTensionArmMonitorMinAngle(value) => {
                self.winder_tension_arm_monitor_state.min_angle = value;
            }
            Mutation::SetWinderTensionArmMonitorMaxAngle(value) => {
                self.winder_tension_arm_monitor_state.max_angle = value;
            }
            Mutation::SetAddonTensionArmMonitorEnabled(enabled) => {
                self.addon_tension_arm_monitor_state.enabled = enabled;
            }
            Mutation::SetAddonTensionArmMonitorMinAngle(value) => {
                self.addon_tension_arm_monitor_state.min_angle = value;
            }
            Mutation::SetAddonTensionArmMonitorMaxAngle(value) => {
                self.addon_tension_arm_monitor_state.max_angle = value;
            }
            Mutation::SetSlaveTensionArmMonitorEnabled(enabled) => {
                self.slave_tension_arm_monitor_state.enabled = enabled;
            }
            Mutation::SetSlaveTensionArmMonitorMinAngle(value) => {
                self.slave_tension_arm_monitor_state.min_angle = value;
            }
            Mutation::SetSlaveTensionArmMonitorMaxAngle(value) => {
                self.slave_tension_arm_monitor_state.max_angle = value;
            }
            Mutation::SetOptris1MonitorEnabled(enabled) => {
                self.optris_1_monitor_state.enabled = enabled;
            }
            Mutation::SetOptris1MonitorMinVoltage(value) => {
                self.optris_1_monitor_state.min_voltage = value;
            }
            Mutation::SetOptris1MonitorMaxVoltage(value) => {
                self.optris_1_monitor_state.max_voltage = value;
            }
            Mutation::SetOptris2MonitorEnabled(enabled) => {
                self.optris_2_monitor_state.enabled = enabled;
            }
            Mutation::SetOptris2MonitorMinVoltage(value) => {
                self.optris_2_monitor_state.min_voltage = value;
            }
            Mutation::SetOptris2MonitorMaxVoltage(value) => {
                self.optris_2_monitor_state.max_voltage = value;
            }
            Mutation::SetSleepTimerEnabled(enabled) => {
                self.sleep_timer_state.enabled = enabled;
                self.sleep_timer_state.triggered = false;
            }
            Mutation::SetSleepTimerTimeout(timeout) => {
                self.sleep_timer_state.timeout_seconds = timeout;
                self.sleep_timer_state.remaining_seconds = timeout;
                self.sleep_timer_state.triggered = false;
            }
            Mutation::ResetSleepTimer => {
                self.sleep_timer_state.remaining_seconds = self.sleep_timer_state.timeout_seconds;
                self.sleep_timer_state.triggered = false;
            }
            Mutation::SetOrderNumber(value) => {
                self.order_info_state.order_number = value;
            }
            Mutation::SetSerialNumber(value) => {
                self.order_info_state.serial_number = value;
            }
            Mutation::SetProductDescription(value) => {
                self.order_info_state.product_description = value;
            }
            Mutation::SetValveEnabled(enabled) => {
                self.valve_state.enabled = enabled;
            }
            Mutation::SetValveManualOverride(value) => {
                self.valve_state.manual_override = value;
            }
            Mutation::SetValveOnDistanceMm(value) => {
                self.valve_state.on_distance_mm = value;
            }
            Mutation::SetValveOffDistanceMm(value) => {
                self.valve_state.off_distance_mm = value;
            }
        }

        self.emit_state();
        Ok(())
    }

    fn api_event_namespace(
        &mut self,
    ) -> std::option::Option<control_core::socketio::namespace::Namespace> {
        self.namespace.namespace.clone()
    }

    fn api_get_sender(&self) -> smol::channel::Sender<MachineMessage> {
        self.api_sender.clone()
    }
}
