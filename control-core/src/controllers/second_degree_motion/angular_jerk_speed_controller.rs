use std::time::Instant;

use units::{
    angular_acceleration::radian_per_second_squared,
    angular_jerk::radian_per_second_cubed,
    angular_velocity::radian_per_second,
    f64::{AngularAcceleration, AngularJerk, AngularVelocity},
};

use super::acceleration_position_controller::MotionControllerError;
use super::jerk_speed_controller::JerkSpeedController;

/// Angular Jerk Speed Controller with proper physical units
///
/// This controller provides angular motion control with proper SI units for rotational systems.
/// It wraps the core JerkSpeedController and provides unit-typed interfaces for angular velocity
/// (AngularVelocity), angular acceleration (AngularAcceleration), and angular jerk (AngularJerk).
///
/// The controller manages smooth rotational speed profiles with configurable limits and tolerances,
/// ensuring that angular velocity, acceleration, and jerk constraints are respected during motion.
/// This is particularly useful for motor control, servo systems, and any rotational machinery
/// requiring smooth speed transitions.
///
/// # Example
/// ```ignore
/// use units::{
///     angular_velocity::revolution_per_minute,
///     angular_acceleration::revolution_per_minute_per_second,
///     angular_jerk::revolution_per_minute_per_second_squared,
///     f64::{AngularVelocity, AngularAcceleration, AngularJerk}
/// };
///
/// let max_speed = AngularVelocity::new::<revolution_per_minute>(3000.0);
/// let max_accel = AngularAcceleration::new::<revolution_per_minute_per_second>(1000.0);
/// let max_jerk = AngularJerk::new::<revolution_per_minute_per_second_squared>(500.0);
///
/// let controller = AngularJerkSpeedController::new_simple(
///     Some(max_speed),
///     max_accel,
///     max_jerk,
/// );
/// ```
#[derive(Debug)]
pub struct AngularJerkSpeedController {
    controller: JerkSpeedController,
    last_update: Option<Instant>,
}

impl AngularJerkSpeedController {
    /// Create a new angular speed controller with jerk limits
    ///
    /// This constructor allows full control over all angular velocity, acceleration, and jerk limits.
    /// Use this when you need asymmetric limits or fine-grained control over the rotational motion profile.
    ///
    /// # Parameters (ordered: speed, acceleration, jerk)
    /// * `min_speed` - Optional minimum allowed angular velocity (None for no limit)
    /// * `max_speed` - Optional maximum allowed angular velocity (None for no limit)
    /// * `min_acceleration` - Minimum angular acceleration (typically negative for deceleration)
    /// * `max_acceleration` - Maximum angular acceleration (typically positive)
    /// * `min_jerk` - Minimum angular jerk for decreasing acceleration (typically negative)
    /// * `max_jerk` - Maximum angular jerk for increasing acceleration (typically positive)
    ///
    /// # Returns
    /// A new AngularJerkSpeedController instance
    ///
    /// # Example
    /// ```ignore
    /// use units::{
    ///     angular_velocity::revolution_per_minute,
    ///     angular_acceleration::revolution_per_minute_per_second,
    ///     angular_jerk::revolution_per_minute_per_second_squared,
    ///     f64::{AngularVelocity, AngularAcceleration, AngularJerk}
    /// };
    ///
    /// // Create controller with asymmetric limits
    /// let controller = AngularJerkSpeedController::new(
    ///     Some(AngularVelocity::new::<revolution_per_minute>(-1000.0)), // Min: -1000 RPM
    ///     Some(AngularVelocity::new::<revolution_per_minute>(3000.0)),  // Max: +3000 RPM
    ///     AngularAcceleration::new::<revolution_per_minute_per_second>(-800.0), // Min accel
    ///     AngularAcceleration::new::<revolution_per_minute_per_second>(1200.0), // Max accel
    ///     AngularJerk::new::<revolution_per_minute_per_second_squared>(-400.0), // Min jerk
    ///     AngularJerk::new::<revolution_per_minute_per_second_squared>(600.0),  // Max jerk
    /// );
    /// ```
    pub fn new(
        min_speed: Option<AngularVelocity>,
        max_speed: Option<AngularVelocity>,
        min_acceleration: AngularAcceleration,
        max_acceleration: AngularAcceleration,
        min_jerk: AngularJerk,
        max_jerk: AngularJerk,
    ) -> Self {
        Self {
            controller: JerkSpeedController::new(
                min_speed.map(|speed| speed.get::<radian_per_second>()),
                max_speed.map(|speed| speed.get::<radian_per_second>()),
                min_acceleration.get::<radian_per_second_squared>(),
                max_acceleration.get::<radian_per_second_squared>(),
                min_jerk.get::<radian_per_second_cubed>(),
                max_jerk.get::<radian_per_second_cubed>(),
            ),
            last_update: None,
        }
    }

    /// Create a new simple angular jerk speed controller with symmetric limits
    ///
    /// This is a convenience constructor that creates symmetric limits around zero for all parameters.
    /// This is the most common use case and simplifies controller setup for typical rotational applications.
    ///
    /// The created limits are:
    /// - Angular velocity limits: [-speed, +speed] (if speed is Some), no limits if None
    /// - Angular acceleration limits: [-acceleration, +acceleration]
    /// - Angular jerk limits: [-jerk, +jerk]
    /// - Default tolerances: 1e-6 for both position and speed
    ///
    /// # Parameters (ordered: speed, acceleration, jerk)
    /// * `speed` - Optional maximum angular velocity magnitude (None for no limits, Some(x) creates [-x, +x])
    /// * `acceleration` - Maximum angular acceleration magnitude (creates limits [-acceleration, +acceleration])
    /// * `jerk` - Maximum angular jerk magnitude (creates limits [-jerk, +jerk])
    ///
    /// # Returns
    /// A new AngularJerkSpeedController instance
    ///
    /// # Panics
    /// Panics if acceleration or jerk values are negative or zero
    ///
    /// # Example
    /// ```ignore
    /// use units::{
    ///     angular_velocity::revolution_per_minute,
    ///     angular_acceleration::revolution_per_minute_per_second,
    ///     angular_jerk::revolution_per_minute_per_second_squared,
    ///     f64::{AngularVelocity, AngularAcceleration, AngularJerk}
    /// };
    ///
    /// // Create controller with symmetric limits
    /// let max_speed = AngularVelocity::new::<revolution_per_minute>(2000.0);
    /// let max_accel = AngularAcceleration::new::<revolution_per_minute_per_second>(1000.0);
    /// let max_jerk = AngularJerk::new::<revolution_per_minute_per_second_squared>(500.0);
    ///
    /// let controller = AngularJerkSpeedController::new_simple(
    ///     Some(max_speed),  // Angular velocity limits: [-2000, +2000] RPM
    ///     max_accel,        // Angular acceleration limits: [-1000, +1000] RPM/s
    ///     max_jerk,         // Angular jerk limits: [-500, +500] RPM/s²
    /// );
    ///
    /// // Create controller with no speed limits
    /// let unlimited_controller = AngularJerkSpeedController::new_simple(
    ///     None,       // No angular velocity limits
    ///     max_accel,  // Angular acceleration limits: [-1000, +1000] RPM/s
    ///     max_jerk,   // Angular jerk limits: [-500, +500] RPM/s²
    /// );
    /// ```
    pub fn new_simple(
        speed: Option<AngularVelocity>,
        acceleration: AngularAcceleration,
        jerk: AngularJerk,
    ) -> Self {
        let controller = JerkSpeedController::new_simple(
            speed.map(|s| s.get::<radian_per_second>()),
            acceleration.get::<radian_per_second_squared>(),
            jerk.get::<radian_per_second_cubed>(),
        );

        Self {
            controller,
            last_update: None,
        }
    }

    /// Update the controller with a new target angular velocity
    ///
    /// Calculates the next angular velocity in the motion profile based on the target and elapsed time.
    /// The controller automatically manages angular acceleration and jerk to reach the target angular
    /// velocity while respecting all configured limits.
    ///
    /// # Parameters
    /// * `target_speed` - The desired target angular velocity
    /// * `t` - Current timestamp for calculating time delta
    ///
    /// # Returns
    /// The current angular velocity after applying the motion profile for this time step
    ///
    /// # Example
    /// ```ignore
    /// use std::time::Instant;
    /// use units::{angular_velocity::revolution_per_minute, f64::AngularVelocity};
    ///
    /// let mut controller = AngularJerkSpeedController::new_simple(/* ... */);
    /// let target = AngularVelocity::new::<revolution_per_minute>(1500.0);
    ///
    /// let current_speed = controller.update(target, Instant::now());
    /// ```
    pub fn update(&mut self, target_speed: AngularVelocity, t: Instant) -> AngularVelocity {
        // Calculate dt from the last update
        let dt = if let Some(last_t) = self.last_update {
            t.duration_since(last_t).as_secs_f64()
        } else {
            0.0 // First update, no time has passed
        };
        self.last_update = Some(t);

        // Convert target to raw value, update controller, and convert result back to AngularVelocity
        let target_raw = target_speed.get::<radian_per_second>();
        let speed_raw = self.controller.update(target_raw, dt);
        AngularVelocity::new::<radian_per_second>(speed_raw)
    }

    /// Get the current angular velocity
    ///
    /// Returns the controller's current angular velocity in the motion profile.
    ///
    /// # Returns
    /// Current angular velocity as an AngularVelocity value
    pub fn get_speed(&self) -> AngularVelocity {
        AngularVelocity::new::<radian_per_second>(self.controller.get_speed())
    }

    /// Get the target angular velocity
    ///
    /// Returns the current target angular velocity that the controller is trying to reach.
    ///
    /// # Returns
    /// Target angular velocity as an AngularVelocity value
    pub fn get_target_speed(&self) -> AngularVelocity {
        AngularVelocity::new::<radian_per_second>(self.controller.get_target_speed())
    }

    /// Get the minimum angular velocity limit
    ///
    /// Returns the minimum angular velocity constraint, if one is set.
    ///
    /// # Returns
    /// Minimum angular velocity limit as an Option<AngularVelocity>, None if no limit is set
    pub fn get_min_speed(&self) -> Option<AngularVelocity> {
        self.controller
            .get_min_speed()
            .map(AngularVelocity::new::<radian_per_second>)
    }

    /// Get the maximum angular velocity limit
    ///
    /// Returns the maximum angular velocity constraint, if one is set.
    ///
    /// # Returns
    /// Maximum angular velocity limit as an Option<AngularVelocity>, None if no limit is set
    pub fn get_max_speed(&self) -> Option<AngularVelocity> {
        self.controller
            .get_max_speed()
            .map(AngularVelocity::new::<radian_per_second>)
    }

    /// Set the minimum angular velocity limit
    pub fn set_min_speed(
        &mut self,
        min_speed: Option<AngularVelocity>,
    ) -> Result<(), MotionControllerError> {
        self.controller
            .set_min_speed(min_speed.map(|speed| speed.get::<radian_per_second>()))
    }

    /// Set the maximum angular velocity limit
    pub fn set_max_speed(
        &mut self,
        max_speed: Option<AngularVelocity>,
    ) -> Result<(), MotionControllerError> {
        self.controller
            .set_max_speed(max_speed.map(|speed| speed.get::<radian_per_second>()))
    }

    /// Get the current angular acceleration
    pub fn get_acceleration(&self) -> AngularAcceleration {
        AngularAcceleration::new::<radian_per_second_squared>(self.controller.get_acceleration())
    }

    /// Set the minimum angular acceleration
    pub fn set_min_acceleration(
        &mut self,
        min_acceleration: AngularAcceleration,
    ) -> Result<(), MotionControllerError> {
        self.controller
            .set_min_acceleration(min_acceleration.get::<radian_per_second_squared>())
    }

    /// Set the maximum angular acceleration
    pub fn set_max_acceleration(
        &mut self,
        max_acceleration: AngularAcceleration,
    ) -> Result<(), MotionControllerError> {
        self.controller
            .set_max_acceleration(max_acceleration.get::<radian_per_second_squared>())
    }

    /// Get the current angular jerk
    pub fn get_jerk(&self) -> AngularJerk {
        AngularJerk::new::<radian_per_second_cubed>(self.controller.get_jerk())
    }

    /// Set the minimum angular jerk
    pub fn set_min_jerk(&mut self, min_jerk: AngularJerk) -> Result<(), MotionControllerError> {
        self.controller
            .set_min_jerk(min_jerk.get::<radian_per_second_cubed>())
    }

    /// Set the maximum angular jerk
    pub fn set_max_jerk(&mut self, max_jerk: AngularJerk) -> Result<(), MotionControllerError> {
        self.controller
            .set_max_jerk(max_jerk.get::<radian_per_second_cubed>())
    }

    /// Reset the controller to a new angular velocity and acceleration
    ///
    /// This resets all internal state including:
    /// - Current angular velocity to the provided value
    /// - Current angular acceleration to 0
    /// - Current angular jerk to 0
    /// - Target angular velocity to the current velocity
    ///
    /// # Parameters
    /// - `velocity`: The new current angular velocity
    pub fn reset(&mut self, velocity: AngularVelocity) -> Result<(), MotionControllerError> {
        self.controller.reset(velocity.get::<radian_per_second>())?;
        self.last_update = None;
        Ok(())
    }
}
