use std::time::Instant;

use uom::si::{
    acceleration::meter_per_second_squared,
    f64::{Acceleration, Jerk, Velocity},
    jerk::meter_per_second_cubed,
    velocity::meter_per_second,
};

use super::jerk_speed_controller::JerkSpeedController;

/// Linear Jerk Speed Controller with proper physical units
#[derive(Debug)]
pub struct LinearJerkSpeedController {
    controller: JerkSpeedController,
    last_update: Option<Instant>,
}

impl LinearJerkSpeedController {
    /// Create a new linear speed controller with jerk limits
    pub fn new(
        min_speed: Option<Velocity>,
        max_speed: Option<Velocity>,
        min_acceleration: Acceleration,
        max_acceleration: Acceleration,
        min_jerk: Jerk,
        max_jerk: Jerk,
    ) -> Self {
        Self {
            controller: JerkSpeedController::new(
                min_speed.map(|speed| speed.get::<meter_per_second>()),
                max_speed.map(|speed| speed.get::<meter_per_second>()),
                min_acceleration.get::<meter_per_second_squared>(),
                max_acceleration.get::<meter_per_second_squared>(),
                min_jerk.get::<meter_per_second_cubed>(),
                max_jerk.get::<meter_per_second_cubed>(),
            ),
            last_update: None,
        }
    }

    /// Update the controller with a new target speed
    pub fn update(&mut self, target_speed: Velocity, t: Instant) -> Velocity {
        // Calculate dt from the last update
        let dt = if let Some(last_t) = self.last_update {
            t.duration_since(last_t).as_secs_f64()
        } else {
            0.0 // First update, no time has passed
        };
        self.last_update = Some(t);

        // Convert target to raw value, update controller, and convert result back to Velocity
        let target_raw = target_speed.get::<meter_per_second>();
        let speed_raw = self.controller.update(target_raw, dt);
        Velocity::new::<meter_per_second>(speed_raw)
    }

    /// Get the current speed
    pub fn get_speed(&self) -> Velocity {
        Velocity::new::<meter_per_second>(self.controller.get_speed())
    }

    /// Get the target speed
    pub fn get_target_speed(&self) -> Velocity {
        Velocity::new::<meter_per_second>(self.controller.get_target_speed())
    }

    /// Get the minimum velocity limit
    pub fn get_min_speed(&self) -> Option<Velocity> {
        self.controller
            .get_min_speed()
            .map(|speed| Velocity::new::<meter_per_second>(speed))
    }

    /// Get the maximum velocity limit
    pub fn get_max_speed(&self) -> Option<Velocity> {
        self.controller
            .get_max_speed()
            .map(|speed| Velocity::new::<meter_per_second>(speed))
    }

    /// Set the minimum velocity limit
    pub fn set_min_speed(&mut self, min_speed: Option<Velocity>) {
        self.controller
            .set_min_speed(min_speed.map(|speed| speed.get::<meter_per_second>()));
    }

    /// Set the maximum velocity limit
    pub fn set_max_speed(&mut self, max_speed: Option<Velocity>) {
        self.controller
            .set_max_speed(max_speed.map(|speed| speed.get::<meter_per_second>()));
    }

    /// Get the current acceleration
    pub fn get_acceleration(&self) -> Acceleration {
        Acceleration::new::<meter_per_second_squared>(self.controller.get_acceleration())
    }

    /// Set the minimum acceleration
    pub fn set_min_acceleration(&mut self, min_acceleration: Acceleration) {
        self.controller
            .set_min_acceleration(min_acceleration.get::<meter_per_second_squared>());
    }

    /// Set the maximum acceleration
    pub fn set_max_acceleration(&mut self, max_acceleration: Acceleration) {
        self.controller
            .set_max_acceleration(max_acceleration.get::<meter_per_second_squared>());
    }

    /// Get the current jerk
    pub fn get_jerk(&self) -> Jerk {
        Jerk::new::<meter_per_second_cubed>(self.controller.get_jerk())
    }

    /// Set the minimum jerk
    pub fn set_min_jerk(&mut self, min_jerk: Jerk) {
        self.controller
            .set_min_jerk(min_jerk.get::<meter_per_second_cubed>());
    }

    /// Set the maximum jerk
    pub fn set_max_jerk(&mut self, max_jerk: Jerk) {
        self.controller
            .set_max_jerk(max_jerk.get::<meter_per_second_cubed>());
    }

    /// Reset the controller to a new velocity and acceleration
    ///
    /// This resets all internal state including:
    /// - Current velocity to the provided value
    /// - Current acceleration to the provided value (optional, defaults to 0)
    /// - Current jerk to 0
    /// - Target velocity to the current velocity
    ///
    /// # Parameters
    /// - `velocity`: The new current velocity
    /// - `acceleration`: The new current acceleration (optional, defaults to 0)
    pub fn reset(&mut self, velocity: Velocity, acceleration: Option<Acceleration>) {
        let acceleration_value = acceleration.map(|a| a.get::<meter_per_second_squared>());
        self.controller
            .reset(velocity.get::<meter_per_second>(), acceleration_value);
        self.last_update = None;
    }
}
