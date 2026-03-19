use std::time::Duration;

use control_core::socketio::{
    event::{BuildEvent, GenericEvent},
    namespace::{CacheFn, CacheableEvents, cache_duration},
};
use control_core_derive::BuildEvent;
use serde::{Deserialize, Serialize};
use stahlwerk_extension::ff01::Entry;

#[derive(Serialize, Debug, Clone, Default, BuildEvent)]
pub struct LiveValues {
    pub weight_peak: Option<f64>,
    pub weight_prev: Option<f64>,
}

impl CacheableEvents<Self> for LiveValues {
    fn event_value(&self) -> GenericEvent {
        self.build().into()
    }

    fn event_cache_fn(&self) -> CacheFn {
        cache_duration(Duration::from_secs(60 * 60), Duration::from_secs(1))
    }
}

#[derive(Debug, Clone, BuildEvent, Serialize)]
pub struct State {
    pub is_default_state: bool,
    pub plates_counted: u32,
    pub current_entry: Option<Entry>,
}

impl CacheableEvents<Self> for State {
    fn event_value(&self) -> GenericEvent {
        self.build().into()
    }

    fn event_cache_fn(&self) -> CacheFn {
        cache_duration(Duration::from_secs(60 * 60), Duration::from_secs(1))
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum Mutation {
    Tare,
    ClearTare,
    ClearLights,
}