use std::str::FromStr;

use crate::app_state::APP_STATE;
use control_core::socketio::room_id::RoomId;
use socketioxide::extract::SocketRef;
use socketioxide::layer::SocketIoLayer;
use tokio::spawn;

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
    let room_id = match RoomId::from_str(socket.ns()) {
        Ok(room_id) => room_id,
        Err(err) => {
            log::error!(
                "[{}::setup_namespace] Failed to parse room id: {}",
                module_path!(),
                err
            );
            return;
        }
    };

    // Set up disconnect handler
    setup_disconnection(&socket, room_id.clone());

    // Set up connection
    spawn(async move {
        setup_connection(socket, room_id).await;
    });
}

fn setup_disconnection(socket: &SocketRef, room_id: RoomId) {
    socket.on_disconnect(|socket: SocketRef| {
        spawn(async move {
            log::debug!("Socket disconnected {}", socket.id);
            let mut socketio_rooms_guard = APP_STATE.socketio_setup.rooms.write().await;

            // remove from machine room
            socketio_rooms_guard
                .apply_mut(room_id.clone(), |room_interface| match room_interface {
                    Ok(room_interface) => {
                        room_interface.unsubscribe(socket.clone());
                    }
                    Err(err) => {
                        log::error!(
                            "[{}::on_disconnect_machine_ns] Room {:?} not found: {}",
                            module_path!(),
                            room_id,
                            err
                        );
                    }
                })
                .await;
        });
    });
}

async fn setup_connection(socket: SocketRef, room_id: RoomId) {
    log::info!("Socket connected {}", socket.id);
    let mut socketio_rooms_guard = APP_STATE.socketio_setup.rooms.write().await;
    socketio_rooms_guard
        .apply_mut(room_id.clone(), |room_interface| match room_interface {
            Ok(room_interface) => {
                room_interface.subscribe(socket.clone());
                room_interface.reemit(socket);
            }
            Err(err) => {
                log::error!(
                    "[{}::on_connect_machine_ns] Room {:?} not found: {}",
                    module_path!(),
                    room_id,
                    err
                );
            }
        })
        .await;
}
