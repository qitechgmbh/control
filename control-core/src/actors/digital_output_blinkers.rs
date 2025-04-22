use super::Actor;
use ethercat_hal::io::digital_output::DigitalOutput;
use std::{
    future::Future,
    pin::Pin,
    time::{Duration, Instant},
};

/// Set a series of digital outputs high in sequence with a given interval
pub struct DigitalOutputBlinkers {
    last_toggle: Instant,
    outputs: Vec<Option<DigitalOutput>>,
    index: usize,
    interval: Duration,
    enabled: bool,
    amount: usize,
}

impl DigitalOutputBlinkers {
    pub fn new(outputs: Vec<Option<DigitalOutput>>, interval: Duration, amount: usize) -> Self {
        Self {
            last_toggle: Instant::now(),
            outputs,
            interval,
            enabled: true,
            index: 0,
            amount,
        }
    }

    pub fn set_interval(&mut self, interval: Duration) {
        self.interval = interval;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl Actor for DigitalOutputBlinkers {
    fn act(&mut self, now: Instant) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            if now - self.last_toggle > self.interval {
                if let Some(output) = &self.outputs[self.index] {
                    (output.write)(true.into()).await;
                }
                let index_end =
                    (self.index + self.outputs.len() - self.amount) % self.outputs.len();
                if let Some(output) = &self.outputs[index_end] {
                    (output.write)(false.into()).await;
                }
                self.last_toggle = now;
                self.index = (self.index + 1) % self.outputs.len();
            }
        })
    }
}
