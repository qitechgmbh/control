use std::{
    collections::VecDeque,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};

use socketioxide::extract::SocketRef;

use super::event::GenericEvent;

/// A queue for managing events for a specific socket
#[derive(Debug)]
pub struct SocketQueue {
    pub queue: Arc<Mutex<VecDeque<Arc<GenericEvent>>>>,
    pub is_flushing: Arc<AtomicBool>,
}

impl SocketQueue {
    /// Create a new socket queue for the given socket ID
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            is_flushing: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Add an event to the queue
    pub fn push(&self, event: Arc<GenericEvent>) {
        if let Ok(mut queue) = self.queue.lock() {
            queue.push_back(event);
        }
    }

    /// Send a single event with retry logic
    async fn send_event(socket: &SocketRef, event: Arc<GenericEvent>) {
        // retry loop for each event
        loop {
            // check if socket is still connected
            if !socket.connected() {
                break; // Exit the loop if the socket is not connected
            }

            match socket.emit("event", event.as_ref()) {
                Ok(_) => break, // Successfully emitted, exit loop
                Err(e) => match e {
                    socketioxide::SendError::Serialize(_) => {
                        // no reason in retrying serialization errors
                        break;
                    }
                    socketioxide::SendError::Socket(socket_error) => match socket_error {
                        socketioxide::SocketError::InternalChannelFull => {
                            continue; // Retry sending the event
                        }
                        socketioxide::SocketError::Closed => {
                            // Socket is closed, no point in retrying
                            break;
                        }
                    },
                },
            }
        }
    }

    /// Force flush events asynchronously (assumes flushing flag is already set)
    pub fn flush(&self, socket: SocketRef) {
        // Check if we're already flushing
        if self
            .is_flushing
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            // Already flushing, return early
            return;
        }

        let queue = Arc::clone(&self.queue);
        let is_flushing = Arc::clone(&self.is_flushing);

        // Spawn a smol task to flush the queue
        smol::spawn(async move {
            // Process events directly from the queue without collecting first
            let mut event_cnt = 0;
            let start_t = Instant::now();
            loop {
                let event = {
                    if let Ok(mut queue_guard) = queue.lock() {
                        queue_guard.pop_front()
                    } else {
                        break; // Exit if we can't lock the queue
                    }
                };

                let Some(event) = event else {
                    break; // No more events in queue
                };

                Self::send_event(&socket, event).await;
                event_cnt += 1;
            }
            let elapsed = start_t.elapsed();

            if event_cnt > 10 {
                log::info!(
                    "Flushed {} events for socket {} in {:?}ms",
                    event_cnt,
                    socket.id,
                    elapsed.as_millis()
                );
            }

            // Reset the flushing flag
            is_flushing.store(false, Ordering::SeqCst);
        })
        .detach();
    }
}
