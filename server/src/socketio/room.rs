use super::{
    event::{Event, EventType},
    messages::ethercat_devices_event::EthercatDevicesEvent,
};
use crate::app_state::APP_STATE;
use serde::{Deserialize, Serialize};
use socketioxide::extract::{Data, SocketRef};
use std::collections::HashMap;
use tokio::spawn;

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomJoinEvent {
    pub room: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomLeaveEvent {
    pub room: String,
}

pub fn on_room_join(socket: SocketRef, Data(data): Data<RoomJoinEvent>) {
    // log
    log::debug!("Socket {} joined room {}", socket.id, data.room);

    // add socket to the room buffer
    let room = data.room.clone();
    let socket_clone = socket.clone();
    spawn(async move {
        let mut socketio_rooms_guard = APP_STATE.socketio_rooms.write().await;
        socketio_rooms_guard.room(room).join(socket_clone);
    });

    // join the room
    socket.join(data.room);
}

pub fn on_room_leave(socket: SocketRef, Data(data): Data<RoomLeaveEvent>) {
    // log
    log::debug!("Socket {} left room {}", socket.id, data.room);

    // remove socket from the room buffer
    let room = data.room.clone();
    let socket_clone = socket.clone();
    spawn(async move {
        let mut socketio_rooms_guard = APP_STATE.socketio_rooms.write().await;
        socketio_rooms_guard.room(room).leave(socket_clone);
    });

    // leave the room
    socket.leave(data.room);
}

pub struct Rooms {
    room_map: HashMap<String, Room>,
}

impl Rooms {
    pub fn new() -> Self {
        Self {
            room_map: HashMap::new(),
        }
    }

    pub fn room(&mut self, room: String) -> &mut Room {
        // get the buffer or create a new one
        let room = self
            .room_map
            .entry(room.clone())
            .or_insert_with(|| Room::new(room));

        room
    }

    // give an iter of all rooms
    pub fn rooms(&self) -> impl Iterator<Item = &Room> {
        self.room_map.values()
    }
}

pub struct Room {
    room: String,
    sockets: Vec<SocketRef>,
    last_ethercat_device_event: Option<Event<EthercatDevicesEvent>>,
}

impl Room {
    pub fn new(room: String) -> Self {
        Self {
            room,
            sockets: Vec::new(),
            last_ethercat_device_event: None,
        }
    }

    pub fn join(&mut self, socket: SocketRef) {
        // add the socket to the list
        self.sockets.push(socket.clone());
        self.reemit(socket);
    }

    pub fn leave(&mut self, socket: SocketRef) {
        // remove the socket from the list
        self.sockets.retain(|s| s.id != socket.id);
    }

    pub fn buffer(&mut self, event: EventType) {
        // log
        log::debug!("Remembering event for room {}", self.room);

        // remember the event
        match event {
            EventType::EthercatDevicesEvent(devices_message) => {
                self.last_ethercat_device_event = Some(devices_message);
            }
        }
    }

    fn reemit(&mut self, socket: SocketRef) {
        // log
        log::debug!("Reemitting event for room {}", self.room);

        // reemit the event
        if let Some(event) = &self.last_ethercat_device_event {
            socket.emit(event.name.clone(), &event);
        }
    }

    pub fn emit<T: ?Sized + Serialize>(&self, name: String, event: &T) {
        // log
        log::debug!("Emitting event for room {}", self.room);

        // emit the event
        for socket in self.sockets.iter() {
            let _ = socket.emit(name.clone(), &event);
        }
    }
}
