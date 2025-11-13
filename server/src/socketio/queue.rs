use crate::app_state::SharedState;
use std::{sync::Arc, time::Instant};
use tracing::{debug, info, instrument, trace};

/// Send a single event with retry logic
#[instrument(skip_all)]
async fn send_event_with_retry(
    socket: &socketioxide::extract::SocketRef,
    event: &Arc<control_core::socketio::event::GenericEvent>,
) {
    // retry loop for each event
    loop {
        // check if socket is still connected
        if !socket.connected() {
            trace!(
                socket_id = ?socket.id,
                event = %event.name,
                "Socket disconnected, skipping event"
            );
            break; // Exit the loop if the socket is not connected
        }

        match socket.emit("event", event.as_ref()) {
            Ok(_) => {
                trace!(
                    socket_id = ?socket.id,
                    event = %event.name,
                    timestamp = event.ts,
                    "Successfully emitted event"
                );
                break; // Successfully emitted, exit loop
            }
            Err(e) => match e {
                socketioxide::SendError::Serialize(serialize_error) => {
                    trace!(
                        socket_id = ?socket.id,
                        event = %event.name,
                        error = %serialize_error,
                        "Serialization error, skipping event"
                    );
                    break; // no reason in retrying serialization errors
                }
                socketioxide::SendError::Socket(socket_error) => match socket_error {
                    socketioxide::SocketError::InternalChannelFull => {
                        continue; // Retry sending the event
                    }
                    socketioxide::SocketError::Closed => {
                        trace!(
                            socket_id = ?socket.id,
                            event = %event.name,
                            "Socket closed, skipping event"
                        );
                        break; // Socket is closed, no point in retrying
                    }
                },
            },
        }
    }
}

pub async fn socketio_queue_worker(app_state: &SharedState) {
    tracing::info!("SocketIO global queue listener started");
    let mut event_count = 0;
    let mut batch_start = Instant::now();

    while let Ok((socket, event)) = app_state.socketio_setup.socket_queue_rx.recv().await {
        event_count += 1;

        send_event_with_retry(&socket, &event).await;

        if batch_start.elapsed().as_secs() >= 5 {
            let elapsed = batch_start.elapsed();
            if event_count > 0 {
                debug!(
                    "[{}::socketio_queue_worker] Processed {} events in {:.2?} ({:.1} events/s)",
                    module_path!(),
                    event_count,
                    elapsed,
                    event_count as f64 / elapsed.as_secs_f64(),
                );
            }
            event_count = 0;
            batch_start = Instant::now();
        }
    }

    info!("SocketIO global queue listener stopped");
}

pub async fn start_socketio_queue(app_state: Arc<SharedState>) {
    let app_state = app_state.as_ref();
    loop {
        let res = socketio_queue_worker(app_state).await;
        tracing::error!("SocketIO task finished, but should never finish: {:?}", res);
        tracing::error!("Restarting SocketIO...");
    }
}
