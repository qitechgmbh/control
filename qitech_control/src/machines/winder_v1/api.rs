mod winder2_imports {
    pub use std::time::Instant;

    pub use serde::Deserialize;
    pub use serde::Serialize;

    pub use super::super::Winder_V1;
    pub use super::super::Winder2Mode;
    pub use super::super::puller_speed_controller::GearRatio;
    pub use super::super::puller_speed_controller::PullerRegulationMode;
}

use qitech_framework::ScalarValue;
use qitech_framework::machine::TypeWrapper;
use qitech_framework::machine::error::CommandExecuteResult;
use qitech_framework::machine::resource::Measurement;
use qitech_framework::machine::resource::StateProperty;
use qitech_lib::units::Angle;
use qitech_lib::units::AngularVelocity;
use qitech_lib::units::Length;
use qitech_lib::units::Velocity;
use qitech_lib::units::angle::degree;
pub use winder2_imports::*;

use crate::machines::winder_v1::PULLER_PORT;
use crate::machines::winder_v1::SPOOL_PORT;
use crate::machines::winder_v1::spool_speed_controller::SpoolSpeedControllerType;

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub enum Mode {
    #[default]
    Standby,
    Hold,
    Pull,
    Wind,
}

impl TypeWrapper for Mode {
    type Type = Mode;
    type Input = Mode;

    fn into_scalar(value: &Self::Type) -> ScalarValue {
        ScalarValue::Enum(Some(
            match value {
                Mode::Standby => "standby",
                Mode::Pull => "pull",
                Mode::Hold => "hold",
                Mode::Wind => "wind",
            }
            .to_string(),
        ))
    }

    fn convert_input(input: Self::Input) -> Self::Type {
        input
    }

    fn deserialize_json(raw: &str) -> serde_json::Result<Self::Type> {
        serde_json::from_str(raw)
    }
}

impl From<Winder2Mode> for Mode {
    fn from(mode: Winder2Mode) -> Self {
        match mode {
            Winder2Mode::Standby => Self::Standby,
            Winder2Mode::Hold => Self::Hold,
            Winder2Mode::Pull => Self::Pull,
            Winder2Mode::Wind => Self::Wind,
        }
    }
}

impl From<Mode> for Winder2Mode {
    fn from(mode: Mode) -> Self {
        match mode {
            Mode::Standby => Self::Standby,
            Mode::Hold => Self::Hold,
            Mode::Pull => Self::Pull,
            Mode::Wind => Self::Wind,
        }
    }
}

#[derive(Deserialize)]
pub enum Mutation {
    // Traverse
    /// Position in mm from home point
    SetTraverseLimitOuter(f64),
    /// Position in mm from home point
    SetTraverseLimitInner(f64),
    /// Step size in mm for traverse movement
    SetTraverseStepSize(f64),
    /// Padding in mm for traverse movement limits
    SetTraversePadding(f64),
    GotoTraverseLimitOuter,
    GotoTraverseLimitInner,
    /// Find home point
    GotoTraverseHome,
    EnableTraverseLaserpointer(bool),

    // Puller
    /// on = speed, off = stop
    SetPullerRegulationMode(PullerRegulationMode),
    SetPullerTargetSpeed(f64),
    SetPullerTargetDiameter(f64),
    SetPullerForward(bool),
    SetPullerGearRatio(GearRatio),

    // Spool Speed Controller
    SetSpoolRegulationMode(SpoolSpeedControllerType),
    SetSpoolMinMaxMinSpeed(f64),
    SetSpoolMinMaxMaxSpeed(f64),
    SetSpoolForward(bool),

    // Adaptive Spool Speed Controller Parameters
    SetSpoolAdaptiveTensionTarget(f64),
    SetSpoolAdaptiveRadiusLearningRate(f64),
    SetSpoolAdaptiveMaxSpeedMultiplier(f64),
    SetSpoolAdaptiveAccelerationFactor(f64),
    SetSpoolAdaptiveDeaccelerationUrgencyMultiplier(f64),

    // Spool Auto Stop/Pull
    SetSpoolAutomaticRequiredMeters(f64),
    SetSpoolAutomaticAction(SpoolAutomaticActionMode),
    ResetSpoolProgress,

    // Tension Arm
    ZeroTensionArmAngle,

    // Mode
    SetMode(Mode),

    // Set puller adaptive reference machine
    /// Maximum speed change as a percentage of base speed (0.0–100.0)
    SetPullerAdaptiveMaxSpeedChangePercent(f64),
    /// Minimum meters between consecutive adjustments
    SetPullerAdaptiveAdjustmentIntervalMeters(f64),
    /// Step size per adjustment as a percentage of base speed (0.0–100.0)
    SetPullerAdaptiveStepPercent(f64),
    /// Inner deadzone: max deviation from target (mm) that requires no correction
    SetPullerAdaptiveAcceptedDifference(f64),
}

pub struct Measurements {
    /// traverse position in mm
    pub traverse_position: Measurement<Option<Length>>,
    /// puller speed in m/min
    pub puller_speed: Measurement<Velocity>,
    /// spool rpm
    pub spool_rpm: Measurement<AngularVelocity>,
    /// tension arm angle in degrees
    pub tension_arm_angle: Measurement<Angle>,
    // spool progress in meters (pulled distance of filament)
    pub spool_progress: Measurement<Length>,
}

pub struct States {
    /// traverse state
    pub traverse_state: TraverseState,
    /// puller state
    pub puller_state: PullerState,
    /// spool automatic action state and progress
    pub spool_automatic_action_state: SpoolAutomaticActionState,
    /// mode state
    pub mode_state: ModeState,
    /// tension arm state
    pub tension_arm_state: TensionArmState,
    /// spool speed controller state
    pub spool_speed_controller_state: SpoolSpeedControllerState,
}

pub struct TraverseState {
    /// min position in mm
    pub limit_inner: StateProperty<Length>,
    /// max position in mm
    pub limit_outer: StateProperty<Length>,
    /// is going to position in
    pub is_going_in: StateProperty<bool>,
    /// is going to position out
    pub is_going_out: StateProperty<bool>,
    /// if is homed
    pub is_homed: StateProperty<bool>,
    /// if is homing
    pub is_going_home: StateProperty<bool>,
    /// if is traversing
    pub is_traversing: StateProperty<bool>,
    /// laserpointer is on
    pub laserpointer: StateProperty<bool>,
    /// step size in mm
    pub step_size: StateProperty<Length>,
    /// padding in mm
    pub padding: StateProperty<Length>,
    /// can go in (to inner limit)
    pub can_go_in: StateProperty<bool>,
    /// can go out (to outer limit)
    pub can_go_out: StateProperty<bool>,
    /// can home
    pub can_go_home: StateProperty<bool>,
}

pub struct PullerState {
    /// regulation type
    pub regulation: StateProperty<PullerRegulationMode>,
    /// target speed in m/min
    pub target_speed: StateProperty<Velocity>,
    /// forward rotation direction
    pub forward: StateProperty<bool>,
    /// gear ratio for winding speed
    pub gear_ratio: StateProperty<GearRatio>,

    /// Maximum speed change as a percentage of base speed (0.0–100.0)
    pub adaptive_speed_delta_max: StateProperty<f64>,
    /// Minimum meters between consecutive adjustments
    pub adaptive_adjustment_distance: StateProperty<Length>,
    /// Step size per adjustment as a percentage of base speed (0.0–100.0)
    pub adaptive_change_per_step: StateProperty<f64>,
    /// Inner deadzone: max deviation from target (mm) that requires no correction
    pub allowed_diameter_deviation: StateProperty<Length>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq)]
pub enum SpoolAutomaticActionMode {
    #[default]
    NoAction,
    Pull,
    Hold,
}

impl TypeWrapper for SpoolAutomaticActionMode {
    type Type = SpoolAutomaticActionMode;
    type Input = SpoolAutomaticActionMode;

    fn into_scalar(value: &Self::Type) -> ScalarValue {
        ScalarValue::Enum(Some(
            match value {
                SpoolAutomaticActionMode::NoAction => "no_action",
                SpoolAutomaticActionMode::Pull => "pull",
                SpoolAutomaticActionMode::Hold => "hold",
            }
            .to_string(),
        ))
    }

    fn convert_input(input: Self::Input) -> Self::Type {
        input
    }

    fn deserialize_json(raw: &str) -> serde_json::Result<Self::Type> {
        serde_json::from_str(raw)
    }
}

pub struct SpoolAutomaticActionState {
    pub spool_required_meters: StateProperty<Length>,
    pub spool_automatic_action_mode: StateProperty<SpoolAutomaticActionMode>,
}

pub struct ModeState {
    /// mode
    pub mode: StateProperty<Mode>,
    /// can wind
    pub can_wind: StateProperty<bool>,
}

pub struct TensionArmState {
    /// is zeroed
    pub zeroed: StateProperty<bool>,
}

pub struct SpoolSpeedControllerState {
    /// regulation mode
    pub regulation_mode: StateProperty<SpoolSpeedControllerType>,
    /// min speed in rpm for minmax mode
    pub minmax_min_speed: StateProperty<AngularVelocity>,
    /// max speed in rpm for minmax mode
    pub minmax_max_speed: StateProperty<AngularVelocity>,
    /// tension target for adaptive mode (0.0-1.0)
    pub adaptive_tension_target: StateProperty<f64>,
    /// radius learning rate for adaptive mode
    pub adaptive_radius_learning_rate: StateProperty<f64>,
    /// max speed multiplier for adaptive mode
    pub adaptive_max_speed_multiplier: StateProperty<f64>,
    /// acceleration factor for adaptive mode
    pub adaptive_acceleration_factor: StateProperty<f64>,
    /// deacceleration urgency multiplier for adaptive mode
    pub adaptive_deacceleration_urgency_multiplier: StateProperty<f64>,
    /// forward rotation direction
    pub forward: StateProperty<bool>,
}

// --- resource updates ---
impl Winder_V1 {
    pub fn update_measurements(&mut self) {
        let angle_deg = self.tension_arm.get_angle().unwrap();

        // Wrap [270;<360] to [-90; 0]
        // This is done to reduce flicker in the graphs around the zero point
        let angle_deg = if angle_deg >= Angle::new::<degree>(270.0) {
            angle_deg - Angle::new::<degree>(360.0)
        } else {
            angle_deg
        };

        let puller_ref = &mut *self.puller.borrow_mut();

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
        let spool_ref = &mut *self.spool.borrow_mut();
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
        self.states.mode_state.mode.set(self.mode.clone().into());
        self.states.mode_state.can_wind.set(self.can_wind());

        // --- update tension arm state ---
        self.states
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

        // --- update traverse states ---
        let s = &mut self.states.traverse_state;

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
        let regulation = self.puller_speed_controller.regulation_mode.clone();
        let target_speed = self.puller_speed_controller.target_speed;
        let forward = self.puller_speed_controller.forward;
        let gear_ratio = self.puller_speed_controller.gear_ratio;

        let adaptive_speed_delta_max = self.puller_speed_controller.adaptive.speed_delta_max();
        let adaptive_adjustment_distance =
            self.puller_speed_controller.adaptive.adjustment_distance();
        let adaptive_change_per_step = self.puller_speed_controller.adaptive.increase_per_step();
        let allowed_diameter_deviation = self.puller_speed_controller.adaptive.tolerance_limit();

        // --- update puller state ---
        let s = &mut self.states.puller_state;

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
        let regulation_mode = self.spool_speed_controller.get_type().clone();
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
        let s = &mut self.states.spool_speed_controller_state;

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
        let spool_automatic_action_mode = self.spool_automatic_action.mode.clone();

        // --- update spool automatic action state ---
        let s = &mut self.states.spool_automatic_action_state;

        s.spool_required_meters.set(spool_required_meters);
        s.spool_automatic_action_mode
            .set(spool_automatic_action_mode);
    }
}

// --- commands ---
impl Winder_V1 {
    pub fn cmd_enable_traverse_laserpointer(&mut self, value: bool) -> CommandExecuteResult {
        self.set_laser(value);
        Ok(())
    }

    pub fn execute_mutation(&mut self, mutation: Mutation) -> CommandExecuteResult {
        match mutation {
            Mutation::EnableTraverseLaserpointer(enable) => self.set_laser(enable),
            Mutation::SetMode(mode) => self.set_mode(&mode.into()),
            Mutation::SetTraverseLimitOuter(limit) => self.traverse_set_limit_outer(limit),
            Mutation::SetTraverseLimitInner(limit) => self.traverse_set_limit_inner(limit),
            Mutation::SetTraverseStepSize(size) => self.traverse_set_step_size(size),
            Mutation::SetTraversePadding(padding) => self.traverse_set_padding(padding),
            Mutation::GotoTraverseLimitOuter => self.traverse_goto_limit_outer(),
            Mutation::GotoTraverseLimitInner => self.traverse_goto_limit_inner(),
            Mutation::GotoTraverseHome => self.traverse_goto_home(),
            Mutation::SetPullerRegulationMode(regulation) => self.puller_set_regulation(regulation),
            Mutation::SetPullerTargetSpeed(value) => self.puller_set_target_speed(value),
            Mutation::SetPullerTargetDiameter(_) => todo!(),
            Mutation::SetPullerForward(value) => self.puller_set_forward(value),
            Mutation::SetPullerGearRatio(gear_ratio) => self.puller_set_gear_ratio(gear_ratio),
            Mutation::SetSpoolRegulationMode(mode) => self.spool_set_regulation_mode(mode),
            Mutation::SetSpoolMinMaxMinSpeed(speed) => self.spool_set_minmax_min_speed(speed),
            Mutation::SetSpoolMinMaxMaxSpeed(speed) => self.spool_set_minmax_max_speed(speed),
            Mutation::SetSpoolForward(value) => self.spool_set_forward(value),
            Mutation::SetSpoolAdaptiveTensionTarget(value) => {
                self.spool_set_adaptive_tension_target(value)
            }
            Mutation::SetSpoolAdaptiveRadiusLearningRate(value) => {
                self.spool_set_adaptive_radius_learning_rate(value)
            }
            Mutation::SetSpoolAdaptiveMaxSpeedMultiplier(value) => {
                self.spool_set_adaptive_max_speed_multiplier(value)
            }
            Mutation::SetSpoolAdaptiveAccelerationFactor(value) => {
                self.spool_set_adaptive_acceleration_factor(value)
            }
            Mutation::SetSpoolAdaptiveDeaccelerationUrgencyMultiplier(value) => {
                self.spool_set_adaptive_deacceleration_urgency_multiplier(value)
            }
            Mutation::SetSpoolAutomaticRequiredMeters(meters) => {
                self.set_spool_automatic_required_meters(meters)
            }
            Mutation::SetSpoolAutomaticAction(mode) => self.set_spool_automatic_mode(mode),
            Mutation::ResetSpoolProgress => self.stop_or_pull_spool_reset(Instant::now()),
            Mutation::ZeroTensionArmAngle => self.tension_arm_zero(),

            // puller adaptive speed algorithm
            Mutation::SetPullerAdaptiveMaxSpeedChangePercent(v) => {
                self.puller_set_adaptive_max_speed_change_percent(v)
            }
            Mutation::SetPullerAdaptiveAdjustmentIntervalMeters(v) => {
                self.puller_set_adaptive_adjustment_interval_meters(v)
            }
            Mutation::SetPullerAdaptiveStepPercent(v) => self.puller_set_adaptive_step_percent(v),
            Mutation::SetPullerAdaptiveAcceptedDifference(v) => {
                self.puller_set_adaptive_accepted_difference(v)
            }
        }

        Ok(())
    }
}
