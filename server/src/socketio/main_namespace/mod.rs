use control_core::socketio::{
    event::{Event, GenericEvent},
    namespace::{CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_one_event},
};
use ethercat_devices_event::EthercatDevicesEvent;
use ethercat_interface_discovery_event::EthercatInterfaceDiscoveryEvent;
use machines_event::MachinesEvent;
use tracing::instrument;

pub mod ethercat_devices_event;
pub mod ethercat_interface_discovery_event;
pub mod machines_event;

pub struct MainRoom(pub Namespace);

impl MainRoom {
    pub fn new() -> Self {
        Self(Namespace::new())
    }
}

impl NamespaceCacheingLogic<MainNamespaceEvents> for MainRoom
where
    MainNamespaceEvents: CacheableEvents<MainNamespaceEvents>,
{
    #[instrument(skip_all)]
    fn emit_cached(&mut self, event: MainNamespaceEvents) {
        let buffer_fn = event.event_cache_fn();
        let generic_event = match event.event_value() {
            Ok(event) => event,
            Err(err) => {
                tracing::error!("Failed to event.event_value(): {:?}", err);
                return;
            }
        };
        self.0.emit_cached(&generic_event, &buffer_fn);
    }
}

#[derive(Clone)]
pub enum MainNamespaceEvents {
    MachinesEvent(Event<MachinesEvent>),
    EthercatDevicesEvent(Event<EthercatDevicesEvent>),
    EthercatInterfaceDiscoveryEvent(Event<EthercatInterfaceDiscoveryEvent>),
}

impl CacheableEvents<MainNamespaceEvents> for MainNamespaceEvents {
    fn event_value(&self) -> Result<GenericEvent, serde_json::Error> {
        match self {
            MainNamespaceEvents::EthercatDevicesEvent(event) => event.clone().try_into(),
            MainNamespaceEvents::EthercatInterfaceDiscoveryEvent(event) => event.clone().try_into(),
            MainNamespaceEvents::MachinesEvent(event) => event.clone().try_into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        match self {
            MainNamespaceEvents::EthercatDevicesEvent(_) => cache_one_event(),
            MainNamespaceEvents::EthercatInterfaceDiscoveryEvent(_) => cache_one_event(),
            MainNamespaceEvents::MachinesEvent(_) => cache_one_event(),
        }
    }
}
