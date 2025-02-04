use crate::ethercat_drivers::{
    actor::Actor, io::digital_output::DigitalOutput, utils::traits::ArcRwLock,
};
use std::time::Duration;

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
    fn act(&mut self, _now_ts: u64) {
        let toggle_duration = self.interval.as_nanos() as u64;
        let state = (self.output.state)();
        if state.output_ts - self.last_toggle > toggle_duration {
            match state.value {
                true => (self.output.write)(false),
                false => (self.output.write)(true),
            }
            self.last_toggle = state.output_ts;
        }
    }
}

impl ArcRwLock for DigitalOutputBlinker {}
