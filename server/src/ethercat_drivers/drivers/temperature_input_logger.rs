use crate::ethercat_drivers::{
    actor::Actor, io::temperature_input::TemperatureInput, utils::traits::ArcRwLock,
};
use std::{future::Future, pin::Pin};

pub struct TemperatureInputLogger {
    input: TemperatureInput,
}

impl TemperatureInputLogger {
    pub fn new(input: TemperatureInput) -> Self {
        Self { input }
    }
}

impl Actor for TemperatureInputLogger {
    fn act(&mut self, _now_ts: u64) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let state = (self.input.state)().await;
            log::debug!("TemperatureInputLogger: {:?}", state);
        })
    }
}

impl ArcRwLock for TemperatureInputLogger {}
