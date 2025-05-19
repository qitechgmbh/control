use std::time::Instant;

use control_core::controllers::linear_acceleration::LinearAccelerationController;
use serde::{Deserialize, Serialize};
use uom::{
    ConstZero,
    si::{
        angular_velocity::revolution_per_second,
        f64::{Acceleration, AngularVelocity, Length, Velocity},
        length::{meter, millimeter},
        velocity::meter_per_second,
    },
};

#[derive(Debug)]
pub struct PullerSpeedController {
    enabled: bool,
    pub target_speed: Velocity,
    pub target_diameter: Length,
    pub regulation_mode: PullerRegulationMode,
    /// Linear acceleration controller to dampen speed change
    acceleration_controller: LinearAccelerationController,
}

impl PullerSpeedController {
    pub fn new(
        acceleration: Acceleration,
        target_speed: Velocity,
        target_diameter: Length,
    ) -> Self {
        Self {
            enabled: false,
            target_speed,
            target_diameter,
            regulation_mode: PullerRegulationMode::Speed,
            acceleration_controller: LinearAccelerationController::new(
                acceleration,
                -acceleration,
                Velocity::ZERO,
            ),
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

    fn get_speed(&mut self, t: Instant) -> Velocity {
        let speed = match self.enabled {
            true => match self.regulation_mode {
                PullerRegulationMode::Speed => self.target_speed,
                PullerRegulationMode::Diameter => unimplemented!(),
            },
            false => Velocity::ZERO,
        };

        self.acceleration_controller.update(speed, t)
    }

    pub fn speed_to_angular_velocity(speed: Velocity) -> AngularVelocity {
        // The diameter of the wheel is 80mm
        let diameter = Length::new::<millimeter>(80.0);
        let circumfence = diameter * std::f64::consts::PI;

        // convert linear speed to angular speed
        let angular_speed = AngularVelocity::new::<revolution_per_second>(
            speed.get::<meter_per_second>() / circumfence.get::<meter>(),
        );

        angular_speed
    }

    pub fn angular_velocity_to_speed(angular_speed: AngularVelocity) -> Velocity {
        // The diameter of the wheel is 80mm
        let diameter = Length::new::<millimeter>(80.0);
        let circumfence = diameter * std::f64::consts::PI;

        // convert angular speed to linear speed
        let speed = Velocity::new::<meter_per_second>(
            angular_speed.get::<revolution_per_second>() * circumfence.get::<meter>(),
        );

        speed
    }

    pub fn get_angular_velocity(&mut self, t: Instant) -> AngularVelocity {
        let speed = self.get_speed(t);
        return Self::speed_to_angular_velocity(speed);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PullerRegulationMode {
    Speed,
    Diameter,
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use core::f64;

    #[test]
    fn test_speed_to_angular_velocity() {
        // Test case 1: Standard positive velocity
        let speed1 = Velocity::new::<meter_per_second>(1.0);
        // Diameter = 80mm = 0.08m
        // Circumference = π * 0.08 ≈ 0.25133m
        // Expected angular velocity = 1.0 / 0.25133 ≈ 3.9789 rev/s
        let expected1 =
            AngularVelocity::new::<revolution_per_second>(1.0 / (0.08 * std::f64::consts::PI));
        let result1 = PullerSpeedController::speed_to_angular_velocity(speed1);
        assert_relative_eq!(
            result1.get::<revolution_per_second>(),
            expected1.get::<revolution_per_second>(),
            epsilon = f64::EPSILON
        );

        // Test case 2: Zero velocity
        let speed2 = Velocity::new::<meter_per_second>(0.0);
        let result2 = PullerSpeedController::speed_to_angular_velocity(speed2);
        assert_relative_eq!(
            result2.get::<revolution_per_second>(),
            0.0,
            epsilon = f64::EPSILON
        );

        // Test case 3: Negative velocity (reverse direction)
        let speed3 = Velocity::new::<meter_per_second>(-2.0);
        let expected3 =
            AngularVelocity::new::<revolution_per_second>(-2.0 / (0.08 * std::f64::consts::PI));
        let result3 = PullerSpeedController::speed_to_angular_velocity(speed3);
        assert_relative_eq!(
            result3.get::<revolution_per_second>(),
            expected3.get::<revolution_per_second>(),
            epsilon = f64::EPSILON
        );
    }
}
