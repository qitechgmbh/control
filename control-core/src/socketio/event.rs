use std::time::{SystemTime, UNIX_EPOCH};

use erased_serde::Serialize as ErasedSerialize;
use serde::Serialize;

#[derive(Serialize)]
pub struct GenericEvent {
    pub name: String,
    pub data: Box<dyn ErasedSerialize + Send + Sync>,
    /// Timestamp in milliseconds
    pub ts: u64,
}

impl std::fmt::Debug for GenericEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GenericEvent")
            .field("name", &self.name)
            .field("data", &"[erased]")
            .field("ts", &self.ts)
            .finish()
    }
}

pub trait BuildEvent: Serialize + Send + Sync + Clone {
    fn build(&self) -> Event<Self>;
}

#[derive(Debug, Clone, Serialize)]
pub struct Event<T>
where
    T: Serialize + Send + Sync + 'static,
{
    pub name: String,
    pub data: T,
    /// Timestamp in milliseconds
    pub ts: u64,
}

impl<T> From<Event<T>> for GenericEvent
where
    T: Serialize + Send + Sync + 'static,
{
    fn from(event: Event<T>) -> Self {
        Self {
            name: event.name,
            data: Box::new(event.data),
            ts: event.ts,
        }
    }
}

impl<T> From<&Event<T>> for GenericEvent
where
    T: Serialize + Clone + Send + Sync + 'static,
{
    fn from(event: &Event<T>) -> Self {
        Self {
            name: event.name.clone(),
            data: Box::new(event.data.clone()),
            ts: event.ts,
        }
    }
}

impl<T> Event<T>
where
    T: Serialize + Clone + Send + Sync + 'static,
{
    pub fn new(event: &str, data: T) -> Self {
        Self {
            name: event.to_string(),
            data,
            ts: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }
}
