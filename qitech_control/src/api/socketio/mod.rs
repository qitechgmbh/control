// mod namespace;
mod event;
mod namespace;
pub use event::Event;
use socketioxide::layer::SocketIoLayer;

mod main;
mod namespace_id;
use std::str::FromStr;

use namespace_id::NamespaceId;
use socketioxide::{ParserConfig, SocketIoBuilder};
use socketioxide::extract::SocketRef;

pub fn init() -> SocketIoLayer {
    let (layer, io) = SocketIoBuilder::new()
        .max_buffer_size(1024)
        .with_parser(ParserConfig::msgpack())
        .build_layer();

    // --- init on connect handler for main namespace ---
    io.ns("/main", move |socket: SocketRef| async move {
        handle_socket_connection(socket, ());
    });

    layer
}

async fn handle_socket_connection(socket: SocketRef, app_state: ()) {
    let Ok(namespace_id) = NamespaceId::from_str(socket.ns()) else {
        return;
    };

    if let NamespaceId::Machine(_) = namespace_id {
        /*
        let map = &mut namespaces_guard.machine_namespaces;
        if !map.contains_key(&namespace_id_clone) {
            // Clone the sender from your main namespace
            // Now create the namespace
            let ns = Namespace::new(socket_queue_tx);
            map.insert(namespace_id_clone.clone(), ns);
        }
        */
    }

    panic!("Okay that works");
}


/*
fn setup_disconnection(
    socket: SocketRef,
    namespace_id: NamespaceId,
    app_state: Arc<SharedAppState>,
) {
    socket.on_disconnect(move |socket: SocketRef| async move {
        let namespace_id = namespace_id.clone();
        let app_state = app_state.clone();

        // write-lock to mutate namespaces
        let mut namespaces_guard = app_state.socketio_setup.namespaces.write().await;

        match namespaces_guard.apply_mut(namespace_id.clone()).await {
            Ok(namespace) => {
                namespace.unsubscribe(socket.clone());
            }
            Err(err) => {}
        }
    })
}
*/
