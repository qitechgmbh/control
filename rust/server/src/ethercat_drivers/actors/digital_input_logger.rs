use crate::ethercat_drivers::{
    actor::Actor, io::digital_input::DigitalInput, utils::traits::ArcRwLock,
};
/// Log the state of a digital input
pub struct DigitalInputLogger {
    input: DigitalInput,
}

impl DigitalInputLogger {
    pub fn new(input: DigitalInput) -> Self {
        Self { input }
    }
}

impl Actor for DigitalInputLogger {
    fn act(&mut self, _now_ts: u64) {
        let state = (self.input.state)();
        log::debug!("DigitalInputLogger: {}", state.value);
    }
}

impl ArcRwLock for DigitalInputLogger {}
