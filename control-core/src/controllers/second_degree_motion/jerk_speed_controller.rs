use super::acceleration_position_controller::{
    AccelerationPositionController, MotionControllerError,
};

/// Controller for speed-based motion with jerk limits.
///
/// This controller manages motion profiles where speed is the primary controlled variable,
/// with constraints on acceleration (rate of change of speed) and jerk (rate of change of acceleration).
/// It wraps an AccelerationPositionController but remaps the parameters to represent one higher
/// degree of derivation in the motion hierarchy.
///
/// The control hierarchy is:
/// - **Speed**: The primary controlled variable (analogous to position in position control)
/// - **Acceleration**: First derivative of speed (analogous to velocity in position control)  
/// - **Jerk**: Second derivative of speed, rate of change of acceleration (analogous to acceleration in position control)
///
/// This controller is particularly useful for applications where you need to control rotational
/// or linear speed with smooth acceleration profiles, such as motor control, conveyor systems,
/// or any application requiring smooth speed transitions.
///
/// # Example
/// ```ignore
/// // Create a controller for speed control with jerk limiting
/// let controller = JerkSpeedController::new_simple(
///     Some(100.0),  // Max speed ±100 units/s
///     50.0,         // Max acceleration ±50 units/s²
///     25.0,         // Max jerk ±25 units/s³
/// );
///
/// // Update towards target speed
/// let current_speed = controller.update(80.0, 0.01);  // Target 80 units/s, dt=10ms
/// ```
#[derive(Debug)]
pub struct JerkSpeedController {
    base_controller: AccelerationPositionController,
}

impl JerkSpeedController {
    /// Create a new speed controller with the given constraints
    ///
    /// This constructor allows full control over all speed, acceleration, and jerk limits.
    /// Use this when you need asymmetric limits or fine-grained control over the motion profile.
    ///
    /// # Parameters (ordered: speed, acceleration, jerk)
    /// * `min_speed` - Optional minimum allowed speed (None for no limit)
    /// * `max_speed` - Optional maximum allowed speed (None for no limit)
    /// * `min_acceleration` - Minimum acceleration (typically negative for deceleration)
    /// * `max_acceleration` - Maximum acceleration (typically positive)
    /// * `min_jerk` - Minimum jerk for decreasing acceleration (typically negative)
    /// * `max_jerk` - Maximum jerk for increasing acceleration (typically positive)
    ///
    /// # Returns
    /// A new JerkSpeedController instance
    ///
    /// # Panics
    /// Panics if the underlying AccelerationPositionController cannot be created with the given parameters
    ///
    /// # Example
    /// ```ignore
    /// // Create controller with asymmetric limits
    /// let controller = JerkSpeedController::new(
    ///     Some(-50.0),  // Min speed: -50 units/s
    ///     Some(100.0),  // Max speed: +100 units/s
    ///     -30.0,        // Min acceleration: -30 units/s²
    ///     40.0,         // Max acceleration: +40 units/s²
    ///     -20.0,        // Min jerk: -20 units/s³
    ///     25.0,         // Max jerk: +25 units/s³
    /// );
    /// ```
    pub fn new(
        min_speed: Option<f64>,
        max_speed: Option<f64>,
        min_acceleration: f64,
        max_acceleration: f64,
        min_jerk: f64,
        max_jerk: f64,
    ) -> Self {
        // Create the base controller with renamed parameters
        let base_controller = AccelerationPositionController::new(
            min_jerk,         // min_speed in the base controller
            max_jerk,         // max_speed in the base controller
            min_acceleration, // min_acceleration in the base controller
            max_acceleration, // max_acceleration in the base controller
            min_speed,        // min_position in the base controller
            max_speed,        // max_position in the base controller
            1e-6,             // position_tolerance
            1e-6,             // speed_tolerance
        )
        .expect("Failed to create AccelerationPositionController");

        Self { base_controller }
    }

    /// Create a new speed controller with simple symmetric limits
    ///
    /// This is a convenience constructor that creates symmetric limits around zero for all parameters.
    /// This is the most common use case and simplifies controller setup for typical applications.
    ///
    /// The created limits are:
    /// - Speed limits: [-speed, +speed] (if speed is Some), no limits if None
    /// - Acceleration limits: [-acceleration, +acceleration]  
    /// - Jerk limits: [-jerk, +jerk]
    /// - Default tolerances: 1e-6 for both position and speed
    ///
    /// # Parameters (ordered: speed, acceleration, jerk)
    /// * `speed` - Optional maximum speed magnitude (None for no limits, Some(x) creates [-x, +x])
    /// * `acceleration` - Maximum acceleration magnitude (creates limits [-acceleration, +acceleration])
    /// * `jerk` - Maximum jerk magnitude (creates limits [-jerk, +jerk])
    ///
    /// # Returns
    /// A new JerkSpeedController instance
    ///
    /// # Panics
    /// Panics if acceleration or jerk values are negative or zero
    ///
    /// # Example
    /// ```ignore
    /// // Create controller with symmetric limits
    /// let controller = JerkSpeedController::new_simple(
    ///     Some(100.0),  // Speed limits: [-100, +100] units/s
    ///     50.0,         // Acceleration limits: [-50, +50] units/s²
    ///     25.0,         // Jerk limits: [-25, +25] units/s³
    /// );
    ///
    /// // Create controller with no speed limits
    /// let unlimited_controller = JerkSpeedController::new_simple(
    ///     None,    // No speed limits
    ///     30.0,    // Acceleration limits: [-30, +30] units/s²
    ///     15.0,    // Jerk limits: [-15, +15] units/s³
    /// );
    /// ```
    pub fn new_simple(speed: Option<f64>, acceleration: f64, jerk: f64) -> Self {
        if acceleration <= 0.0 || jerk <= 0.0 {
            panic!("Acceleration and jerk must be positive values");
        }

        Self::new(
            speed.map(|s| -s), // min_speed
            speed,             // max_speed
            -acceleration,     // min_acceleration
            acceleration,      // max_acceleration
            -jerk,             // min_jerk
            jerk,              // max_jerk
        )
    }

    /// Update the controller state based on the target speed and time step
    ///
    /// Calculates the next speed in the motion profile based on the target speed and elapsed time.
    /// The controller automatically manages acceleration and jerk to reach the target speed
    /// while respecting all configured limits.
    ///
    /// # Parameters
    /// * `target_speed` - The desired target speed to reach
    /// * `dt` - Time step in seconds since the last update
    ///
    /// # Returns
    /// The current speed after applying the motion profile for this time step
    ///
    /// # Panics
    /// Panics if the underlying controller update fails (should not happen in normal operation)
    ///
    /// # Example
    /// ```ignore
    /// let mut controller = JerkSpeedController::new_simple(Some(100.0), 50.0, 25.0);
    ///
    /// // Control loop
    /// loop {
    ///     let current_speed = controller.update(target_speed, 0.01);  // 10ms update cycle
    ///     // Use current_speed for your application
    /// }
    /// ```
    pub fn update(&mut self, target_speed: f64, dt: f64) -> f64 {
        // Call the base controller's update method with the target speed
        self.base_controller
            .update(target_speed, dt)
            .expect("Failed to update controller")
    }

    /// Get the current speed
    ///
    /// Returns the controller's current speed in the motion profile.
    ///
    /// # Returns
    /// Current speed as a floating-point value
    pub const fn get_speed(&self) -> f64 {
        self.base_controller.get_position()
    }

    /// Get the current acceleration
    ///
    /// Returns the controller's current acceleration (rate of change of speed).
    ///
    /// # Returns
    /// Current acceleration as a floating-point value
    pub const fn get_acceleration(&self) -> f64 {
        self.base_controller.get_speed()
    }

    /// Get the current jerk
    ///
    /// Returns the controller's current jerk (rate of change of acceleration).
    ///
    /// # Returns
    /// Current jerk as a floating-point value
    pub const fn get_jerk(&self) -> f64 {
        self.base_controller.get_acceleration()
    }

    /// Get the target speed
    ///
    /// Returns the current target speed that the controller is trying to reach.
    ///
    /// # Returns
    /// Target speed as a floating-point value
    pub const fn get_target_speed(&self) -> f64 {
        self.base_controller.get_target_position()
    }

    /// Get the minimum speed limit
    ///
    /// Returns the minimum speed constraint, if one is set.
    ///
    /// # Returns
    /// Minimum speed limit as an Option<f64>, None if no limit is set
    pub const fn get_min_speed(&self) -> Option<f64> {
        self.base_controller.get_min_position()
    }

    /// Get the maximum speed limit
    ///
    /// Returns the maximum speed constraint, if one is set.
    ///
    /// # Returns
    /// Maximum speed limit as an Option<f64>, None if no limit is set
    pub const fn get_max_speed(&self) -> Option<f64> {
        self.base_controller.get_max_position()
    }

    /// Set the minimum speed limit
    ///
    /// Updates the minimum speed constraint for the controller.
    ///
    /// # Parameters
    /// * `min_speed` - New minimum speed limit (None to remove limit)
    ///
    /// # Returns
    /// Result indicating success or failure of the operation
    ///
    /// # Errors
    /// Returns MotionControllerError if the new limit would be invalid
    /// (e.g., greater than the maximum speed limit)
    pub fn set_min_speed(&mut self, min_speed: Option<f64>) -> Result<(), MotionControllerError> {
        self.base_controller.set_min_position(min_speed)
    }

    /// Set the maximum speed limit
    ///
    /// Updates the maximum speed constraint for the controller.
    ///
    /// # Parameters
    /// * `max_speed` - New maximum speed limit (None to remove limit)
    ///
    /// # Returns
    /// Result indicating success or failure of the operation
    ///
    /// # Errors
    /// Returns MotionControllerError if the new limit would be invalid
    /// (e.g., less than the minimum speed limit)
    pub fn set_max_speed(&mut self, max_speed: Option<f64>) -> Result<(), MotionControllerError> {
        self.base_controller.set_max_position(max_speed)
    }

    /// Set the minimum acceleration
    ///
    /// Updates the minimum acceleration constraint for the controller.
    ///
    /// # Parameters
    /// * `min_acceleration` - New minimum acceleration limit (typically negative for deceleration)
    ///
    /// # Returns
    /// Result indicating success or failure of the operation
    ///
    /// # Errors
    /// Returns MotionControllerError if the new limit would be invalid
    /// (e.g., greater than the maximum acceleration limit)
    pub fn set_min_acceleration(
        &mut self,
        min_acceleration: f64,
    ) -> Result<(), MotionControllerError> {
        self.base_controller.set_min_speed(min_acceleration)
    }

    /// Set the maximum acceleration
    ///
    /// Updates the maximum acceleration constraint for the controller.
    ///
    /// # Parameters
    /// * `max_acceleration` - New maximum acceleration limit (typically positive)
    ///
    /// # Returns
    /// Result indicating success or failure of the operation
    ///
    /// # Errors
    /// Returns MotionControllerError if the new limit would be invalid
    /// (e.g., less than the minimum acceleration limit or negative/zero value)
    pub fn set_max_acceleration(
        &mut self,
        max_acceleration: f64,
    ) -> Result<(), MotionControllerError> {
        self.base_controller.set_max_speed(max_acceleration)
    }

    /// Set the minimum jerk
    ///
    /// Updates the minimum jerk constraint for the controller.
    ///
    /// # Parameters
    /// * `min_jerk` - New minimum jerk limit (typically negative for decreasing acceleration)
    ///
    /// # Returns
    /// Result indicating success or failure of the operation
    ///
    /// # Errors
    /// Returns MotionControllerError if the new limit would be invalid
    /// (e.g., greater than the maximum jerk limit)
    pub fn set_min_jerk(&mut self, min_jerk: f64) -> Result<(), MotionControllerError> {
        self.base_controller.set_min_acceleration(min_jerk)
    }

    /// Set the maximum jerk
    ///
    /// Updates the maximum jerk constraint for the controller.
    ///
    /// # Parameters
    /// * `max_jerk` - New maximum jerk limit (typically positive for increasing acceleration)
    ///
    /// # Returns
    /// Result indicating success or failure of the operation
    ///
    /// # Errors
    /// Returns MotionControllerError if the new limit would be invalid
    /// (e.g., less than the minimum jerk limit or negative/zero value)
    pub fn set_max_jerk(&mut self, max_jerk: f64) -> Result<(), MotionControllerError> {
        self.base_controller.set_max_acceleration(max_jerk)
    }

    /// Reset the controller to a new speed and acceleration
    ///
    /// This resets all internal state of the controller to start fresh from a new state.
    /// The controller will reset its speed, clear any accumulated motion profile state,
    /// and set the target speed to the current speed.
    ///
    /// # Parameters
    /// * `speed` - The new current speed to reset to
    ///
    /// # Returns
    /// Result indicating success or failure of the reset operation
    ///
    /// # Errors
    /// Returns MotionControllerError if the speed is outside the configured limits
    ///
    /// # Example
    /// ```ignore
    /// let mut controller = JerkSpeedController::new_simple(Some(100.0), 50.0, 25.0);
    ///
    /// // Reset to zero speed with zero acceleration
    /// controller.reset(0.0)?;
    ///
    /// // Reset to specific speed
    /// controller.reset(25.0)?;
    /// ```
    pub fn reset(&mut self, speed: f64) -> Result<(), MotionControllerError> {
        // Reset to the speed position (treating speed as position in the base controller)
        self.base_controller.reset(speed)?;
        Ok(())
    }
}
