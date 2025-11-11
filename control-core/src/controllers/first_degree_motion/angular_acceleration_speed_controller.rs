use std::time::Instant;

use units::{
    angular_acceleration::radian_per_second_squared,
    angular_velocity::radian_per_second,
    f64::{AngularAcceleration, AngularVelocity},
};

use super::acceleration_speed_controller::AccelerationSpeedController;

/// [`LinearAngularAccelerationController`] wraps [`LinearAccelerationController`]
/// to handle angular velocities and accelerations.
#[derive(Debug)]
pub struct AngularAccelerationSpeedController {
    pub controller: AccelerationSpeedController,
}

impl AngularAccelerationSpeedController {
    pub fn new(
        min_speed: Option<AngularVelocity>,
        max_speed: Option<AngularVelocity>,
        min_acceleration: AngularAcceleration,
        max_acceleration: AngularAcceleration,
        initial_speed: AngularVelocity,
    ) -> Self {
        Self {
            controller: AccelerationSpeedController::new(
                min_speed.map(|s| s.get::<radian_per_second>()),
                max_speed.map(|s| s.get::<radian_per_second>()),
                min_acceleration.get::<radian_per_second_squared>(),
                max_acceleration.get::<radian_per_second_squared>(),
                initial_speed.get::<radian_per_second>(),
            ),
        }
    }

    /// Creates a new angular acceleration speed controller with simplified parameters.
    /// Sets min_acceleration to -max_acceleration for symmetric behavior.
    /// No speed limits are applied.
    pub fn new_simple(
        max_acceleration: AngularAcceleration,
        initial_speed: AngularVelocity,
    ) -> Self {
        Self::new(
            None,              // min_speed
            None,              // max_speed
            -max_acceleration, // min_acceleration (deceleration)
            max_acceleration,  // max_acceleration
            initial_speed,
        )
    }

    pub fn update(&mut self, target_speed: AngularVelocity, t: Instant) -> AngularVelocity {
        let target_speed = target_speed.get::<radian_per_second>();
        let new_speed = self.controller.update(target_speed, t);
        AngularVelocity::new::<radian_per_second>(new_speed)
    }

    pub fn reset(&mut self, initial_speed: AngularVelocity) {
        let initial_speed = initial_speed.get::<radian_per_second>();
        self.controller.reset(initial_speed);
    }

    pub fn set_max_acceleration(&mut self, acceleration: AngularAcceleration) {
        self.controller
            .set_max_acceleration(acceleration.get::<radian_per_second_squared>());
    }
    pub fn set_min_acceleration(&mut self, deceleration: AngularAcceleration) {
        self.controller
            .set_min_acceleration(deceleration.get::<radian_per_second_squared>());
    }

    pub fn get_min_speed(&self) -> Option<AngularVelocity> {
        self.controller
            .get_min_speed()
            .map(AngularVelocity::new::<radian_per_second>)
    }

    pub fn get_max_speed(&self) -> Option<AngularVelocity> {
        self.controller
            .get_max_speed()
            .map(AngularVelocity::new::<radian_per_second>)
    }

    pub fn set_min_speed(&mut self, min_speed: Option<AngularVelocity>) {
        self.controller
            .set_min_speed(min_speed.map(|s| s.get::<radian_per_second>()));
    }

    pub fn set_max_speed(&mut self, max_speed: Option<AngularVelocity>) {
        self.controller
            .set_max_speed(max_speed.map(|s| s.get::<radian_per_second>()));
    }
}
