use crate::{MachineApi, MachineMessage};

use super::new::SensorMachine;

impl MachineApi for SensorMachine {
    fn api_get_sender(&self) -> smol::channel::Sender<MachineMessage> {
        self.api_sender.clone()
    }

    fn api_mutate(&mut self, _value: serde_json::Value) -> anyhow::Result<()> {
        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<control_core::socketio::namespace::Namespace> {
        None
    }
}
