use std::{pin::Pin, time::Instant};

use ethercat_hal::io::temperature_input::TemperatureInput;

use super::Actor;

#[derive(Debug)]
pub struct TemperatureInputGetter {
    input: TemperatureInput,
    pub temperature: f32,
    pub wiring_error: bool,
}

impl TemperatureInputGetter {
    pub fn new(input: TemperatureInput) -> Self {
        Self {
            input,
            temperature: 0.0,
            wiring_error: false,
        }
    }
}

impl Actor for TemperatureInputGetter {
    fn act(&mut self, _now: Instant) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let input = (self.input.state)().await.input;
            self.temperature = input.temperature;
            self.wiring_error = input.overvoltage | input.undervoltage;
            if self.wiring_error {
                self.temperature = 0.0;
            }
        })
    }
}
