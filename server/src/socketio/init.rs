use crate::app_state::APP_STATE;
use control_core::socketio::room::{RoomSubscribeEvent, RoomUnsubscribeEvent};
use socketioxide::extract::{Data, SocketRef};
use socketioxide::layer::SocketIoLayer;
use tokio::spawn;

pub async fn init_socketio() -> SocketIoLayer {
    // create
    let (socketio_layer, io) = socketioxide::SocketIo::new_layer();

    // set the on connect handler for main namespace
    io.ns("/", on_connect);

    // set the io to the app state
    let mut socketio_guard = APP_STATE.socketio_setup.socketio.write().await;
    socketio_guard.replace(io);
    drop(socketio_guard);

    return socketio_layer;
}

fn on_connect(socket: SocketRef) {
    // log
    log::info!("Socket connected {}", socket.id);

    // set the on disconnect handler
    socket.on_disconnect(on_disconnect);

    // if joined room
    socket.on("subscribe", on_room_subscribe);

    // if left room
    socket.on("unsibscribe", on_room_unsubscribe);
}

fn on_disconnect(socket: SocketRef) {
    // log
    log::debug!("Socket disconnected {}", socket.id);

    // remove from every room
    let socket_clone = socket.clone();

    // iterate all rooms
    spawn(async move {
        let mut socketio_rooms_guard = APP_STATE.socketio_setup.rooms.write().await;
        socketio_rooms_guard
            .for_each_mut(|_, room_interface| {
                if let Ok(room_interface) = room_interface {
                    room_interface.unsubscribe(socket_clone.clone());
                }
            })
            .await;
    });
}

pub fn on_room_subscribe(socket: SocketRef, Data(data): Data<RoomSubscribeEvent>) {
    // log
    log::info!("Socket {} subscribed room {:?}", socket.id, data.room_id);

    // add socket to the room buffer
    let room_id = data.room_id.clone();
    let socket_clone = socket.clone();
    spawn(async move {
        let mut socketio_rooms_guard = APP_STATE.socketio_setup.rooms.write().await;
        socketio_rooms_guard
            .apply_mut(room_id.clone(), |room_interface| {
                if let Ok(room_interface) = room_interface {
                    room_interface.subscribe(socket_clone.clone());
                    room_interface.reemit(socket_clone);
                } else {
                    log::error!(
                        "[{}::on_room_subscribe] Room {:?} not found",
                        module_path!(),
                        room_id
                    );
                }
            })
            .await;
    });
}

pub fn on_room_unsubscribe(socket: SocketRef, Data(data): Data<RoomUnsubscribeEvent>) {
    // log
    log::info!("Socket {} unsubscribed room {:?}", socket.id, data.room_id);

    // remove socket from the room buffer
    let room_id = data.room_id.clone();
    let socket_clone = socket.clone();
    spawn(async move {
        let mut socketio_rooms_guard = APP_STATE.socketio_setup.rooms.write().await;
        socketio_rooms_guard
            .apply_mut(room_id.clone(), |room_interface| {
                if let Ok(room_interface) = room_interface {
                    room_interface.unsubscribe(socket_clone);
                } else {
                    log::error!(
                        "[{}::on_room_unsubscribe] Room {:?} not found",
                        module_path!(),
                        room_id
                    );
                }
            })
            .await;
    });
}
