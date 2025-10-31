use std::time::Instant;

use units::{
    angle::radian,
    angular_acceleration::radian_per_second_squared,
    angular_velocity::radian_per_second,
    f64::{Angle, AngularAcceleration, AngularVelocity},
};

use super::acceleration_position_controller::{
    AccelerationPositionController, MotionControllerError,
};

/// Angular Acceleration Position Controller with proper physical units
#[derive(Debug)]
pub struct AngularAccelerationPositionController {
    controller: AccelerationPositionController,
    last_update: Option<Instant>,
}

impl AngularAccelerationPositionController {
    /// Create a new angular position controller with acceleration limits
    pub fn new(
        min_position: Option<Angle>,
        max_position: Option<Angle>,
        min_speed: AngularVelocity,
        max_speed: AngularVelocity,
        min_acceleration: AngularAcceleration,
        max_acceleration: AngularAcceleration,
    ) -> Self {
        Self {
            controller: AccelerationPositionController::new(
                min_speed.get::<radian_per_second>(),                // min_speed
                max_speed.get::<radian_per_second>(),                // max_speed
                min_acceleration.get::<radian_per_second_squared>(), // min_acceleration
                max_acceleration.get::<radian_per_second_squared>(), // max_acceleration
                min_position.map(|angle| angle.get::<radian>()),     // min_position
                max_position.map(|angle| angle.get::<radian>()),     // max_position
                1e-6,                                                // position_tolerance
                1e-6,                                                // speed_tolerance
            )
            .expect("Failed to create AccelerationPositionController"),
            last_update: None,
        }
    }

    /// Create a new angular position controller with simple symmetric limits
    ///
    /// This is a convenience constructor that creates symmetric limits:
    /// - Position limits: [-position, +position] (if position is Some)
    /// - Speed limits: [-speed, +speed]
    /// - Acceleration limits: [-acceleration, +acceleration]
    /// - Default tolerances: 1e-6 for both position and speed
    ///
    /// # Parameters (ordered: position, speed, acceleration)
    /// - `position`: Optional maximum position magnitude (None for no limits, Some(x) creates [-x, +x])
    /// - `speed`: Maximum angular velocity magnitude (creates limits [-speed, +speed])
    /// - `acceleration`: Maximum angular acceleration magnitude (creates limits [-acceleration, +acceleration])
    pub fn new_simple(
        position: Option<Angle>,
        speed: AngularVelocity,
        acceleration: AngularAcceleration,
    ) -> Self {
        Self {
            controller: AccelerationPositionController::new_simple(
                position.map(|angle| angle.get::<radian>()),
                speed.get::<radian_per_second>(),
                acceleration.get::<radian_per_second_squared>(),
            )
            .expect("Failed to create AccelerationPositionController"),
            last_update: None,
        }
    }

    /// Update the controller with a new target angle
    pub fn update(&mut self, target_angle: Angle, t: Instant) -> Angle {
        // Calculate dt from the last update
        let dt = if let Some(last_t) = self.last_update {
            t.duration_since(last_t).as_secs_f64()
        } else {
            0.0 // First update, no time has passed
        };
        self.last_update = Some(t);

        // Update controller with raw angle values
        let result = self
            .controller
            .update(target_angle.get::<radian>(), dt)
            .expect("Failed to update AccelerationPositionController");
        Angle::new::<radian>(result)
    }

    /// Get the current angle position
    pub fn get_position(&self) -> Angle {
        Angle::new::<radian>(self.controller.get_position())
    }

    /// Get the target angle
    pub fn get_target_position(&self) -> Angle {
        Angle::new::<radian>(self.controller.get_target_position())
    }

    /// Get the minimum angle position limit
    pub fn get_min_position(&self) -> Option<Angle> {
        self.controller.get_min_position().map(Angle::new::<radian>)
    }

    /// Get the maximum angle position limit
    pub fn get_max_position(&self) -> Option<Angle> {
        self.controller.get_max_position().map(Angle::new::<radian>)
    }

    /// Set the minimum angle position limit
    pub fn set_min_position(
        &mut self,
        min_position: Option<Angle>,
    ) -> Result<(), MotionControllerError> {
        self.controller
            .set_min_position(min_position.map(|angle| angle.get::<radian>()))
    }

    /// Set the maximum angle position limit
    pub fn set_max_position(
        &mut self,
        max_position: Option<Angle>,
    ) -> Result<(), MotionControllerError> {
        self.controller
            .set_max_position(max_position.map(|angle| angle.get::<radian>()))
    }

    /// Get the current angular velocity
    pub fn get_speed(&self) -> AngularVelocity {
        AngularVelocity::new::<radian_per_second>(self.controller.get_speed())
    }

    /// Set the minimum angular velocity
    pub fn set_min_speed(
        &mut self,
        min_speed: AngularVelocity,
    ) -> Result<(), MotionControllerError> {
        self.controller
            .set_min_speed(min_speed.get::<radian_per_second>())
    }

    /// Set the maximum angular velocity
    pub fn set_max_speed(
        &mut self,
        max_speed: AngularVelocity,
    ) -> Result<(), MotionControllerError> {
        self.controller
            .set_max_speed(max_speed.get::<radian_per_second>())
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

    /// Resets the controller to a specific angular position
    ///
    /// This method completely reinitializes the controller state, setting:
    /// - Current angular position to the specified value
    /// - Target angular position to the specified value (stops any ongoing motion)
    /// - Current angular velocity to 0
    /// - Current angular acceleration to 0
    /// - Motion phase to Idle
    /// - Clears any motion planning state and timing information
    ///
    /// Use this method when you need to teleport the controlled object to a new angular position
    /// or when recovering from errors/emergency stops.
    ///
    /// # Arguments
    /// * `position` - The new angular position to reset to
    ///
    /// # Returns
    /// Returns `Ok(())` on success, or `Err(MotionControllerError)` if the position is outside configured limits.
    ///
    /// # Errors
    /// - `InvalidPositionLimits`: If the specified position is outside the configured min/max position limits
    ///
    /// # Example
    /// ```ignore
    /// use control_core::controllers::second_degree_motion::angular_acceleration_position_controller::AngularAccelerationPositionController;
    /// use units::f64::Angle;
    /// use units::angle::degree;
    ///
    /// let mut controller = AngularAccelerationPositionController::new_simple(
    ///     Some(Angle::new::<degree>(180.0)),
    ///     // ... other parameters
    /// )?;
    ///
    /// // Reset to 45 degrees
    /// let new_position = Angle::new::<degree>(45.0);
    /// controller.reset(new_position)?;
    /// ```
    pub fn reset(&mut self, position: Angle) -> Result<(), MotionControllerError> {
        self.controller.reset(position.get::<radian>())?;
        self.last_update = None;
        Ok(())
    }
}
