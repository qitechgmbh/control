use super::Actor;
use ethercat_hal::io::digital_output::DigitalOutput;
use std::{future::Future, pin::Pin, time::Instant};

/// Set a digital output high and low with a given interval
#[derive(Debug)]
pub struct DigitalOutputSetter {
    output: DigitalOutput,
    enabled: bool,
}

impl DigitalOutputSetter {
    pub fn new(output: DigitalOutput) -> Self {
        Self {
            output,
            enabled: false,
        }
    }
    pub fn set(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    pub fn get(&self) -> bool {
        self.enabled
    }
    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }
}

impl Actor for DigitalOutputSetter {
    fn act(&mut self, _now: Instant) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            {
                (self.output.write)(self.enabled.into()).await;
            }
        })
    }
}
