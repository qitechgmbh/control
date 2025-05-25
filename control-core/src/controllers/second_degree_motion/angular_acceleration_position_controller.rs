use std::time::Instant;

use uom::si::{
    angle::radian,
    angular_acceleration::radian_per_second_squared,
    angular_velocity::radian_per_second,
    f64::{Angle, AngularAcceleration, AngularVelocity},
};

use super::acceleration_position_controller::AccelerationPositionController;

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
                min_position.map(|angle| angle.get::<radian>()),
                max_position.map(|angle| angle.get::<radian>()),
                min_speed.get::<radian_per_second>(),
                max_speed.get::<radian_per_second>(),
                min_acceleration.get::<radian_per_second_squared>(),
                max_acceleration.get::<radian_per_second_squared>(),
            ),
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
        let result = self.controller.update(target_angle.get::<radian>(), dt);
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
        self.controller
            .get_min_position()
            .map(|angle| Angle::new::<radian>(angle))
    }

    /// Get the maximum angle position limit
    pub fn get_max_position(&self) -> Option<Angle> {
        self.controller
            .get_max_position()
            .map(|angle| Angle::new::<radian>(angle))
    }

    /// Set the minimum angle position limit
    pub fn set_min_position(&mut self, min_position: Option<Angle>) {
        self.controller
            .set_min_position(min_position.map(|angle| angle.get::<radian>()));
    }

    /// Set the maximum angle position limit
    pub fn set_max_position(&mut self, max_position: Option<Angle>) {
        self.controller
            .set_max_position(max_position.map(|angle| angle.get::<radian>()));
    }

    /// Get the current angular velocity
    pub fn get_speed(&self) -> AngularVelocity {
        AngularVelocity::new::<radian_per_second>(self.controller.get_speed())
    }

    /// Set the minimum angular velocity
    pub fn set_min_speed(&mut self, min_speed: AngularVelocity) {
        self.controller
            .set_min_speed(min_speed.get::<radian_per_second>());
    }

    /// Set the maximum angular velocity
    pub fn set_max_speed(&mut self, max_speed: AngularVelocity) {
        self.controller
            .set_max_speed(max_speed.get::<radian_per_second>());
    }

    /// Get the current angular acceleration
    pub fn get_acceleration(&self) -> AngularAcceleration {
        AngularAcceleration::new::<radian_per_second_squared>(self.controller.get_acceleration())
    }

    /// Set the minimum angular acceleration
    pub fn set_min_acceleration(&mut self, min_acceleration: AngularAcceleration) {
        self.controller
            .set_min_acceleration(min_acceleration.get::<radian_per_second_squared>());
    }

    /// Set the maximum angular acceleration
    pub fn set_max_acceleration(&mut self, max_acceleration: AngularAcceleration) {
        self.controller
            .set_max_acceleration(max_acceleration.get::<radian_per_second_squared>());
    }

    /// Reset the controller to a new angular position and velocity
    ///
    /// This resets all internal state including:
    /// - Current angular position to the provided value
    /// - Current angular velocity to the provided value (optional, defaults to 0)
    /// - Current angular acceleration to 0
    /// - Target angular position to the current position
    ///
    /// # Parameters
    /// - `position`: The new current angular position
    /// - `velocity`: The new current angular velocity (optional, defaults to 0)
    pub fn reset(&mut self, position: Angle, velocity: Option<AngularVelocity>) {
        let velocity_value = velocity.map(|v| v.get::<radian_per_second>());
        self.controller
            .reset(position.get::<radian>(), velocity_value);
    }
}
