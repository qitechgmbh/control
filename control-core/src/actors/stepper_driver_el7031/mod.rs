use super::Actor;
use core::panic;
use el70xx_velocity_converter::EL7031VelocityCalculator;
use ethercat_hal::{
    devices::el7031::coe::EL7031SpeedRange, io::stepper_velocity_el7031::StepperVelocityEL7031,
};
use std::{future::Future, pin::Pin};

pub mod el70xx_velocity_converter;

/// Set a digital output high and low with a given interval
#[derive(Debug)]
pub struct StepperDriverEl7031 {
    stepper: StepperVelocityEL7031,
    enabled: bool,
    velocity: i16,
    converter: EL7031VelocityCalculator,
}

impl StepperDriverEl7031 {
    pub fn new(stepper: StepperVelocityEL7031, speed_range: &EL7031SpeedRange) -> Self {
        Self {
            stepper,
            enabled: false,
            velocity: 0,
            converter: EL7031VelocityCalculator::new(speed_range),
        }
    }
    pub fn set_speed(&mut self, steps_per_second: i32) {
        self.velocity = self.converter.steps_to_velocity(steps_per_second)
    }
    pub fn get_speed(&self) -> i32 {
        self.converter.velocity_to_steps(self.velocity) as i32
    }
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Actor for StepperDriverEl7031 {
    fn act(&mut self, _now_ts: u64) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let state = match (self.stepper.state)().await {
                Ok(state) => state,
                Err(e) => {
                    panic!("Error while reading StepperVelocity {:?}", e);
                }
            };
            let mut output = state.output.clone();

            // set the output
            output.stm_control.enable = self.enabled;
            output.stm_velocity.velocity = self.velocity;

            println!("StepperDriverEl7031: {:?}", output);

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
