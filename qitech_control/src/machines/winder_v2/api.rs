use std::time::Instant;

use qitech_framework::machine::{
    ActError, ActErrorImpact, ActErrorKind, ActResult, CommandExecuteResult, Measurement,
    OperationCapability,
};
use qitech_lib::units::angle::degree;
use qitech_lib::units::length::millimeter;
use qitech_lib::units::{Angle, AngularVelocity, Length, Velocity};

use crate::machines::winder_v2::types::{Mode, TraverseMode};
use crate::machines::winder_v2::{LASER_PORT, PULLER_PORT, SPOOL_PORT, TRAVERSE_PORT, WinderV1};

pub use super::puller_speed_controller::{GearRatio, PullerRegulationMode};

// --- config properties ---
impl<const VARIANT: usize> WinderV1<VARIANT> {
    pub fn on_traverse_limit_inner_changed(&mut self) -> ActResult {
        let limit_inner = self.traverse_controller.limit_inner.get();
        let offet_min = Length::new::<millimeter>(0.9);
        _ = self
            .traverse_controller
            .limit_outer
            .set(limit_inner + offet_min);
        Ok(())
    }

    pub fn on_puller_regulation_mode_changed(&mut self) -> ActResult {
        self.puller_speed_controller.adaptive.reset_modulation();
        Ok(())
    }

    pub fn on_spool_regulation_mode_changed(&mut self) -> ActResult {
        self.spool_speed_controller.on_control_mode_changed();
        Ok(())
    }

    pub fn on_spool_min_speed_changed(&mut self) -> ActResult {
        _ = self
            .spool_speed_controller
            .min_max_controller
            .on_speed_min_changed();
        Ok(())
    }

    pub fn on_spool_max_speed_changed(&mut self) -> ActResult {
        _ = self
            .spool_speed_controller
            .min_max_controller
            .on_speed_max_changed();
        Ok(())
    }
}

// --- measurements ---
pub struct Measurements {
    pub puller_speed: Measurement<Velocity>,
    pub spool_rpm: Measurement<AngularVelocity>,
    pub tension_arm_angle: Measurement<Angle>,
    pub spool_progress: Measurement<Length>,
}

// --- commands ---
impl<const VARIANT: usize> WinderV1<VARIANT> {
    pub fn traverse_goto_home(&mut self) -> CommandExecuteResult {
        self.traverse_controller.goto_home();
        Ok(())
    }

    pub fn traverse_goto_limit_outer(&mut self) -> CommandExecuteResult {
        self.traverse_controller.goto_limit_outer();
        Ok(())
    }

    pub fn traverse_goto_limit_inner(&mut self) -> CommandExecuteResult {
        self.traverse_controller.goto_limit_inner();
        Ok(())
    }

    pub fn traverse_laser_enable(&mut self) -> CommandExecuteResult {
        self.laser_enabled.set(false);
        self.laser.borrow_mut().set_output(LASER_PORT, true);
        Ok(())
    }

    pub fn traverse_laser_disable(&mut self) -> CommandExecuteResult {
        self.laser_enabled.set(true);
        self.laser.borrow_mut().set_output(LASER_PORT, false);
        Ok(())
    }

    pub fn spool_reset_progress(&mut self) -> CommandExecuteResult {
        self.stop_or_pull_spool_reset(Instant::now());
        Ok(())
    }

    pub fn tension_arm_set_zero(&mut self) -> CommandExecuteResult {
        if let Err(e) = self.tension_arm.set_zero() {
            return Err(ActError {
                kind: ActErrorKind::Custom(e),
                impact: ActErrorImpact::Degraded,
            });
        }

        Ok(())
    }
}

// --- command capability checks ---
impl<const VARIANT: usize> WinderV1<VARIANT> {
    pub fn can_enter_wind_mode(&self) -> OperationCapability {
        if !self.tension_arm.zeroed.get() {
            return OperationCapability::Forbidden {
                reason: "tension arm is not zeroed".to_string(),
            };
        }

        if !self.traverse_controller.is_homed() {
            return OperationCapability::Forbidden {
                reason: "traverse is not homed".to_string(),
            };
        }

        if self.traverse_controller.is_going_home() {
            return OperationCapability::Forbidden {
                reason: "traverse is currently homing".to_string(),
            };
        }

        if self.mode.get() == Mode::Wind {
            return OperationCapability::Forbidden {
                reason: "winder is already in wind mode".to_string(),
            };
        }

        OperationCapability::Allowed
    }

    pub fn traverse_can_goto_home(&self) -> OperationCapability {
        if self.traverse_mode == TraverseMode::Standby {
            return OperationCapability::Forbidden {
                reason: "traverse is in standby".to_string(),
            };
        }

        if self.traverse_controller.is_going_home() {
            return OperationCapability::Forbidden {
                reason: "traverse is already going home".to_string(),
            };
        }

        if self.traverse_controller.is_traversing() {
            return OperationCapability::Forbidden {
                reason: "traverse is currently traversing".to_string(),
            };
        }

        if self.mode.get() == Mode::Wind {
            return OperationCapability::Forbidden {
                reason: "winder is in wind mode".to_string(),
            };
        }

        OperationCapability::Allowed
    }

    pub fn traverse_can_goto_limit_outer(&self) -> OperationCapability {
        if !self.traverse_controller.is_homed() {
            return OperationCapability::Forbidden {
                reason: "traverse is not homed".to_string(),
            };
        }

        if self.traverse_mode == TraverseMode::Standby {
            return OperationCapability::Forbidden {
                reason: "traverse is in standby".to_string(),
            };
        }

        if self.traverse_controller.is_going_out() {
            return OperationCapability::Forbidden {
                reason: "traverse is already going out".to_string(),
            };
        }

        if self.traverse_controller.is_going_home() {
            return OperationCapability::Forbidden {
                reason: "traverse is currently homing".to_string(),
            };
        }

        if self.traverse_controller.is_traversing() {
            return OperationCapability::Forbidden {
                reason: "traverse is currently traversing".to_string(),
            };
        }

        if self.mode.get() == Mode::Wind {
            return OperationCapability::Forbidden {
                reason: "winder is in wind mode".to_string(),
            };
        }

        OperationCapability::Allowed
    }
}

// NOTES:
// -> code that adjust values for export must be removed
// reason is we yield the raw measurements not transformed ones!!

// --- resource updates ---
impl<const VARIANT: usize> WinderV1<VARIANT> {
    pub fn update_measurements(&mut self) {
        let angle_deg = self.tension_arm.angle().unwrap();

        // Wrap [270;<360] to [-90; 0]
        // This is done to reduce flicker in the graphs around the zero point
        let angle_deg = if angle_deg >= Angle::new::<degree>(270.0) {
            angle_deg - Angle::new::<degree>(360.0)
        } else {
            angle_deg
        };

        let puller_ref = self.puller.borrow_mut();

        // Calculate puller speed from current motor steps
        let steps_per_second = puller_ref.get_speed(PULLER_PORT);
        let angular_velocity = self
            .puller_speed_controller
            .converter
            .steps_to_angular_velocity(steps_per_second as f64);

        let motor_speed = self
            .puller_speed_controller
            .angular_velocity_to_speed(angular_velocity);

        // Divide by gear ratio to get actual puller/material speed
        let puller_speed = motor_speed / self.puller_speed_controller.get_gear_ratio().multiplier();

        drop(puller_ref);

        let spool_ref = self.spool.borrow_mut();

        // Calculate spool RPM from current motor steps (always positive regardless of direction)
        let spool_rpm = self
            .spool_step_converter
            .steps_to_angular_velocity(spool_ref.get_speed(SPOOL_PORT) as f64)
            .abs();

        // --- write now ---
        self.measurements.puller_speed.set(puller_speed.abs());
        self.measurements.spool_rpm.set(spool_rpm);
        self.measurements.tension_arm_angle.set(angle_deg);
        self.measurements
            .spool_progress
            .set(self.spool_automatic_action.progress);
    }
}

// --- utils ---
impl<const VARIANT: usize> WinderV1<VARIANT> {
    /// Implement Mode
    pub fn set_mode(&mut self, mode: Mode) -> ActResult {
        self.mode.set(mode);
        self.set_spool_mode(mode);
        self.set_puller_mode(mode);
        self.set_traverse_mode(mode);
        Ok(())
    }

    /// Apply the mode changes to the spool
    ///
    /// It contains a transition matrix for atomic changes.
    /// It will set [`Self::spool_mode`]
    fn set_traverse_mode(&mut self, mode: Mode) {
        // Convert to `Winder2Mode` to `TraverseMode`
        let mode: TraverseMode = mode.into();
        // If coming out of standby
        if self.traverse_mode == TraverseMode::Standby && mode != TraverseMode::Standby {
            let mut traverse = self.traverse.borrow_mut();
            let traverse_ref = &mut *traverse;
            traverse_ref.set_enabled(TRAVERSE_PORT, true);
            self.traverse_controller.set_enabled(true);
            drop(traverse);
        }

        // If going into standby
        if mode == TraverseMode::Standby && self.traverse_mode != TraverseMode::Standby {
            let mut traverse = self.traverse.borrow_mut();
            let traverse_ref = &mut *traverse;
            // If we are going into standby, we need to stop the traverse
            traverse_ref.set_enabled(TRAVERSE_PORT, false);
            self.traverse_controller.set_enabled(false);
            drop(traverse);
        }

        {
            let mut traverse = self.traverse.borrow_mut();
            let traverse_ref = &mut *traverse;
            // Transition matrix
            match self.traverse_mode {
                TraverseMode::Standby => match mode {
                    TraverseMode::Standby => {}
                    TraverseMode::Hold => {
                        // From [`TraverseMode::Standby`] to [`TraverseMode::Hold`]
                        traverse_ref.set_enabled(TRAVERSE_PORT, true);
                        self.traverse_controller.set_enabled(true);
                        self.traverse_controller.goto_home();
                    }
                    TraverseMode::Traverse => {
                        // From [`TraverseMode::Standby`] to [`TraverseMode::Wind`]
                        traverse_ref.set_enabled(TRAVERSE_PORT, true);
                        self.traverse_controller.set_enabled(true);
                        self.traverse_controller.start_traversing();
                    }
                },
                TraverseMode::Hold => match mode {
                    TraverseMode::Standby => {
                        // From [`TraverseMode::Hold`] to [`TraverseMode::Standby`]
                        traverse_ref.set_enabled(TRAVERSE_PORT, false);
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
                        traverse_ref.set_enabled(TRAVERSE_PORT, false);
                        self.traverse_controller.set_enabled(false);
                    }
                    TraverseMode::Hold => {
                        // From [`TraverseMode::Wind`] to [`TraverseMode::Hold`]
                        self.traverse_controller.goto_home();
                    }
                    TraverseMode::Traverse => {}
                },
            }
        }

        // Update the internal state
        self.traverse_mode = mode;
    }
}
