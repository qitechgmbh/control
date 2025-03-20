use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::room::room_id::RoomId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventContentType<T> {
    Data(T),
    Error(String),
    Warning(String),
}

#[derive(Debug, Clone, Serialize)]
pub struct GenericEvent {
    pub name: String,
    pub content: EventContentType<Value>,
    /// Timestamp in milliseconds
    pub ts: i64,
}

impl GenericEvent {
    pub fn include_room_id(&self, room_id: &RoomId) -> GenericRoomEvent {
        GenericRoomEvent {
            room_id: room_id.clone(),
            event: self.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GenericRoomEvent {
    pub room_id: RoomId,
    #[serde(flatten)]
    pub event: GenericEvent,
}

#[derive(Debug, Clone, Serialize)]
pub struct Event<Data>
where
    Data: Serialize,
{
    pub name: String,
    pub content: EventContentType<Data>,
    /// Timestamp in milliseconds
    pub ts: i64,
}

impl<Data> From<Event<Data>> for GenericEvent
where
    Data: Serialize,
{
    fn from(event: Event<Data>) -> Self {
        Self {
            name: event.name,
            content: match event.content {
                EventContentType::Data(data) => {
                    EventContentType::Data(serde_json::to_value(data).unwrap())
                }
                EventContentType::Error(error) => EventContentType::Error(error),
                EventContentType::Warning(warning) => EventContentType::Warning(warning),
            },
            ts: event.ts,
        }
    }
}

impl<Data> From<&Event<Data>> for GenericEvent
where
    Data: Serialize,
{
    fn from(event: &Event<Data>) -> Self {
        Self {
            name: event.name.clone(),
            content: match &event.content {
                EventContentType::Data(data) => {
                    EventContentType::Data(serde_json::to_value(data).unwrap())
                }
                EventContentType::Error(error) => EventContentType::Error(error.clone()),
                EventContentType::Warning(warning) => EventContentType::Warning(warning.clone()),
            },
            ts: event.ts,
        }
    }
}

impl<Data> Event<Data>
where
    Data: Serialize + Clone,
{
    pub fn data(event: &str, data: Data) -> Self {
        Self {
            name: event.to_string(),
            content: EventContentType::Data(data),
            ts: chrono::Utc::now().timestamp_millis(),
        }
    }

    pub fn error(event: &str, error: &str) -> Self {
        Self {
            name: event.to_string(),
            content: EventContentType::Error(error.to_string()),
            ts: chrono::Utc::now().timestamp_millis(),
        }
    }

    pub fn warning(event: &str, warning: &str) -> Self {
        Self {
            name: event.to_string(),
            content: EventContentType::Warning(warning.to_string()),
            ts: chrono::Utc::now().timestamp_millis(),
        }
    }
}

pub trait EventBuilder<Data>
where
    Data: Serialize + Clone,
{
    fn name(&self) -> String;
    fn build(&self) -> Event<Data>;
    fn warning(&self, warning: &str) -> Event<Data> {
        Event::warning(&self.name(), warning)
    }
    fn error(&self, error: &str) -> Event<Data> {
        Event::error(&self.name(), error)
    }
}
