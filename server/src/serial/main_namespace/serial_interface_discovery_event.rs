use control_core::socketio::event::Event;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SerialInterfaceDiscoveryEvent {
    Discovering(bool),
    Done(String),
}

impl SerialInterfaceDiscoveryEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("SerialInterfaceDiscoveryEvent", self.clone())
    }
}
