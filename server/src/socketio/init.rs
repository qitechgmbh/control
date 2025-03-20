use super::room::room_id::RoomId;
use crate::app_state::APP_STATE;
use serde::{Deserialize, Serialize};
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
        let mut socketio_rooms_guard = APP_STATE.socketio_setup.rooms.write().await;
        let mut ethercat_setup_guard = APP_STATE.ethercat_setup.write().await;
        let ethercat_setup_guard = match ethercat_setup_guard.as_mut() {
            Some(ethercat_setup_guard) => ethercat_setup_guard,
            None => {
                log::error!(
                    "[{}::on_disconnect] Ethercat setup not found",
                    module_path!()
                );
                return;
            }
        };
        for room in socket.rooms() {
            socketio_rooms_guard
                .use_mut(
                    ethercat_setup_guard,
                    room.to_string().into(),
                    |room_interface| {
                        if let Ok(room_interface) = room_interface {
                            room_interface.leave(socket_clone.clone());
                        } else {
                            log::error!(
                                "[{}::on_disconnect] Room {} not found",
                                module_path!(),
                                room
                            );
                        }
                    },
                )
                .await;
        }
    });
}

pub fn on_room_join(socket: SocketRef, Data(data): Data<RoomJoinEvent>) {
    // log
    log::info!("Socket {} joined room {}", socket.id, data.room_id);

    // add socket to the room buffer
    let room_id = data.room_id.clone();
    let socket_clone = socket.clone();
    spawn(async move {
        let mut socketio_rooms_guard = APP_STATE.socketio_setup.rooms.write().await;
        let mut ethercat_setup_guard = APP_STATE.ethercat_setup.write().await;
        let ethercat_setup_guard = match ethercat_setup_guard.as_mut() {
            Some(ethercat_setup_guard) => ethercat_setup_guard,
            None => {
                log::error!(
                    "[{}::on_room_join] Ethercat setup not found",
                    module_path!()
                );
                return;
            }
        };
        socketio_rooms_guard
            .use_mut(ethercat_setup_guard, room_id.clone(), |room_interface| {
                if let Ok(room_interface) = room_interface {
                    room_interface.join(socket_clone);
                } else {
                    log::error!(
                        "[{}::on_room_join] Room {} not found",
                        module_path!(),
                        room_id
                    );
                }
            })
            .await;
    });

    // join the room
    socket.join(data.room_id);
}

pub fn on_room_leave(socket: SocketRef, Data(data): Data<RoomLeaveEvent>) {
    // log
    log::info!("Socket {} left room {}", socket.id, data.room_id);

    // remove socket from the room buffer
    let room_id = data.room_id.clone();
    let socket_clone = socket.clone();
    spawn(async move {
        let mut socketio_rooms_guard = APP_STATE.socketio_setup.rooms.write().await;
        let mut ethercat_setup_guard = APP_STATE.ethercat_setup.write().await;
        let ethercat_setup_guard = match ethercat_setup_guard.as_mut() {
            Some(ethercat_setup_guard) => ethercat_setup_guard,
            None => {
                log::error!(
                    "[{}::on_room_leave] Ethercat setup not found",
                    module_path!()
                );
                return;
            }
        };
        socketio_rooms_guard
            .use_mut(ethercat_setup_guard, room_id.clone(), |room_interface| {
                if let Ok(room_interface) = room_interface {
                    room_interface.leave(socket_clone);
                } else {
                    log::error!(
                        "[{}::on_room_leave] Room {} not found",
                        module_path!(),
                        room_id
                    );
                }
            })
            .await;
    });

    // leave the room
    socket.leave(data.room_id);
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomJoinEvent {
    pub room_id: RoomId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomLeaveEvent {
    pub room_id: RoomId,
}
