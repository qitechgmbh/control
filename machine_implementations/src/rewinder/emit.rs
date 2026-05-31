use super::{
    api::{
        LiveValuesEvent, ModeState, PullerState, RewinderEvents, SourceSpoolState, StateEvent,
        TakeupSpoolState, TensionArmState, TraverseState,
    },
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
    pub(crate) const SOURCE_TENSION_ARM_MIN_ANGLE_DEG: f64 = 20.0;
    pub(crate) const SOURCE_TENSION_ARM_MAX_ANGLE_DEG: f64 = 60.0;
    pub(crate) const SOURCE_TENSION_ARM_SOFT_HIGH_ANGLE_DEG: f64 = 50.0;
    pub(crate) const TAKEUP_TENSION_ARM_MIN_ANGLE_DEG: f64 = 20.0;
    pub(crate) const TAKEUP_TENSION_ARM_LOW_EXIT_ANGLE_DEG: f64 = 25.0;
    pub(crate) const TAKEUP_TENSION_ARM_MAX_ANGLE_DEG: f64 = 90.0;
    const SOURCE_TENSION_HARD_HIGH_SOURCE_FEED_RPM: f64 = 30.0;
    const TAKEUP_TENSION_LOW_TAKEUP_FEED_RPM: f64 = 20.0;

    pub fn set_mode(&mut self, mode: &RewinderMode) {
        let should_update = *mode != RewinderMode::Rewind || self.can_rewind();
        if should_update {
            let entering_rewind =
                !matches!(self.mode, RewinderMode::Rewind) && matches!(mode, RewinderMode::Rewind);
            self.mode = mode.clone();
            self.rewind_phase = if entering_rewind {
                RewindPhase::StartPulling
            } else if matches!(self.mode, RewinderMode::Rewind) {
                self.rewind_phase
            } else {
                RewindPhase::Idle
            };
            self.apply_axis_modes();
        } else if matches!(mode, RewinderMode::Rewind) {
            println!(
                "Rewinder rejected Rewind: {}",
                self.rewind_block_reason().unwrap_or("unknown reason")
            );
        }
        self.emit_state();
    }

    fn set_rewind_phase(&mut self, phase: RewindPhase, reason: &str) {
        if self.rewind_phase != phase {
            println!(
                "Rewinder phase {:?} -> {:?}: {}",
                self.rewind_phase, phase, reason
            );
        }
        self.rewind_phase = phase;
    }

    fn apply_axis_modes(&mut self) {
        let puller_enabled = matches!(
            self.mode,
            RewinderMode::Hold | RewinderMode::Pull | RewinderMode::Rewind
        );
        let controller_enabled = matches!(self.mode, RewinderMode::Pull | RewinderMode::Rewind);
        let spool_enabled = matches!(self.mode, RewinderMode::Hold | RewinderMode::Rewind);
        let traverse_enabled = matches!(self.mode, RewinderMode::Hold | RewinderMode::Rewind);

        self.puller
            .borrow_mut()
            .set_enabled(PULLER_PORT, puller_enabled);
        self.puller_speed_controller.set_enabled(controller_enabled);

        self.takeup_spool
            .borrow_mut()
            .set_enabled(TAKEUP_SPOOL_PORT, spool_enabled);
        self.source_spool
            .borrow_mut()
            .set_enabled(SOURCE_SPOOL_PORT, spool_enabled);
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

    fn update_rewind_phase(&mut self) {
        if !matches!(self.mode, RewinderMode::Rewind) {
            self.set_rewind_phase(RewindPhase::Idle, "mode is not Rewind");
            return;
        }

        if matches!(self.rewind_phase, RewindPhase::HardStop) {
            return;
        }

        let Ok(source_angle) = self.source_tension_arm.get_angle() else {
            self.set_rewind_phase(
                RewindPhase::HardStop,
                "failed to read source tension arm angle",
            );
            return;
        };
        let source_angle = source_angle.get::<degree>();

        let Ok(takeup_angle) = self.takeup_tension_arm.get_angle() else {
            self.set_rewind_phase(
                RewindPhase::HardStop,
                "failed to read takeup tension arm angle",
            );
            return;
        };
        let takeup_angle = takeup_angle.get::<degree>();

        if let Some(reason) = self.rewind_block_reason() {
            self.set_rewind_phase(RewindPhase::HardStop, reason);
            return;
        }

        if takeup_angle > Self::TAKEUP_TENSION_ARM_MAX_ANGLE_DEG {
            self.set_rewind_phase(
                RewindPhase::HardStop,
                "takeup tension arm exceeded maximum angle",
            );
            return;
        }

        if source_angle > Self::SOURCE_TENSION_ARM_MAX_ANGLE_DEG {
            self.set_rewind_phase(
                RewindPhase::SourceHigh,
                "source tension arm exceeded maximum angle",
            );
            return;
        }

        let next_phase = match self.rewind_phase {
            RewindPhase::Idle => RewindPhase::StartPulling,
            RewindPhase::StartPulling => {
                if source_angle < Self::SOURCE_TENSION_ARM_MIN_ANGLE_DEG {
                    RewindPhase::StartPulling
                } else if takeup_angle < Self::TAKEUP_TENSION_ARM_MIN_ANGLE_DEG {
                    RewindPhase::TakeupLow
                } else {
                    RewindPhase::Normal
                }
            }
            RewindPhase::Normal => {
                if takeup_angle < Self::TAKEUP_TENSION_ARM_MIN_ANGLE_DEG {
                    RewindPhase::TakeupLow
                } else if source_angle < Self::SOURCE_TENSION_ARM_MIN_ANGLE_DEG {
                    RewindPhase::SourceLow
                } else {
                    RewindPhase::Normal
                }
            }
            RewindPhase::SourceLow => {
                if takeup_angle < Self::TAKEUP_TENSION_ARM_MIN_ANGLE_DEG {
                    RewindPhase::TakeupLow
                } else if source_angle >= Self::SOURCE_TENSION_ARM_MIN_ANGLE_DEG {
                    RewindPhase::Normal
                } else {
                    RewindPhase::SourceLow
                }
            }
            RewindPhase::SourceHigh => {
                if source_angle <= Self::SOURCE_TENSION_ARM_SOFT_HIGH_ANGLE_DEG {
                    if source_angle < Self::SOURCE_TENSION_ARM_MIN_ANGLE_DEG {
                        RewindPhase::SourceLow
                    } else if takeup_angle < Self::TAKEUP_TENSION_ARM_MIN_ANGLE_DEG {
                        RewindPhase::TakeupLow
                    } else {
                        RewindPhase::Normal
                    }
                } else {
                    RewindPhase::SourceHigh
                }
            }
            RewindPhase::TakeupLow => {
                if takeup_angle >= Self::TAKEUP_TENSION_ARM_LOW_EXIT_ANGLE_DEG {
                    if source_angle < Self::SOURCE_TENSION_ARM_MIN_ANGLE_DEG {
                        RewindPhase::SourceLow
                    } else {
                        RewindPhase::Normal
                    }
                } else {
                    RewindPhase::TakeupLow
                }
            }
            RewindPhase::HardStop => RewindPhase::HardStop,
        };
        self.set_rewind_phase(next_phase, "tension range update");
    }

    pub fn sync_puller_speed(&mut self, t: Instant) {
        self.update_rewind_phase();

        let angular_velocity = if self.puller_motion_permitted() {
            self.puller_speed_controller.calc_angular_velocity(t)
        } else {
            AngularVelocity::new::<revolution_per_minute>(0.0)
        };
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
        let angular_velocity = if matches!(self.rewind_phase, RewindPhase::TakeupLow) {
            AngularVelocity::new::<revolution_per_minute>(
                Self::TAKEUP_TENSION_LOW_TAKEUP_FEED_RPM,
            )
        } else if self.takeup_spool_motion_permitted() {
            self.takeup_spool_speed_controller.update_speed(
                t,
                &self.takeup_tension_arm,
                &self.puller_speed_controller,
            )
        } else {
            AngularVelocity::new::<revolution_per_minute>(0.0)
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

    pub fn sync_source_spool_speed(&mut self, t: Instant) {
        let source_angle_below_min = self
            .source_tension_arm
            .get_angle()
            .map(|angle| angle.get::<degree>() < Self::SOURCE_TENSION_ARM_MIN_ANGLE_DEG)
            .unwrap_or(true);

        let angular_velocity = if source_angle_below_min
            && !matches!(self.rewind_phase, RewindPhase::SourceHigh)
        {
            AngularVelocity::new::<revolution_per_minute>(0.0)
        } else if matches!(self.rewind_phase, RewindPhase::SourceHigh) {
            AngularVelocity::new::<revolution_per_minute>(
                Self::SOURCE_TENSION_HARD_HIGH_SOURCE_FEED_RPM,
            )
        } else if self.source_spool_motion_permitted() {
            self.source_spool_speed_controller.update_speed(
                t,
                &self.source_tension_arm,
                &self.puller_speed_controller,
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
                .map(|angle| angle.get::<degree>())
                .unwrap_or_default(),
            source_tension_arm_angle: self
                .source_tension_arm
                .get_angle()
                .map(|angle| angle.get::<degree>())
                .unwrap_or_default(),
        }
    }

    pub fn emit_live_values(&mut self) {
        let event = self.get_live_values().build();
        self.namespace.emit(RewinderEvents::LiveValues(event));
    }

    pub fn build_state_event(&mut self) -> StateEvent {
        let is_default_state = !self.emitted_default_state;
        self.emitted_default_state = true;

        StateEvent {
            is_default_state,
            mode_state: ModeState {
                mode: self.mode.clone().into(),
                can_rewind: self.can_rewind(),
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
