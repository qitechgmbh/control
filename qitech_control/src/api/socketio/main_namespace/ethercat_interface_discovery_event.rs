use control_core::socketio::event::Event;
use serde::{Deserialize, Serialize};

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
