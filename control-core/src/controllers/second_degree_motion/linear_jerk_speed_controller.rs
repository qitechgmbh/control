use std::time::Instant;

use units::{
    acceleration::meter_per_second_squared,
    f64::{Acceleration, Jerk, Velocity},
    jerk::meter_per_second_cubed,
    velocity::meter_per_second,
};

use super::acceleration_position_controller::MotionControllerError;
use super::jerk_speed_controller::JerkSpeedController;

/// Linear Jerk Speed Controller with proper physical units
///
/// This controller provides linear motion control with proper SI units for translational systems.
/// It wraps the core JerkSpeedController and provides unit-typed interfaces for velocity (Velocity),
/// acceleration (Acceleration), and jerk (Jerk).
///
/// The controller manages smooth linear speed profiles with configurable limits and tolerances,
/// ensuring that velocity, acceleration, and jerk constraints are respected during motion.
/// This is particularly useful for linear actuators, conveyor systems, CNC machines, and any
/// translational machinery requiring smooth speed transitions.
///
/// # Example
/// ```ignore
/// use units::{
///     velocity::meter_per_second,
///     acceleration::meter_per_second_squared,
///     jerk::meter_per_second_cubed,
///     f64::{Velocity, Acceleration, Jerk}
/// };
///
/// let max_speed = Velocity::new::<meter_per_second>(5.0);
/// let max_accel = Acceleration::new::<meter_per_second_squared>(2.0);
/// let max_jerk = Jerk::new::<meter_per_second_cubed>(1.0);
///
/// let controller = LinearJerkSpeedController::new_simple(
///     Some(max_speed),
///     max_accel,
///     max_jerk,
/// );
/// ```
#[derive(Debug)]
pub struct LinearJerkSpeedController {
    controller: JerkSpeedController,
    last_update: Option<Instant>,
}

impl LinearJerkSpeedController {
    /// Create a new linear speed controller with jerk limits
    ///
    /// This constructor allows full control over all velocity, acceleration, and jerk limits.
    /// Use this when you need asymmetric limits or fine-grained control over the linear motion profile.
    ///
    /// # Parameters (ordered: speed, acceleration, jerk)
    /// * `min_speed` - Optional minimum allowed velocity (None for no limit)
    /// * `max_speed` - Optional maximum allowed velocity (None for no limit)
    /// * `min_acceleration` - Minimum acceleration (typically negative for deceleration)
    /// * `max_acceleration` - Maximum acceleration (typically positive)
    /// * `min_jerk` - Minimum jerk for decreasing acceleration (typically negative)
    /// * `max_jerk` - Maximum jerk for increasing acceleration (typically positive)
    ///
    /// # Returns
    /// A new LinearJerkSpeedController instance
    ///
    /// # Example
    /// ```ignore
    /// use units::{
    ///     velocity::meter_per_second,
    ///     acceleration::meter_per_second_squared,
    ///     jerk::meter_per_second_cubed,
    ///     f64::{Velocity, Acceleration, Jerk}
    /// };
    ///
    /// // Create controller with asymmetric limits
    /// let controller = LinearJerkSpeedController::new(
    ///     Some(Velocity::new::<meter_per_second>(-2.0)), // Min: -2 m/s
    ///     Some(Velocity::new::<meter_per_second>(5.0)),  // Max: +5 m/s
    ///     Acceleration::new::<meter_per_second_squared>(-3.0), // Min accel
    ///     Acceleration::new::<meter_per_second_squared>(4.0),  // Max accel
    ///     Jerk::new::<meter_per_second_cubed>(-2.0), // Min jerk
    ///     Jerk::new::<meter_per_second_cubed>(3.0),  // Max jerk
    /// );
    /// ```
    pub fn new(
        min_speed: Option<Velocity>,
        max_speed: Option<Velocity>,
        min_acceleration: Acceleration,
        max_acceleration: Acceleration,
        min_jerk: Jerk,
        max_jerk: Jerk,
    ) -> Self {
        Self {
            controller: JerkSpeedController::new(
                min_speed.map(|speed| speed.get::<meter_per_second>()),
                max_speed.map(|speed| speed.get::<meter_per_second>()),
                min_acceleration.get::<meter_per_second_squared>(),
                max_acceleration.get::<meter_per_second_squared>(),
                min_jerk.get::<meter_per_second_cubed>(),
                max_jerk.get::<meter_per_second_cubed>(),
            ),
            last_update: None,
        }
    }

    /// Create a new simple linear jerk speed controller with symmetric limits
    ///
    /// This is a convenience constructor that creates symmetric limits around zero for all parameters.
    /// This is the most common use case and simplifies controller setup for typical linear applications.
    ///
    /// The created limits are:
    /// - Velocity limits: [-speed, +speed] (if speed is Some), no limits if None
    /// - Acceleration limits: [-acceleration, +acceleration]
    /// - Jerk limits: [-jerk, +jerk]
    /// - Default tolerances: 1e-6 for both position and speed
    ///
    /// # Parameters (ordered: speed, acceleration, jerk)
    /// * `speed` - Optional maximum velocity magnitude (None for no limits, Some(x) creates [-x, +x])
    /// * `acceleration` - Maximum acceleration magnitude (creates limits [-acceleration, +acceleration])
    /// * `jerk` - Maximum jerk magnitude (creates limits [-jerk, +jerk])
    ///
    /// # Returns
    /// A new LinearJerkSpeedController instance
    ///
    /// # Panics
    /// Panics if acceleration or jerk values are negative or zero
    ///
    /// # Example
    /// ```ignore
    /// use units::{
    ///     velocity::meter_per_second,
    ///     acceleration::meter_per_second_squared,
    ///     jerk::meter_per_second_cubed,
    ///     f64::{Velocity, Acceleration, Jerk}
    /// };
    ///
    /// // Create controller with symmetric limits
    /// let max_speed = Velocity::new::<meter_per_second>(3.0);
    /// let max_accel = Acceleration::new::<meter_per_second_squared>(2.0);
    /// let max_jerk = Jerk::new::<meter_per_second_cubed>(1.5);
    ///
    /// let controller = LinearJerkSpeedController::new_simple(
    ///     Some(max_speed),  // Velocity limits: [-3, +3] m/s
    ///     max_accel,        // Acceleration limits: [-2, +2] m/s²
    ///     max_jerk,         // Jerk limits: [-1.5, +1.5] m/s³
    /// );
    ///
    /// // Create controller with no speed limits
    /// let unlimited_controller = LinearJerkSpeedController::new_simple(
    ///     None,       // No velocity limits
    ///     max_accel,  // Acceleration limits: [-2, +2] m/s²
    ///     max_jerk,   // Jerk limits: [-1.5, +1.5] m/s³
    /// );
    /// ```
    pub fn new_simple(speed: Option<Velocity>, acceleration: Acceleration, jerk: Jerk) -> Self {
        let controller = JerkSpeedController::new_simple(
            speed.map(|s| s.get::<meter_per_second>()),
            acceleration.get::<meter_per_second_squared>(),
            jerk.get::<meter_per_second_cubed>(),
        );

        Self {
            controller,
            last_update: None,
        }
    }

    /// Update the controller with a new target speed
    pub fn update(&mut self, target_speed: Velocity, t: Instant) -> Velocity {
        // Calculate dt from the last update
        let dt = if let Some(last_t) = self.last_update {
            t.duration_since(last_t).as_secs_f64()
        } else {
            0.0 // First update, no time has passed
        };
        self.last_update = Some(t);

        // Convert target to raw value, update controller, and convert result back to Velocity
        let target_raw = target_speed.get::<meter_per_second>();
        let speed_raw = self.controller.update(target_raw, dt);
        Velocity::new::<meter_per_second>(speed_raw)
    }

    /// Get the current speed
    pub fn get_speed(&self) -> Velocity {
        Velocity::new::<meter_per_second>(self.controller.get_speed())
    }

    /// Get the target speed
    pub fn get_target_speed(&self) -> Velocity {
        Velocity::new::<meter_per_second>(self.controller.get_target_speed())
    }

    /// Get the minimum velocity limit
    pub fn get_min_speed(&self) -> Option<Velocity> {
        self.controller
            .get_min_speed()
            .map(Velocity::new::<meter_per_second>)
    }

    /// Get the maximum velocity limit
    pub fn get_max_speed(&self) -> Option<Velocity> {
        self.controller
            .get_max_speed()
            .map(Velocity::new::<meter_per_second>)
    }

    /// Set the minimum velocity limit
    pub fn set_min_speed(
        &mut self,
        min_speed: Option<Velocity>,
    ) -> Result<(), MotionControllerError> {
        self.controller
            .set_min_speed(min_speed.map(|speed| speed.get::<meter_per_second>()))
    }

    /// Set the maximum velocity limit
    pub fn set_max_speed(
        &mut self,
        max_speed: Option<Velocity>,
    ) -> Result<(), MotionControllerError> {
        self.controller
            .set_max_speed(max_speed.map(|speed| speed.get::<meter_per_second>()))
    }

    /// Get the current acceleration
    pub fn get_acceleration(&self) -> Acceleration {
        Acceleration::new::<meter_per_second_squared>(self.controller.get_acceleration())
    }

    /// Set the minimum acceleration
    pub fn set_min_acceleration(
        &mut self,
        min_acceleration: Acceleration,
    ) -> Result<(), MotionControllerError> {
        self.controller
            .set_min_acceleration(min_acceleration.get::<meter_per_second_squared>())
    }

    /// Set the maximum acceleration
    pub fn set_max_acceleration(
        &mut self,
        max_acceleration: Acceleration,
    ) -> Result<(), MotionControllerError> {
        self.controller
            .set_max_acceleration(max_acceleration.get::<meter_per_second_squared>())
    }

    /// Get the current jerk
    pub fn get_jerk(&self) -> Jerk {
        Jerk::new::<meter_per_second_cubed>(self.controller.get_jerk())
    }

    /// Set the minimum jerk
    pub fn set_min_jerk(&mut self, min_jerk: Jerk) -> Result<(), MotionControllerError> {
        self.controller
            .set_min_jerk(min_jerk.get::<meter_per_second_cubed>())
    }

    /// Set the maximum jerk
    pub fn set_max_jerk(&mut self, max_jerk: Jerk) -> Result<(), MotionControllerError> {
        self.controller
            .set_max_jerk(max_jerk.get::<meter_per_second_cubed>())
    }

    /// Reset the controller to a new velocity and acceleration
    ///
    /// This resets all internal state including:
    /// - Current velocity to the provided value
    /// - Current acceleration to 0
    /// - Current jerk to 0
    /// - Target velocity to the current velocity
    ///
    /// # Parameters
    /// - `velocity`: The new current velocity
    pub fn reset(&mut self, velocity: Velocity) -> Result<(), MotionControllerError> {
        self.controller.reset(velocity.get::<meter_per_second>())?;
        self.last_update = None;
        Ok(())
    }
}
