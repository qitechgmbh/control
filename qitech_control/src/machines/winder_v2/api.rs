use std::time::Instant;

use qitech_framework::machine::{
    ActError, ActErrorImpact, ActResult, CommandExecuteResult, Measurement, OperationCapability,
};

use qitech_lib::units::{AngularVelocity, Length, Velocity};

use crate::machines::winder_v2::types::Mode;
use crate::machines::winder_v2::{LASER_PORT, PULLER_PORT, SPOOL_PORT, WinderV1};

pub use super::puller_speed_controller::{GearRatio, PullerRegulationMode};

// --- config properties ---
impl<const VARIANT: usize> WinderV1<VARIANT> {
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
    pub spool_progress: Measurement<Length>,
}

// --- commands ---
impl<const VARIANT: usize> WinderV1<VARIANT> {
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
}

// --- command capability checks ---
impl<const VARIANT: usize> WinderV1<VARIANT> {
    pub fn can_enter_wind_mode(&self) -> OperationCapability {
        if self.tension_arm.zero.get().is_none() {
            return OperationCapability::Forbidden {
                reason: "tension arm is not zeroed".to_string(),
            };
        }

        /*
        if !self.traverse.is_homed() {
            return OperationCapability::Forbidden {
                reason: "traverse is not homed".to_string(),
            };
        }

        if self.traverse.is_going_home() {
            return OperationCapability::Forbidden {
                reason: "traverse is currently homing".to_string(),
            };
        }
        */

        if self.mode.get() == Mode::Wind {
            return OperationCapability::Forbidden {
                reason: "winder is already in wind mode".to_string(),
            };
        }

        OperationCapability::Allowed
    }
}

// --- resource updates ---
impl<const VARIANT: usize> WinderV1<VARIANT> {
    pub fn update_measurements(&mut self) {
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
        
        if let Err(kind) = self.traverse.apply_mode(mode.into()) {
            return Err(ActError {
                kind,
                impact: ActErrorImpact::Degraded,
            });
        }

        Ok(())
    }
}
