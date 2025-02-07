use crate::app_state::APP_STATE;
use crate::socketio::room::{on_room_join, on_room_leave};
use socketioxide::extract::SocketRef;
use socketioxide::layer::SocketIoLayer;
use tokio::spawn;

pub async fn init_socketio() -> SocketIoLayer {
    // create
    let (socketio_layer, io) = socketioxide::SocketIo::new_layer();

    // set the on connect handler for main namespace
    io.ns("/", on_connect);

    // set the io to the app state
    let mut socketio_guard = APP_STATE.socketio.write().await;
    socketio_guard.replace(io);
    drop(socketio_guard);

    return socketio_layer;
}

fn on_connect(socket: SocketRef) {
    // log
    log::debug!("Socket connected {}", socket.id);

    // set the on disconnect handler
    socket.on_disconnect(on_disconnect);
    // if joined room
    socket.on("join", on_room_join);

    // if left room
    socket.on("leave", on_room_leave);
}

fn on_disconnect(socket: SocketRef) {
    // log
    log::debug!("Socket disconnected {}", socket.id);

    // remove from every room
    let socket_clone = socket.clone();
    spawn(async move {
        let mut socketio_rooms_guard = APP_STATE.socketio_rooms.write().await;
        for room in socket.rooms() {
            socketio_rooms_guard
                .room(room.to_string())
                .leave(socket_clone.clone());
        }
    });
}
