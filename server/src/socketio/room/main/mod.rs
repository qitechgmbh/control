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
    MainRoomEvents: CacheableEvents,
{
    fn emit_cached(&mut self, events: MainRoomEvents) {
        let event = events.event_value();
        let cache_key = events.event_cache_key();
        let buffer_fn = events.event_cache_fn(&cache_key);
        self.0.emit_cached(&event, &cache_key, buffer_fn);
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

impl CacheableEvents for MainRoomEvents {
    fn event_cache_key(&self) -> String {
        match self {
            MainRoomEvents::EthercatSetupEvent(_) => {
                MainRoomEventCacheKeys::EthercatSetupEvent.into()
            }
        }
    }

    fn event_value(&self) -> GenericEvent {
        match self {
            MainRoomEvents::EthercatSetupEvent(event) => event.clone().into(),
        }
    }

    fn event_cache_fn(&self, cache_key: &str) -> CacheFn {
        match cache_key {
            "EthercatSetupEvent" => cache_one_event(),
            _ => unreachable!("[{}] Unknown cache key: {}", module_path!(), cache_key),
        }
    }
}
