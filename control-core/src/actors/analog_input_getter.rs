use super::Actor;
use ethercat_hal::io::analog_input::{AnalogInput, physical::AnalogInputValue};
use std::{future::Future, pin::Pin, time::Instant};

/// Log the state of a analog input
#[derive(Debug)]
pub struct AnalogInputGetter {
    input: AnalogInput,
    normalized: Option<f32>,
}

impl AnalogInputGetter {
    pub fn new(input: AnalogInput) -> Self {
        Self {
            input,
            normalized: None,
        }
    }

    /// Value from -1.0 to 1.0
    pub fn get_normalized(&self) -> Option<f32> {
        self.normalized
    }

    pub fn get_physical(&self) -> Option<AnalogInputValue> {
        match self.get_normalized() {
            Some(normalized) => Some(self.input.range.normalized_to_physical(normalized)),
            None => return None,
        }
    }
}

impl Actor for AnalogInputGetter {
    fn act(&mut self, _now: Instant) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let state = (self.input.state)().await;
            self.normalized = Some(state.input.normalized);
        })
    }
}
