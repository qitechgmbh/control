use super::Actor;
use crate::io::digital_output::DigitalOutput;
use std::{future::Future, pin::Pin, sync::Arc};
use tokio::sync::RwLock;

/// Set a digital output high and low with a given interval
#[derive(Debug)]
pub struct DigitalOutputSetter {
    output: DigitalOutput,
    enabled: bool,
}

impl DigitalOutputSetter {
    pub fn new(output: DigitalOutput) -> Self {
        Self {
            output,
            enabled: true,
        }
    }
    pub fn set(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    pub fn get(&self) -> bool {
        self.enabled
    }
    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }
}

impl Actor for DigitalOutputSetter {
    fn act(&mut self, _now_ts: u64) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            {
                (self.output.write)(self.enabled.into()).await;
            }
        })
    }
}

impl From<DigitalOutputSetter> for Arc<RwLock<DigitalOutputSetter>> {
    fn from(actor: DigitalOutputSetter) -> Self {
        Arc::new(RwLock::new(actor))
    }
}
