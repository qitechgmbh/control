#[cfg(not(feature = "mock-machine"))]
mod winder2_imports {
    pub use super::super::puller_speed_controller::PullerRegulationMode;
    pub use super::super::{TraverseMode, Winder2, Winder2Mode, api, spool_speed_controller};
    pub use crate::buffer1::BufferV1;
    pub use api::{
        LiveValuesEvent, ModeState, PullerState, SpoolAutomaticActionMode,
        SpoolAutomaticActionState, SpoolSpeedControllerState, StateEvent, TensionArmState,
        TraverseState, Winder2Events,
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

#[cfg(not(feature = "mock-machine"))]
pub use winder2_imports::*;

#[cfg(not(feature = "mock-machine"))]
impl Winder2 {
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
            Winder2Mode::Pull => self.calculate_spool_auto_progress_(now),
            Winder2Mode::Wind => self.calculate_spool_auto_progress_(now),
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
                    self.set_mode(&Winder2Mode::Pull);
                }
                SpoolAutomaticActionMode::Hold => {
                    self.stop_or_pull_spool_reset(now);
                    self.set_mode(&Winder2Mode::Hold);
                }
            }
        }
    }
    /// Implement Mode
    pub fn set_mode(&mut self, mode: &Winder2Mode) {
        let should_update = *mode != Winder2Mode::Wind || self.can_wind();

        if should_update {
            // all transitions are allowed
            self.mode = mode.clone();

            // Apply the mode changes to the spool and puller
            self.set_spool_mode(mode);
            self.set_puller_mode(mode);
            self.set_traverse_mode(mode);
        }
        self.emit_state();
    }

    /// Implement Traverse
    pub fn set_laser(&mut self, value: bool) {
        self.laser.set(value);
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

    pub fn emit_live_values(&mut self) {
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
        };

        let event = live_values.build();
        self.namespace.emit(Winder2Events::LiveValues(event));
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
            connected_machine_state: cross_conn,
        }
    }

    pub fn emit_state(&mut self) {
        let state_event = self.build_state_event();
        let event = state_event.build();
        self.namespace.emit(Winder2Events::State(event));
    }

    /// Apply the mode changes to the spool
    ///
    /// It contains a transition matrix for atomic changes.
    /// It will set [`Self::spool_mode`]
    fn set_traverse_mode(&mut self, mode: &Winder2Mode) {
        // Convert to `Winder2Mode` to `TraverseMode`
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
    pub fn puller_set_gear_ratio(&mut self, gear_ratio: super::puller_speed_controller::GearRatio) {
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
