use std::time::{Duration, Instant};

use control_core::{
    controllers::first_degree_motion::angular_acceleration_speed_controller::AngularAccelerationSpeedController,
    helpers::{
        interpolation::{interpolate_exponential, scale},
        moving_time_window::MovingTimeWindow,
    },
};
use units::{
    AngularAcceleration, AngularVelocity, ConstZero, Length, Velocity, angular_acceleration::radian_per_second_squared, angular_velocity::radian_per_second, length::{centimeter, meter}, velocity::meter_per_second
};

use super::AlgorithmInput;
use crate::types::Bounds;

pub struct AdaptiveSpeedAlgorithm {
    constants: Constants,
    variables: Variables,
    variables_default: Variables,

    // state
    speed_factor: Length,
    last_max_speed_factor_update: Option<Instant>,

    // controllers
    acceleration_controller: AngularAccelerationSpeedController,
}

impl AdaptiveSpeedAlgorithm
{
    pub fn compute_speed(&mut self, input: AlgorithmInput) -> AngularVelocity 
    {
        self.update_speed_factor(input.filament_tension, input.t);

        let speed_raw = match input.enabled {
            true => self.compute_raw_speed(input.filament_tension),
            false => self.min_speed(),
        };

        

        todo!();
    }
}

impl AdaptiveSpeedAlgorithm
{
    fn compute_raw_speed(&self, filament_tension: f64) -> AngularVelocity {
        let filament_tension_inverted = 1.0 - filament_tension;

        AngularVelocity::new::<radian_per_second>(scale(
            filament_tension_inverted,
            self.min_speed().get::<radian_per_second>(),
            self.compute_speed_max().get::<radian_per_second>(),
        ))
    }

    fn update_acceleration_limits(&mut self, target_speed: AngularVelocity) 
    {
        

        let speed_target_rad_s = target_speed.get::<radian_per_second>();
        let speed_max_current = self.compute_speed_max(puller);
        let max_speed_rad_s = speed_max_current.get::<radian_per_second>();

        // Base acceleration proportional to current max operating speed
        let base_acceleration = max_speed_rad_s * self.acceleration_factor;

        // Simple urgency factor - dramatically increases near zero
        let urgency_factor = if speed_target_rad_s.abs() < 0.1 {
            self.deacceleration_urgency_multiplier * (1.0 / (speed_target_rad_s.abs() + 0.01))
        } else {
            1.0
        };

        // Final acceleration limit
        let acceleration_limit =
            (base_acceleration * urgency_factor).max(self.config.acceleration_limit_min);

        let acceleration =
            AngularAcceleration::new::<radian_per_second_squared>(acceleration_limit);

        // Update acceleration controller limits
        self.acceleration_controller
            .set_max_acceleration(acceleration);
        self.acceleration_controller
            .set_min_acceleration(-acceleration);
    }

    fn update_speed_factor(&mut self, filament_tension: f64, t: Instant) 
    {
        let delta_t = match self.last_max_speed_factor_update {
            Some(last_update) => t.duration_since(last_update).as_secs_f64(),
            None => {
                // First call, initialize and return early
                self.last_max_speed_factor_update = Some(t);
                return;
            }
        };

        // positive error means too much tension, so we reduce speed
        // negative error means too little tension, so we increase speed
        let tension_error = filament_tension - self.tension_target;

        // Calculate proportional control adjustment
        let proportional_gain = self.radius_learning_rate * delta_t;
        let factor_change = tension_error * proportional_gain;

        // Update the speed factor directly
        let new_factor = (self.speed_factor.get::<centimeter>() + factor_change)
            .clamp(self.config.speed_factor_min, Self::FACTOR_MAX);

        self.config.speed_factor_limits.clamp(value);

        // Convert to cm
        self.speed_factor = Length::new::<centimeter>(new_factor);

        self.last_max_speed_factor_update = Some(t);
    }

    fn compute_speed_max(&self, puller_speed: Velocity) -> AngularVelocity 
    {
        let puller_speed = puller_speed.get::<meter_per_second>();
        let speed_factor = self.speed_factor.get::<meter>();

        // 13
        let speed = (puller_speed / speed_factor) * self.variables.speed_multiplier_max;

        AngularVelocity::new::<radian_per_second>(speed)
    }
}

pub struct Constants 
{
    /// Hard limits for speed
    pub speed_limits_hard: Bounds<AngularVelocity>,

    /// Target normalized tension value (0.0-1.0) that the controller tries to maintain
    pub tension_target: f64,

    /// Proportional control gain for adaptive learning (negative: higher tension reduces speed)
    pub radius_learning_rate: f64,

    /// TODO: add documenation
    pub speed_factor_limits: Bounds<f64>,

    /// if the tension is the lowest, the speed can be up to X the puller speed
    pub speed_multiplier_max: f64,

    /// Base acceleration as a fraction of max possible speed (per second)
    pub acceleration_factor: f64,

    /// Urgency multiplier for near-zero target speeds
    pub deacceleration_urgency_multiplier: f64,

    /// Minimum acceleration limit to prevent completely frozen motion
    pub acceleration_limit_min: AngularVelocity
}

pub struct Variables 
{
    /// Target normalized tension value (0.0-1.0) that the controller tries to maintain
    pub tension_target: f64,

    /// Proportional control gain for adaptive learning (negative: higher tension reduces speed)
    pub radius_learning_rate: f64,

    /// Speed multiplier when tension is at minimum (max speed factor)
    pub speed_multiplier_max: f64,

    /// Base acceleration as a fraction of max possible speed (per second)
    pub acceleration_factor: f64,

    /// Urgency multiplier for near-zero target speeds
    pub deacceleration_urgency_multiplier: f64,
}