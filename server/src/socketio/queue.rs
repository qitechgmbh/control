use crate::{
    app_state::AppState,
    panic::{PanicDetails, send_panic},
};
use smol::channel::Sender;
use std::{sync::Arc, time::Instant};
use tracing::{debug, error, info, instrument, trace};

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

pub fn init_socketio_queue(thread_panic_tx: Sender<PanicDetails>, app_state: Arc<AppState>) {
    std::thread::Builder::new()
        .name("socketio-queue".to_string())
        .spawn(move || {
            send_panic(thread_panic_tx);

            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_io()
                .enable_time()
                .build()
                .expect("Failed to create runtime");

            rt.block_on(async {
                info!("SocketIO global queue listener started");

                let mut event_count = 0;
                let mut batch_start = Instant::now();

                loop {
                    match app_state.socketio_setup.socket_queue_rx.recv().await {
                        Ok((socket, event)) => {
                            event_count += 1;

                            // Handle the received message with retry logic
                            send_event_with_retry(&socket, &event).await;

                            // Log batch statistics every 5 seconds
                            if batch_start.elapsed().as_secs() >= 5 {
                                let elapsed = batch_start.elapsed();
                                if event_count > 0 {
                                    debug!(
                                        "[{}::init_socketio_queue] Processed {} events in {:.2?} ({:.1} events/s)",
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
                        Err(e) => {
                            error!(error = %e, "Error receiving from global socketio queue");
                            info!("SocketIO global queue listener stopping");
                            break;
                        }
                    }
                }
            });
        })
        .expect("Failed to spawn socketio queue thread");
}
