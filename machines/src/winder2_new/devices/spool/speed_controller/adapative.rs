use control_core::{
    controllers::first_degree_motion::angular_acceleration_speed_controller::AngularAccelerationSpeedController,
    helpers::{interpolation::scale, moving_time_window::MovingTimeWindow},
};
use core::f64;
use std::time::Instant;
use units::ConstZero;
use units::angle::degree;
use units::angular_acceleration::radian_per_second_squared;
use units::angular_velocity::{radian_per_second, revolution_per_minute};
use units::f64::*;
use units::length::{centimeter, meter};
use units::velocity::meter_per_second;

use crate::{helpers::{Clamp, clamp_revolution_uom}, winder2_new::devices::spool::speed_controller::SpeedController};
use crate::winder2_new::devices::{Puller, TensionArm};

use super::helpers::FilamentTensionCalculator;

#[derive(Debug)]
pub struct AdaptiveSpeedController {
    /// Last commanded angular velocity sent to the spool motor
    last_speed: AngularVelocity,
    /// Whether the speed controller is enabled (false = always returns zero speed)
    enabled: bool,
    /// Acceleration controller to smooth speed transitions and prevent sudden changes
    acceleration_controller: AngularAccelerationSpeedController,
    /// Calculator for converting tension arm angle to normalized filament tension
    filament_calc: FilamentTensionCalculator,
    /// Moving window of recent speeds (in rad/s) used for dynamic acceleration limit calculation
    speed_time_window: MovingTimeWindow<f64>,
    /// Factor to control/calculate the feed speed based on filament tension
    speed_factor: Length,
    /// Timestamp of last max speed factor update, used for time-aware learning rate calculation
    last_max_speed_factor_update: Option<Instant>,

    /// Target normalized tension value (0.0-1.0) that the controller tries to maintain
    tension_target: f64,
    /// Proportional control gain for adaptive learning (negative: higher tension reduces speed)
    radius_learning_rate: f64,
    /// Speed multiplier when tension is at minimum (max speed factor)
    max_speed_multiplier: f64,
    /// Base acceleration as a fraction of max possible speed (per second)
    acceleration_factor: f64,
    /// Urgency multiplier for near-zero target speeds
    deacceleration_urgency_multiplier: f64,
}

// Constants
impl AdaptiveSpeedController 
{
    /// Maximum speed limit (in RPM) used to initialize the acceleration controller
    const INITIAL_MAX_SPEED_RPM: f64 = 150.0;

    /// Absolute safety limit (in RPM) that the spool speed can never exceed to protect hardware
    const SAFETY_MAX_SPEED_RPM: f64 = 600.0;

    /// Time window duration (in seconds) for tracking recent speed history in acceleration calculations
    const SPEED_WINDOW_DURATION_SECS: u64 = 5;

    /// Maximum number of speed samples stored in the moving time window
    const SPEED_WINDOW_MAX_SAMPLES: usize = 10;

    /// Maximum angle (in degrees) of the tension arm, representing loosest filament tension
    const TENSION_ARM_MAX_ANGLE_DEG: f64 = 90.0;

    /// Minimum angle (in degrees) of the tension arm, representing tightest filament tension
    const TENSION_ARM_MIN_ANGLE_DEG: f64 = 20.0;

    /// Target normalized tension value (0.0-1.0) that the controller tries to maintain
    const TENSION_TARGET: f64 = 0.7;

    /// Proportional control gain for adaptive learning (negative: higher tension reduces speed)
    const RADIUS_LEARNING_RATE: f64 = 0.5;

    const FACTOR_MIN: f64 = 4.25;

    const FACTOR_MAX: f64 = 20.0;

    /// if the tension is the lowest, the speed can be up to X the puller speed
    const MAX_SPEED_MULTIPLIER: f64 = 4.0;

    /// Base acceleration as a fraction of max possible speed (per second)
    const ACCELERATION_FACTOR: f64 = 0.2; // 20% of max speed per second

    /// Urgency multiplier for near-zero target speeds
    const DEACCELERATION_URGENCY_MULTIPLIER: f64 = 15.0;

    /// Minimum acceleration limit to prevent completely frozen motion
    const MIN_ACCELERATION_LIMIT: f64 = 0.5; // rad/s²
}

// getter + setter
impl AdaptiveSpeedController
{
    pub const fn is_enabled(&self) -> bool 
    {
        self.enabled
    }

    pub const fn set_enabled(&mut self, enabled: bool) 
    {
        self.enabled = enabled;
    }

    pub fn speed(&self) -> AngularVelocity {
        self.last_speed
    }

    pub fn set_speed(&mut self, speed: AngularVelocity) {
        self.last_speed = speed;
        self.acceleration_controller.reset(speed);
    }

    pub fn speed_factor(&self) -> Length {
        self.speed_factor
    }

    // Getters and setters for the new configurable parameters
    pub const fn tension_target(&self) -> f64 {
        self.tension_target
    }

    pub const fn set_tension_target(&mut self, value: f64) {
        self.tension_target = value.clamp(0.0, 1.0);
    }

    pub const fn radius_learning_rate(&self) -> f64 {
        self.radius_learning_rate
    }

    pub const fn set_radius_learning_rate(&mut self, radius_learning_rate: f64) {
        self.radius_learning_rate = radius_learning_rate.max(0.0);
    }

    pub const fn max_speed_multiplier(&self) -> f64 {
        self.max_speed_multiplier
    }

    pub const fn set_max_speed_multiplier(&mut self, max_speed_multiplier: f64) {
        self.max_speed_multiplier = max_speed_multiplier.max(0.1);
    }

    pub const fn acceleration_factor(&self) -> f64 {
        self.acceleration_factor
    }

    pub const fn set_acceleration_factor(&mut self, acceleration_factor: f64) {
        self.acceleration_factor = acceleration_factor.clamp(0.01, 1.0);
    }

    pub const fn deacceleration_urgency_multiplier(&self) -> f64 {
        self.deacceleration_urgency_multiplier
    }

    pub const fn set_deacceleration_urgency_multiplier(
        &mut self,
        deacceleration_urgency_multiplier: f64,
    ) {
        self.deacceleration_urgency_multiplier = deacceleration_urgency_multiplier.max(1.0);
    }
}

// public interface
impl AdaptiveSpeedController
{
    pub fn new() -> Self 
    {
        use revolution_per_minute as rpm;

        let max_speed = AngularVelocity::new::<rpm>(Self::INITIAL_MAX_SPEED_RPM);

        Self {
            last_speed: AngularVelocity::ZERO,
            enabled: false,
            acceleration_controller: AngularAccelerationSpeedController::new(
                Some(AngularVelocity::ZERO),
                Some(max_speed),
                -AngularAcceleration::ZERO, // Will be dynamically adjusted
                AngularAcceleration::ZERO,  // Will be dynamically adjusted
                AngularVelocity::ZERO,
            ),
            filament_calc: FilamentTensionCalculator::new(
                Angle::new::<degree>(Self::TENSION_ARM_MAX_ANGLE_DEG),
                Angle::new::<degree>(Self::TENSION_ARM_MIN_ANGLE_DEG),
            ),
            speed_time_window: MovingTimeWindow::new(
                std::time::Duration::from_secs(Self::SPEED_WINDOW_DURATION_SECS),
                Self::SPEED_WINDOW_MAX_SAMPLES,
            ),
            speed_factor: Length::new::<centimeter>(4.25),
            last_max_speed_factor_update: None,
            tension_target: Self::TENSION_TARGET,
            radius_learning_rate: Self::RADIUS_LEARNING_RATE,
            max_speed_multiplier: Self::MAX_SPEED_MULTIPLIER,
            acceleration_factor: Self::ACCELERATION_FACTOR,
            deacceleration_urgency_multiplier: Self::DEACCELERATION_URGENCY_MULTIPLIER,
        }
    }

    pub fn update_speed(
        &mut self,
        t: Instant, 
        tension_arm: &TensionArm, 
        puller: &Puller
    ) -> AngularVelocity 
    {
        let target_speed = self.calculate_speed(t, tension_arm, puller);

        let enabled_speed = if self.enabled {
            target_speed
        } else {
            AngularVelocity::ZERO
        };

        let accelerated_speed = self.accelerate_speed(enabled_speed, puller, t);

        // Store speed before clamping to preserve the actual commanded value
        self.last_speed = accelerated_speed;

        self.clamp_speed(accelerated_speed)
    }

    pub fn reset(&mut self) {
        self.last_speed = AngularVelocity::ZERO;
        self.acceleration_controller.reset(AngularVelocity::ZERO);
        self.speed_factor = Length::new::<centimeter>(4.25);
        self.last_max_speed_factor_update = None;
        self.tension_target = Self::TENSION_TARGET;
        self.radius_learning_rate = Self::RADIUS_LEARNING_RATE;
        self.max_speed_multiplier = Self::MAX_SPEED_MULTIPLIER;
        self.acceleration_factor = Self::ACCELERATION_FACTOR;
        self.deacceleration_urgency_multiplier = Self::DEACCELERATION_URGENCY_MULTIPLIER;
    }
}

// utils
impl AdaptiveSpeedController 
{
    fn calculate_speed(
        &mut self,
        t: Instant,
        tension_arm: &TensionArm,
        puller: &Puller,
    ) -> AngularVelocity {
        let min_speed = AngularVelocity::ZERO;
        let max_speed = self.max_speed(puller).abs();

        // Calculate filament tension from arm angle
        let tension_arm_revolution = clamp_revolution_uom(
            tension_arm.get_angle(),
            self.filament_calc.get_max_angle(), // Inverted because min angle = max tension
            self.filament_calc.get_min_angle(),
        );

        // Return minimum speed if tension arm is at limits
        if matches!(tension_arm_revolution.clamp, Clamp::Min | Clamp::Max) 
        {
            return min_speed;
        }

        // 1.0 means maximum tension (high angle, low speed)
        // 0.0 means minimum tension (low angle, high speed)
        let filament_tension = self.filament_calc.calc_filament_tension(tension_arm_revolution.value);

        self.update_speed_factor(filament_tension, t);

        // Calculate speed based on inverted tension (lower tension = higher speed)

        AngularVelocity::new::<radian_per_second>(scale(
            1.0 - filament_tension,
            min_speed.get::<radian_per_second>(),
            max_speed.get::<radian_per_second>(),
        ))
    }

    fn accelerate_speed(
        &mut self,
        target_speed: AngularVelocity,
        puller: &Puller,
        t: Instant,
    ) -> AngularVelocity 
    {
        let target_speed_rad_s = target_speed.get::<radian_per_second>();
        let current_max_speed = self.max_speed(puller);
        let max_speed_rad_s = current_max_speed.get::<radian_per_second>();

        // Base acceleration proportional to current max operating speed
        let base_acceleration = max_speed_rad_s * self.acceleration_factor;

        // Simple urgency factor - dramatically increases near zero
        let urgency_factor = if target_speed_rad_s.abs() < 0.1 {
            self.deacceleration_urgency_multiplier * (1.0 / (target_speed_rad_s.abs() + 0.01))
        } else {
            1.0
        };

        // Final acceleration limit
        let acceleration_limit =
            (base_acceleration * urgency_factor).max(Self::MIN_ACCELERATION_LIMIT);

        let acceleration =
            AngularAcceleration::new::<radian_per_second_squared>(acceleration_limit);

        // Update acceleration controller limits
        self.acceleration_controller.set_max_acceleration(acceleration);
        self.acceleration_controller.set_min_acceleration(-acceleration);

        let new_speed = self.acceleration_controller.update(target_speed, t);

        // Record for diagnostics
        self.speed_time_window.update(new_speed.get::<radian_per_second>(), t);

        new_speed
    }

    fn clamp_speed(&self, speed: AngularVelocity) -> AngularVelocity 
    {
        use revolution_per_minute as rpm;

        let min_speed = AngularVelocity::ZERO;
        let max_speed = AngularVelocity::new::<rpm>(Self::SAFETY_MAX_SPEED_RPM);

        speed.max(min_speed).min(max_speed)
    }

    fn update_speed_factor(&mut self, filament_tension: f64, t: Instant) 
    {
        let delta_t = match self.last_max_speed_factor_update 
        {
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
            .clamp(Self::FACTOR_MIN, Self::FACTOR_MAX);

        self.speed_factor = Length::new::<centimeter>(new_factor); // Convert to cm

        self.last_max_speed_factor_update = Some(t);
    }

    fn max_speed(&self, puller: &Puller) -> AngularVelocity 
    {
        let puller_speed = puller.output_speed().get::<meter_per_second>();
        let speed_factor = self.speed_factor.get::<meter>();

        let speed = (puller_speed / speed_factor) * self.max_speed_multiplier;

        AngularVelocity::new::<radian_per_second>(speed)
    }
}

impl SpeedController for AdaptiveSpeedController
{
    fn speed(&self) -> AngularVelocity 
    {
        todo!()
    }

    fn set_speed(&mut self, speed: AngularVelocity) 
    {
        todo!()
    }

    fn is_enabled(&self) -> bool 
    {
        todo!()
    }

    fn set_enabled(&mut self, enabled: bool) 
    {
        todo!()
    }

    fn update_speed(&mut self, t: Instant, tension_arm: &TensionArm, puller: &Puller) 
    {
        todo!()
    }
    
    fn reset(&mut self) {
        todo!()
    }
}