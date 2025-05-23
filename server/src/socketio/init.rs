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
                "[{}::handle_socket_connection] Failed to parse namespace id: {}",
                module_path!(),
                err
            );
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
            log::debug!("Socket disconnected {}", socket.id);
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
    log::info!("Socket connected {}", socket.id);
    smol::block_on(async {
        let mut socketio_namespaces_guard = app_state.socketio_setup.namespaces.write().await;
        socketio_namespaces_guard
            .apply_mut(namespace_id.clone(), &app_state, |namespace_interface| {
                match namespace_interface {
                    Ok(namespace_interface) => {
                        namespace_interface.subscribe(socket.clone());
                        namespace_interface.reemit(socket);
                    }
                    Err(err) => {
                        log::error!(
                            "[{}::on_connect_machine_ns] Namespace {:?} not found: {}",
                            module_path!(),
                            namespace_id,
                            err
                        );
                    }
                }
            })
            .await;
    });
}
