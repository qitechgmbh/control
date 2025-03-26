use super::room_id::RoomId;
use crate::socketio::event::GenericEvent;
use serde::{Deserialize, Serialize};
use socketioxide::extract::SocketRef;
use std::{collections::HashMap, time::Duration};

pub trait RoomInterface {
    /// Adds a socket to the room.
    ///
    /// # Arguments
    ///
    /// * `socket` - A reference to the socket to be added
    fn subscribe(&mut self, socket: SocketRef);

    /// Removes a socket from the room.
    ///
    /// # Arguments
    ///
    /// * `socket` - A reference to the socket to be removed
    fn unsubscribe(&mut self, socket: SocketRef);

    /// Re-emits cached events to a specific socket.
    ///
    /// This is typically used when a socket reconnects or joins an existing room
    /// to bring it up to date with the current state.
    ///
    /// # Arguments
    ///
    /// * `socket` - A reference to the socket that will receive the cached events
    fn reemit(&mut self, socket: SocketRef);

    /// Emits an event to all sockets in the room.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to be emitted
    fn emit(&mut self, event: &GenericEvent);

    /// Caches an event with a specific key for later retrieval.
    ///
    /// # Arguments
    ///
    /// * `cache_key` - A string key to identify the cache
    /// * `event` - The event to be cached
    /// * `buffer_fn` - A function that defines how the event should be added to the cache buffer
    fn cache(
        &mut self,
        event: &GenericEvent,
        buffer_fn: Box<dyn Fn(&mut Vec<GenericEvent>, &GenericEvent) -> ()>,
    );

    /// Emits an event to all sockets in the room and caches it.
    ///
    /// This is a convenience method that combines the functionality of
    /// [`Room::emit`] and [`Room::cache`].
    ///
    /// # Arguments
    ///
    /// * `event` - The event to be emitted and cached
    /// * `cache_key` - A string key to identify the cache
    /// * `buffer_fn` - A function that defines how the event should be added to the cache buffer
    fn emit_cached(
        &mut self,
        event: &GenericEvent,
        buffer_fn: Box<dyn Fn(&mut Vec<GenericEvent>, &GenericEvent) -> ()>,
    );

    fn room_id(&self) -> &RoomId;
}

#[derive(Debug)]
pub struct Room {
    pub room_id: RoomId,
    sockets: Vec<SocketRef>,
    events: HashMap<String, Vec<GenericEvent>>,
}

impl Room {
    pub fn new(room_id: RoomId) -> Self {
        Self {
            room_id,
            sockets: vec![],
            events: HashMap::new(),
        }
    }
}

impl RoomInterface for Room {
    fn room_id(&self) -> &RoomId {
        &self.room_id
    }

    fn subscribe(&mut self, socket: SocketRef) {
        // add the socket to the list
        self.sockets.push(socket.clone());
    }

    fn unsubscribe(&mut self, socket: SocketRef) {
        // remove the socket from the list
        self.sockets.retain(|s| s.id != socket.id);
    }

    fn reemit(&mut self, socket: SocketRef) {
        for (_, events) in self.events.iter() {
            for event in events.iter() {
                let _ = socket.emit("event", &event.include_room_id(&self.room_id));
            }
        }
    }

    fn emit(&mut self, event: &GenericEvent) {
        for socket in self.sockets.iter() {
            let _ = socket.emit("event", &event.include_room_id(&self.room_id));
        }
    }

    fn cache(
        &mut self,
        event: &GenericEvent,
        buffer_fn: Box<dyn Fn(&mut Vec<GenericEvent>, &GenericEvent) -> ()>,
    ) {
        let mut cached_events_for_key = self
            .events
            .entry(event.name.clone())
            .or_insert_with(Vec::new);
        buffer_fn(&mut cached_events_for_key, event);
    }

    fn emit_cached(
        &mut self,
        event: &GenericEvent,
        buffer_fn: Box<dyn Fn(&mut Vec<GenericEvent>, &GenericEvent) -> ()>,
    ) {
        // cache the event
        self.cache(event, buffer_fn);

        // emit the event
        self.emit(event);
    }
}

pub trait RoomBufferCacheKey {
    fn to_key(&self) -> String;
}

pub trait RoomCacheingLogic<V>
where
    V: CacheableEvents<V>,
{
    fn emit_cached(&mut self, event: V);
}

pub trait CacheableEvents<Events> {
    fn event_value(&self) -> GenericEvent;
    fn event_cache_fn(&self) -> CacheFn;
}

pub type CacheFn = Box<dyn Fn(&mut Vec<GenericEvent>, &GenericEvent) -> ()>;

/// [BufferFn] that stores the last n events
pub fn cache_n_events(n: usize) -> CacheFn {
    Box::new(move |events, event| {
        if events.len() >= n {
            events.remove(0);
        }
        events.push(event.clone());
    })
}

/// [BufferFn] that stores only one event
pub fn cache_one_event() -> CacheFn {
    cache_n_events(1)
}

/// [BufferFn] that stores events for a certain duration
pub fn cache_duration(duration: Duration) -> CacheFn {
    Box::new(move |events, event| {
        let now = chrono::Utc::now();
        let cutoff_time = now - chrono::Duration::from_std(duration).unwrap_or_default();
        let cutoff_millis = cutoff_time.timestamp_millis();

        // Since events are ordered by increasing ts, we can find the first index
        // that should be kept and truncate everything before it
        if let Some(keep_idx) = events.iter().position(|evt| evt.ts >= cutoff_millis) {
            events.drain(0..keep_idx);
        } else if !events.is_empty() {
            // All events are too old
            events.clear();
        }

        events.push(event.clone());
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomSubscribeEvent {
    pub room_id: RoomId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomUnsubscribeEvent {
    pub room_id: RoomId,
}
