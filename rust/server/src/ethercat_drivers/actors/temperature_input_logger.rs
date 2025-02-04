use crate::ethercat_drivers::{
    actor::Actor, io::temperature_input::TemperatureInput, utils::traits::ArcRwLock,
};

/// Log the state of a temperature input
pub struct TemperatureInputLogger {
    input: TemperatureInput,
}

impl TemperatureInputLogger {
    pub fn new(input: TemperatureInput) -> Self {
        Self { input }
    }
}

impl Actor for TemperatureInputLogger {
    fn act(&mut self, _now_ts: u64) {
        let state = (self.input.state)();
        log::debug!("TemperatureInputLogger: {:?}C", state.value);
    }
}

impl ArcRwLock for TemperatureInputLogger {}
