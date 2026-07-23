use super::rewind_control::ArmConfig;
use super::{
    LASER_PORT, PULL_MODE_SOURCE_ASSIST_MAX_RPM, PULL_MODE_SOURCE_ASSIST_RPM_PER_M_PER_MIN,
    PULLER_PORT, RewindPhase, Rewinder, RewinderMode, SOURCE_SPOOL_PORT, TAKEUP_SPOOL_PORT,
    api::{
        HardStopEvent, LiveValuesEvent, ModeState, PrepareControlState, PullerState,
        RewindAutomaticActionState, RewinderEvents, SourceSpoolState, StateEvent, TakeupSpoolState,
        TensionArmControlState, TensionArmState, TraverseState,
    },
};
use crate::winder2::spool_speed_controller::SpoolSpeedControllerType;
use control_core::socketio::namespace::NamespaceCacheingLogic;
use qitech_lib::{
    ethercat_hal::io::digital_output::DigitalOutputDevice,
    units::{
        angular_velocity::revolution_per_minute,
        f64::*,
        length::{meter, millimeter},
        velocity::meter_per_minute,
    },
};
use std::cell::RefMut;
use std::time::Instant;

impl Rewinder {
    pub fn set_mode(&mut self, mode: &RewinderMode) {
        if self.hold_decelerating_from_rewind {
            if matches!(mode, RewinderMode::Hold | RewinderMode::Standby) {
                self.pending_mode_after_rewind_deceleration = Some(mode.clone());
            } else {
                tracing::warn!(
                    "Rewinder rejected {:?}: waiting for rewind deceleration to finish",
                    mode
                );
            }
            self.emit_state();
            return;
        }

        let should_update = match mode {
            RewinderMode::Rewind => self.can_rewind(),
            RewinderMode::Prepare => self.prepare_block_reason().is_none(),
            RewinderMode::Standby | RewinderMode::Hold | RewinderMode::Pull => true,
        };
        if should_update {
            let entering_rewind =
                !matches!(self.mode, RewinderMode::Rewind) && matches!(mode, RewinderMode::Rewind);
            let exiting_rewind =
                matches!(self.mode, RewinderMode::Rewind) && !matches!(mode, RewinderMode::Rewind);
            let entering_pull =
                !matches!(self.mode, RewinderMode::Pull) && matches!(mode, RewinderMode::Pull);
            let entering_prepare = !matches!(self.mode, RewinderMode::Prepare)
                && matches!(mode, RewinderMode::Prepare);
            let entering_hold =
                !matches!(self.mode, RewinderMode::Hold) && matches!(mode, RewinderMode::Hold);
            let hold_from_standby = entering_hold && matches!(self.mode, RewinderMode::Standby);
            if exiting_rewind {
                self.save_current_traverse_as_start_position();
                self.pending_mode_after_rewind_deceleration = Some(mode.clone());
                self.hold_decelerating_from_rewind = true;
                self.capture_hold_deceleration_state();
                self.emit_state();
                return;
            }
            if entering_hold {
                self.rewind_control.reset_motion();
                self.capture_hold_deceleration_state();
            } else if !matches!(mode, RewinderMode::Hold) {
                self.hold_decelerating_from_rewind = false;
                self.pending_mode_after_rewind_deceleration = None;
            }
            self.mode = mode.clone();
            self.rewind_phase = if matches!(mode, RewinderMode::Rewind) {
                RewindPhase::Validate
            } else {
                RewindPhase::Idle
            };
            if entering_rewind {
                let now = Instant::now();
                let zero_speed = Velocity::new::<meter_per_minute>(0.0);
                self.rewind_control.reset_for_rewind(now);
                self.puller_speed_controller.reset_speed(zero_speed);
                self.takeup_spool_speed_controller
                    .set_speed(AngularVelocity::new::<revolution_per_minute>(0.0));
                self.source_spool_speed_controller
                    .set_speed(AngularVelocity::new::<revolution_per_minute>(0.0));
            }
            if entering_pull {
                self.puller_speed_controller
                    .reset_speed(Velocity::new::<meter_per_minute>(0.0));
                self.source_spool_speed_controller
                    .set_speed(AngularVelocity::new::<revolution_per_minute>(0.0));
                let _ = self
                    .source_spool
                    .borrow_mut()
                    .set_speed(SOURCE_SPOOL_PORT, 0.0);
            }
            if entering_prepare {
                let now = Instant::now();
                self.rewind_control.reset_for_prepare(now);
                self.puller_speed_controller
                    .reset_speed(Velocity::new::<meter_per_minute>(0.0));
                self.takeup_spool_speed_controller
                    .set_speed(AngularVelocity::new::<revolution_per_minute>(0.0));
                self.source_spool_speed_controller
                    .set_speed(AngularVelocity::new::<revolution_per_minute>(0.0));
                let _ = self
                    .takeup_spool
                    .borrow_mut()
                    .set_speed(TAKEUP_SPOOL_PORT, 0.0);
                let _ = self
                    .source_spool
                    .borrow_mut()
                    .set_speed(SOURCE_SPOOL_PORT, 0.0);
            }
            self.apply_mode_to_axes(mode);
            if hold_from_standby {
                self.traverse_controller.goto_home();
            }
            if entering_rewind {
                let start_position =
                    self.clamp_traverse_position(self.active_rewind_start_position());
                self.traverse_controller.set_target_position(start_position);
                self.traverse_controller.goto_target_position();
            }
            if matches!(mode, RewinderMode::Pull) {
                self.rewind_control.reset_motion();
            }
        } else if matches!(mode, RewinderMode::Rewind) {
            tracing::warn!(
                "Rewinder rejected Rewind: {}",
                self.rewind_block_reason().unwrap_or("unknown reason")
            );
        } else if matches!(mode, RewinderMode::Prepare) {
            tracing::warn!(
                "Rewinder rejected Prepare: {}",
                self.prepare_block_reason().unwrap_or("unknown reason")
            );
        }
        self.emit_state();
    }

    fn get_laser(&mut self) -> RefMut<'_, dyn DigitalOutputDevice> {
        self.digital_outputs.borrow_mut()
    }

    pub fn set_laser(&mut self, value: bool) {
        self.laser_enabled = value;
        let mut laser = self.get_laser();
        laser.set_output(LASER_PORT, value);
        drop(laser);
        self.emit_state();
    }

    pub(crate) fn save_current_traverse_as_start_position(&mut self) {
        {
            let traverse = &*self.traverse.borrow();
            self.traverse_controller.sync_position(traverse);
        }
        if let Some(position) = self.traverse_controller.get_current_position() {
            self.traverse_start_position = self.clamp_traverse_position(position);
            self.traverse_controller
                .set_target_position(self.traverse_start_position);
        }
    }

    pub(crate) fn save_current_traverse_as_resume_position(&mut self) {
        {
            let traverse = &*self.traverse.borrow();
            self.traverse_controller.sync_position(traverse);
        }
        if let Some(position) = self.traverse_controller.get_current_position() {
            self.resume_traverse_position = Some(self.clamp_traverse_position(position));
        }
    }

    fn capture_hold_deceleration_state(&mut self) {
        let puller_speed = self.measured_puller_line_speed();
        self.rewind_control.puller_command_m_per_min = puller_speed.get::<meter_per_minute>().abs();

        let takeup_steps_per_second = {
            let takeup_spool = &*self.takeup_spool.borrow();
            takeup_spool.get_speed(TAKEUP_SPOOL_PORT)
        };
        self.rewind_control.takeup_follower.command_rpm = self
            .takeup_spool_step_converter
            .steps_to_angular_velocity(takeup_steps_per_second as f64)
            .get::<revolution_per_minute>()
            .abs();

        let source_steps_per_second = {
            let source_spool = &*self.source_spool.borrow();
            source_spool.get_speed(SOURCE_SPOOL_PORT)
        };
        self.rewind_control.source_follower.command_rpm = self
            .source_spool_step_converter
            .steps_to_angular_velocity(source_steps_per_second as f64)
            .get::<revolution_per_minute>()
            .abs();

        self.rewind_control.last_update = Some(Instant::now());
        self.rewind_control.last_dt_s = 0.0;
    }

    fn finish_hold_deceleration_if_stopped(&mut self) {
        if !self.hold_decelerating_from_rewind {
            return;
        }

        let puller_stopped = self.rewind_control.puller_command_m_per_min <= 0.02;
        let takeup_stopped = self.rewind_control.takeup_follower.command_rpm <= 0.5;
        let source_stopped = self.rewind_control.source_follower.command_rpm <= 0.5;
        if puller_stopped && takeup_stopped && source_stopped {
            let target_mode = self
                .pending_mode_after_rewind_deceleration
                .take()
                .unwrap_or(RewinderMode::Hold);
            self.hold_decelerating_from_rewind = false;
            self.mode = target_mode.clone();
            self.rewind_phase = RewindPhase::Idle;
            self.rewind_control.reset_motion();
            if matches!(target_mode, RewinderMode::Prepare) {
                self.rewind_control.reset_for_prepare(Instant::now());
            }
            self.apply_mode_to_axes(&target_mode);
            self.emit_state();
        }
    }

    pub fn sync_puller_speed(&mut self, t: Instant) {
        if !self.update_prepare_control(t) {
            self.update_rewind_sequence(t);
        }

        let angular_velocity = if matches!(self.mode, RewinderMode::Hold) {
            self.rewind_control.decelerate_motion_at(t);
            let speed = self.rewind_control.puller_command_speed();
            let angular_velocity = self
                .puller_speed_controller
                .speed_to_angular_velocity(speed);
            if self.puller_speed_controller.forward {
                angular_velocity
            } else {
                -angular_velocity
            }
        } else if self.puller_speed_output_permitted() {
            if matches!(self.mode, RewinderMode::Rewind | RewinderMode::Prepare) {
                let target_speed = self.puller_speed_controller.get_target_speed();
                self.puller_speed_controller
                    .set_target_speed(self.rewind_control.puller_command_speed());
                let angular_velocity = self.puller_speed_controller.calc_angular_velocity(t);
                self.puller_speed_controller.set_target_speed(target_speed);
                angular_velocity
            } else {
                self.puller_speed_controller.calc_angular_velocity(t)
            }
        } else {
            AngularVelocity::new::<revolution_per_minute>(0.0)
        };
        let actual_line_speed = self.puller_angular_velocity_to_line_speed(angular_velocity);
        if matches!(
            self.rewind_phase,
            RewindPhase::Precharge | RewindPhase::CrawlStart | RewindPhase::Rewind
        ) {
            self.rewind_control.update_followers(
                actual_line_speed.abs(),
                self.takeup_spool_diameter_mm,
                self.source_spool_diameter_mm,
                self.rewind_control.last_dt_s,
            );
        } else if !matches!(self.mode, RewinderMode::Prepare) && !self.hold_decelerating_from_rewind
        {
            self.rewind_control.source_follower.force_zero();
            self.rewind_control.takeup_follower.force_zero();
        }
        self.finish_hold_deceleration_if_stopped();
        let steps_per_second = self
            .puller_speed_controller
            .converter
            .angular_velocity_to_steps(angular_velocity);
        {
            let puller = &mut *self.puller.borrow_mut();
            let _ = puller.set_speed(PULLER_PORT, steps_per_second);
        }
    }

    pub fn sync_takeup_spool_speed(&mut self, t: Instant) {
        let angular_velocity = if matches!(self.mode, RewinderMode::Hold) {
            self.rewind_control.takeup_command_angular_velocity()
        } else if self.takeup_spool_speed_output_permitted() {
            if matches!(self.mode, RewinderMode::Prepare | RewinderMode::Rewind) {
                self.rewind_control.takeup_command_angular_velocity()
            } else {
                let angular_velocity = self.takeup_spool_speed_controller.update_speed(
                    t,
                    &self.takeup_tension_arm,
                    &self.puller_speed_controller,
                );
                angular_velocity
            }
        } else {
            let angular_velocity = AngularVelocity::new::<revolution_per_minute>(0.0);
            self.takeup_spool_speed_controller
                .set_speed(angular_velocity);
            angular_velocity
        };
        if matches!(
            self.mode,
            RewinderMode::Hold | RewinderMode::Prepare | RewinderMode::Rewind
        ) || !self.takeup_spool_speed_output_permitted()
        {
            self.takeup_spool_speed_controller
                .set_speed(angular_velocity);
        }

        let directed_angular_velocity = if self.takeup_spool_speed_controller.get_forward() {
            angular_velocity
        } else {
            -angular_velocity
        };

        let steps_per_second = self
            .takeup_spool_step_converter
            .angular_velocity_to_steps(directed_angular_velocity);
        let takeup_spool = &mut *self.takeup_spool.borrow_mut();
        let _ = takeup_spool.set_speed(TAKEUP_SPOOL_PORT, steps_per_second);
    }

    pub fn sync_source_spool_speed(&mut self, _t: Instant) {
        let angular_velocity = if self.source_spool_speed_output_permitted() {
            if matches!(self.mode, RewinderMode::Pull) {
                AngularVelocity::new::<revolution_per_minute>(self.pull_mode_source_assist_rpm())
            } else if matches!(self.mode, RewinderMode::Hold) {
                self.rewind_control.source_command_angular_velocity()
            } else {
                self.rewind_control.source_command_angular_velocity()
            }
        } else {
            AngularVelocity::new::<revolution_per_minute>(0.0)
        };
        let source_forward = self.takeup_spool_speed_controller.get_forward();
        self.source_spool_speed_controller
            .set_forward(source_forward);

        let directed_angular_velocity = if source_forward {
            angular_velocity
        } else {
            -angular_velocity
        };

        let steps_per_second = self
            .source_spool_step_converter
            .angular_velocity_to_steps(directed_angular_velocity);
        let source_spool = &mut *self.source_spool.borrow_mut();
        let _ = source_spool.set_speed(SOURCE_SPOOL_PORT, steps_per_second);
    }

    fn puller_angular_velocity_to_line_speed(&self, angular_velocity: AngularVelocity) -> Velocity {
        self.puller_speed_controller
            .angular_velocity_to_speed(angular_velocity)
    }

    fn measured_puller_line_speed(&self) -> Velocity {
        let puller_steps_per_second = {
            let puller_ref = &*self.puller.borrow();
            puller_ref.get_speed(PULLER_PORT)
        };
        let puller_angular_velocity = self
            .puller_speed_controller
            .converter
            .steps_to_angular_velocity(puller_steps_per_second as f64);
        self.puller_angular_velocity_to_line_speed(puller_angular_velocity)
    }

    fn pull_mode_source_assist_rpm(&self) -> f64 {
        let puller_line_speed = self.measured_puller_line_speed();
        (puller_line_speed.get::<meter_per_minute>().abs()
            * PULL_MODE_SOURCE_ASSIST_RPM_PER_M_PER_MIN)
            .min(PULL_MODE_SOURCE_ASSIST_MAX_RPM)
    }

    pub fn get_live_values(&self) -> LiveValuesEvent {
        let puller_speed = self.measured_puller_line_speed();

        let takeup_spool_steps_per_second = {
            let takeup_spool_ref = &*self.takeup_spool.borrow();
            takeup_spool_ref.get_speed(TAKEUP_SPOOL_PORT)
        };
        let source_spool_steps_per_second = {
            let source_spool_ref = &*self.source_spool.borrow();
            source_spool_ref.get_speed(SOURCE_SPOOL_PORT)
        };

        LiveValuesEvent {
            traverse_position: self
                .traverse_controller
                .get_current_position()
                .map(|position| position.get::<millimeter>()),
            puller_speed: puller_speed.get::<meter_per_minute>().abs(),
            takeup_spool_rpm: self
                .takeup_spool_step_converter
                .steps_to_angular_velocity(takeup_spool_steps_per_second as f64)
                .get::<revolution_per_minute>()
                .abs(),
            source_spool_rpm: self
                .source_spool_step_converter
                .steps_to_angular_velocity(source_spool_steps_per_second as f64)
                .get::<revolution_per_minute>()
                .abs(),
            takeup_tension_arm_angle: self
                .takeup_tension_arm
                .get_angle()
                .map(Self::normalize_tension_arm_angle_deg)
                .unwrap_or_default(),
            source_tension_arm_angle: self
                .source_tension_arm
                .get_angle()
                .map(Self::normalize_tension_arm_angle_deg)
                .unwrap_or_default(),
            rewind_progress: self.rewind_automatic_action.progress.get::<meter>(),
        }
    }

    pub fn emit_live_values(&mut self) {
        let event = self.get_live_values().build();
        self.namespace.emit(RewinderEvents::LiveValues(event));
    }

    pub(crate) fn emit_hard_stop(&mut self, event: HardStopEvent) {
        self.namespace.emit(RewinderEvents::HardStop(event.build()));
    }

    pub fn build_state_event(&mut self) -> StateEvent {
        let is_default_state = !self.emitted_default_state;
        self.emitted_default_state = true;
        let can_rewind = if self.hold_decelerating_from_rewind {
            false
        } else if matches!(self.mode, RewinderMode::Rewind) {
            self.active_rewind_block_reason().is_none()
        } else {
            self.can_rewind()
        };
        self.last_can_rewind = can_rewind;
        let displayed_mode = if self.hold_decelerating_from_rewind {
            self.pending_mode_after_rewind_deceleration
                .clone()
                .unwrap_or(RewinderMode::Hold)
        } else {
            self.mode.clone()
        };

        StateEvent {
            is_default_state,
            mode_state: ModeState {
                mode: displayed_mode.into(),
                can_rewind,
            },
            traverse_state: TraverseState {
                limit_inner: self
                    .traverse_controller
                    .get_limit_inner()
                    .get::<millimeter>(),
                limit_outer: self
                    .traverse_controller
                    .get_limit_outer()
                    .get::<millimeter>(),
                position_in: 0.0,
                position_out: self
                    .traverse_controller
                    .get_current_position()
                    .map(|position| position.get::<millimeter>())
                    .unwrap_or_default(),
                start_position: self.traverse_start_position.get::<millimeter>(),
                is_going_in: self.traverse_controller.is_going_in(),
                is_going_out: self.traverse_controller.is_going_out(),
                is_going_to_start: self.traverse_controller.is_going_to_target(),
                is_homed: self.traverse_controller.is_homed(),
                is_going_home: self.traverse_controller.is_going_home(),
                is_traversing: self.traverse_controller.is_traversing(),
                step_size: self.traverse_controller.get_step_size().get::<millimeter>(),
                padding: self.traverse_controller.get_padding().get::<millimeter>(),
                laserpointer: self.laser_enabled,
            },
            puller_state: PullerState {
                target_speed: self
                    .puller_speed_controller
                    .get_target_speed()
                    .get::<meter_per_minute>(),
            },
            takeup_spool_state: TakeupSpoolState {
                regulation_mode: self.takeup_spool_speed_controller.get_type().clone(),
                diameter_mm: self.takeup_spool_diameter_mm,
                minmax_min_speed: self
                    .takeup_spool_speed_controller
                    .get_minmax_min_speed()
                    .get::<revolution_per_minute>(),
                minmax_max_speed: self
                    .takeup_spool_speed_controller
                    .get_minmax_max_speed()
                    .get::<revolution_per_minute>(),
                adaptive_tension_target: self
                    .takeup_spool_speed_controller
                    .get_adaptive_tension_target(),
                adaptive_radius_learning_rate: self
                    .takeup_spool_speed_controller
                    .get_adaptive_radius_learning_rate(),
                adaptive_max_speed_multiplier: self
                    .takeup_spool_speed_controller
                    .get_adaptive_max_speed_multiplier(),
                adaptive_acceleration_factor: self
                    .takeup_spool_speed_controller
                    .get_adaptive_acceleration_factor(),
                adaptive_deacceleration_urgency_multiplier: self
                    .takeup_spool_speed_controller
                    .get_adaptive_deacceleration_urgency_multiplier(),
            },
            source_spool_state: SourceSpoolState {
                diameter_mm: self.source_spool_diameter_mm,
                adaptive_tension_target: self
                    .source_spool_speed_controller
                    .get_adaptive_tension_target(),
            },
            rewind_automatic_action_state: RewindAutomaticActionState {
                required_meters: self.rewind_automatic_action.target_length.get::<meter>(),
                mode: self.rewind_automatic_action.mode.clone(),
            },
            takeup_tension_arm_state: TensionArmState {
                zeroed: self.takeup_tension_arm.zeroed,
            },
            source_tension_arm_state: TensionArmState {
                zeroed: self.source_tension_arm.zeroed,
            },
            takeup_tension_arm_control_state: self.rewind_control.config.takeup_arm.into(),
            source_tension_arm_control_state: self.rewind_control.config.source_arm.into(),
            prepare_control_state: PrepareControlState {
                tolerance_angle: self.rewind_control.config.prepare.settle_tolerance_deg,
                settle_rate: self.rewind_control.config.prepare.settle_rate_deg_per_s,
            },
        }
    }

    pub fn emit_state(&mut self) {
        let event = self.build_state_event().build();
        self.namespace.emit(RewinderEvents::State(event));
    }

    pub fn puller_set_target_speed(&mut self, target_speed: f64) {
        self.puller_speed_controller
            .set_target_speed(Velocity::new::<meter_per_minute>(target_speed));
        if !self.puller_motion_permitted() {
            self.puller_speed_controller
                .reset_speed(Velocity::new::<meter_per_minute>(0.0));
        }
        self.emit_state();
    }

    pub fn takeup_spool_set_regulation_mode(&mut self, mode: SpoolSpeedControllerType) {
        if !self.settings_edit_permitted() {
            self.emit_state();
            return;
        }

        self.takeup_spool_speed_controller.set_type(mode);
        self.takeup_spool_speed_controller
            .set_speed(AngularVelocity::new::<revolution_per_minute>(0.0));
        self.emit_state();
    }

    pub fn takeup_spool_set_minmax_min_speed(&mut self, speed_rpm: f64) {
        if !self.settings_edit_permitted() {
            self.emit_state();
            return;
        }

        let speed = AngularVelocity::new::<revolution_per_minute>(speed_rpm);
        if let Err(e) = self
            .takeup_spool_speed_controller
            .set_minmax_min_speed(speed)
        {
            tracing::error!("Failed to set takeup spool min speed: {:?}", e);
        }
        self.emit_state();
    }

    pub fn takeup_spool_set_minmax_max_speed(&mut self, speed_rpm: f64) {
        if !self.settings_edit_permitted() {
            self.emit_state();
            return;
        }

        let speed = AngularVelocity::new::<revolution_per_minute>(speed_rpm);
        if let Err(e) = self
            .takeup_spool_speed_controller
            .set_minmax_max_speed(speed)
        {
            tracing::error!("Failed to set takeup spool max speed: {:?}", e);
        }
        self.emit_state();
    }

    pub fn takeup_spool_set_adaptive_tension_target(&mut self, tension_target: f64) {
        if !self.settings_edit_permitted() {
            self.emit_state();
            return;
        }

        self.takeup_spool_speed_controller
            .set_adaptive_tension_target(tension_target);
        self.emit_state();
    }

    pub fn takeup_spool_set_adaptive_radius_learning_rate(&mut self, value: f64) {
        if !self.settings_edit_permitted() {
            self.emit_state();
            return;
        }

        self.takeup_spool_speed_controller
            .set_adaptive_radius_learning_rate(value);
        self.emit_state();
    }

    pub fn takeup_spool_set_adaptive_max_speed_multiplier(&mut self, value: f64) {
        if !self.settings_edit_permitted() {
            self.emit_state();
            return;
        }

        self.takeup_spool_speed_controller
            .set_adaptive_max_speed_multiplier(value);
        self.emit_state();
    }

    pub fn takeup_spool_set_adaptive_acceleration_factor(&mut self, value: f64) {
        if !self.settings_edit_permitted() {
            self.emit_state();
            return;
        }

        self.takeup_spool_speed_controller
            .set_adaptive_acceleration_factor(value);
        self.emit_state();
    }

    pub fn takeup_spool_set_adaptive_deacceleration_urgency_multiplier(&mut self, value: f64) {
        if !self.settings_edit_permitted() {
            self.emit_state();
            return;
        }

        self.takeup_spool_speed_controller
            .set_adaptive_deacceleration_urgency_multiplier(value);
        self.emit_state();
    }

    pub fn takeup_spool_set_diameter(&mut self, diameter_mm: f64) {
        if self.settings_edit_permitted()
            && diameter_mm.is_finite()
            && (10.0..=500.0).contains(&diameter_mm)
        {
            self.takeup_spool_diameter_mm = Some(diameter_mm);
        }
        self.emit_state();
    }

    pub fn source_spool_set_diameter(&mut self, diameter_mm: f64) {
        if self.settings_edit_permitted()
            && diameter_mm.is_finite()
            && (10.0..=500.0).contains(&diameter_mm)
        {
            self.source_spool_diameter_mm = Some(diameter_mm);
        }
        self.emit_state();
    }

    pub fn source_spool_set_adaptive_tension_target(&mut self, tension_target: f64) {
        if !self.settings_edit_permitted() {
            self.emit_state();
            return;
        }

        self.source_spool_speed_controller
            .set_adaptive_tension_target(tension_target);
        self.emit_state();
    }

    pub fn set_takeup_tension_arm_control(&mut self, state: TensionArmControlState) {
        self.set_tension_arm_control(false, state);
    }

    pub fn set_source_tension_arm_control(&mut self, state: TensionArmControlState) {
        self.set_tension_arm_control(true, state);
    }

    fn set_tension_arm_control(&mut self, source: bool, state: TensionArmControlState) {
        if !self.settings_edit_permitted() {
            self.emit_state();
            return;
        }

        let current = if source {
            self.rewind_control.config.source_arm
        } else {
            self.rewind_control.config.takeup_arm
        };

        let Some(config) = build_arm_config(current, state) else {
            self.emit_state();
            return;
        };

        if source {
            self.rewind_control.config.source_arm = config;
        } else {
            self.rewind_control.config.takeup_arm = config;
        }
        self.emit_state();
    }

    pub fn set_prepare_control(&mut self, state: PrepareControlState) {
        if self.settings_edit_permitted()
            && (1.0..=20.0).contains(&state.tolerance_angle)
            && (0.1..=30.0).contains(&state.settle_rate)
        {
            self.rewind_control.config.prepare.settle_tolerance_deg = state.tolerance_angle;
            self.rewind_control.config.prepare.settle_rate_deg_per_s = state.settle_rate;
        }
        self.emit_state();
    }

    fn settings_edit_permitted(&self) -> bool {
        matches!(self.mode, RewinderMode::Standby | RewinderMode::Hold)
    }

    pub fn takeup_tension_arm_zero(&mut self) {
        self.takeup_tension_arm.zero();
        self.emit_live_values();
        self.emit_state();
    }

    pub fn source_tension_arm_zero(&mut self) {
        self.source_tension_arm.zero();
        self.emit_live_values();
        self.emit_state();
    }

    pub fn traverse_set_limit_inner(&mut self, limit: f64) {
        if !self.settings_edit_permitted() {
            self.emit_state();
            return;
        }

        let new_inner = Length::new::<millimeter>(limit);
        let current_outer = self.traverse_controller.get_limit_outer();
        if Self::validate_traverse_limits(new_inner, current_outer) {
            self.traverse_controller.set_limit_inner(new_inner);
            self.traverse_start_position =
                self.clamp_traverse_position(self.traverse_start_position);
            self.resume_traverse_position = self
                .resume_traverse_position
                .map(|position| self.clamp_traverse_position(position));
            self.traverse_controller
                .set_target_position(self.traverse_start_position);
        }
        self.emit_state();
    }

    pub fn traverse_set_limit_outer(&mut self, limit: f64) {
        if !self.settings_edit_permitted() {
            self.emit_state();
            return;
        }

        let new_outer = Length::new::<millimeter>(limit);
        let current_inner = self.traverse_controller.get_limit_inner();
        if Self::validate_traverse_limits(current_inner, new_outer) {
            self.traverse_controller.set_limit_outer(new_outer);
            self.traverse_start_position =
                self.clamp_traverse_position(self.traverse_start_position);
            self.resume_traverse_position = self
                .resume_traverse_position
                .map(|position| self.clamp_traverse_position(position));
            self.traverse_controller
                .set_target_position(self.traverse_start_position);
        }
        self.emit_state();
    }

    pub fn traverse_set_start_position(&mut self, position: f64) {
        if !self.settings_edit_permitted() {
            self.emit_state();
            return;
        }

        let position = Length::new::<millimeter>(position);
        self.traverse_start_position = self.clamp_traverse_position(position);
        self.resume_traverse_position = None;
        self.traverse_controller
            .set_target_position(self.traverse_start_position);
        self.emit_state();
    }

    pub fn traverse_set_step_size(&mut self, step_size: f64) {
        if !self.settings_edit_permitted() {
            self.emit_state();
            return;
        }

        self.traverse_controller
            .set_step_size(Length::new::<millimeter>(step_size));
        self.emit_state();
    }

    pub fn traverse_set_padding(&mut self, padding: f64) {
        if !self.settings_edit_permitted() {
            self.emit_state();
            return;
        }

        self.traverse_controller
            .set_padding(Length::new::<millimeter>(padding));
        self.emit_state();
    }

    pub fn traverse_goto_limit_inner(&mut self) {
        self.traverse_controller.goto_limit_inner();
        self.emit_state();
    }

    pub fn traverse_goto_limit_outer(&mut self) {
        self.traverse_controller.goto_limit_outer();
        self.emit_state();
    }

    pub fn traverse_goto_start_position(&mut self) {
        self.resume_traverse_position = None;
        self.traverse_controller
            .set_target_position(self.traverse_start_position);
        self.traverse_controller.goto_target_position();
        self.emit_state();
    }

    pub fn traverse_goto_home(&mut self) {
        self.traverse_controller.goto_home();
        self.emit_state();
    }
}

fn build_arm_config(current: ArmConfig, state: TensionArmControlState) -> Option<ArmConfig> {
    current
        .with_hard_range(state.hard_min_angle, state.hard_max_angle)?
        .with_start_range(state.start_min_angle, state.start_max_angle)?
        .with_target(state.target_angle)
}

impl From<ArmConfig> for TensionArmControlState {
    fn from(config: ArmConfig) -> Self {
        Self {
            hard_min_angle: config.hard_min_deg,
            hard_max_angle: config.hard_max_deg,
            start_min_angle: config.start_min_deg,
            start_max_angle: config.start_max_deg,
            target_angle: config.target_deg,
        }
    }
}
