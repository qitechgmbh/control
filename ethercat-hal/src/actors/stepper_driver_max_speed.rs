use super::Actor;
use crate::io::digital_output::DigitalOutput;
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, RwLock},
};

/// Set a digital output high and low with a given interval
pub struct StepperDriverMaxSpeed {
    pulse: DigitalOutput,
}

impl StepperDriverMaxSpeed {
    pub fn new(output: DigitalOutput) -> Self {
        Self { pulse: output }
    }
}

impl Actor for StepperDriverMaxSpeed {
    fn act(&mut self, _now_ts: u64) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let state = (self.pulse.state)().await;
            match state.output.into() {
                true => (self.pulse.write)(false.into()).await,
                false => (self.pulse.write)(true.into()).await,
            }
        })
    }
}

impl From<StepperDriverMaxSpeed> for Arc<RwLock<StepperDriverMaxSpeed>> {
    fn from(actor: StepperDriverMaxSpeed) -> Self {
        Arc::new(RwLock::new(actor))
    }
}
