use super::Actor;
use crate::io::digital_input::DigitalInput;
use std::{future::Future, pin::Pin, sync::Arc};
use tokio::sync::RwLock;
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
    fn act(&mut self, _now_ts: u64) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let state = (self.input.state)().await;
            log::debug!("DigitalInputLogger: {}", state.input.value);
        })
    }
}

impl From<DigitalInputLogger> for Arc<RwLock<DigitalInputLogger>> {
    fn from(actor: DigitalInputLogger) -> Self {
        Arc::new(RwLock::new(actor))
    }
}
