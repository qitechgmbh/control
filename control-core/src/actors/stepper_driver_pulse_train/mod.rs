use super::Actor;
use encoder::Encoder;
use ethercat_hal::io::pulse_train_output::PulseTrainOutput;
use uom::si::{acceleration::Acceleration, angle::Angle, angular_acceleration::AngularAcceleration, angular_velocity::revolution_per_second, quantities::Acceleration};
use std::{future::Future, pin::Pin};
use stepper_driver::{measurements::Acceleration, StepperDriver};

pub mod encoder;

/// Set a digital output high and low with a given interval
#[derive(Debug)]
pub struct StepperDriverPulseTrain {
    pulse: PulseTrainOutput,
    encoder: Encoder,
    frequency: i32,
    stepper_aclulator: StepperDriver,
}

impl<'ptodevice> StepperDriverPulseTrain {
    pub fn new(output: PulseTrainOutput) -> Self {
        Self {
            pulse: output,
            encoder: Encoder::new(),
            frequency: 0,
            stepper_aclulator: StepperDriver::new(
                max_acceleration = 
                max_speed,
                min_position,
                max_position,
                pulse_time,
                min_pulse_offset,
                min_direction_offset,
                steps_per_revolution,
                radius,
            ),
        }
    }
}

impl<'ptodevice> Actor for StepperDriverPulseTrain {
    fn act(&mut self, _now_ts: u64) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let state = (self.pulse.state)().await;
            let mut output = state.output.clone();

            let x = AngularAcceleration::new::<per_second_sqared

            // sync the counter to the encoder
            self.encoder.update(
                state.input.counter_value,
                state.input.counter_overflow,
                state.input.counter_underflow,
            );

            // write frequency
            output.frequency_value = self.frequency;

            (self.pulse.write)(output).await;
        })
    }
}

impl<'ptodevice> StepperDriverPulseTrain {
    pub fn set_frequency(&mut self, frequency: i32) {
        self.frequency = frequency;
    }
}
