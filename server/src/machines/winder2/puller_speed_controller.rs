use std::time::Instant;

use control_core::controllers::linear_acceleration::LinearAccelerationController;
use control_core::converters::linear_step_converter::LinearStepConverter;
use serde::{Deserialize, Serialize};
use uom::{
    ConstZero,
    si::f64::{Acceleration, AngularVelocity, Length, Velocity},
};

#[derive(Debug)]
pub struct PullerSpeedController {
    enabled: bool,
    pub target_speed: Velocity,
    pub target_diameter: Length,
    pub regulation_mode: PullerRegulationMode,
    /// Forward rotation direction. If false, applies negative sign to speed
    pub forward: bool,
    /// Linear acceleration controller to dampen speed change
    acceleration_controller: LinearAccelerationController,
    /// Converter for linear to angular transformations
    pub converter: LinearStepConverter,
}

impl PullerSpeedController {
    pub fn new(
        acceleration: Acceleration,
        target_speed: Velocity,
        target_diameter: Length,
        converter: LinearStepConverter,
    ) -> Self {
        Self {
            enabled: false,
            target_speed,
            target_diameter,
            regulation_mode: PullerRegulationMode::Speed,
            forward: true,
            acceleration_controller: LinearAccelerationController::new(
                acceleration,
                -acceleration,
                Velocity::ZERO,
            ),
            converter,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn set_target_speed(&mut self, target: Velocity) {
        self.target_speed = target;
    }

    pub fn set_target_diameter(&mut self, target: Length) {
        self.target_diameter = target;
    }

    pub fn set_regulation_mode(&mut self, regulation: PullerRegulationMode) {
        self.regulation_mode = regulation;
    }

    pub fn set_forward(&mut self, forward: bool) {
        self.forward = forward;
    }

    fn get_speed(&mut self, t: Instant) -> Velocity {
        let speed = match self.enabled {
            true => match self.regulation_mode {
                PullerRegulationMode::Speed => self.target_speed,
                PullerRegulationMode::Diameter => unimplemented!(),
            },
            false => Velocity::ZERO,
        };

        let speed = if self.forward { speed } else { -speed };

        self.acceleration_controller.update(speed, t)
    }

    pub fn speed_to_angular_velocity(&self, speed: Velocity) -> AngularVelocity {
        // Use the converter to transform from linear velocity to angular velocity
        self.converter.velocity_to_angular_velocity(speed)
    }

    pub fn angular_velocity_to_speed(&self, angular_speed: AngularVelocity) -> Velocity {
        // Use the converter to transform from angular velocity to linear velocity
        self.converter.angular_velocity_to_velocity(angular_speed)
    }

    pub fn get_angular_velocity(&mut self, t: Instant) -> AngularVelocity {
        let speed = self.get_speed(t);
        self.speed_to_angular_velocity(speed)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PullerRegulationMode {
    Speed,
    Diameter,
}
