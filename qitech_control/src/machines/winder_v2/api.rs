use std::time::Instant;

use qitech_framework::machine::ActError;
use qitech_framework::machine::ActErrorImpact;
use qitech_framework::machine::ActResult;
use qitech_framework::machine::CommandExecuteResult;
use qitech_framework::machine::Measurement;
use qitech_framework::machine::OperationCapability;
use qitech_lib::units::AngularVelocity;
use qitech_lib::units::Length;

use crate::machines::winder_v2::LASER_PORT;
use crate::machines::winder_v2::SPOOL_PORT;
use crate::machines::winder_v2::WinderV1;
use crate::machines::winder_v2::types::Mode;

/*
// --- config properties ---
impl<const VARIANT: usize> WinderV1<VARIANT> {
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
    */

// --- measurements ---
pub struct Measurements {
    pub spool_rpm: Measurement<AngularVelocity>,
    pub spool_progress: Measurement<Length>,
}

// --- commands ---
impl<const VARIANT: usize> WinderV1<VARIANT> {
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
        let spool_ref = self.spool.borrow_mut();

        // Calculate spool RPM from current motor steps (always positive regardless of direction)
        let spool_rpm = self
            .spool_step_converter
            .steps_to_angular_velocity(spool_ref.get_speed(SPOOL_PORT) as f64)
            .abs();

        // --- write now ---
        self.measurements.spool_rpm.set(spool_rpm);
        self.measurements
            .spool_progress
            .set(self.spool_automatic_action.progress);
    }
}
