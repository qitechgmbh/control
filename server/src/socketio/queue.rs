use crate::app_state::SharedState;
use std::{sync::Arc};
use tracing::{error, info, instrument, trace};

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
            info!(
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

pub async fn start_socketio_queue(app_state: Arc<SharedState>) {
    let app_state = app_state.as_ref();    
    loop {
        let res = app_state.socketio_setup.socket_queue_rx.recv().await;
        match res {
            Ok((socket,event)) => send_event_with_retry(&socket, &event).await,
            Err(e) => {                
                error!("SocketIO global queue listener stopped: {:?}", e);
                break;
            },
        }
    }
}
