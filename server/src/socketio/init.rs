use std::str::FromStr;
use std::sync::Arc;

use crate::app_state::AppState;
use control_core::socketio::namespace_id::NamespaceId;
use socketioxide::extract::SocketRef;
use socketioxide::layer::SocketIoLayer;

pub async fn init_socketio(app_state: &Arc<AppState>) -> SocketIoLayer {
    // create
    let (socketio_layer, io) = socketioxide::SocketIo::new_layer();

    // Clone app_state for the first handler
    let app_state_main = app_state.clone();

    // set the on connect handler for main namespace
    io.ns("/main", move |socket: SocketRef| {
        handle_socket_connection(socket, app_state_main.clone());
    });

    // Clone app_state for the second handler
    let app_state_machine = app_state.clone();

    match io.dyn_ns(
        "/machine/{vendor}/{machine}/{serial}",
        move |socket: SocketRef| {
            handle_socket_connection(socket, app_state_machine.clone());
        },
    ) {
        Ok(_) => {
            // log
            log::info!("Machine namespace created");
        }
        Err(err) => {
            // log
            log::error!(
                "[{}::init_socketio] Failed to parse machine namespace: {}",
                module_path!(),
                err
            );
        }
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
            log::error!(
                "[{}::handle_socket_connection] Failed to parse namespace id '{}': {}, disconnecting socket {}",
                module_path!(),
                socket.ns(),
                err,
                socket.id
            );
            socket.disconnect().ok(); // Disconnect invalid namespace connections immediately
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

        smol::block_on(async move {
            log::debug!(
                "Socket disconnected {} from namespace {}",
                socket.id,
                namespace_id
            );
            let mut socketio_namespaces_guard = app_state.socketio_setup.namespaces.write().await;

            // remove from machine namespace
            socketio_namespaces_guard
                .apply_mut(namespace_id.clone(), &app_state, |namespace_interface| {
                    match namespace_interface {
                        Ok(namespace_interface) => {
                            namespace_interface.unsubscribe(socket.clone());
                        }
                        Err(err) => {
                            log::error!(
                                "[{}::on_disconnect_machine_ns] Namespace {:?} not found: {}",
                                module_path!(),
                                namespace_id,
                                err
                            );
                        }
                    }
                })
                .await;
        });
    });
}

fn setup_connection(socket: SocketRef, namespace_id: NamespaceId, app_state: Arc<AppState>) {
    log::info!(
        "Socket connected {} to namespace {}",
        socket.id,
        namespace_id
    );
    smol::block_on(async {
        let mut socketio_namespaces_guard = app_state.socketio_setup.namespaces.write().await;
        
        // Try to apply to existing namespace first
        let mut namespace_found = false;
        socketio_namespaces_guard
            .apply_mut(namespace_id.clone(), &app_state, |namespace_interface| {
                match namespace_interface {
                    Ok(namespace_interface) => {
                        // Successfully found and subscribed to existing namespace
                        namespace_interface.subscribe(socket.clone());
                        namespace_interface.reemit(socket.clone());
                        namespace_found = true;
                    }
                    Err(err) => {
                        // Namespace doesn't exist yet - this is expected during startup
                        // We'll handle this case below by queueing the connection
                        log::debug!(
                            "[{}::setup_connection] Namespace {:?} not found: {}",
                            module_path!(),
                            namespace_id,
                            err
                        );
                    }
                }
            })
            .await;
        
        // Handle cases where namespace was not found
        if !namespace_found {
            match namespace_id {
                NamespaceId::Machine(machine_identification_unique) => {
                    // Machine namespace not ready yet, add to pending connections queue
                    // This handles the timing issue where clients reconnect during server startup
                    // before machines are initialized from EtherCAT setup
                    log::info!(
                        "Machine namespace not ready yet, adding socket {} to pending connections for machine {:?}",
                        socket.id,
                        machine_identification_unique
                    );
                    socketio_namespaces_guard.add_pending_machine_connection(machine_identification_unique, socket);
                }
                NamespaceId::Main => {
                    // Main namespace should always exist, if it doesn't something is wrong
                    // Disconnect the socket to prevent resource leaks
                    log::error!(
                        "Main namespace not found for socket {}, disconnecting",
                        socket.id
                    );
                    socket.disconnect().ok(); // Ignore errors during disconnect
                }
            }
        }
    });
}
