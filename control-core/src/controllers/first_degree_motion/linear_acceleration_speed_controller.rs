use std::time::Instant;

use uom::si::{
    acceleration::meter_per_second_squared,
    f64::{Acceleration, Velocity},
    velocity::meter_per_second,
};

use super::acceleration_speed_controller::AccelerationSpeedController;

/// [`LinearAccelerationController`] wraps [`LinearF64AccelerationController`]
/// to handle linear velocities and accelerations.
#[derive(Debug)]
pub struct LinearAccelerationLimitingController {
    pub controller: AccelerationSpeedController,
}

impl LinearAccelerationLimitingController {
    pub fn new(
        acceleration: Acceleration,
        deceleration: Acceleration,
        initial_speed: Velocity,
    ) -> Self {
        Self {
            controller: AccelerationSpeedController::new(
                acceleration.get::<meter_per_second_squared>(),
                deceleration.get::<meter_per_second_squared>(),
                initial_speed.get::<meter_per_second>(),
            ),
        }
    }
    pub fn update(&mut self, target_speed: Velocity, t: Instant) -> Velocity {
        let target_speed = target_speed.get::<meter_per_second>();
        let new_speed = self.controller.update(target_speed, t);
        return Velocity::new::<meter_per_second>(new_speed);
    }
    pub fn reset(&mut self, initial_speed: Velocity) {
        let initial_speed = initial_speed.get::<meter_per_second>();
        self.controller.reset(initial_speed);
    }
    pub fn set_acceleration(&mut self, acceleration: Acceleration) {
        self.controller
            .set_acceleration(acceleration.get::<meter_per_second_squared>());
    }
    pub fn set_deceleration(&mut self, deceleration: Acceleration) {
        self.controller
            .set_deceleration(deceleration.get::<meter_per_second_squared>());
    }
}
