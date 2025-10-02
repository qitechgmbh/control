use std::sync::Arc;

use control_core::socketio::{
    event::{Event, GenericEvent},
    namespace::{CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_one_event},
};
use ethercat_devices_event::EthercatDevicesEvent;
use ethercat_interface_discovery_event::EthercatInterfaceDiscoveryEvent;
use machines_event::MachinesEvent;
use smol::channel::Sender;
use socketioxide::extract::SocketRef;
use tracing::instrument;

pub mod ethercat_devices_event;
pub mod ethercat_interface_discovery_event;
pub mod machines_event;

pub struct MainRoom {
    pub namespace: Namespace,
}

impl MainRoom {
    pub fn new(socket_queue_tx: Sender<(SocketRef, Arc<GenericEvent>)>) -> Self {
        Self {
            namespace: Namespace::new(socket_queue_tx),
        }
    }
}

impl NamespaceCacheingLogic<MainNamespaceEvents> for MainRoom
where
    MainNamespaceEvents: CacheableEvents<MainNamespaceEvents>,
{
    #[instrument(skip_all)]
    fn emit(&mut self, event: MainNamespaceEvents) {
        let buffer_fn = event.event_cache_fn();
        let generic_event = Arc::new(event.event_value());
        self.namespace.emit(generic_event, &buffer_fn);
    }
}

#[derive(Clone)]
pub enum MainNamespaceEvents {
    MachinesEvent(Event<MachinesEvent>),
    EthercatDevicesEvent(Event<EthercatDevicesEvent>),
    EthercatInterfaceDiscoveryEvent(Event<EthercatInterfaceDiscoveryEvent>),
}

impl CacheableEvents<Self> for MainNamespaceEvents {
    fn event_value(&self) -> GenericEvent {
        match self {
            Self::EthercatDevicesEvent(event) => event.into(),
            Self::EthercatInterfaceDiscoveryEvent(event) => event.into(),
            Self::MachinesEvent(event) => event.into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        match self {
            Self::EthercatDevicesEvent(_) => cache_one_event(),
            Self::EthercatInterfaceDiscoveryEvent(_) => cache_one_event(),
            Self::MachinesEvent(_) => cache_one_event(),
        }
    }
}
