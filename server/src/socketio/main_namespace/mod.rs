use control_core::socketio::{
    event::{Event, GenericEvent},
    namespace::{
        cache_one_event, CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic,
        NamespaceInterface,
    },
};
use ethercat_interface_discovery_event::EthercatInterfaceDiscoveryEvent;
use ethercat_setup_event::EthercatSetupEvent;

pub mod ethercat_interface_discovery_event;
pub mod ethercat_setup_event;

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
    fn emit_cached(&mut self, event: MainNamespaceEvents) {
        println!("Emitting event: {:?}", event);
        let buffer_fn = event.event_cache_fn();
        let generic_event = match event.event_value() {
            Ok(event) => event,
            Err(err) => {
                log::error!(
                    "[{}::emit_cached] Failed to event.event_value(): {:?}",
                    module_path!(),
                    err
                );
                return;
            }
        };
        self.0.emit_cached(&generic_event, buffer_fn);
    }
}

#[derive(Debug, Clone)]
pub enum MainNamespaceEvents {
    EthercatSetupEvent(Event<EthercatSetupEvent>),
    EthercatInterfaceDiscoveryEvent(Event<EthercatInterfaceDiscoveryEvent>),
}

impl CacheableEvents<MainNamespaceEvents> for MainNamespaceEvents {
    fn event_value(&self) -> Result<GenericEvent, serde_json::Error> {
        match self {
            MainNamespaceEvents::EthercatSetupEvent(event) => event.clone().try_into(),
            MainNamespaceEvents::EthercatInterfaceDiscoveryEvent(event) => event.clone().try_into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        match self {
            MainNamespaceEvents::EthercatSetupEvent(_) => cache_one_event(),
            MainNamespaceEvents::EthercatInterfaceDiscoveryEvent(_) => cache_one_event(),
        }
    }
}
