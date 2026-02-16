mod gluetex_imports {
    pub use super::super::controllers::puller_speed_controller::PullerRegulationMode;
    pub use super::super::controllers::spool_speed_controller;
    pub use super::super::{Gluetex, GluetexMode, TraverseMode, api};
    pub use crate::buffer1::BufferV1;
    pub use api::{
        GluetexEvents, LiveValuesEvent, ModeState, PullerState, SpoolAutomaticActionMode,
        SpoolAutomaticActionState, SpoolSpeedControllerState, StateEvent, TensionArmState,
        TraverseState,
    };
    pub use control_core::socketio::event::BuildEvent;
    pub use control_core::socketio::namespace::NamespaceCacheingLogic;
    pub use std::time::Instant;
    pub use units::{
        angle::degree,
        angular_velocity::revolution_per_minute,
        f64::*,
        length::{meter, millimeter},
        velocity::meter_per_minute,
    };
}

pub use gluetex_imports::*;

impl Gluetex {
    /// Implement Spool
    /// called by `act`
    pub fn sync_spool_speed(&mut self, t: Instant) {
        let angular_velocity = self.spool_speed_controller.update_speed(
            t,
            &self.tension_arm,
            &self.puller_speed_controller,
        );

        // Apply direction based on forward setting
        let directed_angular_velocity = if self.spool_speed_controller.get_forward() {
            angular_velocity
        } else {
            -angular_velocity
        };

        let steps_per_second = self
            .spool_step_converter
            .angular_velocity_to_steps(directed_angular_velocity);
        let _ = self.spool.set_speed(steps_per_second);
    }

    pub fn stop_or_pull_spool(&mut self, now: Instant) {
        if matches!(
            self.spool_automatic_action.mode,
            SpoolAutomaticActionMode::NoAction
        ) {
            self.calculate_spool_auto_progress_(now);
            return;
        }

        match self.mode {
            GluetexMode::Pull => self.calculate_spool_auto_progress_(now),
            GluetexMode::Wind => self.calculate_spool_auto_progress_(now),
            _ => {
                self.spool_automatic_action.progress_last_check = now;
                return;
            }
        }

        if self.spool_automatic_action.progress >= self.spool_automatic_action.target_length {
            match self.spool_automatic_action.mode {
                SpoolAutomaticActionMode::NoAction => (),
                SpoolAutomaticActionMode::Pull => {
                    self.stop_or_pull_spool_reset(now);
                    self.set_mode(&GluetexMode::Pull);
                }
                SpoolAutomaticActionMode::Hold => {
                    self.stop_or_pull_spool_reset(now);
                    self.set_mode(&GluetexMode::Hold);
                }
            }
        }
    }
    /// Implement Mode
    pub fn set_mode(&mut self, mode: &GluetexMode) {
        let should_update = *mode != GluetexMode::Wind || self.can_wind();

        if should_update {
            // all transitions are allowed
            self.mode = mode.clone();

            // Apply the mode changes to the spool, puller, slave puller, and traverse
            self.set_spool_mode(mode);
            self.set_puller_mode(mode);
            self.set_slave_puller_mode(mode);
            self.set_traverse_mode(mode);
        }
        self.update_status_output();
        self.emit_state();
    }

    /// Set operation mode (safety monitoring level)
    pub fn set_operation_mode(&mut self, mode: &super::OperationMode) {
        self.operation_mode = mode.clone();
        // Reset activity timer when changing operation mode
        self.reset_sleep_timer();
        self.emit_state();
    }

    /// Implement Traverse
    pub fn set_laser(&mut self, value: bool) {
        self.laser.set(value);
        self.emit_state();
    }

    pub fn set_extra_output(&mut self, channel: api::ExtraOutputChannel, enabled: bool) {
        let output = match channel {
            api::ExtraOutputChannel::Output1 => &mut self.extra_outputs[0],
            api::ExtraOutputChannel::Output2 => &mut self.extra_outputs[1],
            api::ExtraOutputChannel::Output3 => &mut self.extra_outputs[2],
            api::ExtraOutputChannel::Output4 => &mut self.extra_outputs[3],
            api::ExtraOutputChannel::Output5 => &mut self.extra_outputs[4],
            api::ExtraOutputChannel::Output6 => &mut self.extra_outputs[5],
            api::ExtraOutputChannel::Output7 => &mut self.extra_outputs[6],
            api::ExtraOutputChannel::Output8 => &mut self.extra_outputs[7],
        };
        output.set(enabled);
        self.emit_state();
    }

    pub fn traverse_set_limit_inner(&mut self, limit: f64) {
        let new_inner = Length::new::<millimeter>(limit);
        let current_outer = self.traverse_controller.get_limit_outer();

        // Validate the new inner limit against current outer limit
        if !Self::validate_traverse_limits(new_inner, current_outer) {
            // Don't update if validation fails - keep the current value
            return;
        }
        self.traverse_controller.set_limit_inner(new_inner);
        self.emit_state();
    }

    pub fn traverse_set_limit_outer(&mut self, limit: f64) {
        let new_outer = Length::new::<millimeter>(limit);
        let current_inner = self.traverse_controller.get_limit_inner();

        // Validate the new outer limit against current inner limit
        if !Self::validate_traverse_limits(current_inner, new_outer) {
            // Don't update if validation fails - keep the current value
            return;
        }

        self.traverse_controller.set_limit_outer(new_outer);
        self.emit_state();
    }

    pub fn traverse_set_step_size(&mut self, step_size: f64) {
        let step_size = Length::new::<millimeter>(step_size);
        self.traverse_controller.set_step_size(step_size);
        self.emit_state();
    }

    pub fn traverse_set_padding(&mut self, padding: f64) {
        let padding = Length::new::<millimeter>(padding);
        self.traverse_controller.set_padding(padding);
        self.emit_state();
    }

    pub fn traverse_goto_limit_inner(&mut self) {
        if self.can_go_in() {
            self.traverse_controller.goto_limit_inner();
        }
        self.emit_state();
    }

    pub fn traverse_goto_limit_outer(&mut self) {
        if self.can_go_out() {
            self.traverse_controller.goto_limit_outer();
        }
        self.emit_state();
    }

    pub fn traverse_goto_home(&mut self) {
        if self.can_go_home() {
            self.traverse_controller.goto_home();
        }
        self.emit_state();
    }

    pub fn get_state(&mut self) -> StateEvent {
        self.build_state_event()
    }

    pub fn get_live_values(&mut self) -> LiveValuesEvent {
        self.build_live_values_event()
    }

    fn build_live_values_event(&mut self) -> LiveValuesEvent {
        let angle_deg = self.tension_arm.get_angle().get::<degree>();

        // Wrap [270;<360] to [-90; 0]
        // This is done to reduce flicker in the graphs around the zero point
        let angle_deg = if angle_deg >= 270.0 {
            angle_deg - 360.0
        } else {
            angle_deg
        };

        // Calculate puller speed from current motor steps
        let steps_per_second = self.puller.get_speed();
        let angular_velocity = self
            .puller_speed_controller
            .converter
            .steps_to_angular_velocity(steps_per_second as f64);
        let motor_speed = self
            .puller_speed_controller
            .angular_velocity_to_speed(angular_velocity);

        // Divide by gear ratio to get actual puller/material speed
        let puller_speed = motor_speed / self.puller_speed_controller.get_gear_ratio().multiplier();

        // Calculate spool RPM from current motor steps (always positive regardless of direction)
        let spool_rpm = self
            .spool_step_converter
            .steps_to_angular_velocity(self.spool.get_speed() as f64)
            .get::<revolution_per_minute>()
            .abs();

        let live_values = LiveValuesEvent {
            traverse_position: self
                .traverse_controller
                .get_current_position()
                .map(|x| x.get::<millimeter>()),
            puller_speed: puller_speed.get::<meter_per_minute>().abs(),
            spool_rpm,
            tension_arm_angle: angle_deg,
            spool_progress: self.spool_automatic_action.progress.get::<meter>(),
            temperature_1: self
                .temperature_controller_1
                .get_temperature()
                .unwrap_or(0.0),
            temperature_2: self
                .temperature_controller_2
                .get_temperature()
                .unwrap_or(0.0),
            temperature_3: self
                .temperature_controller_3
                .get_temperature()
                .unwrap_or(0.0),
            temperature_4: self
                .temperature_controller_4
                .get_temperature()
                .unwrap_or(0.0),
            temperature_5: self
                .temperature_controller_5
                .get_temperature()
                .unwrap_or(0.0),
            temperature_6: self
                .temperature_controller_6
                .get_temperature()
                .unwrap_or(0.0),
            heater_1_power: self.temperature_controller_1.get_heating_element_wattage(),
            heater_2_power: self.temperature_controller_2.get_heating_element_wattage(),
            heater_3_power: self.temperature_controller_3.get_heating_element_wattage(),
            heater_4_power: self.temperature_controller_4.get_heating_element_wattage(),
            heater_5_power: self.temperature_controller_5.get_heating_element_wattage(),
            heater_6_power: self.temperature_controller_6.get_heating_element_wattage(),
            slave_puller_speed: {
                let steps_per_second = self.slave_puller.get_speed();
                let angular_velocity = self
                    .slave_puller_speed_controller
                    .converter
                    .steps_to_angular_velocity(steps_per_second as f64);
                let speed = self
                    .slave_puller_speed_controller
                    .converter
                    .angular_velocity_to_velocity(angular_velocity);
                speed.get::<meter_per_minute>().abs()
            },
            slave_tension_arm_angle: {
                let angle = self.slave_tension_arm.get_angle().get::<degree>();
                // Wrap [270;<360] to [-90; 0]
                if angle >= 270.0 { angle - 360.0 } else { angle }
            },
            addon_tension_arm_angle: {
                let angle = self.addon_tension_arm.get_angle().get::<degree>();
                // Wrap [270;<360] to [-90; 0]
                if angle >= 270.0 { angle - 360.0 } else { angle }
            },
            optris_1_voltage: {
                use ethercat_hal::io::analog_input::physical::AnalogInputValue;
                use units::electric_potential::volt;
                match self.optris_1.get_physical() {
                    AnalogInputValue::Potential(v) => v.get::<volt>(),
                    _ => 0.0,
                }
            },
            optris_2_voltage: {
                use ethercat_hal::io::analog_input::physical::AnalogInputValue;
                use units::electric_potential::volt;
                match self.optris_2.get_physical() {
                    AnalogInputValue::Potential(v) => v.get::<volt>(),
                    _ => 0.0,
                }
            },
        };

        live_values
    }

    pub fn emit_live_values(&mut self) {
        let live_values = self.build_live_values_event();
        let event = live_values.build();
        self.namespace.emit(GluetexEvents::LiveValues(event));
    }

    pub fn build_state_event(&mut self) -> StateEvent {
        use crate::MachineCrossConnectionState;

        let connected_machine = self.connected_machines.get(0);
        let ident = match connected_machine {
            Some(machine) => Some(machine.ident.clone()),
            None => None,
        };
        let cross_conn = MachineCrossConnectionState {
            machine_identification_unique: ident,
            is_available: connected_machine.is_some(),
        };

        StateEvent {
            is_default_state: !std::mem::replace(&mut self.emitted_default_state, true),
            status_out: self.status_out.get(),
            traverse_state: TraverseState {
                limit_inner: self
                    .traverse_controller
                    .get_limit_inner()
                    .get::<millimeter>(),
                limit_outer: self
                    .traverse_controller
                    .get_limit_outer()
                    .get::<millimeter>(),
                position_in: self
                    .traverse_controller
                    .get_limit_inner()
                    .get::<millimeter>(),
                position_out: self
                    .traverse_controller
                    .get_limit_outer()
                    .get::<millimeter>(),
                is_going_in: self.traverse_controller.is_going_in(),
                is_going_out: self.traverse_controller.is_going_out(),
                is_homed: self.traverse_controller.is_homed(),
                is_going_home: self.traverse_controller.is_going_home(),
                is_traversing: self.traverse_controller.is_traversing(),
                laserpointer: self.laser.get(),
                step_size: self.traverse_controller.get_step_size().get::<millimeter>(),
                padding: self.traverse_controller.get_padding().get::<millimeter>(),
                can_go_in: self.can_go_in(),
                can_go_out: self.can_go_out(),
                can_go_home: self.can_go_home(),
            },
            puller_state: PullerState {
                regulation: self.puller_speed_controller.regulation_mode.clone(),
                target_speed: self
                    .puller_speed_controller
                    .target_speed
                    .get::<meter_per_minute>(),
                target_diameter: self
                    .puller_speed_controller
                    .target_diameter
                    .get::<millimeter>(),
                forward: self.puller_speed_controller.forward,
                gear_ratio: self.puller_speed_controller.gear_ratio,
            },
            mode_state: ModeState {
                mode: self.mode.clone().into(),
                operation_mode: self.operation_mode.clone().into(),
                can_wind: self.can_wind(),
            },
            tension_arm_state: TensionArmState {
                zeroed: self.tension_arm.zeroed,
            },
            spool_speed_controller_state: SpoolSpeedControllerState {
                regulation_mode: self.spool_speed_controller.get_type().clone(),
                minmax_min_speed: self
                    .spool_speed_controller
                    .get_minmax_min_speed()
                    .get::<revolution_per_minute>(),
                minmax_max_speed: self
                    .spool_speed_controller
                    .get_minmax_max_speed()
                    .get::<revolution_per_minute>(),
                adaptive_tension_target: self.spool_speed_controller.get_adaptive_tension_target(),
                adaptive_radius_learning_rate: self
                    .spool_speed_controller
                    .get_adaptive_radius_learning_rate(),
                adaptive_max_speed_multiplier: self
                    .spool_speed_controller
                    .get_adaptive_max_speed_multiplier(),
                adaptive_acceleration_factor: self
                    .spool_speed_controller
                    .get_adaptive_acceleration_factor(),
                adaptive_deacceleration_urgency_multiplier: self
                    .spool_speed_controller
                    .get_adaptive_deacceleration_urgency_multiplier(),
                forward: self.spool_speed_controller.get_forward(),
            },
            spool_automatic_action_state: SpoolAutomaticActionState {
                spool_required_meters: self.spool_automatic_action.target_length.get::<meter>(),
                spool_automatic_action_mode: self.spool_automatic_action.mode.clone(),
            },
            heating_states: api::HeatingStates {
                enabled: self.heating_enabled,
                zone_1: api::HeatingState {
                    target_temperature: self
                        .temperature_controller_1
                        .heating
                        .target_temperature
                        .get::<units::thermodynamic_temperature::degree_celsius>(),
                    wiring_error: self.temperature_controller_1.heating.wiring_error,
                    autotuning_active: self.temperature_controller_1.is_autotuning(),
                    autotuning_progress: self.temperature_controller_1.get_autotuning_progress(),
                },
                zone_2: api::HeatingState {
                    target_temperature: self
                        .temperature_controller_2
                        .heating
                        .target_temperature
                        .get::<units::thermodynamic_temperature::degree_celsius>(),
                    wiring_error: self.temperature_controller_2.heating.wiring_error,
                    autotuning_active: self.temperature_controller_2.is_autotuning(),
                    autotuning_progress: self.temperature_controller_2.get_autotuning_progress(),
                },
                zone_3: api::HeatingState {
                    target_temperature: self
                        .temperature_controller_3
                        .heating
                        .target_temperature
                        .get::<units::thermodynamic_temperature::degree_celsius>(),
                    wiring_error: self.temperature_controller_3.heating.wiring_error,
                    autotuning_active: self.temperature_controller_3.is_autotuning(),
                    autotuning_progress: self.temperature_controller_3.get_autotuning_progress(),
                },
                zone_4: api::HeatingState {
                    target_temperature: self
                        .temperature_controller_4
                        .heating
                        .target_temperature
                        .get::<units::thermodynamic_temperature::degree_celsius>(),
                    wiring_error: self.temperature_controller_4.heating.wiring_error,
                    autotuning_active: self.temperature_controller_4.is_autotuning(),
                    autotuning_progress: self.temperature_controller_4.get_autotuning_progress(),
                },
                zone_5: api::HeatingState {
                    target_temperature: self
                        .temperature_controller_5
                        .heating
                        .target_temperature
                        .get::<units::thermodynamic_temperature::degree_celsius>(),
                    wiring_error: self.temperature_controller_5.heating.wiring_error,
                    autotuning_active: self.temperature_controller_5.is_autotuning(),
                    autotuning_progress: self.temperature_controller_5.get_autotuning_progress(),
                },
                zone_6: api::HeatingState {
                    target_temperature: self
                        .temperature_controller_6
                        .heating
                        .target_temperature
                        .get::<units::thermodynamic_temperature::degree_celsius>(),
                    wiring_error: self.temperature_controller_6.heating.wiring_error,
                    autotuning_active: self.temperature_controller_6.is_autotuning(),
                    autotuning_progress: self.temperature_controller_6.get_autotuning_progress(),
                },
            },
            heating_pid_settings: api::HeatingPidStates {
                zone_1: api::HeatingPidSettings {
                    ki: self.temperature_controller_1.pid.get_ki(),
                    kp: self.temperature_controller_1.pid.get_kp(),
                    kd: self.temperature_controller_1.pid.get_kd(),
                    zone: String::from("zone_1"),
                },
                zone_2: api::HeatingPidSettings {
                    ki: self.temperature_controller_2.pid.get_ki(),
                    kp: self.temperature_controller_2.pid.get_kp(),
                    kd: self.temperature_controller_2.pid.get_kd(),
                    zone: String::from("zone_2"),
                },
                zone_3: api::HeatingPidSettings {
                    ki: self.temperature_controller_3.pid.get_ki(),
                    kp: self.temperature_controller_3.pid.get_kp(),
                    kd: self.temperature_controller_3.pid.get_kd(),
                    zone: String::from("zone_3"),
                },
                zone_4: api::HeatingPidSettings {
                    ki: self.temperature_controller_4.pid.get_ki(),
                    kp: self.temperature_controller_4.pid.get_kp(),
                    kd: self.temperature_controller_4.pid.get_kd(),
                    zone: String::from("zone_4"),
                },
                zone_5: api::HeatingPidSettings {
                    ki: self.temperature_controller_5.pid.get_ki(),
                    kp: self.temperature_controller_5.pid.get_kp(),
                    kd: self.temperature_controller_5.pid.get_kd(),
                    zone: String::from("zone_5"),
                },
                zone_6: api::HeatingPidSettings {
                    ki: self.temperature_controller_6.pid.get_ki(),
                    kp: self.temperature_controller_6.pid.get_kp(),
                    kd: self.temperature_controller_6.pid.get_kd(),
                    zone: String::from("zone_6"),
                },
            },
            connected_machine_state: cross_conn,
            addon_motor_3_state: api::AddonMotor5State {
                enabled: self.addon_motor_3_controller.is_enabled(),
                forward: self.addon_motor_3_controller.is_forward(),
                master_ratio: self.addon_motor_3_controller.get_master_ratio(),
                slave_ratio: self.addon_motor_3_controller.get_slave_ratio(),
                konturlaenge_mm: self.addon_motor_3_controller.get_konturlaenge_mm(),
                pause_mm: self.addon_motor_3_controller.get_pause_mm(),
                pattern_state: format!("{:?}", self.addon_motor_3_controller.get_pattern_state()),
            },
            addon_motor_4_state: api::AddonMotorState {
                enabled: self.addon_motor_4_controller.is_enabled(),
                forward: self.addon_motor_4_controller.is_forward(),
                master_ratio: self.addon_motor_4_controller.get_master_ratio(),
                slave_ratio: self.addon_motor_4_controller.get_slave_ratio(),
            },
            addon_motor_5_state: api::AddonMotorState {
                enabled: self.addon_motor_5_controller.is_enabled(),
                forward: self.addon_motor_5_controller.is_forward(),
                master_ratio: self.addon_motor_5_controller.get_master_ratio(),
                slave_ratio: self.addon_motor_5_controller.get_slave_ratio(),
            },
            addon_motor_5_tension_control_state: api::AddonMotorTensionControlState {
                enabled: self.addon_motor_5_tension_controller.is_enabled(),
                target_angle: self
                    .addon_motor_5_tension_controller
                    .get_target_angle()
                    .get::<degree>(),
                sensitivity: self
                    .addon_motor_5_tension_controller
                    .get_sensitivity()
                    .get::<degree>(),
                min_speed_factor: self.addon_motor_5_tension_controller.get_min_speed_factor(),
                max_speed_factor: self.addon_motor_5_tension_controller.get_max_speed_factor(),
            },
            slave_puller_state: api::SlavePullerState {
                enabled: self.slave_puller_user_enabled,
                forward: self.slave_puller_speed_controller.get_forward(),
                target_angle: self
                    .slave_puller_speed_controller
                    .get_target_angle()
                    .get::<degree>(),
                sensitivity: self
                    .slave_puller_speed_controller
                    .get_sensitivity()
                    .get::<degree>(),
                min_speed_factor: self.slave_puller_speed_controller.get_min_speed_factor(),
                max_speed_factor: self.slave_puller_speed_controller.get_max_speed_factor(),
                tension_arm: api::SlaveTensionArmState {
                    zeroed: self.slave_tension_arm.zeroed,
                },
            },
            addon_tension_arm_state: TensionArmState {
                zeroed: self.addon_tension_arm.zeroed,
            },
            winder_tension_arm_monitor_state: api::TensionArmMonitorState {
                enabled: self.winder_tension_arm_monitor.config.enabled,
                min_angle: self
                    .winder_tension_arm_monitor
                    .config
                    .min_angle
                    .get::<degree>(),
                max_angle: self
                    .winder_tension_arm_monitor
                    .config
                    .max_angle
                    .get::<degree>(),
                triggered: self.winder_tension_arm_monitor.triggered,
            },
            addon_tension_arm_monitor_state: api::TensionArmMonitorState {
                enabled: self.addon_tension_arm_monitor.config.enabled,
                min_angle: self
                    .addon_tension_arm_monitor
                    .config
                    .min_angle
                    .get::<degree>(),
                max_angle: self
                    .addon_tension_arm_monitor
                    .config
                    .max_angle
                    .get::<degree>(),
                triggered: self.addon_tension_arm_monitor.triggered,
            },
            slave_tension_arm_monitor_state: api::TensionArmMonitorState {
                enabled: self.slave_tension_arm_monitor.config.enabled,
                min_angle: self
                    .slave_tension_arm_monitor
                    .config
                    .min_angle
                    .get::<degree>(),
                max_angle: self
                    .slave_tension_arm_monitor
                    .config
                    .max_angle
                    .get::<degree>(),
                triggered: self.slave_tension_arm_monitor.triggered,
            },
            optris_1_monitor_state: api::VoltageMonitorState {
                enabled: self.optris_1_monitor.config.enabled,
                min_voltage: self.optris_1_monitor.config.min_voltage,
                max_voltage: self.optris_1_monitor.config.max_voltage,
                triggered: self.optris_1_monitor.triggered,
            },
            optris_2_monitor_state: api::VoltageMonitorState {
                enabled: self.optris_2_monitor.config.enabled,
                min_voltage: self.optris_2_monitor.config.min_voltage,
                max_voltage: self.optris_2_monitor.config.max_voltage,
                triggered: self.optris_2_monitor.triggered,
            },
            sleep_timer_state: api::SleepTimerState {
                enabled: self.sleep_timer.config.enabled,
                timeout_seconds: self.sleep_timer.config.timeout_seconds,
                remaining_seconds: self.get_sleep_timer_remaining_seconds(),
                triggered: self.sleep_timer.triggered,
            },
            order_info_state: api::OrderInfoState {
                order_number: self.order_info.order_number,
                serial_number: self.order_info.serial_number,
                product_description: self.order_info.product_description.clone(),
            },
            extra_outputs_state: api::ExtraOutputsState {
                output_1: self.extra_outputs[0].get(),
                output_2: self.extra_outputs[1].get(),
                output_3: self.extra_outputs[2].get(),
                output_4: self.extra_outputs[3].get(),
                output_5: self.extra_outputs[4].get(),
                output_6: self.extra_outputs[5].get(),
                output_7: self.extra_outputs[6].get(),
                output_8: self.extra_outputs[7].get(),
            },
            valve_state: api::ValveState {
                enabled: self.valve_controller.is_enabled(),
                manual_override: self.valve_controller.get_manual_override(),
                on_distance_mm: self.valve_controller.get_on_distance_mm(),
                off_distance_mm: self.valve_controller.get_off_distance_mm(),
                pattern_state: self.valve_controller.get_pattern_state(),
                accumulated_distance: self.valve_controller.get_accumulated_distance(),
                valve_output: self.valve_controller.get_desired_state(),
            },
        }
    }

    pub fn emit_state(&mut self) {
        let state_event = self.build_state_event();
        let event = state_event.build();
        self.namespace.emit(GluetexEvents::State(event));
    }

    /// Apply the mode changes to the spool
    ///
    /// It contains a transition matrix for atomic changes.
    /// It will set [`Self::spool_mode`]
    fn set_traverse_mode(&mut self, mode: &GluetexMode) {
        // Convert to `GluetexMode` to `TraverseMode`
        let mode: TraverseMode = mode.clone().into();

        // If coming out of standby
        if self.traverse_mode == TraverseMode::Standby && mode != TraverseMode::Standby {
            self.traverse.set_enabled(true);
            self.traverse_controller.set_enabled(true);
        }

        // If going into standby
        if mode == TraverseMode::Standby && self.traverse_mode != TraverseMode::Standby {
            // If we are going into standby, we need to stop the traverse
            self.traverse.set_enabled(false);
            self.traverse_controller.set_enabled(false);
        }

        // Transition matrix
        match self.traverse_mode {
            TraverseMode::Standby => match mode {
                TraverseMode::Standby => {}
                TraverseMode::Hold => {
                    // From [`TraverseMode::Standby`] to [`TraverseMode::Hold`]
                    self.traverse.set_enabled(true);
                    self.traverse_controller.set_enabled(true);
                    self.traverse_controller.goto_home();
                }
                TraverseMode::Traverse => {
                    // From [`TraverseMode::Standby`] to [`TraverseMode::Wind`]
                    self.traverse.set_enabled(true);
                    self.traverse_controller.set_enabled(true);
                    self.traverse_controller.start_traversing();
                }
            },
            TraverseMode::Hold => match mode {
                TraverseMode::Standby => {
                    // From [`TraverseMode::Hold`] to [`TraverseMode::Standby`]
                    self.traverse.set_enabled(false);
                    self.traverse_controller.set_enabled(false);
                }
                TraverseMode::Hold => {}
                TraverseMode::Traverse => {
                    // From [`TraverseMode::Hold`] to [`TraverseMode::Wind`]
                    self.traverse_controller.start_traversing();
                }
            },
            TraverseMode::Traverse => match mode {
                TraverseMode::Standby => {
                    // From [`TraverseMode::Wind`] to [`TraverseMode::Standby`]
                    self.traverse.set_enabled(false);
                    self.traverse_controller.set_enabled(false);
                }
                TraverseMode::Hold => {
                    // From [`TraverseMode::Wind`] to [`TraverseMode::Hold`]
                    self.traverse_controller.goto_home();
                }
                TraverseMode::Traverse => {}
            },
        }

        // Update the internal state
        self.traverse_mode = mode;
        self.emit_state();
    }

    /// Implement Tension Arm
    pub fn tension_arm_zero(&mut self) {
        self.tension_arm.zero();
        self.emit_live_values(); // For angle update
        // For state update
        self.emit_state();
    }

    pub fn set_spool_automatic_required_meters(&mut self, meters: f64) {
        self.spool_automatic_action.target_length = Length::new::<meter>(meters);
        self.emit_state();
    }

    pub fn set_spool_automatic_mode(&mut self, mode: SpoolAutomaticActionMode) {
        self.spool_automatic_action.mode = mode;
        self.emit_state();
    }

    pub fn puller_set_regulation(&mut self, puller_regulation_mode: PullerRegulationMode) {
        self.puller_speed_controller
            .set_regulation_mode(puller_regulation_mode);
        self.emit_state();
    }

    /// Set target speed in m/min
    pub fn puller_set_target_speed(&mut self, target_speed: f64) {
        // Convert m/min to velocity
        let target_speed = Velocity::new::<meter_per_minute>(target_speed);
        self.puller_speed_controller.set_target_speed(target_speed);
        self.emit_state();
    }

    /// Set target diameter in mm
    pub fn puller_set_target_diameter(&mut self, target_diameter: f64) {
        // Convert m/min to velocity
        let target_diameter = Length::new::<millimeter>(target_diameter);
        self.puller_speed_controller
            .set_target_diameter(target_diameter);
        self.emit_state();
    }

    /// Set forward direction
    pub fn puller_set_forward(&mut self, forward: bool) {
        self.puller_speed_controller.set_forward(forward);
        self.emit_state();
    }

    /// Set gear ratio for winding speed
    pub fn puller_set_gear_ratio(
        &mut self,
        gear_ratio: super::controllers::puller_speed_controller::GearRatio,
    ) {
        self.puller_speed_controller.set_gear_ratio(gear_ratio);
        self.emit_state();
    }

    // Spool Speed Controller API methods
    pub fn spool_set_regulation_mode(
        &mut self,
        regulation_mode: spool_speed_controller::SpoolSpeedControllerType,
    ) {
        self.spool_speed_controller.set_type(regulation_mode);
        self.emit_state();
    }

    /// Set minimum speed for minmax mode in RPM
    pub fn spool_set_minmax_min_speed(&mut self, min_speed_rpm: f64) {
        let min_speed = AngularVelocity::new::<revolution_per_minute>(min_speed_rpm);
        if let Err(e) = self.spool_speed_controller.set_minmax_min_speed(min_speed) {
            tracing::error!("Failed to set spool min speed: {:?}", e);
        }
        self.emit_state();
    }

    /// Set maximum speed for minmax mode in RPM
    pub fn spool_set_minmax_max_speed(&mut self, max_speed_rpm: f64) {
        let max_speed = AngularVelocity::new::<revolution_per_minute>(max_speed_rpm);
        if let Err(e) = self.spool_speed_controller.set_minmax_max_speed(max_speed) {
            tracing::error!("Failed to set spool max speed: {:?}", e);
        }
        self.emit_state();
    }

    /// Set tension target for adaptive mode (0.0-1.0)
    pub fn spool_set_adaptive_tension_target(&mut self, tension_target: f64) {
        self.spool_speed_controller
            .set_adaptive_tension_target(tension_target);
        self.emit_state();
    }

    /// Set target temperature for a heating zone
    pub fn set_target_temperature(&mut self, target_temperature: f64, zone: api::HeatingZone) {
        let target_temp = units::f64::ThermodynamicTemperature::new::<
            units::thermodynamic_temperature::degree_celsius,
        >(target_temperature);

        match zone {
            api::HeatingZone::Zone1 => self
                .temperature_controller_1
                .set_target_temperature(target_temp),
            api::HeatingZone::Zone2 => self
                .temperature_controller_2
                .set_target_temperature(target_temp),
            api::HeatingZone::Zone3 => self
                .temperature_controller_3
                .set_target_temperature(target_temp),
            api::HeatingZone::Zone4 => self
                .temperature_controller_4
                .set_target_temperature(target_temp),
            api::HeatingZone::Zone5 => self
                .temperature_controller_5
                .set_target_temperature(target_temp),
            api::HeatingZone::Zone6 => self
                .temperature_controller_6
                .set_target_temperature(target_temp),
        }
        self.emit_state();
    }

    /// Enable or disable all heating zones
    pub fn set_heating_enabled(&mut self, enabled: bool) {
        self.heating_enabled = enabled;
        if enabled {
            self.temperature_controller_1.allow_heating();
            self.temperature_controller_2.allow_heating();
            self.temperature_controller_3.allow_heating();
            self.temperature_controller_4.allow_heating();
            self.temperature_controller_5.allow_heating();
            self.temperature_controller_6.allow_heating();
        } else {
            self.temperature_controller_1.disallow_heating();
            self.temperature_controller_2.disallow_heating();
            self.temperature_controller_3.disallow_heating();
            self.temperature_controller_4.disallow_heating();
            self.temperature_controller_5.disallow_heating();
            self.temperature_controller_6.disallow_heating();
        }
        self.update_status_output();
    }

    pub fn update_status_output(&mut self) {
        let any_heating_active = self.temperature_controller_1.heating.heating
            || self.temperature_controller_2.heating.heating
            || self.temperature_controller_3.heating.heating
            || self.temperature_controller_4.heating.heating
            || self.temperature_controller_5.heating.heating
            || self.temperature_controller_6.heating.heating
            || self.temperature_controller_1.is_autotuning()
            || self.temperature_controller_2.is_autotuning()
            || self.temperature_controller_3.is_autotuning()
            || self.temperature_controller_4.is_autotuning()
            || self.temperature_controller_5.is_autotuning()
            || self.temperature_controller_6.is_autotuning();

        let machine_active = self.mode != GluetexMode::Standby || self.heating_enabled;
        let status_on = machine_active || any_heating_active;
        self.status_out.set(status_on);
    }

    /// Configure PID parameters for a heating zone
    pub fn configure_heating_pid(&mut self, settings: api::HeatingPidSettings) {
        match settings.zone.as_str() {
            "zone_1" => {
                self.temperature_controller_1
                    .pid
                    .configure(settings.ki, settings.kp, settings.kd);
            }
            "zone_2" => {
                self.temperature_controller_2
                    .pid
                    .configure(settings.ki, settings.kp, settings.kd);
            }
            "zone_3" => {
                self.temperature_controller_3
                    .pid
                    .configure(settings.ki, settings.kp, settings.kd);
            }
            "zone_4" => {
                self.temperature_controller_4
                    .pid
                    .configure(settings.ki, settings.kp, settings.kd);
            }
            "zone_5" => {
                self.temperature_controller_5
                    .pid
                    .configure(settings.ki, settings.kp, settings.kd);
            }
            "zone_6" => {
                self.temperature_controller_6
                    .pid
                    .configure(settings.ki, settings.kp, settings.kd);
            }
            _ => tracing::warn!("Unknown heating zone: {}", settings.zone),
        }
        self.emit_state();
    }

    /// Start PID auto-tuning for a heating zone
    pub fn start_heating_autotune(&mut self, zone: api::HeatingZone, target_temp: f64) {
        use units::thermodynamic_temperature::degree_celsius;
        let target = ThermodynamicTemperature::new::<degree_celsius>(target_temp);

        match zone {
            api::HeatingZone::Zone1 => {
                self.temperature_controller_1.start_autotuning(target);
                tracing::info!(
                    "Started auto-tuning for zone 1 with target {}°C",
                    target_temp
                );
            }
            api::HeatingZone::Zone2 => {
                self.temperature_controller_2.start_autotuning(target);
                tracing::info!(
                    "Started auto-tuning for zone 2 with target {}°C",
                    target_temp
                );
            }
            api::HeatingZone::Zone3 => {
                self.temperature_controller_3.start_autotuning(target);
                tracing::info!(
                    "Started auto-tuning for zone 3 with target {}°C",
                    target_temp
                );
            }
            api::HeatingZone::Zone4 => {
                self.temperature_controller_4.start_autotuning(target);
                tracing::info!(
                    "Started auto-tuning for zone 4 with target {}°C",
                    target_temp
                );
            }
            api::HeatingZone::Zone5 => {
                self.temperature_controller_5.start_autotuning(target);
                tracing::info!(
                    "Started auto-tuning for zone 5 with target {}°C",
                    target_temp
                );
            }
            api::HeatingZone::Zone6 => {
                self.temperature_controller_6.start_autotuning(target);
                tracing::info!(
                    "Started auto-tuning for zone 6 with target {}°C",
                    target_temp
                );
            }
        }
        self.emit_state();
    }

    /// Stop PID auto-tuning for a heating zone
    pub fn stop_heating_autotune(&mut self, zone: api::HeatingZone) {
        match zone {
            api::HeatingZone::Zone1 => {
                self.temperature_controller_1.stop_autotuning();
                tracing::info!("Stopped auto-tuning for zone 1");
            }
            api::HeatingZone::Zone2 => {
                self.temperature_controller_2.stop_autotuning();
                tracing::info!("Stopped auto-tuning for zone 2");
            }
            api::HeatingZone::Zone3 => {
                self.temperature_controller_3.stop_autotuning();
                tracing::info!("Stopped auto-tuning for zone 3");
            }
            api::HeatingZone::Zone4 => {
                self.temperature_controller_4.stop_autotuning();
                tracing::info!("Stopped auto-tuning for zone 4");
            }
            api::HeatingZone::Zone5 => {
                self.temperature_controller_5.stop_autotuning();
                tracing::info!("Stopped auto-tuning for zone 5");
            }
            api::HeatingZone::Zone6 => {
                self.temperature_controller_6.stop_autotuning();
                tracing::info!("Stopped auto-tuning for zone 6");
            }
        }
        self.emit_state();
    }

    /// Check for completed auto-tuning and emit results
    pub fn check_autotuning_results(&mut self) {
        let mut did_complete = false;
        // Check each zone for completed auto-tuning
        if let Some((kp, ki, kd)) = self.temperature_controller_1.get_autotuning_result() {
            tracing::info!(
                "Auto-tuning completed for zone 1: kp={}, ki={}, kd={}",
                kp,
                ki,
                kd
            );
            self.emit_autotuning_complete("zone_1", kp, ki, kd);
            did_complete = true;
        }
        if let Some((kp, ki, kd)) = self.temperature_controller_2.get_autotuning_result() {
            tracing::info!(
                "Auto-tuning completed for zone 2: kp={}, ki={}, kd={}",
                kp,
                ki,
                kd
            );
            self.emit_autotuning_complete("zone_2", kp, ki, kd);
            did_complete = true;
        }
        if let Some((kp, ki, kd)) = self.temperature_controller_3.get_autotuning_result() {
            tracing::info!(
                "Auto-tuning completed for zone 3: kp={}, ki={}, kd={}",
                kp,
                ki,
                kd
            );
            self.emit_autotuning_complete("zone_3", kp, ki, kd);
            did_complete = true;
        }
        if let Some((kp, ki, kd)) = self.temperature_controller_4.get_autotuning_result() {
            tracing::info!(
                "Auto-tuning completed for zone 4: kp={}, ki={}, kd={}",
                kp,
                ki,
                kd
            );
            self.emit_autotuning_complete("zone_4", kp, ki, kd);
            did_complete = true;
        }
        if let Some((kp, ki, kd)) = self.temperature_controller_5.get_autotuning_result() {
            tracing::info!(
                "Auto-tuning completed for zone 5: kp={}, ki={}, kd={}",
                kp,
                ki,
                kd
            );
            self.emit_autotuning_complete("zone_5", kp, ki, kd);
            did_complete = true;
        }
        if let Some((kp, ki, kd)) = self.temperature_controller_6.get_autotuning_result() {
            tracing::info!(
                "Auto-tuning completed for zone 6: kp={}, ki={}, kd={}",
                kp,
                ki,
                kd
            );
            self.emit_autotuning_complete("zone_6", kp, ki, kd);
            did_complete = true;
        }

        if did_complete {
            self.emit_state();
        }
    }

    /// Emit auto-tuning completion event
    fn emit_autotuning_complete(&mut self, zone: &str, kp: f64, ki: f64, kd: f64) {
        let event = api::HeatingAutoTuneCompleteEvent {
            zone: zone.to_string(),
            kp,
            ki,
            kd,
        }
        .build();

        self.namespace
            .emit(api::GluetexEvents::HeatingAutoTuneComplete(event));
    }

    /// Set radius learning rate for adaptive mode
    pub fn spool_set_adaptive_radius_learning_rate(&mut self, radius_learning_rate: f64) {
        self.spool_speed_controller
            .set_adaptive_radius_learning_rate(radius_learning_rate);
        self.emit_state();
    }

    /// Set max speed multiplier for adaptive mode
    pub fn spool_set_adaptive_max_speed_multiplier(&mut self, max_speed_multiplier: f64) {
        self.spool_speed_controller
            .set_adaptive_max_speed_multiplier(max_speed_multiplier);
        self.emit_state();
    }

    /// Set acceleration factor for adaptive mode
    pub fn spool_set_adaptive_acceleration_factor(&mut self, acceleration_factor: f64) {
        self.spool_speed_controller
            .set_adaptive_acceleration_factor(acceleration_factor);
        self.emit_state();
    }

    /// Set deacceleration urgency multiplier for adaptive mode
    pub fn spool_set_adaptive_deacceleration_urgency_multiplier(
        &mut self,
        deacceleration_urgency_multiplier: f64,
    ) {
        self.spool_speed_controller
            .set_adaptive_deacceleration_urgency_multiplier(deacceleration_urgency_multiplier);
        self.emit_state();
    }

    /// Set forward rotation direction
    pub fn spool_set_forward(&mut self, forward: bool) {
        self.spool_speed_controller.set_forward(forward);
        self.emit_state();
    }
}
