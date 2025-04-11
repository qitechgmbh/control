use super::Actor;
use ethercat_hal::io::temperature_input::TemperatureInput;
use std::{future::Future, pin::Pin, time::Instant};

/// Log the state of a temperature input
pub struct TemperatureInputLogger {
    input: TemperatureInput,
}

impl TemperatureInputLogger {
    pub fn new(input: TemperatureInput) -> Self {
        Self { input }
    }
}

impl Actor for TemperatureInputLogger {
    fn act(&mut self, _now: Instant) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let state = (self.input.state)().await;
            println!("TemperatureInputLogger: {:?}C", state.input.temperature);
        })
    }
}
