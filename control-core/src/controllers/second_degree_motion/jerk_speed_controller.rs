use super::acceleration_position_controller::AccelerationPositionController;

/// Controller for speed-based motion with jerk limits.
///
/// This controller wraps an AccelerationPositionController but changes the parameter names
/// to represent one higher degree of derivation:
/// - Speed (the primary controlled variable)
/// - Acceleration (the first derivative of speed)
/// - Jerk (the second derivative of speed, rate of change of acceleration)
#[derive(Debug)]
pub struct JerkSpeedController {
    base_controller: AccelerationPositionController,
}

impl JerkSpeedController {
    /// Create a new speed controller with the given constraints
    ///
    /// # Parameters (ordered: speed, acceleration, jerk)
    /// - `min_speed`: Optional minimum allowed speed (None for no limit)
    /// - `max_speed`: Optional maximum allowed speed (None for no limit)
    /// - `min_acceleration`: Minimum acceleration (negative value)
    /// - `max_acceleration`: Maximum acceleration (positive value)
    /// - `min_jerk`: Minimum jerk for decreasing acceleration (negative value)
    /// - `max_jerk`: Maximum jerk for increasing acceleration (positive value)
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
            min_speed,        // min_position in the base controller
            max_speed,        // max_position in the base controller
            min_acceleration, // min_speed in the base controller
            max_acceleration, // max_speed in the base controller
            min_jerk,         // min_acceleration in the base controller
            max_jerk,         // max_acceleration in the base controller
        );

        JerkSpeedController { base_controller }
    }

    /// Update the controller state based on the target speed and time step.
    ///
    /// Parameters:
    /// - target_speed: The target speed to reach
    /// - dt: Time step in seconds
    ///
    /// Returns:
    /// - current_speed: Updated speed
    pub fn update(&mut self, target_speed: f64, dt: f64) -> f64 {
        // Call the base controller's update method with the target speed
        self.base_controller.update(target_speed, dt)
    }

    /// Get the current speed
    pub fn get_speed(&self) -> f64 {
        self.base_controller.get_position()
    }

    /// Get the current acceleration
    pub fn get_acceleration(&self) -> f64 {
        self.base_controller.get_speed()
    }

    /// Get the current jerk
    pub fn get_jerk(&self) -> f64 {
        self.base_controller.get_acceleration()
    }

    /// Get the target speed
    pub fn get_target_speed(&self) -> f64 {
        self.base_controller.get_target_position()
    }

    /// Get the minimum speed limit
    pub fn get_min_speed(&self) -> Option<f64> {
        self.base_controller.get_min_position()
    }

    /// Get the maximum speed limit
    pub fn get_max_speed(&self) -> Option<f64> {
        self.base_controller.get_max_position()
    }

    /// Set the minimum speed limit
    pub fn set_min_speed(&mut self, min_speed: Option<f64>) {
        self.base_controller.set_min_position(min_speed);
    }

    /// Set the maximum speed limit
    pub fn set_max_speed(&mut self, max_speed: Option<f64>) {
        self.base_controller.set_max_position(max_speed);
    }

    /// Set the min acceleration
    pub fn set_min_acceleration(&mut self, min_acceleration: f64) {
        self.base_controller.set_min_speed(min_acceleration);
    }

    /// Set the max acceleration
    pub fn set_max_acceleration(&mut self, max_acceleration: f64) {
        self.base_controller.set_max_speed(max_acceleration);
    }

    /// Set the min jerk
    pub fn set_min_jerk(&mut self, min_jerk: f64) {
        self.base_controller.set_min_acceleration(min_jerk);
    }

    /// Set the max jerk
    pub fn set_max_jerk(&mut self, max_jerk: f64) {
        self.base_controller.set_max_acceleration(max_jerk);
    }

    /// Reset the controller to a new speed and acceleration
    ///
    /// This resets all internal state including:
    /// - Current speed to the provided value
    /// - Current acceleration to the provided value (optional, defaults to 0.0)
    /// - Current jerk to 0
    /// - Target speed to the current speed
    ///
    /// # Parameters
    /// - `speed`: The new current speed
    /// - `acceleration`: The new current acceleration (optional, defaults to 0.0)
    pub fn reset(&mut self, speed: f64, acceleration: Option<f64>) {
        let acceleration = acceleration.unwrap_or(0.0);
        // Reset using the base controller with mapped parameters
        self.base_controller.reset(speed, Some(acceleration));
    }
}
