use super::namespace_id::NamespaceId;
use crate::SharedAppState;
use control_core::socketio::namespace::Namespace;
//use crate::apis::socketio::namespaces::Namespace;
use machine_implementations::MachineMessage;
use socketioxide::ParserConfig;
use socketioxide::extract::SocketRef;
use socketioxide::layer::SocketIoLayer;
use std::str::FromStr;
use std::sync::Arc;

pub async fn init_socketio(app_state: Arc<SharedAppState>) -> SocketIoLayer {
    // create
    let (socketio_layer, io) = socketioxide::SocketIoBuilder::new()
        .max_buffer_size(1024)
        .with_parser(ParserConfig::msgpack())
        .build_layer();

    // Clone app_state for the first handler
    let app_state_main = app_state.clone();

    // set the on connect handler for main namespace
    io.ns("/main", move |socket: SocketRef| async move {
        handle_socket_connection(socket, app_state_main.clone());
    });

    // Clone app_state for the second handler
    let app_state_machine = app_state.clone();

    if let Err(err) = io.dyn_ns(
        "/machine/{vendor}/{machine}/{serial}",
        move |socket: SocketRef| async move {
            handle_socket_connection(socket, app_state_machine.clone());
        },
    ) {
        tracing::error!("Failed to detect machine namespace: {}", err);
    }

    // set the io to the app state
    let mut socketio_guard = app_state.socketio_setup.socketio.write().await;
    socketio_guard.replace(io);

    socketio_layer
}

fn handle_socket_connection(socket: SocketRef, app_state: Arc<SharedAppState>) {
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

fn setup_disconnection(socket: SocketRef, namespace_id: NamespaceId, app_state: Arc<SharedAppState>) {
    socket.on_disconnect(move |socket: SocketRef| async move {
        let namespace_id = namespace_id.clone();
        let app_state = app_state.clone();

        // Spawn async task to avoid blocking
        tokio::spawn(async move {
            tracing::info!(
                "Socket disconnected from namespace socket={:?} namespace={}",
                socket.id,
                namespace_id,
            );

            // write-lock to mutate namespaces
            let mut namespaces_guard = app_state.socketio_setup.namespaces.write().await;

            match namespaces_guard.apply_mut(namespace_id.clone()).await {
                Ok(namespace) => {
                    namespace.unsubscribe(socket.clone());
                    tracing::info!(
                        "Socket unsubscribed from namespace socket={:?} namespace={}",
                        socket.id,
                        namespace_id
                    );
                }
                Err(err) => {
                    tracing::info!(
                        "Failed to unsubscribe socket from namespace socket={:?} namespace={} err={:?}",
                        socket.id,
                        namespace_id,
                        err
                    );
                }
            }
            if let NamespaceId::Machine(ident) = namespace_id.clone() {
                    match app_state.machines_with_channel.get(&ident) {
                        Some(sender) => {
                            let _ = sender.send(MachineMessage::UnsubscribeNamespace).await;
                        },
                        None => tracing::info!("sender doesnt exist for: {}",ident),
                    };
                }else{
                }
        });
    })
}

fn setup_connection(socket: SocketRef, namespace_id: NamespaceId, app_state: Arc<SharedAppState>) {
    let socket_clone = socket.clone();
    let namespace_id_clone = namespace_id.clone();
    let app_state_clone = app_state.clone();

    tokio::spawn(async move {
        let guard = app_state_clone.socketio_setup.namespaces.read().await;
        let socket_queue_tx =  guard.main_namespace.namespace.socket_queue_tx.clone();
        drop(guard);

        let mut namespaces_guard = app_state_clone.socketio_setup.namespaces.write().await;
        // Ensure machine namespace exists before applying
        if let NamespaceId::Machine(_) = namespace_id_clone {
            let map = &mut namespaces_guard.machine_namespaces;
            if !map.contains_key(&namespace_id_clone) {
                tracing::info!(
                    "Registering new machine namespace: {}",
                    namespace_id_clone
                );
                // Clone the sender from your main namespace
                // Now create the namespace
                let ns = Namespace::new(socket_queue_tx);
                map.insert(namespace_id_clone.clone(), ns);
            }
        }

        // Apply and subscribe the socket
        match  namespaces_guard
            .apply_mut(namespace_id_clone.clone())
            .await
        {
            Ok(namespace) => {
                namespace.subscribe(socket_clone.clone());
                namespace.reemit(socket_clone);

                if let NamespaceId::Machine(ident) = namespace_id_clone {
                    match app_state.clone().machines_with_channel.get(&ident) {
                        Some(sender) => {
                            tracing::info!("subscribing namespace to {}",ident);
                            let _ = sender.send(MachineMessage::SubscribeNamespace(namespace.clone())).await;
                        },
                        None => tracing::info!("sender doesnt exist for: {}",ident),
                    };
                }

            }
            Err(err) => {
                    tracing::warn!(
                        "Couldn't subscribe socket to namespace, disconnecting socket={:?} namespace={} error={:?}",
                        socket_clone.id,
                        namespace_id_clone,
                        err
                    );
                    let _ = socket_clone.disconnect();
                }
            }
        }
    );

    tracing::info!(
        "Socket connected to namespace socket={:?} namespace={}",
        socket.id,
        namespace_id,
    );
}
