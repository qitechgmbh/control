use std::str::FromStr;

use crate::app_state::APP_STATE;
use control_core::socketio::namespace_id::NamespaceId;
use socketioxide::extract::SocketRef;
use socketioxide::layer::SocketIoLayer;

pub async fn init_socketio() -> SocketIoLayer {
    // create
    let (socketio_layer, io) = socketioxide::SocketIo::new_layer();

    // set the on connect handler for main namespace
    io.ns("/main", setup_namespace);

    match io.dyn_ns("/machine/{vendor}/{machine}/{serial}", setup_namespace) {
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
    let mut socketio_guard = APP_STATE.socketio_setup.socketio.write().await;
    socketio_guard.replace(io);

    return socketio_layer;
}

fn setup_namespace(socket: SocketRef) {
    let namespace_id = match NamespaceId::from_str(socket.ns()) {
        Ok(namespace_id) => namespace_id,
        Err(err) => {
            log::error!(
                "[{}::setup_namespace] Failed to parse namespace id: {}",
                module_path!(),
                err
            );
            return;
        }
    };

    // Set up disconnect handler
    setup_disconnection(&socket, namespace_id.clone());

    // Set up connection
    smol::block_on(setup_connection(socket, namespace_id));
}

fn setup_disconnection(socket: &SocketRef, namepsace_id: NamespaceId) {
    socket.on_disconnect(|socket: SocketRef| {
        smol::block_on(async move {
            log::debug!("Socket disconnected {}", socket.id);
            let mut socketio_namespaces_guard = APP_STATE.socketio_setup.namespaces.write().await;

            // remove from machine namespace
            socketio_namespaces_guard
                .apply_mut(
                    namepsace_id.clone(),
                    |namespace_interface| match namespace_interface {
                        Ok(namespace_interface) => {
                            namespace_interface.unsubscribe(socket.clone());
                        }
                        Err(err) => {
                            log::error!(
                                "[{}::on_disconnect_machine_ns] Namespace {:?} not found: {}",
                                module_path!(),
                                namepsace_id,
                                err
                            );
                        }
                    },
                )
                .await;
        });
    });
}

async fn setup_connection(socket: SocketRef, namespace_id: NamespaceId) {
    log::info!("Socket connected {}", socket.id);
    let mut socketio_namespaces_guard = APP_STATE.socketio_setup.namespaces.write().await;
    socketio_namespaces_guard
        .apply_mut(
            namespace_id.clone(),
            |namespace_interface| match namespace_interface {
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
            },
        )
        .await;
}
