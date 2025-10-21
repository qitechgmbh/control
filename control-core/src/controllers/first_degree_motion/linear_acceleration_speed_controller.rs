use std::time::Instant;

use units::{
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
        min_speed: Option<Velocity>,
        max_speed: Option<Velocity>,
        acceleration: Acceleration,
        deceleration: Acceleration,
        initial_speed: Velocity,
    ) -> Self {
        Self {
            controller: AccelerationSpeedController::new(
                min_speed.map(|s| s.get::<meter_per_second>()),
                max_speed.map(|s| s.get::<meter_per_second>()),
                acceleration.get::<meter_per_second_squared>(),
                deceleration.get::<meter_per_second_squared>(),
                initial_speed.get::<meter_per_second>(),
            ),
        }
    }
    /// Creates a new linear acceleration speed controller with simplified parameters.
    /// Sets deceleration to -acceleration for symmetric behavior.
    /// No speed limits are applied.
    pub fn new_simple(acceleration: Acceleration, initial_speed: Velocity) -> Self {
        Self::new(
            None,          // min_speed
            None,          // max_speed
            acceleration,  // acceleration
            -acceleration, // deceleration (negative for deceleration)
            initial_speed,
        )
    }
    pub fn update(&mut self, target_speed: Velocity, t: Instant) -> Velocity {
        let target_speed = target_speed.get::<meter_per_second>();
        let new_speed = self.controller.update(target_speed, t);
        Velocity::new::<meter_per_second>(new_speed)
    }
    pub fn reset(&mut self, initial_speed: Velocity) {
        let initial_speed = initial_speed.get::<meter_per_second>();
        self.controller.reset(initial_speed);
    }
    pub fn set_acceleration(&mut self, acceleration: Acceleration) {
        self.controller
            .set_max_acceleration(acceleration.get::<meter_per_second_squared>());
    }
    pub fn set_deceleration(&mut self, deceleration: Acceleration) {
        self.controller
            .set_min_acceleration(deceleration.get::<meter_per_second_squared>());
    }
    pub fn get_min_speed(&self) -> Option<Velocity> {
        self.controller
            .get_min_speed()
            .map(Velocity::new::<meter_per_second>)
    }
    pub fn get_max_speed(&self) -> Option<Velocity> {
        self.controller
            .get_max_speed()
            .map(Velocity::new::<meter_per_second>)
    }
    pub fn set_min_speed(&mut self, min_speed: Option<Velocity>) {
        self.controller
            .set_min_speed(min_speed.map(|s| s.get::<meter_per_second>()));
    }
    pub fn set_max_speed(&mut self, max_speed: Option<Velocity>) {
        self.controller
            .set_max_speed(max_speed.map(|s| s.get::<meter_per_second>()));
    }
}
