use crate::socketio::event::GenericEvent;
use smol::channel::Sender;
use socketioxide::extract::SocketRef;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct Namespace {
    pub sockets: Vec<SocketRef>,
    pub events: HashMap<String, Vec<Arc<GenericEvent>>>,
    pub socket_queue_tx: Sender<(SocketRef, Arc<GenericEvent>)>,
}

impl Namespace {
    pub fn new(socket_queue_tx: Sender<(SocketRef, Arc<GenericEvent>)>) -> Self {
        Self {
            sockets: vec![],
            events: HashMap::new(),
            socket_queue_tx,
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
        self.sockets.push(socket);
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
    }

    /// Disconnects all clients in the namespace.
    ///
    /// This will forcefully disconnect all sockets and clear the socket list.
    #[instrument(skip_all)]
    pub fn disconnect_all(&mut self) {
        tracing::info!("Disconnecting {} sockets in namespace", self.sockets.len());

        // Disconnect each socket
        for socket in &self.sockets {
            let _ = socket.clone().disconnect(); // Ignore errors if socket is already disconnected
        }

        // Clear the socket list
        self.sockets.clear();
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
        // Collect events grouped by name/kind with their counts for sorting
        let mut event_groups: Vec<(&String, &Vec<Arc<GenericEvent>>)> =
            self.events.iter().collect();

        // Sort by event count (ascending - lowest count first)
        event_groups.sort_by(|a, b| a.1.len().cmp(&b.1.len()));

        // Emit events in order of lowest count first
        for (_event_name, events) in event_groups {
            for event in events {
                // Send to global queue instead of per-socket queue
                self.send_to_queue(&socket, event, "reemit");
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
    fn cache(
        &mut self,
        event: Arc<GenericEvent>,
        buffer_fn: &Box<dyn Fn(&mut Vec<Arc<GenericEvent>>, &Arc<GenericEvent>)>,
    ) {
        let cached_events_for_key = self.events.entry(event.name.clone()).or_default();
        buffer_fn(cached_events_for_key, &event);
    }

    /// Emits an event to all sockets in the namespace and caches it.
    ///
    ///
    /// # Arguments
    ///
    /// * `event` - The event to be emitted and cached
    /// * `buffer_fn` - A function that defines how the event should be added to the cache buffer
    #[instrument(skip_all)]
    pub fn emit(
        &mut self,
        event: Arc<GenericEvent>,
        buffer_fn: &Box<dyn Fn(&mut Vec<Arc<GenericEvent>>, &Arc<GenericEvent>)>,
    ) {
        // cache the event
        self.cache(event.clone(), buffer_fn);
        // emit the event - inlined from emit function
        // Send to global queue for each socket in the namespace
        for socket in self.sockets.clone() {
            self.send_to_queue(&socket, &event, "emit");
        }
    }

    /// Sends an event to the global queue for a specific socket.
    ///
    /// # Arguments
    ///
    /// * `socket` - The socket to send the event to
    /// * `event` - The event to be sent
    /// * `operation` - A description of the operation for logging purposes (e.g., "reemit", "emit")
    #[instrument(skip_all)]
    fn send_to_queue(&self, socket: &SocketRef, event: &Arc<GenericEvent>, operation: &str) {
        tracing::trace!(
            socket_id = ?socket.id,
            event = %event.name,
            operation = %operation,
            "Sending event to global queue"
        );
        match self
            .socket_queue_tx
            .try_send((socket.clone(), event.clone()))
        {
            Ok(_) => {
                tracing::trace!(
                    socket_id = ?socket.id,
                    event = %event.name,
                    operation = %operation,
                    "Successfully sent event to global queue"
                );
            }
            Err(e) => {
                tracing::error!(
                    socket_id = ?socket.id,
                    event = %event.name,
                    operation = %operation,
                    error = %e,
                    "Failed to send event to global queue"
                );
            }
        }
    }
}

impl Drop for Namespace {
    fn drop(&mut self) {
        //self.disconnect_all();
    }
}

pub trait NamespaceBufferCacheKey {
    fn to_key(&self) -> String;
}

pub trait NamespaceCacheingLogic<V>
where
    V: CacheableEvents<V>,
{
    fn emit(&mut self, event: V);
}

pub trait CacheableEvents<Events> {
    fn event_value(&self) -> GenericEvent;
    fn event_cache_fn(&self) -> CacheFn;
}

pub type CacheFn = Box<dyn Fn(&mut Vec<Arc<GenericEvent>>, &Arc<GenericEvent>)>;

/// [`BufferFn`] that stores the last n events
pub fn cache_n_events(n: usize) -> CacheFn {
    Box::new(move |events, event| {
        if events.len() >= n {
            events.remove(0);
        }
        events.push(event.clone());
    })
}

/// [`BufferFn`] that stores only one event
pub fn cache_one_event() -> CacheFn {
    cache_n_events(1)
}

/// [`BufferFn`] that stores first and last event
///
/// The primary use case of this function is to cache both the default state of a machine, which should be emitted first,
/// and the last event, which is the most recent state of the machine.
pub fn cache_first_and_last_event() -> CacheFn {
    Box::new(move |events, event| {
        // if the events length 0 or 1, we just push the event
        if events.is_empty() || events.len() == 1 {
            events.push(event.clone());
            return;
        }
        // if the event length is 2 we remove the last event and append a new one
        if events.len() == 2 {
            events.remove(1);
            events.push(event.clone());
        }
    })
}

/// [`BufferFn`] that stores events for a certain duration
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

    use serde::Serialize;

    use super::*;

    #[derive(Debug, Clone, Serialize)]
    struct TestEventData {
        value: usize,
    }

    #[test]
    fn test_cache_n_events() {
        // Create a cache function that stores the last 2 events
        let cache_fn = cache_n_events(2);

        // Create namespace with dummy queue_tx
        let (queue_tx, _queue_rx) = smol::channel::unbounded();
        let mut namespace = Namespace::new(queue_tx);
        assert!(namespace.events.is_empty());

        // Add event
        let event1 = Arc::new(GenericEvent {
            name: "test_event".to_string(),
            data: Box::new(TestEventData { value: 1 }),
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
            data: Box::new(TestEventData { value: 2 }),
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
            data: Box::new(TestEventData { value: 3 }),
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
        let (queue_tx, _queue_rx) = smol::channel::unbounded();

        // Create namespace
        let mut namespace = Namespace::new(queue_tx);
        assert!(namespace.events.is_empty());

        // Add events every 100ms for 20 seconds
        for i in 0..200 {
            let event = Arc::new(GenericEvent {
                name: "test_event".to_string(),
                data: Box::new(TestEventData { value: i }),
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
