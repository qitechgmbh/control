use serde_json::Value;
use smol::lock::Mutex;
use std::sync::Arc;

use crate::socketio::namespace::Namespace;

pub trait MachineApi {
    fn api_mutate(&mut self, value: Value) -> Result<(), anyhow::Error>;
    fn api_event_namespace(&mut self) -> Arc<Mutex<Namespace>>;
}
