use std::{sync::Arc, time::Duration};

use super::{WaterCooling, WaterCoolingMode};
use control_core::{
    machines::api::MachineApi,
    socketio::{
        event::{Event, GenericEvent},
        namespace::{
            CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_duration,
            cache_one_event,
        },
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use smol::channel::Sender;
use socketioxide::extract::SocketRef;
use tracing::instrument;

#[derive(Serialize, Debug, Clone)]
pub struct MotorStateEvent {
    start: bool,
    forward_rotation: bool,
}

#[derive(Serialize, Debug, Clone)]
pub struct CoolingStateEvent {
    pub temperature: f64,
    pub target_temperature: f64,
}

impl CoolingStateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("WaterCoolingStateEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct ModeEvent {
    pub mode: WaterCoolingMode,
}

impl ModeEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("ModeStateEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct CoolingPowerEvent {
    pub wattage: f64,
}

impl CoolingPowerEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("CoolingPowerEvent", self.clone())
    }
}

pub enum WaterCoolingEvents {
    ModeEvent(Event<ModeEvent>),
    CoolingStateEvent(Event<CoolingStateEvent>),
    CoolingPowerEvent(Event<CoolingPowerEvent>),
}

#[derive(Deserialize, Serialize)]
enum Mutation {
    CoolingSetMode(WaterCoolingMode),
    CoolingSetTargetTemperature(f64),
}

#[derive(Debug)]
pub struct WaterCoolingNamespace {
    pub namespace: Namespace,
}

impl NamespaceCacheingLogic<WaterCoolingEvents> for WaterCoolingNamespace {
    #[instrument(skip_all)]
    fn emit(&mut self, events: WaterCoolingEvents) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        self.namespace.emit(event, &buffer_fn);
    }
}

impl WaterCoolingNamespace {
    pub fn new(socket_queue_tx: Sender<(SocketRef, Arc<GenericEvent>)>) -> Self {
        Self {
            namespace: Namespace::new(socket_queue_tx),
        }
    }
}

impl CacheableEvents<WaterCoolingEvents> for WaterCoolingEvents {
    fn event_value(&self) -> GenericEvent {
        match self {
            WaterCoolingEvents::ModeEvent(event) => event.into(),
            WaterCoolingEvents::CoolingStateEvent(event) => event.into(),
            WaterCoolingEvents::CoolingPowerEvent(event) => event.into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        let cache_one_hour = cache_duration(Duration::from_secs(60 * 60), Duration::from_secs(1));
        let cache_one = cache_one_event();

        match self {
            WaterCoolingEvents::ModeEvent(_) => cache_one,
            WaterCoolingEvents::CoolingStateEvent(_) => cache_one,
            WaterCoolingEvents::CoolingPowerEvent(_) => cache_one_hour,
        }
    }
}

impl MachineApi for WaterCooling {
    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        // there are multiple Modbus Frames that are "prebuilt"
        let control: Mutation = serde_json::from_value(request_body)?;
        match control {
            Mutation::CoolingSetMode(mode) => self.set_mode_state(mode),

            Mutation::CoolingSetTargetTemperature(temp) => self.set_target_temperature(temp),
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> &mut Namespace {
        &mut self.namespace.namespace
    }
}
