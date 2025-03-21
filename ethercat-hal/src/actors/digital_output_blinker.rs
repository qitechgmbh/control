use super::Actor;
use crate::io::digital_output::DigitalOutput;
use std::{future::Future, pin::Pin, sync::Arc, time::Duration};
use tokio::sync::RwLock;

/// Set a digital output high and low with a given interval
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

impl Actor for DigitalOutputBlinker {
    fn act(&mut self, _now_ts: u64) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            {
                let toggle_duration = self.interval.as_nanos() as u64;
                let state = (self.output.state)().await;
                if state.output_ts - self.last_toggle > toggle_duration {
                    match state.output.value {
                        true => (self.output.write)(false).await,
                        false => (self.output.write)(true).await,
                    }
                    self.last_toggle = state.output_ts;
                }
            }
        })
    }
}

impl From<DigitalOutputBlinker> for Arc<RwLock<DigitalOutputBlinker>> {
    fn from(actor: DigitalOutputBlinker) -> Self {
        Arc::new(RwLock::new(actor))
    }
}
