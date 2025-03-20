use super::Actor;
use crate::io::analog_input::AnalogInput;
use std::{future::Future, pin::Pin, sync::Arc};
use tokio::sync::RwLock;

/// Log the state of a analog input
#[derive(Debug)]
pub struct AnalogInputLogger {
    input: AnalogInput,
}

impl AnalogInputLogger {
    pub fn new(input: AnalogInput) -> Self {
        Self { input }
    }
}

impl Actor for AnalogInputLogger {
    fn act(&mut self, _now_ts: u64) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let state = (self.input.state)().await;
            log::info!(
                "AnalogInputLogger: {} (normalized) {} (absolute)",
                state.input.normalized,
                state.input.absolute
            );
        })
    }
}

impl From<AnalogInputLogger> for Arc<RwLock<AnalogInputLogger>> {
    fn from(actor: AnalogInputLogger) -> Self {
        Arc::new(RwLock::new(actor))
    }
}
