use std::time::Instant;

use qitech_framework::machine::{
    ActResult, CommandExecuteResult, ConfigProperty, Measurement, StateProperty,
};
use qitech_lib::units::angle::degree;
use qitech_lib::units::length::millimeter;
use qitech_lib::units::{Angle, AngularVelocity, Length, Velocity};

use crate::machines::winder_v2::types::Mode;
use crate::machines::winder_v2::{LASER_PORT, PULLER_PORT, SPOOL_PORT, Winder2Mode, WinderV1};

pub use super::puller_speed_controller::{GearRatio, PullerRegulationMode};

// --- config properties ---
pub struct ConfigProperties {
    // --- traverse ---
    pub traverse_limit_inner: ConfigProperty<Length>,
    pub traverse_limit_outer: ConfigProperty<Length>,
}

impl<const VARIANT: usize> WinderV1<VARIANT> {
    pub fn on_traverse_limit_inner_changed(&mut self) -> ActResult {
        self.traverse_set_limit_inner(
            self.config_props
                .traverse_limit_inner
                .get_as::<millimeter>(),
        );

        Ok(())
    }

    pub fn on_traverse_limit_outer_changed(&mut self) -> ActResult {
        self.traverse_set_limit_outer(
            self.config_props
                .traverse_limit_outer
                .get_as::<millimeter>(),
        );

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

pub struct StateProperties {
    pub traverse_state: TraverseStateProperties,
    pub mode_state: ModeStateProperties,
    pub tension_arm_state: TensionArmStateProperties,
}

pub struct TraverseStateProperties {
    pub limit_inner: StateProperty<Length>,
    pub limit_outer: StateProperty<Length>,
    pub is_going_in: StateProperty<bool>,
    pub is_going_out: StateProperty<bool>,
    pub is_homed: StateProperty<bool>,
    pub is_going_home: StateProperty<bool>,
    pub is_traversing: StateProperty<bool>,
    pub laserpointer: StateProperty<bool>,
    pub can_go_in: StateProperty<bool>,
    pub can_go_out: StateProperty<bool>,
    pub can_go_home: StateProperty<bool>,
}

pub struct ModeStateProperties {
    pub mode: StateProperty<Mode>,
    pub can_wind: StateProperty<bool>,
}

pub struct TensionArmStateProperties {
    pub zeroed: StateProperty<bool>,
}

// --- measurements ---
pub struct Measurements {
    pub traverse_position: Measurement<Option<Length>>,
    pub puller_speed: Measurement<Velocity>,
    pub spool_rpm: Measurement<AngularVelocity>,
    pub tension_arm_angle: Measurement<Angle>,
    pub spool_progress: Measurement<Length>,
}

// --- commands ---
impl<const VARIANT: usize> WinderV1<VARIANT> {
    pub fn cmd_enter_standby_mode(&mut self) -> CommandExecuteResult {
        self.set_mode(&Winder2Mode::Standby);
        Ok(())
    }

    pub fn cmd_enter_hold_mode(&mut self) -> CommandExecuteResult {
        self.set_mode(&Winder2Mode::Hold);
        Ok(())
    }

    pub fn cmd_enter_pull_mode(&mut self) -> CommandExecuteResult {
        self.set_mode(&Winder2Mode::Pull);
        Ok(())
    }

    pub fn cmd_enter_wind_mode(&mut self) -> CommandExecuteResult {
        self.set_mode(&Winder2Mode::Wind);
        Ok(())
    }

    pub fn cmd_traverse_goto_home(&mut self) -> CommandExecuteResult {
        self.traverse_goto_home();
        Ok(())
    }

    pub fn cmd_traverse_goto_limit_outer(&mut self) -> CommandExecuteResult {
        self.traverse_goto_limit_outer();
        Ok(())
    }

    pub fn cmd_traverse_goto_limit_inner(&mut self) -> CommandExecuteResult {
        self.traverse_goto_limit_inner();
        Ok(())
    }

    pub fn cmd_traverse_laser_enable(&mut self) -> CommandExecuteResult {
        self.laser_enabled = true;
        self.laser.borrow_mut().set_output(LASER_PORT, true);
        Ok(())
    }

    pub fn cmd_traverse_laser_disable(&mut self) -> CommandExecuteResult {
        self.laser_enabled = false;
        self.laser.borrow_mut().set_output(LASER_PORT, false);
        Ok(())
    }

    pub fn cmd_spool_reset_progress(&mut self) -> CommandExecuteResult {
        self.stop_or_pull_spool_reset(Instant::now());
        Ok(())
    }

    pub fn cmd_tension_arm_set_zero(&mut self) -> CommandExecuteResult {
        self.tension_arm_zero();
        Ok(())
    }
}

// --- resource updates ---
impl<const VARIANT: usize> WinderV1<VARIANT> {
    pub fn update_measurements(&mut self) {
        let angle_deg = self.tension_arm.get_angle().unwrap();

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
        self.measurements
            .traverse_position
            .set(self.traverse_controller.get_current_position());

        self.measurements.puller_speed.set(puller_speed.abs());
        self.measurements.spool_rpm.set(spool_rpm);
        self.measurements.tension_arm_angle.set(angle_deg);
        self.measurements
            .spool_progress
            .set(self.spool_automatic_action.progress);
    }

    pub fn update_states(&mut self) {
        self.update_state_traverse();

        // --- update mode state ---
        self.state_props
            .mode_state
            .mode
            .set(self.mode.clone().into());
        self.state_props.mode_state.can_wind.set(self.can_wind());

        // --- update tension arm state ---
        self.state_props
            .tension_arm_state
            .zeroed
            .set(self.tension_arm.zeroed);
    }

    fn update_state_traverse(&mut self) {
        // --- precompute traverse state ---
        let limit_inner = self.traverse_controller.get_limit_inner();
        let limit_outer = self.traverse_controller.get_limit_outer();

        let is_going_in = self.traverse_controller.is_going_in();
        let is_going_out = self.traverse_controller.is_going_out();
        let is_homed = self.traverse_controller.is_homed();
        let is_going_home = self.traverse_controller.is_going_home();
        let is_traversing = self.traverse_controller.is_traversing();

        let laserpointer = self.laser_enabled;

        let can_go_in = self.traverse_can_goto_limit_inner();
        let can_go_out = self.traverse_can_goto_limit_outer();
        let can_go_home = self.traverse_can_goto_home();

        // --- update traverse state_props ---
        let s = &mut self.state_props.traverse_state;

        s.limit_inner.set(limit_inner);
        s.limit_outer.set(limit_outer);

        s.is_going_in.set(is_going_in);
        s.is_going_out.set(is_going_out);
        s.is_homed.set(is_homed);
        s.is_going_home.set(is_going_home);
        s.is_traversing.set(is_traversing);

        s.laserpointer.set(laserpointer);

        s.can_go_in.set(can_go_in.is_allowed());
        s.can_go_out.set(can_go_out.is_allowed());
        s.can_go_home.set(can_go_home.is_allowed());
    }
}
