use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
pub struct GenericEvent {
    pub name: String,
    pub data: Value,
    /// Timestamp in milliseconds
    pub ts: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Event<T>
where
    T: Serialize,
{
    pub name: String,
    pub data: T,
    /// Timestamp in milliseconds
    pub ts: i64,
}

impl<T> TryFrom<Event<T>> for GenericEvent
where
    T: Serialize,
{
    type Error = serde_json::Error;

    fn try_from(event: Event<T>) -> Result<Self, Self::Error> {
        Ok(Self {
            name: event.name,
            data: serde_json::to_value(event.data)?,
            ts: event.ts,
        })
    }
}

impl<T> TryFrom<&Event<T>> for GenericEvent
where
    T: Serialize,
{
    type Error = serde_json::Error;

    fn try_from(event: &Event<T>) -> Result<Self, Self::Error> {
        Ok(Self {
            name: event.name.clone(),
            data: serde_json::to_value(&event.data)?,
            ts: event.ts,
        })
    }
}

impl<T> Event<T>
where
    T: Serialize + Clone,
{
    pub fn new(event: &str, data: T) -> Self {
        Self {
            name: event.to_string(),
            data,
            ts: chrono::Utc::now().timestamp_millis(),
        }
    }
}
