use std::time::Duration;

use control_core::{controllers::first_degree_motion::angular_acceleration_speed_controller::AngularAccelerationSpeedController, helpers::{interpolation::{interpolate_exponential, scale}, moving_time_window::MovingTimeWindow}};
use units::{AngularAcceleration, AngularVelocity, ConstZero, angular_acceleration::{radian_per_second_squared, revolution_per_minute_per_second}, angular_velocity::{radian_per_second, revolution_per_minute}};

use crate::types::Bounds;
use super::AlgorithmInput;

pub struct MinMaxSpeedAlgorithm
{
    config: Config,

    /// Unit is AngularVelocity<radian_per_second>
    speed_time_window: MovingTimeWindow<f64>,

    acceleration_controller: AngularAccelerationSpeedController,
}

// public interface
impl MinMaxSpeedAlgorithm
{
    pub fn new(config: Config) -> Self
    {
        let speed_time_window = MovingTimeWindow::new(
            config.speed_window_duration,
            config.speed_window_samples_max,
        );

        let acceleration_controller = AngularAccelerationSpeedController::new(
            Some(config.speed_limits_default.min),
            Some(config.speed_limits_default.max),
            -AngularAcceleration::ZERO, // Will be dynamically adjusted
            AngularAcceleration::ZERO,  // Will be dynamically adjusted
            AngularVelocity::ZERO,
        );

        Self { config, speed_time_window, acceleration_controller }
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

    pub fn reset(&mut self) {
        self.acceleration_controller.reset(AngularVelocity::ZERO);
    }
}

// getters + setters
impl MinMaxSpeedAlgorithm
{
    pub fn min_speed(&self) -> AngularVelocity {
        self.acceleration_controller
            .get_min_speed()
            .unwrap_or(AngularVelocity::ZERO)
    }

    pub fn set_min_speed(&mut self, min_speed: AngularVelocity) {
        let value = self.config.speed_limits_safety.clamp(min_speed);
        self.acceleration_controller.set_min_speed(Some(value));
        self.recompute_acceleration_limits();
    }

    pub fn max_speed(&self) -> AngularVelocity {
        self.acceleration_controller
            .get_max_speed()
            .unwrap_or(AngularVelocity::ZERO)
    }

    pub fn set_max_speed(&mut self, max_speed: AngularVelocity) {
        let value = self.config.speed_limits_safety.clamp(max_speed);
        self.acceleration_controller.set_max_speed(Some(value));
        self.recompute_acceleration_limits();
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
        // highest achieved speed in time window
        let speed_peak = self.speed_time_window.max().abs();

        // highest speed between achieved and current
        let speed_max = speed_peak.max(speed_target.get::<radian_per_second>().abs());

        let acceleration = AngularAcceleration::new::<radian_per_second_squared>(
            speed_max * self.config.magic_factor
        );

        self.acceleration_controller.set_max_acceleration(acceleration);
        self.acceleration_controller.set_min_acceleration(-acceleration);
    }

    /// Called when modifying min/max speed to reset acceleration and 
    /// restore it over the configured duration
    fn recompute_acceleration_limits(&mut self) {
        let factor = self.config.acceleration_duration_after_recompute.as_secs_f64();

        let range = (self.max_speed() - self.min_speed()).get::<revolution_per_minute>();

        // divide range by factor so we start at the first step
        let acceleration = AngularAcceleration::new::<revolution_per_minute_per_second>(
            range / factor,
        );

        self.acceleration_controller.set_max_acceleration(acceleration);
        self.acceleration_controller.set_min_acceleration(-acceleration);
    }
}

pub struct Config
{
    pub speed_limits_default: Bounds<AngularVelocity>,
    pub speed_limits_safety: Bounds<AngularVelocity>,

    /// The magic factor is dependent on the scceleration settings on the puller speed controller to reduce oscillation
    pub magic_factor: f64, // 0.5

    pub speed_max_initial: AngularVelocity,
    pub speed_window_duration: Duration,
    pub speed_window_samples_max: usize,

    pub acceleration_duration_after_recompute: Duration,
}

impl Config
{
    
}