use crate::ethercat_drivers::{
    actor::Actor, io::digital_output::DigitalOutput, utils::traits::ArcRwLock,
};
use std::{future::Future, pin::Pin};

/// Set a digital output high and low with a given interval
pub struct StepperDriverMaxSpeed {
    output: DigitalOutput,
}

impl StepperDriverMaxSpeed {
    pub fn new(output: DigitalOutput) -> Self {
        Self { output }
    }
}

impl Actor for StepperDriverMaxSpeed {
    fn act(&mut self, _now_ts: u64) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let state = (self.output.state)().await;
            match state.value {
                true => (self.output.write)(false).await,
                false => (self.output.write)(true).await,
            }
        })
    }
}

impl ArcRwLock for StepperDriverMaxSpeed {}
