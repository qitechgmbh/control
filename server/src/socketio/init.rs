use std::str::FromStr;
use std::sync::Arc;

use crate::app_state::AppState;
use control_core::socketio::namespace_id::NamespaceId;
use socketioxide::extract::SocketRef;
use socketioxide::layer::SocketIoLayer;
use tracing::info_span;
use tracing_futures::Instrument;

pub async fn init_socketio(app_state: &Arc<AppState>) -> SocketIoLayer {
    // create
    let (socketio_layer, io) = socketioxide::SocketIoBuilder::new()
        .max_buffer_size(1024)
        .build_layer();

    // Clone app_state for the first handler
    let app_state_main = app_state.clone();

    // set the on connect handler for main namespace
    io.ns("/main", move |socket: SocketRef| {
        handle_socket_connection(socket, app_state_main.clone());
    });

    // Clone app_state for the second handler
    let app_state_machine = app_state.clone();

    if let Err(err) = io.dyn_ns(
        "/machine/{vendor}/{machine}/{serial}",
        move |socket: SocketRef| {
            handle_socket_connection(socket, app_state_machine.clone());
        },
    ) {
        tracing::error!("Failed to detect machine namespace: {}", err);
    }

    // set the io to the app state
    let mut socketio_guard = app_state.socketio_setup.socketio.write().await;
    socketio_guard.replace(io);

    return socketio_layer;
}

fn handle_socket_connection(socket: SocketRef, app_state: Arc<AppState>) {
    let namespace_id = match NamespaceId::from_str(socket.ns()) {
        Ok(namespace_id) => namespace_id,
        Err(err) => {
            tracing::error!("Failed to parse NamespaceId: {}", err);
            return;
        }
    };

    // Setup disconnection handler
    setup_disconnection(socket.clone(), namespace_id.clone(), app_state.clone());

    // Setup connection
    setup_connection(socket, namespace_id, app_state);
}

fn setup_disconnection(socket: SocketRef, namespace_id: NamespaceId, app_state: Arc<AppState>) {
    socket.on_disconnect(move |socket: SocketRef| {
        let namespace_id = namespace_id.clone();
        let app_state = app_state.clone();

        // Spawn async task to avoid blocking and potential deadlocks
        smol::spawn(async move {
            tracing::debug!(
                socket = ?socket.id,
                namespace = %namespace_id,
                "Socket disconnected from namespace",
            );
            let mut socketio_namespaces_guard = app_state.socketio_setup.namespaces.write().await;

            // remove from machine namespace
            socketio_namespaces_guard
                .apply_mut(namespace_id.clone(), &app_state, |namespace_interface| {
                    if let Ok(namespace_interface) = namespace_interface {
                        namespace_interface.unsubscribe(socket.clone());
                    }
                })
                .await;
        })
        .detach();
    });
}

fn setup_connection(socket: SocketRef, namespace_id: NamespaceId, app_state: Arc<AppState>) {
    // Spawn async task to avoid blocking and potential deadlocks
    let socket_clone = socket.clone();
    let namespace_id_clone = namespace_id.clone();
    let app_state_clone = app_state.clone();

    let span = info_span!(
        "socketio_connection",
        socket = ?socket_clone.id,
        namespace = %namespace_id_clone,
        "Connecting socket to namespace"
    );

    smol::spawn(
        async move {
            let mut socketio_namespaces_guard =
                app_state_clone.socketio_setup.namespaces.write().await;
            socketio_namespaces_guard
                .apply_mut(
                    namespace_id_clone.clone(),
                    &app_state_clone,
                    |namespace_interface| {
                        match namespace_interface {
                            Ok(namespace_interface) => {
                                // First subscribe the socket
                                namespace_interface.subscribe(socket_clone.clone());

                                // Then re-emit cached events
                                namespace_interface.reemit(socket_clone.clone());
                            }
                            Err(err) => {
                                tracing::warn!(
                                    socket = ?socket_clone.id,
                                    namespace = %namespace_id_clone,
                                    "Couln't subscribe socket to namespace, disconnecting: {:?}",
                                    err
                                );
                                let _ = socket_clone.disconnect();
                            }
                        }
                    },
                )
                .await;
        }
        .instrument(span),
    )
    .detach();

    tracing::info!(
        socket = ?socket.id,
        namespace = %namespace_id,
        "Socket connected to namespace",
    );
}
