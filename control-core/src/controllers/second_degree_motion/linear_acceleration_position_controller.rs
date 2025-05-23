use std::time::Instant;

use uom::si::{
    acceleration::meter_per_second_squared,
    f64::{Acceleration, Length, Velocity},
    length::meter,
    velocity::meter_per_second,
};

use super::acceleration_position_controller::AccelerationPositionController;

/// Linear Acceleration Position Controller with proper physical units
#[derive(Debug)]
pub struct LinearAccelerationPositionController {
    controller: AccelerationPositionController,
    last_update: Option<Instant>,
}

impl LinearAccelerationPositionController {
    /// Create a new linear position controller with acceleration limits
    pub fn new(
        max_speed: Velocity,
        min_speed: Velocity,
        max_acceleration: Acceleration,
        min_acceleration: Acceleration,
    ) -> Self {
        Self {
            controller: AccelerationPositionController::new(
                max_speed.get::<meter_per_second>(),
                min_speed.get::<meter_per_second>(),
                max_acceleration.get::<meter_per_second_squared>(),
                min_acceleration.get::<meter_per_second_squared>(),
            ),
            last_update: None,
        }
    }

    /// Update the controller with a new target position
    pub fn update(&mut self, target_position: Length, t: Instant) -> Length {
        // Calculate dt from the last update
        let dt = if let Some(last_t) = self.last_update {
            t.duration_since(last_t).as_secs_f64()
        } else {
            0.0 // First update, no time has passed
        };
        self.last_update = Some(t);

        // Convert target to raw value, update controller, and convert result back to Length
        let target_raw = target_position.get::<meter>();
        let position_raw = self.controller.update(target_raw, dt);
        Length::new::<meter>(position_raw)
    }

    /// Get the current position
    pub fn get_position(&self) -> Length {
        Length::new::<meter>(self.controller.get_position())
    }

    /// Get the current speed
    pub fn get_speed(&self) -> Velocity {
        Velocity::new::<meter_per_second>(self.controller.get_speed())
    }

    /// Get the current acceleration
    pub fn get_acceleration(&self) -> Acceleration {
        Acceleration::new::<meter_per_second_squared>(self.controller.get_acceleration())
    }

    /// Get the target position
    pub fn get_target_position(&self) -> Length {
        Length::new::<meter>(self.controller.get_target_position())
    }

    /// Set the maximum speed
    pub fn set_max_speed(&mut self, max_speed: Velocity) {
        self.controller
            .set_max_speed(max_speed.get::<meter_per_second>());
    }

    /// Set the minimum speed
    pub fn set_min_speed(&mut self, min_speed: Velocity) {
        self.controller
            .set_min_speed(min_speed.get::<meter_per_second>());
    }

    /// Set the maximum acceleration
    pub fn set_max_acceleration(&mut self, max_acceleration: Acceleration) {
        self.controller
            .set_max_acceleration(max_acceleration.get::<meter_per_second_squared>());
    }

    /// Set the minimum acceleration
    pub fn set_min_acceleration(&mut self, min_acceleration: Acceleration) {
        self.controller
            .set_min_acceleration(min_acceleration.get::<meter_per_second_squared>());
    }
}
