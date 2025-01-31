use crate::ethercat_drivers::{
    actor::Actor, io::digital_input::DigitalInput, utils::traits::ArcRwLock,
};
use std::{future::Future, pin::Pin};

pub struct DigitalInputLogger {
    input: DigitalInput,
}

impl DigitalInputLogger {
    pub fn new(input: DigitalInput) -> Self {
        Self { input }
    }
}

impl Actor for DigitalInputLogger {
    fn act(&mut self, _now_ts: u64) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let state = (self.input.state)().await;
            log::debug!("DigitalInputLogger: {}", state.value);
        })
    }
}

impl ArcRwLock for DigitalInputLogger {}
