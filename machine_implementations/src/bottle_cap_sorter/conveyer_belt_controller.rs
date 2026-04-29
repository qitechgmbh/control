use std::time::Instant;

use control_core::{
    controllers::second_degree_motion::linear_jerk_speed_controller::LinearJerkSpeedController,
    converters::linear_step_converter::LinearStepConverter,
};

use qitech_lib::units::{
    ConstZero,
    Acceleration, AngularVelocity, Jerk, Velocity,
    acceleration::meter_per_second_squared, jerk::meter_per_second_cubed, velocity::meter_per_second,
};

#[derive(Debug)]
pub struct ConveyorBeltController {
    enabled: bool,
    pub target_speed: Velocity,
    /// Linear acceleration controller to dampen speed change
    acceleration_controller: LinearJerkSpeedController,
    /// Converter for linear to angular transformations
    pub converter: LinearStepConverter,
    pub last_speed: Velocity,
}

impl ConveyorBeltController {
    pub fn new(target_speed: Velocity, converter: LinearStepConverter) -> Self {
        let acceleration = Acceleration::new::<meter_per_second_squared>(0.17); // 10 m/min/s = 0.17 m/s²
        let jerk = Jerk::new::<meter_per_second_cubed>(0.33); // 20 m/min/s² = 0.33 m/s³
        let max_speed = Velocity::new::<meter_per_second>(1.67); // 100 m/min = 1.67 m/s

        Self {
            enabled: false,
            target_speed,
            acceleration_controller: LinearJerkSpeedController::new_simple(
                Some(max_speed),
                acceleration,
                jerk,
            ),
            converter,
            last_speed: Velocity::ZERO,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_target_speed(&mut self, target: Velocity) {
        self.target_speed = target;
    }

    pub fn get_target_speed(&self) -> Velocity {
        self.target_speed
    }

    pub fn get_current_speed(&self) -> Velocity {
        self.last_speed
    }

    fn update_speed(&mut self, t: Instant) -> Velocity {
        let speed = match self.enabled {
            true => self.target_speed,
            false => Velocity::ZERO,
        };

        let speed = self.acceleration_controller.update(speed, t);

        self.last_speed = speed;
        speed
    }

    pub fn speed_to_angular_velocity(&self, speed: Velocity) -> AngularVelocity {
        self.converter.velocity_to_angular_velocity(speed)
    }

    pub fn angular_velocity_to_speed(&self, angular_speed: AngularVelocity) -> Velocity {
        self.converter.angular_velocity_to_velocity(angular_speed)
    }

    pub fn calc_speed(&mut self, t: Instant) -> f64 {
        let speed = self.update_speed(t);
        let angular_velocity = self.speed_to_angular_velocity(speed);
        self.converter.angular_velocity_to_steps(angular_velocity)
    }
}
