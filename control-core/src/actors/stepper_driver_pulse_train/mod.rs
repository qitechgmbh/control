use super::Actor;
use encoder::Encoder;
use ethercat_hal::io::pulse_train_output::PulseTrainOutput;
use std::{future::Future, pin::Pin, time::Instant};

pub mod encoder;

/// Set a digital output high and low with a given interval
#[derive(Debug)]
pub struct StepperDriverPulseTrain {
    pulse: PulseTrainOutput,
    encoder: Encoder,
    frequency: i32,
}

impl<'ptodevice> StepperDriverPulseTrain {
    pub fn new(output: PulseTrainOutput) -> Self {
        Self {
            pulse: output,
            encoder: Encoder::new(),
            frequency: 0,
        }
    }
}

impl<'ptodevice> Actor for StepperDriverPulseTrain {
    fn act(&mut self, _now: Instant) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let state = (self.pulse.state)().await;
            let mut output = state.output.clone();

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
