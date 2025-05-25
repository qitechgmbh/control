use std::time::Instant;

use uom::si::{
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
        acceleration: AngularAcceleration,
        deceleration: AngularAcceleration,
        initial_speed: AngularVelocity,
    ) -> Self {
        Self {
            controller: AccelerationSpeedController::new(
                acceleration.get::<radian_per_second_squared>(),
                deceleration.get::<radian_per_second_squared>(),
                initial_speed.get::<radian_per_second>(),
            ),
        }
    }

    pub fn update(&mut self, target_speed: AngularVelocity, t: Instant) -> AngularVelocity {
        let target_speed = target_speed.get::<radian_per_second>();
        let new_speed = self.controller.update(target_speed, t);
        return AngularVelocity::new::<radian_per_second>(new_speed);
    }

    pub fn reset(&mut self, initial_speed: AngularVelocity) {
        let initial_speed = initial_speed.get::<radian_per_second>();
        self.controller.reset(initial_speed);
    }

    pub fn set_acceleration(&mut self, acceleration: AngularAcceleration) {
        self.controller
            .set_acceleration(acceleration.get::<radian_per_second_squared>());
    }
    pub fn set_deceleration(&mut self, deceleration: AngularAcceleration) {
        self.controller
            .set_deceleration(deceleration.get::<radian_per_second_squared>());
    }
}
