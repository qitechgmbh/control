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
    /// Parameters:
    /// - max_acceleration: Maximum acceleration (positive value)
    /// - min_acceleration: Minimum acceleration (negative value)
    /// - max_jerk: Maximum jerk for increasing acceleration (positive value)
    /// - min_jerk: Minimum jerk for decreasing acceleration (negative value)
    pub fn new(max_acceleration: f64, min_acceleration: f64, max_jerk: f64, min_jerk: f64) -> Self {
        // Create the base controller with renamed parameters
        let base_controller = AccelerationPositionController::new(
            max_acceleration, // max_speed in the base controller
            min_acceleration, // min_speed in the base controller
            max_jerk,         // max_acceleration in the base controller
            min_jerk,         // min_acceleration in the base controller
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

    /// Set the max acceleration
    pub fn set_max_acceleration(&mut self, max_acceleration: f64) {
        self.base_controller.set_max_speed(max_acceleration);
    }

    /// Set the min acceleration
    pub fn set_min_acceleration(&mut self, min_acceleration: f64) {
        self.base_controller.set_min_speed(min_acceleration);
    }

    /// Set the max jerk
    pub fn set_max_jerk(&mut self, max_jerk: f64) {
        self.base_controller.set_max_acceleration(max_jerk);
    }

    /// Set the min jerk
    pub fn set_min_jerk(&mut self, min_jerk: f64) {
        self.base_controller.set_min_acceleration(min_jerk);
    }
}
