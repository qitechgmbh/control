use crate::socketio::namespace::NamespaceInterface;
use serde_json::Value;

pub trait MachineApi {
    fn api_mutate(&mut self, value: Value) -> Result<(), anyhow::Error>;
    fn api_event_namespace(&mut self) -> &mut dyn NamespaceInterface;
}
