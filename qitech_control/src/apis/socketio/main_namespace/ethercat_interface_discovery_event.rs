use serde::{Deserialize, Serialize};
use control_core::socketio::event::Event;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum EthercatInterfaceDiscoveryEvent {
    Discovering(bool),
    Done(String),
}

impl EthercatInterfaceDiscoveryEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("EthercatInterfaceDiscoveryEvent", self.clone())
    }
}
