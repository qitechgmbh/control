use super::Actor;
use ethercat_hal::io::digital_input::DigitalInput;
use std::{future::Future, pin::Pin, time::Instant};

/// Log the state of a digital input
#[derive(Debug)]
pub struct DigitalInputGetter {
    input: DigitalInput,
    value: bool,
}

impl DigitalInputGetter {
    pub fn new(input: DigitalInput) -> Self {
        Self {
            input,
            value: false,
        }
    }

    /// Get the current value of the digital input
    pub fn value(&self) -> bool {
        self.value
    }
}

impl Actor for DigitalInputGetter {
    fn act(&mut self, _now: Instant) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let state = (self.input.state)()
                .await
                .expect("Failed to get digital input state");
            self.value = state.input.value;
        })
    }
}
