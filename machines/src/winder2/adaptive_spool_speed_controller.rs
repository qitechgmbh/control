use crate::winder2::{
    clamp_revolution::clamp_revolution_uom, filament_tension::FilamentTensionCalculator,
    puller_speed_controller::PullerSpeedController,
};

use super::{clamp_revolution::Clamping, tension_arm::TensionArm};
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

/// Adaptive spool speed controller that automatically adjusts to maintain optimal filament tension.
///
/// This controller monitors filament tension via the tension arm and learns the appropriate
/// maximum speed based on puller speed and tension feedback. It uses closed-loop control
/// to minimize tension error and applies smooth acceleration to prevent sudden motor commands.
#[derive(Debug)]
pub struct AdaptiveSpoolSpeedController {
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

impl Default for AdaptiveSpoolSpeedController {
    fn default() -> Self {
        Self::new()
    }
}

impl AdaptiveSpoolSpeedController {
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

    /// if the tension is the lowest, the speed can be up to 2x the puller speed
    const MAX_SPEED_MULTIPLIER: f64 = 4.0;

    /// Base acceleration as a fraction of max possible speed (per second)
    const ACCELERATION_FACTOR: f64 = 0.2; // 20% of max speed per second

    /// Urgency multiplier for near-zero target speeds
    const DEACCELERATION_URGENCY_MULTIPLIER: f64 = 15.0;

    /// Minimum acceleration limit to prevent completely frozen motion
    const MIN_ACCELERATION_LIMIT: f64 = 0.5; // rad/s²

    /// Creates a new adaptive spool speed controller with default settings.
    ///
    /// Initializes all internal components and sets baseline parameters.
    ///
    /// # Returns
    /// A new `AdaptiveSpoolSpeedController` instance ready for use.
    pub fn new() -> Self {
        let max_speed = AngularVelocity::new::<revolution_per_minute>(Self::INITIAL_MAX_SPEED_RPM);

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

    /// Calculates the current maximum speed based on puller speed and learned factor.
    ///
    /// Uses the fundamental relationship between linear and angular velocity:
    /// ω = (v / r) × multiplier
    ///
    /// Where:
    /// - ω (omega) = angular velocity in radians per second
    /// - v = linear velocity in meters per second  
    /// - r = radius in meters
    /// - multiplier = MAX_SPEED_MULTIPLIER for speed scaling
    ///
    /// This ensures the spool's rim speed is properly scaled relative to the puller's
    /// linear speed, with the multiplier allowing for tension control adjustments.
    ///
    /// # Parameters
    /// - `puller_speed_controller`: Reference to puller for baseline speed calculation
    ///
    /// # Returns
    /// Current maximum angular velocity for the spool in radians per second
    fn get_max_speed(&self, puller_speed_controller: &PullerSpeedController) -> AngularVelocity {
        AngularVelocity::new::<radian_per_second>(
            (puller_speed_controller.last_speed.get::<meter_per_second>()
                / self.speed_factor.get::<meter>())
                * self.max_speed_multiplier,
        )
    }

    /// Calculates the desired spool speed based on filament tension feedback.
    ///
    /// Reads the tension arm angle, converts it to normalized tension, and calculates
    /// the appropriate speed. Also updates the learned maximum speed based on tension error.
    ///
    /// # Parameters
    /// - `t`: Current timestamp for learning updates
    /// - `tension_arm`: Reference to tension arm for angle measurements
    /// - `puller_speed_controller`: Reference to puller for baseline speed calculations
    ///
    /// # Returns
    /// The calculated target angular velocity for the spool motor
    fn calculate_speed(
        &mut self,
        t: Instant,
        tension_arm: &TensionArm,
        puller_speed_controller: &PullerSpeedController,
    ) -> AngularVelocity {
        let min_speed = AngularVelocity::ZERO;
        let max_speed = self.get_max_speed(puller_speed_controller).abs();

        // Calculate filament tension from arm angle
        let tension_arm_angle = tension_arm.get_angle();
        let (clamped_angle, clamping_state) = clamp_revolution_uom(
            tension_arm_angle,
            self.filament_calc.get_max_angle(), // Inverted because min angle = max tension
            self.filament_calc.get_min_angle(),
        );

        // Return minimum speed if tension arm is at limits
        if matches!(clamping_state, Clamping::Min | Clamping::Max) {
            return min_speed;
        }

        // 1.0 means maximum tension (high angle, low speed)
        // 0.0 means minimum tension (low angle, high speed)
        let filament_tension = self.filament_calc.calc_filament_tension(clamped_angle);

        self.update_speed_factor(filament_tension, t);

        // Calculate speed based on inverted tension (lower tension = higher speed)

        AngularVelocity::new::<radian_per_second>(scale(
            1.0 - filament_tension,
            min_speed.get::<radian_per_second>(),
            max_speed.get::<radian_per_second>(),
        ))
    }

    /// Simplified urgency-weighted acceleration that adapts to current operating conditions.
    ///
    /// This approach uses the current max speed (calculated from puller speed and learned diameter)
    /// as the basis for acceleration limits, ensuring smooth operation across all speed ranges
    /// while providing instant deceleration capability when approaching zero.
    ///
    /// # Parameters
    /// - `target_speed`: Target speed to accelerate towards
    /// - `puller_speed_controller`: Reference to puller for current max speed calculation
    /// - `t`: Current timestamp for the acceleration controller
    ///
    /// # Returns
    /// Smoothly accelerated speed with urgency-weighted limits
    fn accelerate_speed(
        &mut self,
        target_speed: AngularVelocity,
        puller_speed_controller: &PullerSpeedController,
        t: Instant,
    ) -> AngularVelocity {
        let target_speed_rad_s = target_speed.get::<radian_per_second>();
        let current_max_speed = self.get_max_speed(puller_speed_controller);
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
        self.acceleration_controller
            .set_max_acceleration(acceleration);
        self.acceleration_controller
            .set_min_acceleration(-acceleration);

        let new_speed = self.acceleration_controller.update(target_speed, t);

        // Record for diagnostics
        self.speed_time_window
            .update(new_speed.get::<radian_per_second>(), t);

        new_speed
    }

    /// Safety function that enforces absolute speed limits.
    ///
    /// Ensures the spool speed never exceeds safe operational limits to protect hardware.
    /// Acts as a final safety check after all other speed calculations.
    ///
    /// # Parameters
    /// - `speed`: The speed to be safety-checked and clamped
    ///
    /// # Returns
    /// Speed guaranteed to be within safe operational limits
    fn clamp_speed(&self, speed: AngularVelocity) -> AngularVelocity {
        let min_speed = AngularVelocity::ZERO;
        let max_speed = AngularVelocity::new::<revolution_per_minute>(Self::SAFETY_MAX_SPEED_RPM);

        speed.max(min_speed).min(max_speed)
    }

    /// Adaptive learning algorithm that adjusts maximum speed factor based on tension feedback.
    ///
    /// Uses proportional control to adjust the speed factor based on tension error.
    /// The controller learns over time to maintain optimal tension by scaling the
    /// baseline speed derived from puller speed.
    ///
    /// # Parameters
    /// - `filament_tension`: Normalized tension value
    /// - `t`: Current timestamp for time-aware learning rate
    fn update_speed_factor(&mut self, filament_tension: f64, t: Instant) {
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
            .clamp(Self::FACTOR_MIN, Self::FACTOR_MAX);

        self.speed_factor = Length::new::<centimeter>(new_factor); // Convert to cm

        self.last_max_speed_factor_update = Some(t);
    }

    /// Main update function that orchestrates the complete speed control pipeline.
    ///
    /// This is the primary interface that calculates the target speed, applies enable/disable
    /// logic, smooths speed changes, and applies safety limits.
    ///
    /// Call this function every control loop iteration to get the commanded spool speed.
    ///
    /// # Parameters
    /// - `t`: Current timestamp for time-based calculations
    /// - `tension_arm`: Reference to tension arm for feedback
    /// - `puller_speed_controller`: Reference for adaptive speed scaling
    ///
    /// # Returns
    /// Final commanded angular velocity for the spool motor
    pub fn update_speed(
        &mut self,
        t: Instant,
        tension_arm: &TensionArm,
        puller_speed_controller: &PullerSpeedController,
    ) -> AngularVelocity {
        let target_speed = self.calculate_speed(t, tension_arm, puller_speed_controller);

        let enabled_speed = if self.enabled {
            target_speed
        } else {
            AngularVelocity::ZERO
        };

        let accelerated_speed = self.accelerate_speed(enabled_speed, puller_speed_controller, t);

        // Store speed before clamping to preserve the actual commanded value
        self.last_speed = accelerated_speed;

        self.clamp_speed(accelerated_speed)
    }

    /// Enables or disables the speed controller.
    ///
    /// When disabled, the controller returns zero speed regardless of tension feedback.
    ///
    /// # Parameters
    /// - `enabled`: True to enable speed control, false to disable
    pub const fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Returns whether the speed controller is currently enabled.
    ///
    /// # Returns
    /// True if the controller is enabled, false if disabled
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Resets the controller to initial state, clearing all learned parameters.
    ///
    /// Use this when starting a new winding operation or after significant
    /// process changes that would invalidate learned parameters.
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

    /// Returns the last commanded speed from the controller.
    ///
    /// # Returns
    /// The angular velocity that was last commanded to the spool motor
    pub fn get_speed(&self) -> AngularVelocity {
        self.last_speed
    }

    /// Manually sets the current speed, bypassing normal calculation.
    ///
    /// Use this for manual control or when switching control modes to ensure
    /// smooth transitions.
    ///
    /// # Parameters
    /// - `speed`: The angular velocity to set as the current speed
    pub fn set_speed(&mut self, speed: AngularVelocity) {
        self.last_speed = speed;
        self.acceleration_controller.reset(speed);
    }

    pub fn get_speed_factor(&self) -> Length {
        self.speed_factor
    }

    // Getters and setters for the new configurable parameters
    pub const fn get_tension_target(&self) -> f64 {
        self.tension_target
    }

    pub const fn set_tension_target(&mut self, tension_target: f64) {
        self.tension_target = tension_target.clamp(0.0, 1.0);
    }

    pub const fn get_radius_learning_rate(&self) -> f64 {
        self.radius_learning_rate
    }

    pub const fn set_radius_learning_rate(&mut self, radius_learning_rate: f64) {
        self.radius_learning_rate = radius_learning_rate.max(0.0);
    }

    pub const fn get_max_speed_multiplier(&self) -> f64 {
        self.max_speed_multiplier
    }

    pub const fn set_max_speed_multiplier(&mut self, max_speed_multiplier: f64) {
        self.max_speed_multiplier = max_speed_multiplier.max(0.1);
    }

    pub const fn get_acceleration_factor(&self) -> f64 {
        self.acceleration_factor
    }

    pub const fn set_acceleration_factor(&mut self, acceleration_factor: f64) {
        self.acceleration_factor = acceleration_factor.clamp(0.01, 1.0);
    }

    pub const fn get_deacceleration_urgency_multiplier(&self) -> f64 {
        self.deacceleration_urgency_multiplier
    }

    pub const fn set_deacceleration_urgency_multiplier(
        &mut self,
        deacceleration_urgency_multiplier: f64,
    ) {
        self.deacceleration_urgency_multiplier = deacceleration_urgency_multiplier.max(1.0);
    }
}
