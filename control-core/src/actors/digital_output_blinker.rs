use super::Actor;
use ethercat_hal::io::digital_output::DigitalOutput;
use std::{
    future::Future,
    pin::Pin,
    time::{Duration, Instant},
};

/// Set a digital output high and low with a given interval
pub struct DigitalOutputBlinker {
    last_toggle: Instant,
    output: DigitalOutput,
    interval: Duration,
    enabled: bool,
}

impl DigitalOutputBlinker {
    pub fn new(output: DigitalOutput, interval: Duration) -> Self {
        Self {
            last_toggle: Instant::now(),
            output,
            interval,
            enabled: true,
        }
    }
    pub fn set_interval(&mut self, interval: Duration) {
        self.interval = interval;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl Actor for DigitalOutputBlinker {
    fn act(&mut self, now: Instant) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            {
                let state = (self.output.state)().await;
                if now - self.last_toggle > self.interval {
                    match state.output.into() {
                        true => (self.output.write)(false.into()).await,
                        false => (self.output.write)(true.into()).await,
                    }
                    self.last_toggle = now;
                }
            }
        })
    }
}
