use super::Actor;
use ethercat_hal::io::digital_input::DigitalInput;
use std::{future::Future, pin::Pin, time::Instant};

/// Log the state of a digital input
pub struct DigitalInputLogger {
    input: DigitalInput,
}

impl DigitalInputLogger {
    pub fn new(input: DigitalInput) -> Self {
        Self { input }
    }
}

impl Actor for DigitalInputLogger {
    fn act(&mut self, _now: Instant) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let state = (self.input.state)().await;
            println!("DigitalInputLogger: {}", state.input.value);
        })
    }
}
