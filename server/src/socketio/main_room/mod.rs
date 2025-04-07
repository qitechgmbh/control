use control_core::socketio::{
    event::{Event, GenericEvent},
    room::{cache_one_event, CacheFn, CacheableEvents, Room, RoomCacheingLogic, RoomInterface},
};
use ethercat_setup_event::EthercatSetupEvent;

pub mod ethercat_setup_event;

pub struct MainRoom(pub Room);

impl MainRoom {
    pub fn new() -> Self {
        Self(Room::new())
    }
}

impl RoomCacheingLogic<MainRoomEvents> for MainRoom
where
    MainRoomEvents: CacheableEvents<MainRoomEvents>,
{
    fn emit_cached(&mut self, events: MainRoomEvents) {
        let buffer_fn = events.event_cache_fn();
        let event = events.event_value();
        self.0.emit_cached(&event, buffer_fn);
    }
}

#[derive(Debug, Clone)]
pub enum MainRoomEvents {
    EthercatSetupEvent(Event<EthercatSetupEvent>),
}

impl CacheableEvents<MainRoomEvents> for MainRoomEvents {
    fn event_value(&self) -> GenericEvent {
        match self {
            MainRoomEvents::EthercatSetupEvent(event) => event.clone().into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        match self {
            MainRoomEvents::EthercatSetupEvent(_) => cache_one_event(),
        }
    }
}
