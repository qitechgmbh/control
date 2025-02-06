use crate::{actor::Actor, io::digital_output::DigitalOutput, utils::traits::ArcRwLock};
use std::{
    future::Future,
    pin::Pin,
    time::{Duration, SystemTime},
};

/// Set a digital output high and low with a given interval
pub struct StepperDriver {
    // Context
    /// Steps on the motor per revolution
    controller: stepper_driver::StepperDriver,

    // Hardware
    /// Digital output to control the motor
    pulse: DigitalOutput,
    /// Digital output to control the direction of the motor
    direction: DigitalOutput,
}

impl StepperDriver {
    fn new(steps: u16, pulse: DigitalOutput, direction: DigitalOutput) -> Self {
        Self {
            pulse,
            direction,
            controller: stepper_driver::StepperDriver::new(
                200.0,                                  // steps/s^2
                200.0,                                  // steps/s
                i128::MIN,                              // lower limit
                i128::MAX,                              // upper limit
                Duration::from_micros(3).as_secs_f64(), // min pulse width
                Duration::from_micros(3).as_secs_f64(), // min pulse offset
                Duration::from_micros(5).as_secs_f64(), // min direction change offset
                Some(steps as i128),                    // steps per revolution
                None,                                   // radius
            ),
        }
    }
}

impl Actor for StepperDriver {
    fn act(&mut self, _now_ts: u64) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let output = self.controller.output(&SystemTime::now());
            (self.pulse.write)(output.pulse).await;
            (self.direction.write)(output.direction).await;
        })
    }
}

impl ArcRwLock for StepperDriver {}
