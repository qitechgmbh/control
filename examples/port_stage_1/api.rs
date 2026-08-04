use std::time::Instant;

use qitech_framework::machine::error::CommandExecuteResult;
use qitech_framework::machine::{ConfigProperty, Measurement, StateProperty};
use qitech_lib::units::angle::degree;
use qitech_lib::units::length::{meter, millimeter};
use qitech_lib::units::velocity::meter_per_minute;
use qitech_lib::units::{Angle, AngularVelocity, Length, Velocity};

use crate::machines::winder_v2::types::SpoolAutomaticActionMode;
use crate::machines::winder_v2::{PULLER_PORT, SPOOL_PORT, WinderV1, Winder2Mode};
use crate::machines::winder_v2::{spool_speed_controller::SpoolSpeedControllerType, types::Mode};

pub use super::puller_speed_controller::{GearRatio, PullerRegulationMode};

// --- config properties ---
pub struct ConfigProperties {
    // --- traverse ---
    pub traverse_limit_inner: ConfigProperty<Length>,
    pub traverse_limit_outer: ConfigProperty<Length>,
    pub traverse_step_size: ConfigProperty<Length>,
    pub traverse_padding: ConfigProperty<Length>,

    // --- puller ---
    pub puller_regulation_mode: ConfigProperty<PullerRegulationMode>,
    pub puller_target_speed: ConfigProperty<Velocity>,
    pub puller_forward: ConfigProperty<bool>,
    pub puller_gear_ratio: ConfigProperty<GearRatio>,
    pub puller_adaptive_max_speed_change_percent: ConfigProperty<f64>,
    pub puller_adaptive_adjustment_interval: ConfigProperty<Length>,
    pub puller_adaptive_step_percent: ConfigProperty<f64>,
    pub puller_adaptive_accepted_difference: ConfigProperty<Length>,

    // --- spool speed controller ---
    pub spool_regulation_mode: ConfigProperty<SpoolSpeedControllerType>,
    pub spool_min_speed: ConfigProperty<Velocity>,
    pub spool_max_speed: ConfigProperty<Velocity>,
    pub spool_forward: ConfigProperty<bool>,

    // --- adaptive spool speed controller ---
    pub spool_adaptive_tension_target: ConfigProperty<f64>,
    pub spool_adaptive_radius_learning_rate: ConfigProperty<f64>,
    pub spool_adaptive_max_speed_multiplier: ConfigProperty<f64>,
    pub spool_adaptive_acceleration_factor: ConfigProperty<f64>,
    pub spool_adaptive_deacceleration_urgency_multiplier: ConfigProperty<f64>,

    // --- spool automation ---
    pub spool_automatic_required_length: ConfigProperty<Length>,
    pub spool_automatic_action: ConfigProperty<SpoolAutomaticActionMode>,
}

impl WinderV1 {
    pub fn on_traverse_limit_inner_changed(&mut self) -> Result<(), String> {
        self.traverse_set_limit_inner(
            self.config_props
                .traverse_limit_inner
                .get_as::<millimeter>(),
        );

        Ok(())
    }

    pub fn on_traverse_limit_outer_changed(&mut self) -> Result<(), String> {
        self.traverse_set_limit_outer(
            self.config_props
                .traverse_limit_outer
                .get_as::<millimeter>(),
        );

        Ok(())
    }

    pub fn on_traverse_step_size_changed(&mut self) -> Result<(), String> {
        self.traverse_set_step_size(
            self.config_props
                .traverse_step_size
                .get_as::<millimeter>(),
        );
        Ok(())
    }

    pub fn on_traverse_padding_changed(&mut self) -> Result<(), String> {
        self.traverse_set_padding(
            self.config_props
                .traverse_padding
                .get_as::<millimeter>(),
        );
        Ok(())
    }

    pub fn on_puller_regulation_mode_changed(&mut self) -> Result<(), String> {
        self.puller_set_regulation(
            self.config_props.puller_regulation_mode.get(),
        );
        Ok(())
    }

    pub fn on_puller_target_speed_changed(&mut self) -> Result<(), String> {
        self.puller_set_target_speed(
            self.config_props
                .puller_target_speed
                .get_as::<meter_per_minute>(),
        );
        Ok(())
    }

    pub fn on_puller_forward_changed(&mut self) -> Result<(), String> {
        self.puller_set_forward(
            self.config_props.puller_forward.get(),
        );
        Ok(())
    }

    pub fn on_puller_gear_ratio_changed(&mut self) -> Result<(), String> {
        self.puller_set_gear_ratio(
            self.config_props.puller_gear_ratio.get(),
        );
        Ok(())
    }

    pub fn on_puller_adaptive_max_speed_change_percent_changed(
        &mut self,
    ) -> Result<(), String> {
        self.puller_set_adaptive_max_speed_change_percent(
            self.config_props
                .puller_adaptive_max_speed_change_percent
                .get(),
        );
        Ok(())
    }

    pub fn on_puller_adaptive_adjustment_interval_changed(
        &mut self,
    ) -> Result<(), String> {
        self.puller_set_adaptive_adjustment_interval_meters(
            self.config_props
                .puller_adaptive_adjustment_interval
                .get_as::<meter>(),
        );
        Ok(())
    }

    pub fn on_puller_adaptive_step_percent_changed(
        &mut self,
    ) -> Result<(), String> {
        self.puller_set_adaptive_step_percent(
            self.config_props
                .puller_adaptive_step_percent
                .get(),
        );
        Ok(())
    }

    pub fn on_puller_adaptive_accepted_difference_changed(
        &mut self,
    ) -> Result<(), String> {
        self.puller_set_adaptive_accepted_difference(
            self.config_props
                .puller_adaptive_accepted_difference
                .get_as::<millimeter>(),
        );
        Ok(())
    }

    pub fn on_spool_regulation_mode_changed(&mut self) -> Result<(), String> {
        self.spool_set_regulation_mode(
            self.config_props.spool_regulation_mode.get(),
        );
        Ok(())
    }

    pub fn on_spool_min_speed_changed(&mut self) -> Result<(), String> {
        self.spool_set_minmax_min_speed(
            self.config_props
                .spool_min_speed
                .get_as::<meter_per_minute>(),
        );
        Ok(())
    }

    pub fn on_spool_max_speed_changed(&mut self) -> Result<(), String> {
        self.spool_set_minmax_max_speed(
            self.config_props
                .spool_max_speed
                .get_as::<meter_per_minute>(),
        );
        Ok(())
    }

    pub fn on_spool_forward_changed(&mut self) -> Result<(), String> {
        self.spool_set_forward(
            self.config_props.spool_forward.get(),
        );
        Ok(())
    }

    pub fn on_spool_adaptive_tension_target_changed(
        &mut self,
    ) -> Result<(), String> {
        self.spool_set_adaptive_tension_target(
            self.config_props
                .spool_adaptive_tension_target
                .get(),
        );
        Ok(())
    }

    pub fn on_spool_adaptive_radius_learning_rate_changed(
        &mut self,
    ) -> Result<(), String> {
        self.spool_set_adaptive_radius_learning_rate(
            self.config_props
                .spool_adaptive_radius_learning_rate
                .get(),
        );
        Ok(())
    }

    pub fn on_spool_adaptive_max_speed_multiplier_changed(
        &mut self,
    ) -> Result<(), String> {
        self.spool_set_adaptive_max_speed_multiplier(
            self.config_props
                .spool_adaptive_max_speed_multiplier
                .get(),
        );
        Ok(())
    }

    pub fn on_spool_adaptive_acceleration_factor_changed(
        &mut self,
    ) -> Result<(), String> {
        self.spool_set_adaptive_acceleration_factor(
            self.config_props
                .spool_adaptive_acceleration_factor
                .get(),
        );
        Ok(())
    }

    pub fn on_spool_adaptive_deacceleration_urgency_multiplier_changed(
        &mut self,
    ) -> Result<(), String> {
        self.spool_set_adaptive_deacceleration_urgency_multiplier(
            self.config_props
                .spool_adaptive_deacceleration_urgency_multiplier
                .get(),
        );
        Ok(())
    }

    pub fn on_spool_automatic_required_length_changed(
        &mut self,
    ) -> Result<(), String> {
        self.set_spool_automatic_required_meters(
            self.config_props
                .spool_automatic_required_length
                .get_as::<meter>(),
        );
        Ok(())
    }

    pub fn on_spool_automatic_action_changed(&mut self) -> Result<(), String> {
        self.set_spool_automatic_mode(
            self.config_props.spool_automatic_action.get(),
        );
        Ok(())
    }
}

pub struct StateProperties {
    pub traverse_state: TraverseStateProperties,
    pub puller_state: PullerStateProperties,
    pub spool_automatic_action_state: SpoolAutomaticActionStateProperties,
    pub mode_state: ModeStateProperties,
    pub tension_arm_state: TensionArmStateProperties,
    pub spool_speed_controller_state: SpoolSpeedControllerStateProperties,
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
    pub step_size: StateProperty<Length>,
    pub padding: StateProperty<Length>,
    pub can_go_in: StateProperty<bool>,
    pub can_go_out: StateProperty<bool>,
    pub can_go_home: StateProperty<bool>,
}

pub struct PullerStateProperties {
    pub regulation: StateProperty<PullerRegulationMode>,
    pub target_speed: StateProperty<Velocity>,
    pub forward: StateProperty<bool>,
    pub gear_ratio: StateProperty<GearRatio>,
    pub adaptive_speed_delta_max: StateProperty<f64>,
    pub adaptive_adjustment_distance: StateProperty<Length>,
    pub adaptive_change_per_step: StateProperty<f64>,
    pub allowed_diameter_deviation: StateProperty<Length>,
}

pub struct SpoolAutomaticActionStateProperties {
    pub spool_required_meters: StateProperty<Length>,
    pub spool_automatic_action_mode: StateProperty<SpoolAutomaticActionMode>,
}

pub struct ModeStateProperties {
    pub mode: StateProperty<Mode>,
    pub can_wind: StateProperty<bool>,
}

pub struct TensionArmStateProperties {
    pub zeroed: StateProperty<bool>,
}

pub struct SpoolSpeedControllerStateProperties {
    pub regulation_mode: StateProperty<SpoolSpeedControllerType>,
    pub minmax_min_speed: StateProperty<AngularVelocity>,
    pub minmax_max_speed: StateProperty<AngularVelocity>,
    pub adaptive_tension_target: StateProperty<f64>,
    pub adaptive_radius_learning_rate: StateProperty<f64>,
    pub adaptive_max_speed_multiplier: StateProperty<f64>,
    pub adaptive_acceleration_factor: StateProperty<f64>,
    pub adaptive_deacceleration_urgency_multiplier: StateProperty<f64>,
    pub forward: StateProperty<bool>,
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
impl WinderV1 {
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
        self.set_laser(true);
        Ok(())
    }

    pub fn cmd_traverse_laser_disable(&mut self) -> CommandExecuteResult {
        self.set_laser(false);
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
impl WinderV1 {
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
        self.update_state_puller();

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

        // --- update spool speed controller state ---
        self.update_state_spool_speed_controller();

        // --- update spool automatic action state ---
        self.update_state_spool_automatic_action_state();
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
        let step_size = self.traverse_controller.get_step_size();
        let padding = self.traverse_controller.get_padding();

        let can_go_in = self.can_go_in();
        let can_go_out = self.can_go_out();
        let can_go_home = self.can_go_home();

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
        s.step_size.set(step_size);
        s.padding.set(padding);

        s.can_go_in.set(can_go_in);
        s.can_go_out.set(can_go_out);
        s.can_go_home.set(can_go_home);
    }

    fn update_state_puller(&mut self) {
        // --- precompute puller state ---
        let regulation = self.puller_speed_controller.regulation_mode;
        let target_speed = self.puller_speed_controller.target_speed;
        let forward = self.puller_speed_controller.forward;
        let gear_ratio = self.puller_speed_controller.gear_ratio;

        let adaptive_speed_delta_max = self.puller_speed_controller.adaptive.speed_delta_max();
        let adaptive_adjustment_distance =
            self.puller_speed_controller.adaptive.adjustment_distance();
        let adaptive_change_per_step = self.puller_speed_controller.adaptive.increase_per_step();
        let allowed_diameter_deviation = self.puller_speed_controller.adaptive.tolerance_limit();

        // --- update puller state ---
        let s = &mut self.state_props.puller_state;

        s.regulation.set(regulation);
        s.target_speed.set(target_speed);
        s.forward.set(forward);
        s.gear_ratio.set(gear_ratio);

        s.adaptive_speed_delta_max.set(adaptive_speed_delta_max);

        s.adaptive_adjustment_distance
            .set(adaptive_adjustment_distance);

        s.adaptive_change_per_step.set(adaptive_change_per_step);

        s.allowed_diameter_deviation.set(allowed_diameter_deviation);
    }

    fn update_state_spool_speed_controller(&mut self) {
        // --- precompute spool speed controller state ---
        let regulation_mode = *self.spool_speed_controller.get_type();
        let minmax_min_speed = self.spool_speed_controller.get_minmax_min_speed();
        let minmax_max_speed = self.spool_speed_controller.get_minmax_max_speed();

        let adaptive_tension_target = self.spool_speed_controller.get_adaptive_tension_target();

        let adaptive_radius_learning_rate = self
            .spool_speed_controller
            .get_adaptive_radius_learning_rate();

        let adaptive_max_speed_multiplier = self
            .spool_speed_controller
            .get_adaptive_max_speed_multiplier();

        let adaptive_acceleration_factor = self
            .spool_speed_controller
            .get_adaptive_acceleration_factor();

        let adaptive_deacceleration_urgency_multiplier = self
            .spool_speed_controller
            .get_adaptive_deacceleration_urgency_multiplier();

        let forward = self.spool_speed_controller.get_forward();

        // --- update spool speed controller state ---
        let s = &mut self.state_props.spool_speed_controller_state;

        s.regulation_mode.set(regulation_mode);
        s.minmax_min_speed.set(minmax_min_speed);
        s.minmax_max_speed.set(minmax_max_speed);

        s.adaptive_tension_target.set(adaptive_tension_target);

        s.adaptive_radius_learning_rate
            .set(adaptive_radius_learning_rate);

        s.adaptive_max_speed_multiplier
            .set(adaptive_max_speed_multiplier);

        s.adaptive_acceleration_factor
            .set(adaptive_acceleration_factor);

        s.adaptive_deacceleration_urgency_multiplier
            .set(adaptive_deacceleration_urgency_multiplier);

        s.forward.set(forward);
    }

    fn update_state_spool_automatic_action_state(&mut self) {
        // --- precompute spool automatic action state ---
        let spool_required_meters = self.spool_automatic_action.target_length;
        let spool_automatic_action_mode = self.spool_automatic_action.mode;

        // --- update spool automatic action state ---
        let s = &mut self.state_props.spool_automatic_action_state;

        s.spool_required_meters.set(spool_required_meters);
        s.spool_automatic_action_mode
            .set(spool_automatic_action_mode);
    }
}
