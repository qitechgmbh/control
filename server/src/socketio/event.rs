use std::future::Future;

use serde::{Deserialize, Serialize};
use socketioxide_core::adapter::RoomParam;

use crate::app_state::APP_STATE;

use super::messages::ethercat_devices_event::EthercatDevicesEvent;

#[derive(Debug, Clone)]
pub enum EventType {
    EthercatDevicesEvent(Event<EthercatDevicesEvent>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event<M: EventData> {
    #[serde(skip_serializing)]
    pub name: String,
    pub data: Option<M>,
    pub errors: Option<String>,
    pub ts: i64,
}

impl<M: EventData> Event<M> {
    pub fn data(event: String, data: M) -> Self {
        Self {
            name: event,
            data: Some(data),
            errors: None,
            ts: chrono::Utc::now().timestamp(),
        }
    }

    pub fn error(event: String, error: String) -> Self {
        Self {
            name: event,
            data: None,
            errors: Some(error),
            ts: chrono::Utc::now().timestamp(),
        }
    }
}

pub trait EventData: Sized + Send + Clone + Sync + Serialize {
    fn build() -> impl Future<Output = Event<Self>> + Send
    where
        Self: Send + Sync;
    fn to_message_type(message: Event<Self>) -> EventType;

    fn emit(room: impl RoomParam + Clone) {
        let room = room.clone();
        tokio::spawn(async move {
            let message = Self::build().await;
            let message_type = Self::to_message_type(message.clone());

            // for every room
            let mut socketio_buffer_guard = APP_STATE.socketio_buffer.write().await;
            for single_room in room.clone().into_room_iter() {
                // buffer in room
                socketio_buffer_guard
                    .room(single_room.to_string())
                    .buffer(message_type.clone());

                // emit to sockets
                socketio_buffer_guard
                    .room(single_room.to_string())
                    .emit(message.name.clone(), &message);
            }
        });
    }
}
