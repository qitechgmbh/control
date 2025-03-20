pub mod ethercat_setup_event;

use super::{
    room::{cache_one_event, CacheFn, CacheableEvents, Room, RoomCacheingLogic, RoomInterface},
    room_id::RoomId,
};
use crate::socketio::event::{Event, GenericEvent};
use ethercat_setup_event::EthercatSetupEvent;

pub struct MainRoom(pub Room);

impl MainRoom {
    pub fn new() -> Self {
        Self(Room::new(RoomId::Main))
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

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
enum MainRoomEventCacheKeys {
    EthercatSetupEvent,
}

impl From<MainRoomEventCacheKeys> for String {
    fn from(cache_key: MainRoomEventCacheKeys) -> Self {
        match cache_key {
            MainRoomEventCacheKeys::EthercatSetupEvent => "EthercatSetupEvent".to_string(),
        }
    }
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
