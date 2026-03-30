use crate::SharedAppState;
use control_core::socketio::event::GenericEvent;
use std::sync::Arc;
use tracing::{error, info, instrument, trace};

#[instrument(skip_all)]
async fn send_event_with_retry(
    socket: &socketioxide::extract::SocketRef,
    event: &Arc<GenericEvent>,
) {
    loop {
        if !socket.connected() {
            info!(
                socket_id = ?socket.id,
                event = %event.name,
                "Socket disconnected, skipping event"
            );
            break;
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
                    info!(
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
                        info!(
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

pub async fn start_socketio_queue(app_state: Arc<SharedAppState>) {
    let app_state = app_state.as_ref();
    loop {
        let queue = &mut app_state.socketio_setup.socket_queue_rx.write().await;
        let res = queue.recv().await;
        match res {
            Some((socket, event)) => send_event_with_retry(&socket, &event).await,
            None => {
                error!("SocketIO global queue listener stopped");
                break;
            }
        }
    }
}
