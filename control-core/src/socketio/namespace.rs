use crate::socketio::event::GenericEvent;
use socketioxide::{extract::SocketRef, socket::Sid};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tracing::instrument;

use super::socket_queue::SocketQueue;

#[derive(Debug)]
pub struct Namespace {
    pub sockets: Vec<SocketRef>,
    pub events: HashMap<String, Vec<Arc<GenericEvent>>>,
    pub socket_queues: HashMap<Sid, SocketQueue>,
}

impl Namespace {
    pub fn new() -> Self {
        Self {
            sockets: vec![],
            events: HashMap::new(),
            socket_queues: HashMap::new(),
        }
    }
}

impl Namespace {
    /// Adds a socket to the namespace.
    ///
    /// # Arguments
    ///
    /// * `socket` - A reference to the socket to be added
    #[instrument(skip_all)]
    pub fn subscribe(&mut self, socket: SocketRef) {
        // add the socket to the list
        self.sockets.push(socket.clone());
        // create a new queue for this socket
        self.socket_queues.insert(socket.id, SocketQueue::new());
    }

    /// Removes a socket from the namespace.
    ///
    /// # Arguments
    ///
    /// * `socket` - A reference to the socket to be removed
    #[instrument(skip_all)]
    pub fn unsubscribe(&mut self, socket: SocketRef) {
        // remove the socket from the list
        self.sockets.retain(|s| s.id != socket.id);
        // remove the socket's queue
        self.socket_queues.remove(&socket.id);
    }

    /// Re-emits cached events to a specific socket.
    ///
    /// This is typically used when a socket reconnects or joins an existing namespace
    /// to bring it up to date with the current state.
    ///
    /// # Arguments
    ///
    /// * `socket` - A reference to the socket that will receive the cached events
    #[instrument(skip_all)]
    pub fn reemit(&mut self, socket: SocketRef) {
        if let Some(queue) = self.socket_queues.get(&socket.id) {
            // Collect events grouped by name/kind with their counts for sorting
            let mut event_groups: Vec<(&String, &Vec<Arc<GenericEvent>>)> =
                self.events.iter().collect();

            // Sort by event count (ascending - lowest count first)
            event_groups.sort_by(|a, b| a.1.len().cmp(&b.1.len()));

            // Emit events in order of lowest count first
            for (_event_name, events) in event_groups {
                for event in events {
                    queue.push(event.clone());
                }
            }

            queue.flush(socket.clone());
        }
    }

    /// Emits an event to all sockets in the namespace.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to be emitted
    #[instrument(skip_all)]
    pub fn emit(&mut self, event: Arc<GenericEvent>) {
        // Use the new emit function which combines push and flush
        for socket in self.sockets.clone() {
            if let Some(queue) = self.socket_queues.get(&socket.id) {
                queue.push(event.clone());
                queue.flush(socket.clone());
            }
        }
    }

    /// Caches an event with a specific key for later retrieval.
    ///
    /// # Arguments
    ///
    /// * `cache_key` - A string key to identify the cache
    /// * `event` - The event to be cached
    /// * `buffer_fn` - A function that defines how the event should be added to the cache buffer
    #[instrument(skip_all)]
    pub fn cache(
        &mut self,
        event: Arc<GenericEvent>,
        buffer_fn: &Box<dyn Fn(&mut Vec<Arc<GenericEvent>>, &Arc<GenericEvent>) -> ()>,
    ) {
        let mut cached_events_for_key = self
            .events
            .entry(event.name.clone())
            .or_insert_with(Vec::new);
        buffer_fn(&mut cached_events_for_key, &event);
    }

    /// Emits an event to all sockets in the namespace and caches it.
    ///
    /// This is a convenience method that combines the functionality of
    /// [`Namespace::emit`] and [`Namespace::cache`].
    ///
    /// # Arguments
    ///
    /// * `event` - The event to be emitted and cached
    /// * `cache_key` - A string key to identify the cache
    /// * `buffer_fn` - A function that defines how the event should be added to the cache buffer
    #[instrument(skip_all)]
    pub fn emit_cached(
        &mut self,
        event: Arc<GenericEvent>,
        buffer_fn: &Box<dyn Fn(&mut Vec<Arc<GenericEvent>>, &Arc<GenericEvent>) -> ()>,
    ) {
        // cache the event
        self.cache(event.clone(), &buffer_fn);

        // emit the event
        self.emit(event);
    }
}

pub trait NamespaceBufferCacheKey {
    fn to_key(&self) -> String;
}

pub trait NamespaceCacheingLogic<V>
where
    V: CacheableEvents<V>,
{
    fn emit_cached(&mut self, event: V);
}

pub trait CacheableEvents<Events> {
    fn event_value(&self) -> Result<GenericEvent, serde_json::Error>;
    fn event_cache_fn(&self) -> CacheFn;
}

pub type CacheFn = Box<dyn Fn(&mut Vec<Arc<GenericEvent>>, &Arc<GenericEvent>) -> ()>;

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
pub fn cache_duration(duration: Duration, bucket_size: Duration) -> CacheFn {
    Box::new(move |events, event| {
        // Use event.ts instead of system time
        let current_time = event.ts as u128;

        // calculate current bucket & last bucket
        let bucket_size_ms = bucket_size.as_millis().max(1);
        let bucket = current_time / bucket_size_ms; // Avoid division by zero
        let last_bucket = match events.last() {
            Some(last_event) => last_event.ts as u128 / bucket_size_ms,
            None => 0,
        };

        // if the bucket is not larger we early exit
        if bucket <= last_bucket {
            return;
        }

        // Remove old events
        // Since events are ordered by increasing ts, we can find the first index
        // that should be kept and truncate everything before it
        let cutoff_time = current_time.saturating_sub(duration.as_millis());
        let cutoff_millis = cutoff_time as u64;
        if let Some(keep_idx) = events.iter().position(|evt| evt.ts >= cutoff_millis) {
            events.drain(0..keep_idx);
        } else if !events.is_empty() {
            // All events are too old
            events.clear();
        }

        // Add event
        events.push(event.clone());
    })
}

#[cfg(test)]
mod tests {
    use std::cmp::min;

    use super::*;

    #[test]
    fn test_cache_n_events() {
        // Create a cache function that stores the last 2 events
        let cache_fn = cache_n_events(2);

        // Create namespace
        let mut namespace = Namespace::new();
        assert!(namespace.events.is_empty());

        // Add event
        let event1 = Arc::new(GenericEvent {
            name: "test_event".to_string(),
            data: serde_json::json!({"value": 1}),
            ts: 0,
        });
        namespace.cache(event1, &cache_fn);

        // Check that we have one event name in the map
        assert_eq!(namespace.events.len(), 1);
        // Check that we have one event under that name
        assert_eq!(namespace.events.get("test_event").unwrap().len(), 1);
        assert_eq!(namespace.events.get("test_event").unwrap()[0].ts, 0);

        // Add another event
        let event2 = Arc::new(GenericEvent {
            name: "test_event".to_string(),
            data: serde_json::json!({"value": 2}),
            ts: 1,
        });
        namespace.cache(event2, &cache_fn);

        // Still one event name
        assert_eq!(namespace.events.len(), 1);
        // But now two events under that name
        assert_eq!(namespace.events.get("test_event").unwrap().len(), 2);
        assert_eq!(namespace.events.get("test_event").unwrap()[0].ts, 0);
        assert_eq!(namespace.events.get("test_event").unwrap()[1].ts, 1);

        // Add a third event, which should remove the first one
        let event3 = Arc::new(GenericEvent {
            name: "test_event".to_string(),
            data: serde_json::json!({"value": 3}),
            ts: 2,
        });
        namespace.cache(event3, &cache_fn);

        // Still one event name
        assert_eq!(namespace.events.len(), 1);
        // Still two events under that name (because of the limit)
        assert_eq!(namespace.events.get("test_event").unwrap().len(), 2);
        // But now the events should be the second and third ones
        assert_eq!(namespace.events.get("test_event").unwrap()[0].ts, 1);
        assert_eq!(namespace.events.get("test_event").unwrap()[1].ts, 2);
    }

    #[test]
    /// duration: 10 seconds, bucket_size: 1 second
    /// use a for loop that tries to add an event every 100ms
    fn test_cache_duration() {
        // Create a cache function that stores events for 10 seconds
        let duration = Duration::new(10, 0);
        let bucket_size = Duration::new(1, 0);
        let cache_fn = cache_duration(duration, bucket_size);

        // Create namespace
        let mut namespace = Namespace::new();
        assert!(namespace.events.is_empty());

        // Add events every 100ms for 20 seconds
        for i in 0..200 {
            let event = Arc::new(GenericEvent {
                name: "test_event".to_string(),
                data: serde_json::json!({"value": i}),
                ts: (i * 100) as u64,
            });
            namespace.cache(event, &cache_fn);

            let should_have_events = min(i / 10, 11);
            assert_eq!(
                namespace.events.get("test_event").unwrap().len(),
                should_have_events
            );
        }
    }
}
