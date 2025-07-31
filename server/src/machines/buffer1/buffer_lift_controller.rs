use control_core::{
    converters::linear_step_converter::LinearStepConverter,
    uom_extensions::velocity::meter_per_minute,
};
use ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1;
use uom::{
    ConstZero,
    si::{f64::Velocity, velocity::millimeter_per_second},
};

#[derive(Debug)]
pub struct BufferLiftController {
    /// Whether the speed controller is enabled or not
    enabled: bool,
    /// Stepper driver. Controls buffer stepper motor
    pub stepper_driver: StepperVelocityEL70x1,
    // Step Converter
    pub converter: LinearStepConverter,

    /// Fixed constants
    spool_amount: u8,

    /// Variables
    current_input_speed: Velocity,
    target_output_speed: Velocity,
    lift_speed: Velocity,
}

impl BufferLiftController {
    pub fn new(driver: StepperVelocityEL70x1, converter: LinearStepConverter) -> Self {
        Self {
            enabled: false,
            stepper_driver: driver,
            spool_amount: 13,
            converter,

            current_input_speed: Velocity::ZERO,
            target_output_speed: Velocity::ZERO,
            lift_speed: Velocity::ZERO,
        }
    }
}

impl BufferLiftController {
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
}

impl BufferLiftController {
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.stepper_driver.set_enabled(enabled);
    }

    pub fn update_speed(&mut self, speed: Velocity) {
        let steps = self.converter.velocity_to_steps(speed);
        let _ = self.stepper_driver.set_speed(steps);
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
