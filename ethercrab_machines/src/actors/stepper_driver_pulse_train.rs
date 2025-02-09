use std::{future::Future, pin::Pin};

use crate::{actor::Actor, io::digital_output::DigitalOutput, utils::traits::ArcRwLock};

/// Set a digital output high and low with a given interval
pub struct StepperDriverPulseTrain {
    pulse: DigitalOutput,
}

impl StepperDriverPulseTrain {
    pub fn new(output: DigitalOutput) -> Self {
        Self { pulse: output }
    }
}

impl Actor for StepperDriverPulseTrain {
    fn act(&mut self, _now_ts: u64) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let state = (self.pulse.state)().await;
            match state.value {
                true => (self.pulse.write)(false).await,
                false => (self.pulse.write)(true).await,
            }
        })
    }
}

impl ArcRwLock for StepperDriverPulseTrain {}
