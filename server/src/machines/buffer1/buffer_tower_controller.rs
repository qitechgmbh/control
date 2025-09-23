use std::time::Instant;

use control_core::{
    controllers::second_degree_motion::linear_jerk_speed_controller::LinearJerkSpeedController,
    converters::linear_step_converter::LinearStepConverter,
    uom_extensions::{
        acceleration::meter_per_minute_per_second, jerk::meter_per_minute_per_second_squared,
        velocity::meter_per_minute,
    },
};
use ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1;
use uom::{
    ConstZero,
    si::{
        f64::{Acceleration, Jerk, Velocity},
        velocity::millimeter_per_second,
    },
};

#[derive(Debug)]
pub struct BufferTowerController {
    /// Whether the speed controller is enabled or not
    enabled: bool,
    /// Stepper driver. Controls buffer stepper motor
    pub stepper_driver: StepperVelocityEL70x1,
    // Step Converter
    pub converter: LinearStepConverter,

    /// Linear acceleration controller to dampen speed change
    acceleration_controller: LinearJerkSpeedController,

    /// Forward rotation direction. If false, applies negative sign to speed
    pub forward: bool,

    /// Fixed constants
    spool_amount: u8,

    /// Variables
    current_input_speed: Velocity,
    target_output_speed: Velocity,
    lift_speed: Velocity,
}

impl BufferTowerController {
    pub const fn new(driver: StepperVelocityEL70x1) -> Self {
        let acceleration = Acceleration::new::<meter_per_minute_per_second>(5.0);
        let jerk = Jerk::new::<meter_per_minute_per_second_squared>(10.0);
        let speed = Velocity::new::<meter_per_minute>(50.0);
        Self {
            enabled: false,
            stepper_driver: driver,
            spool_amount: 13,
            converter,
            forward: true,
            acceleration_controller: LinearJerkSpeedController::new_simple(
                Some(speed),
                acceleration,
                jerk,
            ),

            current_input_speed: Velocity::ZERO,
            target_output_speed: Velocity::ZERO,
            lift_speed: Velocity::ZERO,
        }
    }
}

impl BufferTowerontroller {
    /// Calculate the speed of the buffer lift from current input speed
    ///
    /// Formula: input_speed / ( 2 * spool_amount )
    pub fn calculate_buffer_lift_speed(&mut self) -> Velocity {
        self.lift_speed = Velocity::new::<millimeter_per_second>(
            (self.current_input_speed.get::<millimeter_per_second>()
                - self.target_output_speed.get::<millimeter_per_second>())
                / (2.0 * self.spool_amount as f64),
        );
        self.lift_speed
    }

    pub fn update_speed(&mut self, t: Instant) -> Velocity {
        let speed = match self.enabled {
            true => self.calculate_buffer_lift_speed(),
            false => Velocity::ZERO,
        };

        let speed = if self.forward { speed } else { -speed };

        self.acceleration_controller.update(speed, t)
    }
}

impl BufferTowerController {
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.stepper_driver.set_enabled(enabled);
    }
    pub fn set_forward(&mut self, forward: bool) {
        self.forward = forward;
    }

    pub fn set_current_input_speed(&mut self, speed: f64) {
        self.current_input_speed = Velocity::new::<meter_per_minute>(speed);
    }
    pub fn set_target_output_speed(&mut self, speed: f64) {
        self.target_output_speed = Velocity::new::<meter_per_minute>(speed);
    }
    pub fn get_current_input_speed(&self) -> Velocity {
        self.current_input_speed
    }
    pub fn get_target_output_speed(&self) -> Velocity {
        self.target_output_speed
    }
    pub fn get_lift_speed(&self) -> Velocity {
        self.lift_speed
    }
}
