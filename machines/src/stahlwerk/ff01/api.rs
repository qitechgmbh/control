use std::time::Duration;

use control_core::socketio::{
    event::{BuildEvent, GenericEvent},
    namespace::{CacheFn, CacheableEvents, cache_duration},
};
use control_core_derive::BuildEvent;
use serde::{Deserialize, Serialize};
use stahlwerk_extension::ff01::Entry;

use crate::stahlwerk::ff01::FF01;

#[derive(Serialize, Debug, Clone, Default, BuildEvent)]
pub struct LiveValues {
    pub weight_peak: Option<f64>,
    pub weight_prev: Option<f64>,
}

impl FF01 {
    pub fn live_values(&self) -> LiveValues {
        LiveValues {
            weight_peak: self.scales.weight_peak(),
            weight_prev: self.scales.weight(),
        }
    }
}

#[derive(Debug, Clone, BuildEvent, Serialize)]
pub struct State {
    pub is_default_state: bool,
    pub plates_counted: u32,
    pub current_entry: Option<Entry>,
}

impl FF01 {
    pub fn state(&self) -> State {
        State { 
            is_default_state: self.base.emitted_default_state, 
            plates_counted:   0, 
            current_entry:    self.workorder_service.current_entry().cloned(), 
        }
    }
}

pub struct ServiceState {
    pub current_entry: Option<Entry>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum Mutation {
    Tare,
    ClearTare,
    ClearLights,
    EnableService,
    DisableService,
    ConnectService,
}

impl FF01 {
    pub fn apply_mutation(&mut self, mutation: Mutation) {
        use Mutation::*;

        match mutation {
            Tare => self.scales.tare(),
            ClearTare => self.scales.tare_clear(),
            ClearLights => self.lights.lights_disable_all(),
            EnableService => {},
            DisableService => {},
            ConnectService => {},
        }

        self.base.state_changed = true;
    }
}

// Junk
impl CacheableEvents<Self> for LiveValues {
    fn event_value(&self) -> GenericEvent {
        self.build().into()
    }

    fn event_cache_fn(&self) -> CacheFn {
        cache_duration(Duration::from_secs(60 * 60), Duration::from_secs(1))
    }
}

impl CacheableEvents<Self> for State {
    fn event_value(&self) -> GenericEvent {
        self.build().into()
    }

    fn event_cache_fn(&self) -> CacheFn {
        cache_duration(Duration::from_secs(60 * 60), Duration::from_secs(1))
    }
}

// service: enabled
// status:  Error: Config not found

// service: Enabled
// status:  Connected

// service: disabled
// status:  N/A
