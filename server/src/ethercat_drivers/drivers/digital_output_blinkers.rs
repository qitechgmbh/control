use crate::ethercat_drivers::{
    actor::Actor, io::digital_output::DigitalOutput, utils::traits::ArcRwLock,
};
use std::{future::Future, pin::Pin, time::Duration};

/// Set a series of digital outputs high in sequence with a given interval
pub struct DigitalOutputBlinkers {
    last_toggle: u64,
    outputs: Vec<Option<DigitalOutput>>,
    index: usize,
    interval: Duration,
    enabled: bool,
    amount: usize,
}

impl DigitalOutputBlinkers {
    pub fn new(outputs: Vec<Option<DigitalOutput>>, interval: Duration, amount: usize) -> Self {
        Self {
            last_toggle: 0,
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
    fn act(&mut self, _now_ts: u64) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let toggle_duration = self.interval.as_nanos() as u64;
            let state = (self.outputs[0].as_ref().unwrap().state)().await;
            if state.output_ts - self.last_toggle > toggle_duration {
                if let Some(output) = &self.outputs[self.index] {
                    (output.write)(true).await;
                }
                let index_end =
                    (self.index + self.outputs.len() - self.amount) % self.outputs.len();
                if let Some(output) = &self.outputs[index_end] {
                    (output.write)(false).await;
                }
                self.last_toggle = state.output_ts;
                self.index = (self.index + 1) % self.outputs.len();
            }
        })
    }
}

impl ArcRwLock for DigitalOutputBlinkers {}
