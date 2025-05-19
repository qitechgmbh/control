use super::Actor;
use core::panic;
use ethercat_hal::{
    helpers::el70xx_velocity_converter::EL70x1VelocityCalculator,
    io::stepper_velocity_el70x1::StepperVelocityEL70x1, shared_config::el70x1::EL70x1SpeedRange,
};
use std::{future::Future, pin::Pin, time::Instant};

/// Set a digital output high and low with a given interval
#[derive(Debug)]
pub struct StepperDriverEL70x1 {
    stepper: StepperVelocityEL70x1,
    enabled: bool,
    velocity: i16,
    pub converter: EL70x1VelocityCalculator,
}

impl StepperDriverEL70x1 {
    pub fn new(stepper: StepperVelocityEL70x1, speed_range: &EL70x1SpeedRange) -> Self {
        Self {
            stepper,
            enabled: false,
            velocity: 0,
            converter: EL70x1VelocityCalculator::new(speed_range),
        }
    }

    /// Set the speed in steps per second
    pub fn set_speed(&mut self, steps_per_second: i32) {
        self.velocity = self.converter.steps_to_velocity(steps_per_second)
    }

    /// Get the speed in steps per second
    pub fn get_speed(&self) -> i32 {
        self.converter.velocity_to_steps(self.velocity) as i32
    }

    /// Enable or disable the stepper
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Get the enabled state of the stepper
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Actor for StepperDriverEL70x1 {
    fn act(&mut self, _now: Instant) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let state = match (self.stepper.state)().await {
                Ok(state) => state,
                Err(e) => {
                    panic!("Error while reading StepperVelocity {:?}", e);
                }
            };
            let mut output = state.output.clone();

            // set the output
            output.enable = self.enabled;
            output.velocity = self.velocity;

            // write the output
            match (self.stepper.write)(output).await {
                Ok(_) => {}
                Err(e) => {
                    panic!("Error while writing StepperVelocity {:?}", e);
                }
            }
        })
    }
}
