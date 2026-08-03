use std::time::Instant;

use qitech_framework::machine::error::CommandExecuteResult;
use qitech_framework::machine::{ConfigProperty, Measurement, StateProperty};
use qitech_lib::units::length::millimeter;
use qitech_lib::units::{Angle, AngularVelocity, Length, Velocity};

use crate::machines::winder_v2::types::SpoolAutomaticActionMode;
use crate::machines::winder_v2::{Winder2, Winder2Mode};
use crate::machines::winder_v2::{spool_speed_controller::SpoolSpeedControllerType, types::Mode};

pub use super::puller_speed_controller::{GearRatio, PullerRegulationMode};

// --- config properties ---
#[derive(Debug, Clone, Default)]
pub struct ConfigProperties {
    // --- traverse ---
    pub traverse_limit_inner: ConfigProperty<Length>,
    pub traverse_limit_outer: ConfigProperty<Length>,
    pub traverse_step_size: ConfigProperty<Length>,
    pub traverse_padding: ConfigProperty<Length>,

    // --- puller ---
    pub puller_regulation_mode: ConfigProperty<PullerRegulationMode>,
    pub puller_target_speed: ConfigProperty<Velocity>,
    pub puller_target_diameter: ConfigProperty<Length>,
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

impl Winder2 {
    pub fn on_traverse_limit_inner_changed(&mut self) -> Result<(), String> {
        self.traverse_set_limit_outer(
            self.config_props.traverse_limit_outer.get_as::<millimeter>()
        );

        Ok(())
    }
}

// --- state properties ---
pub struct StateProperties {
    pub traverse_state: TraverseStateProperties,
    pub puller_state: PullerStateProperties,
    pub spool_automatic_action_state: SpoolAutomaticActionStateProperties,
    pub mode_state: ModeStateProperties,
    pub tension_arm_state: TensionArmStateProperties,
    pub spool_speed_controller_state: SpoolSpeedControllerStateProperties,
}

#[derive(Debug, Clone, Default)]
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

#[derive(Debug, Clone, Default)]
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

#[derive(Debug, Clone, Default)]
pub struct SpoolAutomaticActionStateProperties {
    pub spool_required_meters: StateProperty<Length>,
    pub spool_automatic_action_mode: StateProperty<SpoolAutomaticActionMode>,
}

#[derive(Debug, Clone, Default)]
pub struct ModeStateProperties {
    pub mode: StateProperty<Mode>,
    pub can_wind: StateProperty<bool>,
}

#[derive(Debug, Clone, Default)]
pub struct TensionArmStateProperties {
    pub zeroed: StateProperty<bool>,
}

// --- measurements ---
#[derive(Debug, Clone, Default)]
pub struct Measurements {
    pub traverse_position: Measurement<Option<Length>>,
    pub puller_speed: Measurement<Velocity>,
    pub spool_rpm: Measurement<AngularVelocity>,
    pub tension_arm_angle: Measurement<Angle>,
    pub spool_progress: Measurement<Length>,
}

#[derive(Debug, Clone, Default)]
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

// --- commands ---
impl Winder2 {
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

    pub fn cmd_traverse_enable_laser(&mut self) -> CommandExecuteResult {
        self.set_laser(true);
        Ok(())
    }

    pub fn cmd_traverse_disable_laser(&mut self) -> CommandExecuteResult {
        self.set_laser(false);
        Ok(())
    }

    pub fn cmd_reset_spool_progress(&mut self) -> CommandExecuteResult {
        self.stop_or_pull_spool_reset(Instant::now());
        Ok(())
    }

    pub fn cmd_reset_spool_progress(&mut self) -> CommandExecuteResult {
        self.tension_arm_zero();
        Ok(())
    }
}
