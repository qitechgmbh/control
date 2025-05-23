use std::time::Instant;

use uom::si::{
    angular_acceleration::radian_per_second_squared,
    angular_jerk::radian_per_second_cubed,
    angular_velocity::radian_per_second,
    f64::{AngularAcceleration, AngularJerk, AngularVelocity},
};

use super::jerk_speed_controller::JerkSpeedController;

/// Angular Jerk Speed Controller with proper physical units
#[derive(Debug)]
pub struct AngularJerkSpeedController {
    controller: JerkSpeedController,
    last_update: Option<Instant>,
}

impl AngularJerkSpeedController {
    /// Create a new angular speed controller with jerk limits
    pub fn new(
        max_acceleration: AngularAcceleration,
        min_acceleration: AngularAcceleration,
        max_jerk: AngularJerk,
        min_jerk: AngularJerk,
    ) -> Self {
        Self {
            controller: JerkSpeedController::new(
                max_acceleration.get::<radian_per_second_squared>(),
                min_acceleration.get::<radian_per_second_squared>(),
                max_jerk.get::<radian_per_second_cubed>(),
                min_jerk.get::<radian_per_second_cubed>(),
            ),
            last_update: None,
        }
    }

    /// Update the controller with a new target angular velocity
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
    pub fn get_speed(&self) -> AngularVelocity {
        AngularVelocity::new::<radian_per_second>(self.controller.get_speed())
    }

    /// Get the current angular acceleration
    pub fn get_acceleration(&self) -> AngularAcceleration {
        AngularAcceleration::new::<radian_per_second_squared>(self.controller.get_acceleration())
    }

    /// Get the current angular jerk
    pub fn get_jerk(&self) -> AngularJerk {
        AngularJerk::new::<radian_per_second_cubed>(self.controller.get_jerk())
    }

    /// Get the target angular velocity
    pub fn get_target_speed(&self) -> AngularVelocity {
        AngularVelocity::new::<radian_per_second>(self.controller.get_target_speed())
    }

    /// Set the maximum angular acceleration
    pub fn set_max_acceleration(&mut self, max_acceleration: AngularAcceleration) {
        self.controller
            .set_max_acceleration(max_acceleration.get::<radian_per_second_squared>());
    }

    /// Set the minimum angular acceleration
    pub fn set_min_acceleration(&mut self, min_acceleration: AngularAcceleration) {
        self.controller
            .set_min_acceleration(min_acceleration.get::<radian_per_second_squared>());
    }

    /// Set the maximum angular jerk
    pub fn set_max_jerk(&mut self, max_jerk: AngularJerk) {
        self.controller
            .set_max_jerk(max_jerk.get::<radian_per_second_cubed>());
    }

    /// Set the minimum angular jerk
    pub fn set_min_jerk(&mut self, min_jerk: AngularJerk) {
        self.controller
            .set_min_jerk(min_jerk.get::<radian_per_second_cubed>());
    }
}
