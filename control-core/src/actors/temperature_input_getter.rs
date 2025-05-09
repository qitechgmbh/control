use std::{pin::Pin, time::Instant};

use ethercat_hal::io::temperature_input::TemperatureInput;

use super::Actor;

#[derive(Debug)]
pub struct TemperatureInputGetter {
    input: TemperatureInput,
    pub temperature: f32,
}

impl TemperatureInputGetter {
    pub fn new(input: TemperatureInput) -> Self {
        Self {
            input,
            temperature: 0.0,
        }
    }
}

impl Actor for TemperatureInputGetter {
    fn act(&mut self, _now: Instant) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let temperature = (self.input.state)().await.input.temperature;
            self.temperature = temperature;
        })
    }
}
