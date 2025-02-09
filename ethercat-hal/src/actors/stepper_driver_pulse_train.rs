use std::{future::Future, pin::Pin, sync::Arc};

use tokio::sync::RwLock;

use crate::{actor::Actor, io::digital_output::DigitalOutput};

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

impl From<StepperDriverPulseTrain> for Arc<RwLock<StepperDriverPulseTrain>> {
    fn from(actor: StepperDriverPulseTrain) -> Self {
        Arc::new(RwLock::new(actor))
    }
}
