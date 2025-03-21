use super::Actor;
use crate::io::temperature_input::TemperatureInput;
use std::{future::Future, pin::Pin, sync::Arc};
use tokio::sync::RwLock;

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
    fn act(&mut self, _now_ts: u64) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let state = (self.input.state)().await;
            log::debug!("TemperatureInputLogger: {:?}C", state.input.temperature);
        })
    }
}

impl From<TemperatureInputLogger> for Arc<RwLock<TemperatureInputLogger>> {
    fn from(actor: TemperatureInputLogger) -> Self {
        Arc::new(RwLock::new(actor))
    }
}
