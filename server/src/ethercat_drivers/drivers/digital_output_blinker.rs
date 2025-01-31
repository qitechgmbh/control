use crate::ethercat_drivers::{io::digital_output::DigitalOutput, tick::Tick};
use std::{future::Future, pin::Pin, time::Duration};

pub struct DigitalOutputBlinker {
    last_toggle: u64,
    output: DigitalOutput,
    interval: Duration,
    enabled: bool,
}

impl DigitalOutputBlinker {
    pub fn new(output: DigitalOutput, interval: Duration) -> Self {
        Self {
            last_toggle: 0,
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

impl Tick for DigitalOutputBlinker {
    fn tick(&mut self, now_ts: u64) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let toggle_duration = self.interval.as_nanos() as u64;
            let signal = (self.output.get)().await;
            if now_ts - self.last_toggle > toggle_duration {
                match signal.value {
                    true => (self.output.write)(false).await,
                    false => (self.output.write)(true).await,
                }
                self.last_toggle = now_ts;
            }
        })
    }
}
