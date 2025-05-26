use std::{pin::Pin, time::Instant};

use ethercat_hal::io::temperature_input::TemperatureInput;

use super::Actor;

#[derive(Debug)]
pub struct TemperatureInputGetter {
    input: TemperatureInput,
    temperature: f32,
    wiring_error: bool,
}

impl TemperatureInputGetter {
    pub fn new(input: TemperatureInput) -> Self {
        Self {
            input,
            temperature: 0.0,
            wiring_error: false,
        }
    }

    pub fn get_temperature(&self) -> f32 {
        self.temperature
    }

    pub fn get_wiring_error(&self) -> bool {
        self.wiring_error
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
