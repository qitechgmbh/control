use super::{
    api::{
        LiveValuesEvent, ModeState, PullerState, RewinderEvents, SourceSpoolState, StateEvent,
        TakeupSpoolState, TensionArmState, TraverseState,
    },
    rewind_control::ArmConfig,
    RewindPhase, Rewinder, RewinderMode, PULLER_PORT, SOURCE_SPOOL_PORT, TAKEUP_SPOOL_PORT,
    TRAVERSE_PORT,
};
use crate::winder2::{
    puller_speed_controller::GearRatio, spool_speed_controller::SpoolSpeedControllerType,
};
use control_core::socketio::{event::BuildEvent, namespace::NamespaceCacheingLogic};
use qitech_lib::units::{
    angle::degree, angular_velocity::revolution_per_minute, f64::*, length::millimeter,
    velocity::meter_per_minute,
};
use std::time::Instant;

impl Rewinder {
    pub(crate) const SOURCE_TENSION_ARM_MIN_ANGLE_DEG: f64 = ArmConfig::SOURCE.hard_min_deg;
    pub(crate) const SOURCE_TENSION_ARM_MAX_ANGLE_DEG: f64 = ArmConfig::SOURCE.hard_max_deg;
    pub(crate) const TAKEUP_TENSION_ARM_MIN_ANGLE_DEG: f64 = ArmConfig::TAKEUP.hard_min_deg;
    pub(crate) const TAKEUP_TENSION_ARM_MAX_ANGLE_DEG: f64 = ArmConfig::TAKEUP.hard_max_deg;
    pub(crate) const SOURCE_TENSION_ARM_START_MIN_ANGLE_DEG: f64 = ArmConfig::SOURCE.start_min_deg;
    pub(crate) const SOURCE_TENSION_ARM_START_MAX_ANGLE_DEG: f64 = ArmConfig::SOURCE.start_max_deg;
    pub(crate) const TAKEUP_TENSION_ARM_START_MIN_ANGLE_DEG: f64 = ArmConfig::TAKEUP.start_min_deg;
    pub(crate) const TAKEUP_TENSION_ARM_START_MAX_ANGLE_DEG: f64 = ArmConfig::TAKEUP.start_max_deg;

    pub(crate) fn normalize_tension_arm_angle_deg(angle: Angle) -> f64 {
        let angle_deg = angle.get::<degree>();
        if angle_deg >= 270.0 {
            angle_deg - 360.0
        } else {
            angle_deg
        }
    }

    fn read_tension_arm_angles_deg(&self) -> Result<(f64, f64), &'static str> {
        let source_angle = self
            .source_tension_arm
            .get_angle()
            .map(Self::normalize_tension_arm_angle_deg)
            .map_err(|_| "failed to read source tension arm angle")?;
        let takeup_angle = self
            .takeup_tension_arm
            .get_angle()
            .map(Self::normalize_tension_arm_angle_deg)
            .map_err(|_| "failed to read takeup tension arm angle")?;
        Ok((source_angle, takeup_angle))
    }

    fn set_rewind_phase(&mut self, phase: RewindPhase, reason: &str) {
        if self.rewind_phase != phase {
            println!(
                "Rewinder phase {:?} -> {:?}: {}",
                self.rewind_phase, phase, reason
            );
            self.rewind_control.start_phase(Instant::now());
        }
        if matches!(phase, RewindPhase::FaultHold) {
            self.rewind_hard_stop_reason = Some(reason.to_owned());
        } else {
            self.rewind_hard_stop_reason = None;
        }
        self.rewind_phase = phase;
    }

    fn prepare_rewind_control(&mut self, now: Instant) {
        if !matches!(self.mode, RewinderMode::Rewind) {
            if !matches!(self.rewind_phase, RewindPhase::Idle) {
                self.set_rewind_phase(RewindPhase::Idle, "mode is not Rewind");
            }
            self.rewind_control.reset_motion();
            self.rewind_puller_command_speed = None;
            return;
        }

        if matches!(self.rewind_phase, RewindPhase::FaultHold) {
            self.rewind_control.reset_motion();
            self.rewind_puller_command_speed = Some(Velocity::new::<meter_per_minute>(0.0));
            return;
        }

        if let Some(reason) = self.runtime_rewind_block_reason() {
            self.set_rewind_phase(RewindPhase::FaultHold, reason);
            self.rewind_control.reset_motion();
            self.rewind_puller_command_speed = Some(Velocity::new::<meter_per_minute>(0.0));
            return;
        }

        let Ok((source_angle, takeup_angle)) = self.read_tension_arm_angles_deg() else {
            self.set_rewind_phase(RewindPhase::FaultHold, "failed to read tension arm angle");
            self.rewind_control.reset_motion();
            self.rewind_puller_command_speed = Some(Velocity::new::<meter_per_minute>(0.0));
            return;
        };

        let dt_s = self
            .rewind_control
            .update_arms(source_angle, takeup_angle, now)
            .max(0.0);

        if self.rewind_control.source_arm.zone.is_fault() {
            self.set_rewind_phase(
                RewindPhase::FaultHold,
                "source tension arm is outside rewind range",
            );
        } else if self.rewind_control.takeup_arm.zone.is_fault() {
            self.set_rewind_phase(
                RewindPhase::FaultHold,
                "takeup tension arm is outside rewind range",
            );
        }

        if matches!(self.rewind_phase, RewindPhase::FaultHold) {
            self.rewind_control.reset_motion();
            self.rewind_puller_command_speed = Some(Velocity::new::<meter_per_minute>(0.0));
            return;
        }

        match self.rewind_phase {
            RewindPhase::Idle => self.set_rewind_phase(RewindPhase::Validate, "rewind requested"),
            RewindPhase::Validate => {
                let source_ok = self
                    .rewind_control
                    .config
                    .source_arm
                    .in_start_range(source_angle);
                let takeup_ok = self
                    .rewind_control
                    .config
                    .takeup_arm
                    .in_start_range(takeup_angle);
                if source_ok && takeup_ok {
                    self.set_rewind_phase(RewindPhase::Precharge, "start angles validated");
                }
            }
            RewindPhase::Precharge => {
                if self.rewind_control.phase_elapsed(now)
                    >= self.rewind_control.config.precharge_duration
                {
                    self.set_rewind_phase(RewindPhase::CrawlStart, "precharge settled");
                }
            }
            RewindPhase::CrawlStart => {
                if self.rewind_control.phase_elapsed(now)
                    >= self.rewind_control.config.crawl_duration
                {
                    self.set_rewind_phase(RewindPhase::Rewind, "crawl start complete");
                }
            }
            RewindPhase::Rewind | RewindPhase::FaultHold => {}
        }

        let ui_target_m_per_min = self
            .puller_speed_controller
            .get_target_speed()
            .get::<meter_per_minute>();
        let commanded_target = match self.rewind_phase {
            RewindPhase::Precharge | RewindPhase::Validate | RewindPhase::Idle => 0.0,
            RewindPhase::CrawlStart => ui_target_m_per_min
                .min(self.rewind_control.config.puller_ramp.crawl_speed_m_per_min),
            RewindPhase::Rewind => ui_target_m_per_min,
            RewindPhase::FaultHold => 0.0,
        };

        self.rewind_control
            .update_puller_command(commanded_target, dt_s);
        self.rewind_puller_command_speed = Some(Velocity::new::<meter_per_minute>(
            self.rewind_control.puller_command_m_per_min,
        ));
    }

    pub fn set_mode(&mut self, mode: &RewinderMode) {
        let should_update = *mode != RewinderMode::Rewind || self.can_rewind();
        if should_update {
            let entering_rewind =
                !matches!(self.mode, RewinderMode::Rewind) && matches!(mode, RewinderMode::Rewind);
            self.mode = mode.clone();
            self.rewind_phase = if matches!(mode, RewinderMode::Rewind) {
                RewindPhase::Validate
            } else {
                RewindPhase::Idle
            };
            if entering_rewind {
                let now = Instant::now();
                let zero_speed = Velocity::new::<meter_per_minute>(0.0);
                self.rewind_puller_command_speed = Some(zero_speed);
                self.rewind_control.reset_for_rewind(now);
                self.puller_speed_controller.reset_speed(zero_speed);
                self.takeup_spool_speed_controller
                    .set_speed(AngularVelocity::new::<revolution_per_minute>(0.0));
                self.source_spool_speed_controller
                    .set_speed(AngularVelocity::new::<revolution_per_minute>(0.0));
            }
            self.apply_axis_modes();
            if matches!(mode, RewinderMode::Pull) {
                self.rewind_control.reset_motion();
                self.source_spool_speed_controller.set_enabled(false);
                let _ = self
                    .source_spool
                    .borrow_mut()
                    .set_speed(SOURCE_SPOOL_PORT, 0.0);
                self.source_spool
                    .borrow_mut()
                    .set_enabled(SOURCE_SPOOL_PORT, false);
            }
        } else if matches!(mode, RewinderMode::Rewind) {
            println!(
                "Rewinder rejected Rewind: {}",
                self.rewind_block_reason().unwrap_or("unknown reason")
            );
        }
        self.emit_state();
    }

    fn apply_axis_modes(&mut self) {
        let puller_enabled = matches!(
            self.mode,
            RewinderMode::Hold | RewinderMode::Pull | RewinderMode::Rewind
        );
        let controller_enabled = matches!(self.mode, RewinderMode::Pull | RewinderMode::Rewind);
        let takeup_spool_enabled = matches!(self.mode, RewinderMode::Hold | RewinderMode::Rewind);
        let source_spool_enabled = matches!(self.mode, RewinderMode::Hold | RewinderMode::Rewind);
        let traverse_enabled = matches!(self.mode, RewinderMode::Hold | RewinderMode::Rewind);

        self.puller
            .borrow_mut()
            .set_enabled(PULLER_PORT, puller_enabled);
        self.puller_speed_controller.set_enabled(controller_enabled);

        self.takeup_spool
            .borrow_mut()
            .set_enabled(TAKEUP_SPOOL_PORT, takeup_spool_enabled);
        self.source_spool
            .borrow_mut()
            .set_enabled(SOURCE_SPOOL_PORT, source_spool_enabled);
        self.takeup_spool_speed_controller
            .set_enabled(matches!(self.mode, RewinderMode::Rewind));
        self.source_spool_speed_controller
            .set_enabled(matches!(self.mode, RewinderMode::Rewind));

        self.traverse
            .borrow_mut()
            .set_enabled(TRAVERSE_PORT, traverse_enabled);
        self.traverse_controller.set_enabled(traverse_enabled);
        if matches!(self.mode, RewinderMode::Hold) {
            self.traverse_controller.goto_home();
        }
        if matches!(self.mode, RewinderMode::Rewind) {
            self.traverse_controller.start_traversing();
        }
    }

    pub fn sync_puller_speed(&mut self, t: Instant) {
        self.prepare_rewind_control(t);

        let angular_velocity = if self.puller_motion_permitted() {
            if matches!(self.mode, RewinderMode::Rewind) {
                let target_speed = self.puller_speed_controller.get_target_speed();
                let command_speed =
                    Velocity::new::<meter_per_minute>(self.rewind_control.puller_command_m_per_min);
                self.puller_speed_controller.set_target_speed(command_speed);
                let angular_velocity = self.puller_speed_controller.calc_angular_velocity(t);
                self.puller_speed_controller.set_target_speed(target_speed);
                angular_velocity
            } else {
                self.rewind_puller_command_speed = None;
                self.puller_speed_controller.calc_angular_velocity(t)
            }
        } else {
            self.rewind_puller_command_speed = None;
            AngularVelocity::new::<revolution_per_minute>(0.0)
        };
        let actual_line_speed = self
            .puller_speed_controller
            .angular_velocity_to_speed(angular_velocity)
            / self.puller_speed_controller.get_gear_ratio().multiplier();
        let actual_line_speed_m_per_min = actual_line_speed.get::<meter_per_minute>().abs();
        if matches!(
            self.rewind_phase,
            RewindPhase::Precharge | RewindPhase::CrawlStart | RewindPhase::Rewind
        ) {
            self.rewind_control
                .update_followers(actual_line_speed_m_per_min, self.rewind_control.last_dt_s);
        } else {
            self.rewind_control.source_follower.force_zero();
            self.rewind_control.takeup_follower.force_zero();
        }
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
        let angular_velocity = if self.takeup_spool_motion_permitted() {
            let target_speed = self.puller_speed_controller.get_target_speed();
            if matches!(self.mode, RewinderMode::Rewind) {
                let command_speed =
                    Velocity::new::<meter_per_minute>(self.rewind_control.puller_command_m_per_min);
                self.puller_speed_controller.set_target_speed(command_speed);
            }
            let angular_velocity = self.takeup_spool_speed_controller.update_speed(
                t,
                &self.takeup_tension_arm,
                &self.puller_speed_controller,
            );
            if matches!(self.mode, RewinderMode::Rewind) {
                self.puller_speed_controller.set_target_speed(target_speed);
            }
            angular_velocity
        } else {
            let angular_velocity = AngularVelocity::new::<revolution_per_minute>(0.0);
            self.takeup_spool_speed_controller
                .set_speed(angular_velocity);
            angular_velocity
        };

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
        let angular_velocity = if self.source_spool_motion_permitted() {
            AngularVelocity::new::<revolution_per_minute>(
                self.rewind_control.source_follower.command_rpm,
            )
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

    pub fn get_live_values(&self) -> LiveValuesEvent {
        let puller_steps_per_second = {
            let puller_ref = &*self.puller.borrow();
            puller_ref.get_speed(PULLER_PORT)
        };
        let puller_angular_velocity = self
            .puller_speed_controller
            .converter
            .steps_to_angular_velocity(puller_steps_per_second as f64);
        let motor_speed = self
            .puller_speed_controller
            .angular_velocity_to_speed(puller_angular_velocity);
        let puller_speed = motor_speed / self.puller_speed_controller.get_gear_ratio().multiplier();

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
        }
    }

    pub fn log_rewind_diagnostics(&mut self, now: Instant) {
        if !matches!(self.mode, RewinderMode::Rewind) {
            return;
        }
        if now
            .duration_since(self.last_rewind_diagnostics_log)
            .as_secs_f64()
            < 0.5
        {
            return;
        }
        self.last_rewind_diagnostics_log = now;

        let live_values = self.get_live_values();
        println!(
            "Rewinder diag: phase={:?} puller_target={:.2}m/min puller_command={:.2}m/min puller_accel={:.2}m/min/s puller_actual={:.2}m/min takeup_angle={:.1}deg takeup_filtered={:.1}deg takeup_zone={:?} takeup_rate={:.1}deg/s takeup_controller={:.1}rpm source_angle={:.1}deg source_filtered={:.1}deg source_zone={:?} source_rate={:.1}deg/s source_recovery={} source_ff={:.1}rpm source_trim={:.1}rpm source_target={:.1}rpm source_cmd={:.1}rpm source_ratio={:.2} takeup_actual={:.1}rpm source_actual={:.1}rpm can_rewind={} reason={}",
            self.rewind_phase,
            self.puller_speed_controller
                .get_target_speed()
                .get::<meter_per_minute>(),
            self.rewind_control.puller_command_m_per_min,
            self.rewind_control.puller_accel_m_per_min_s,
            live_values.puller_speed,
            live_values.takeup_tension_arm_angle,
            self.rewind_control.takeup_arm.filtered_deg,
            self.rewind_control.takeup_arm.zone,
            self.rewind_control.takeup_arm.rate_deg_per_s,
            self.takeup_spool_speed_controller
                .get_speed()
                .get::<revolution_per_minute>(),
            live_values.source_tension_arm_angle,
            self.rewind_control.source_arm.filtered_deg,
            self.rewind_control.source_arm.zone,
            self.rewind_control.source_arm.rate_deg_per_s,
            self.rewind_control.source_recovery_active(),
            self.rewind_control.source_follower.feed_forward_rpm,
            self.rewind_control.source_follower.trim_rpm,
            self.rewind_control.source_follower.target_rpm,
            self.rewind_control.source_follower.command_rpm,
            self.rewind_control.source_follower.ratio_rpm_per_m_per_min,
            live_values.takeup_spool_rpm,
            live_values.source_spool_rpm,
            !matches!(self.rewind_phase, RewindPhase::FaultHold),
            if matches!(self.rewind_phase, RewindPhase::FaultHold) {
                self.rewind_hard_stop_reason
                    .as_deref()
                    .unwrap_or("runtime hard stop")
            } else {
                "ok"
            }
        );
    }

    pub fn emit_live_values(&mut self) {
        let event = self.get_live_values().build();
        self.namespace.emit(RewinderEvents::LiveValues(event));
    }

    pub fn build_state_event(&mut self) -> StateEvent {
        let is_default_state = !self.emitted_default_state;
        self.emitted_default_state = true;
        let can_rewind = if matches!(self.mode, RewinderMode::Rewind) {
            self.runtime_rewind_block_reason().is_none()
                && !matches!(self.rewind_phase, RewindPhase::FaultHold)
        } else {
            self.can_rewind()
        };
        self.last_can_rewind = can_rewind;

        StateEvent {
            is_default_state,
            mode_state: ModeState {
                mode: self.mode.clone().into(),
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
                is_going_in: self.traverse_controller.is_going_in(),
                is_going_out: self.traverse_controller.is_going_out(),
                is_homed: self.traverse_controller.is_homed(),
                is_going_home: self.traverse_controller.is_going_home(),
                is_traversing: self.traverse_controller.is_traversing(),
                step_size: self.traverse_controller.get_step_size().get::<millimeter>(),
                padding: self.traverse_controller.get_padding().get::<millimeter>(),
            },
            puller_state: PullerState {
                target_speed: self
                    .puller_speed_controller
                    .get_target_speed()
                    .get::<meter_per_minute>(),
                gear_ratio: self.puller_speed_controller.get_gear_ratio(),
            },
            takeup_spool_state: TakeupSpoolState {
                regulation_mode: self.takeup_spool_speed_controller.get_type().clone(),
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
                adaptive_tension_target: self
                    .source_spool_speed_controller
                    .get_adaptive_tension_target(),
            },
            takeup_tension_arm_state: TensionArmState {
                zeroed: self.takeup_tension_arm.zeroed,
            },
            source_tension_arm_state: TensionArmState {
                zeroed: self.source_tension_arm.zeroed,
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
        self.emit_state();
    }

    pub fn puller_set_gear_ratio(&mut self, gear_ratio: GearRatio) {
        self.puller_speed_controller.set_gear_ratio(gear_ratio);
        self.puller_speed_controller
            .set_target_speed(Velocity::new::<meter_per_minute>(0.0));
        self.emit_state();
    }

    pub fn takeup_spool_set_regulation_mode(&mut self, mode: SpoolSpeedControllerType) {
        self.takeup_spool_speed_controller.set_type(mode);
        self.emit_state();
    }

    pub fn takeup_spool_set_minmax_min_speed(&mut self, speed_rpm: f64) {
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
        self.takeup_spool_speed_controller
            .set_adaptive_tension_target(tension_target);
        self.emit_state();
    }

    pub fn takeup_spool_set_adaptive_radius_learning_rate(&mut self, value: f64) {
        self.takeup_spool_speed_controller
            .set_adaptive_radius_learning_rate(value);
        self.emit_state();
    }

    pub fn takeup_spool_set_adaptive_max_speed_multiplier(&mut self, value: f64) {
        self.takeup_spool_speed_controller
            .set_adaptive_max_speed_multiplier(value);
        self.emit_state();
    }

    pub fn takeup_spool_set_adaptive_acceleration_factor(&mut self, value: f64) {
        self.takeup_spool_speed_controller
            .set_adaptive_acceleration_factor(value);
        self.emit_state();
    }

    pub fn takeup_spool_set_adaptive_deacceleration_urgency_multiplier(&mut self, value: f64) {
        self.takeup_spool_speed_controller
            .set_adaptive_deacceleration_urgency_multiplier(value);
        self.emit_state();
    }

    pub fn source_spool_set_adaptive_tension_target(&mut self, tension_target: f64) {
        self.source_spool_speed_controller
            .set_adaptive_tension_target(tension_target);
        self.emit_state();
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
        let new_inner = Length::new::<millimeter>(limit);
        let current_outer = self.traverse_controller.get_limit_outer();
        if Self::validate_traverse_limits(new_inner, current_outer) {
            self.traverse_controller.set_limit_inner(new_inner);
        }
        self.emit_state();
    }

    pub fn traverse_set_limit_outer(&mut self, limit: f64) {
        let new_outer = Length::new::<millimeter>(limit);
        let current_inner = self.traverse_controller.get_limit_inner();
        if Self::validate_traverse_limits(current_inner, new_outer) {
            self.traverse_controller.set_limit_outer(new_outer);
        }
        self.emit_state();
    }

    pub fn traverse_set_step_size(&mut self, step_size: f64) {
        self.traverse_controller
            .set_step_size(Length::new::<millimeter>(step_size));
        self.emit_state();
    }

    pub fn traverse_set_padding(&mut self, padding: f64) {
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

    pub fn traverse_goto_home(&mut self) {
        self.traverse_controller.goto_home();
        self.emit_state();
    }
}
