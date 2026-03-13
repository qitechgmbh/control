use std::{ops::Bound, time::{Duration, Instant}};

use control_core::{controllers::first_degree_motion::angular_acceleration_speed_controller::AngularAccelerationSpeedController, helpers::{interpolation::{interpolate_exponential, scale}, moving_time_window::MovingTimeWindow}};
use units::{AngularAcceleration, AngularVelocity, ConstZero, Length, Velocity, angular_acceleration::{radian_per_second_squared, revolution_per_minute_per_second}, angular_velocity::{radian_per_second, revolution_per_minute}, length::meter, velocity::meter_per_second};

mod filament_tension;
use filament_tension::FilamentTensionCalculator;

pub mod min_max;
pub use min_max::MinMaxSpeedAlgorithm;

mod adapative;
pub use adapative::AdaptiveSpeedController;

use crate::{types::Bounds, winder::devices::{Puller, TensionArm}};

pub trait SpeedController
{
    fn speed(&self) -> AngularVelocity;
    fn set_speed(&mut self, speed: AngularVelocity);

    fn set_enabled(&mut self, enabled: bool);

    fn reset(&mut self);

    fn update_speed(
        &mut self, 
        t: Instant,
        multiplier: f64,
        tension_arm: &TensionArm, 
        puller: &Puller
    ) -> AngularVelocity;
}


pub struct TheSpeedController 
{
    enabled: bool,
    speed: AngularVelocity,
    filament_tension_calculator: FilamentTensionCalculator,
}

pub struct Config 
{
    speed_max: AngularVelocity,
    speed_max_safety: AngularVelocity,

    speed_window_duration: Duration,
    speed_window_max_samples: u32,

    tension_target: f64,
}

impl TheSpeedController
{
    /// Maximum speed limit (in RPM) used to initialize the acceleration controller
    const INITIAL_MAX_SPEED_RPM: f64 = 150.0;

    pub fn new(config: Config) -> Self
    {
        let acceleration_controller =  AngularAccelerationSpeedController::new(
            Some(AngularVelocity::ZERO),
            Some(config.speed_max),
            -AngularAcceleration::ZERO, // Will be dynamically adjusted
            AngularAcceleration::ZERO,  // Will be dynamically adjusted
            AngularVelocity::ZERO,
        );
    }

    fn update_acceleration(
        &mut self, 
        t: Instant,
        target_speed: AngularVelocity, 
        acceleration: AngularAcceleration, 
    ) -> AngularVelocity
    {
        // Update acceleration controller limits
        self.acceleration_controller
            .set_max_acceleration(acceleration);
        self.acceleration_controller
            .set_min_acceleration(-acceleration);

        let new_speed = self.acceleration_controller.update(target_speed, t);

        // Record for diagnostics
        self.speed_time_window.update(new_speed.get::<radian_per_second>(), t);

        new_speed
    }

    fn clamp_speed(&mut self, speed: AngularVelocity) -> AngularVelocity {
        let min_speed = self.min_speed();
        let max_speed = self.max_speed();

        if speed < min_speed {
            AngularVelocity::ZERO
        } else if speed > max_speed {
            max_speed
        } else {
            speed
        }
    }

    fn compute_speed(&mut self)
    {

    }

    fn compute_acceleration(&mut self)
    {

    }
}

pub struct AlgorithmInput
{
    enabled: bool,
    t: Instant,
    filament_tension: f64,
    puller_speed: Velocity,
}

pub struct MinMaxSpeedAlgorithm
{
    /// Unit is angular velocity in rad/s
    speed_time_window: MovingTimeWindow<f64>,

    acceleration_controller: AngularAccelerationSpeedController,
}

pub struct MinMaxSpeedAlgorithmConfig
{
    pub speed_max_initial: AngularVelocity,
    pub speed_window_duration: Duration,
    pub speed_window_samples_max: usize,
}

impl MinMaxSpeedAlgorithm
{
    pub fn new(config: MinMaxSpeedAlgorithmConfig) -> Self
    {
        let speed_time_window = MovingTimeWindow::new(
            config.speed_window_duration,
            config.speed_window_samples_max,
        );

        let acceleration_controller = AngularAccelerationSpeedController::new(
            Some(AngularVelocity::ZERO),
            Some(config.speed_max_initial),
            -AngularAcceleration::ZERO, // Will be dynamically adjusted
            AngularAcceleration::ZERO,  // Will be dynamically adjusted
            AngularVelocity::ZERO,
        );

        Self { speed_time_window, acceleration_controller }
    }

    pub fn compute_speed(&mut self, input: AlgorithmInput) -> AngularVelocity 
    {
        let speed_raw = match input.enabled {
            true => self.compute_raw_speed(input.filament_tension),
            false => self.min_speed(),
        };

        self.update_acceleration_limits(speed_raw);

        let speed = self.acceleration_controller.update(speed_raw, input.t);

        // add new speed to the time window
        self.speed_time_window.update(speed.get::<radian_per_second>(), input.t);

        speed
    }

    pub fn set_max_speed(&mut self, max_speed: AngularVelocity) {
        self.acceleration_controller.set_max_speed(Some(max_speed));
        self.recompute_acceleration_limits();
    }

    pub fn set_min_speed(&mut self, min_speed: AngularVelocity) {
        self.acceleration_controller.set_min_speed(Some(min_speed));
        self.recompute_acceleration_limits();
    }

    pub fn min_speed(&self) -> AngularVelocity {
        self.acceleration_controller
            .get_min_speed()
            .unwrap_or(AngularVelocity::ZERO)
    }

    pub fn max_speed(&self) -> AngularVelocity {
        self.acceleration_controller
            .get_max_speed()
            .unwrap_or(AngularVelocity::ZERO)
    }

    pub fn reset(&mut self) {
        self.acceleration_controller.reset(AngularVelocity::ZERO);
    }
}

// utils
impl MinMaxSpeedAlgorithm
{
    fn compute_raw_speed(&self, filament_tension: f64) -> AngularVelocity
    {
        let filament_tension_inverted = 
            1.0 - filament_tension;

        let filament_tension_exponential = 
            interpolate_exponential(filament_tension_inverted, 2.0);

        AngularVelocity::new::<radian_per_second>(scale(
            filament_tension_exponential,
            self.min_speed().get::<radian_per_second>(),
            self.max_speed().get::<radian_per_second>(),
        ))
    }

    fn update_acceleration_limits(&mut self, speed_target: AngularVelocity)
    {
        // The magic factor is dependent on the scceleration settings on the puller speed controller to reduce oscillation
        const MAGIC_FACTOR: f64 = 0.5;

        // highest achieved speed in time window
        let speed_peak = self.speed_time_window.max().abs();

        // highest speed between achieved and current
        let speed_max = speed_peak.max(speed_target.get::<radian_per_second>().abs());

        let acceleration = AngularAcceleration::new::<radian_per_second_squared>(
            speed_max * MAGIC_FACTOR
        );

        self.acceleration_controller.set_max_acceleration(acceleration);
        self.acceleration_controller.set_min_acceleration(-acceleration);
    }

    fn recompute_acceleration_limits(&mut self) {
        // Set acceleration to 1/4 of the range between min and max speed
        // The spool will accelerate from min to max speed in 4 seconds
        let min_speed = self.min_speed();
        let max_speed = self.max_speed();
        let range = max_speed - min_speed;
        let acceleration = AngularAcceleration::new::<revolution_per_minute_per_second>(
            range.get::<revolution_per_minute>() / 4.0,
        );
        self.acceleration_controller.set_max_acceleration(acceleration);
        self.acceleration_controller.set_min_acceleration(-acceleration);
    }
}








pub struct AdaptiveSpeedAlgorithm
{
    config: AdaptiveSpeedAlgorithmConfig,

    speed_factor: Length,

    /// Target normalized tension value (0.0-1.0) that the controller tries to maintain
    tension_target: f64,
    /// Proportional control gain for adaptive learning (negative: higher tension reduces speed)
    radius_learning_rate: f64,
    /// Speed multiplier when tension is at minimum (max speed factor)
    speed_multiplier_max: f64,
    /// Base acceleration as a fraction of max possible speed (per second)
    acceleration_factor: f64,
    /// Urgency multiplier for near-zero target speeds
    deacceleration_urgency_multiplier: f64,
}

pub struct AdaptiveSpeedAlgorithmConfig
{
    /// Safety speed limits which can never be exceeded
    speed_limits_safety:  Bounds<Velocity>,

    speed_factor_limits:    Bounds<f64>,
    speed_multiplier_max:   f64,

    /// Minimum acceleration limit to prevent completely frozen motion
    acceleration_limit_min: f64, 
}

impl AdaptiveSpeedAlgorithm
{
    fn compute_speed(&mut self, enabled: bool, filament_tension: f64, puller: &Puller)
    {
        let min_speed = AngularVelocity::ZERO;
        let max_speed = self.compute_max_speed(puller).abs();

        let speed = match enabled {
            true => self.calculate_speed(t, tension_arm, puller),
            false => AngularVelocity::ZERO,
        };
    }

    fn accelerate_speed(
        &mut self,
        target_speed: AngularVelocity,
        puller: &Puller,
        t: Instant,
    ) -> AngularAcceleration {
        let target_speed_rad_s = target_speed.get::<radian_per_second>();
        let current_max_speed = self.compute_max_speed(puller);
        let max_speed_rad_s = current_max_speed.get::<radian_per_second>();

        // Base acceleration proportional to current max operating speed
        let base_acceleration = max_speed_rad_s * self.acceleration_factor;

        // Simple urgency factor - dramatically increases near zero
        let urgency_factor = 
            if target_speed_rad_s.abs() < 0.1 {
                self.deacceleration_urgency_multiplier * (1.0 / (target_speed_rad_s.abs() + 0.01))
            } else {
                1.0
            };

        // Final acceleration limit
        let acceleration_limit =
            (base_acceleration * urgency_factor).max(self.config.acceleration_limit_min);

        AngularAcceleration::new::<radian_per_second_squared>(acceleration_limit)
    }

    fn compute_max_speed(&self, puller: &Puller) -> AngularVelocity 
    {
        let puller_speed = puller.output_speed().get::<meter_per_second>();
        let speed_factor = self.speed_factor.get::<meter>();

        let speed = (puller_speed / speed_factor) * self.speed_multiplier_max;

        AngularVelocity::new::<radian_per_second>(speed)
    }
}