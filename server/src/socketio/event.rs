use super::events::ethercat_devices_event::EthercatDevicesEvent;
use crate::app_state::APP_STATE;
use serde::{Deserialize, Serialize};
use socketioxide_core::adapter::RoomParam;
use std::future::Future;

#[derive(Debug, Clone)]
pub enum EventType {
    EthercatDevicesEvent(Event<EthercatDevicesEvent>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventStatus {
    #[serde(rename = "no_data")]
    NoData, // Frontend only
    #[serde(rename = "success")]
    Success,
    #[serde(rename = "warning")]
    Warning,
    #[serde(rename = "error")]
    Error,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event<M: EventData> {
    #[serde(skip_serializing)]
    pub name: String,
    pub data: Option<M>,
    pub error: Option<String>,
    pub warning: Option<String>,
    pub status: EventStatus,
    pub ts: i64,
}

impl<M: EventData> Event<M> {
    /**
     * Normal emit. Either emits data or error.
     */
    pub async fn emit(&self, room: impl RoomParam + Clone) {
        let room = room.clone();
        // tokio::spawn(async move {
        let message_type = M::to_event_type(self.clone());

        // for every room
        let mut socketio_rooms_guard = APP_STATE.socketio_rooms.write().await;
        for single_room in room.clone().into_room_iter() {
            // buffer in room
            socketio_rooms_guard
                .room(single_room.to_string())
                .buffer(message_type.clone());

            // emit to sockets
            socketio_rooms_guard
                .room(single_room.to_string())
                .emit(self.name.clone(), &self);
        }
    }
}

impl<M: EventData> Event<M> {
    pub fn data(event: String, data: M) -> Self {
        Self {
            name: event,
            data: Some(data),
            error: None,
            warning: None,
            status: EventStatus::Success,
            ts: chrono::Utc::now().timestamp_millis(),
        }
    }

    pub fn error(event: String, error: String) -> Self {
        Self {
            name: event,
            data: None,
            error: Some(error),
            warning: None,
            status: EventStatus::Error,
            ts: chrono::Utc::now().timestamp_millis(),
        }
    }

    pub fn warning(event: String, warning: String) -> Self {
        Self {
            name: event,
            data: None,
            error: None,
            warning: Some(warning),
            status: EventStatus::Warning,
            ts: chrono::Utc::now().timestamp_millis(),
        }
    }
}

pub trait EventData: Sized + Send + Clone + Sync + Serialize {
    fn build() -> impl Future<Output = Event<Self>> + Send
    where
        Self: Send + Sync;
    fn build_warning(warning: String) -> Event<Self>;
    fn to_event_type(message: Event<Self>) -> EventType;
}
