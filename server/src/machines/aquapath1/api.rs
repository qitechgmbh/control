use std::{sync::Arc, time::Duration};

use super::{AquaPathV1, AquaPathV1Mode};
use control_core::{
    machines::api::MachineApi,
    socketio::{
        event::{Event, GenericEvent},
        namespace::{
            CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_duration,
            cache_first_and_last_event,
        },
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use smol::channel::Sender;
use socketioxide::extract::SocketRef;
use tracing::instrument;
use uom::si::{
    f64::{ThermodynamicTemperature, VolumeRate},
    thermodynamic_temperature::degree_celsius,
    volume_rate::liter_per_minute,
};

#[derive(Serialize, Debug, Clone, Default)]
pub struct LiveValuesEvent {
    pub front_flow: f64,
    pub back_flow: f64,
    pub front_temperature: f64,
    pub back_temperature: f64,
}

impl LiveValuesEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("LiveValuesEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct StateEvent {
    pub is_default_state: bool,
    /// mode state
    pub mode_state: ModeState,
    pub flow_states: FlowStates,
    pub temp_states: TempStates,
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct TempStates {
    pub front: TempState,
    pub back: TempState,
}

#[derive(Serialize, Debug, Clone)]
pub struct TempState {
    pub temperature: f64,
    pub target_temperature: f64,
}

#[derive(Serialize, Debug, Clone)]
pub struct ModeState {
    pub mode: AquaPathV1Mode,
}
#[derive(Serialize, Debug, Clone)]
pub struct FlowStates {
    pub front: FlowState,
    pub back: FlowState,
}
#[derive(Serialize, Debug, Clone)]
pub struct FlowState {
    pub flow: f64,
    pub target_flow: f64,
}

pub enum AquaPathV1Events {
    LiveValues(Event<LiveValuesEvent>),
    State(Event<StateEvent>),
}

#[derive(Deserialize, Serialize)]
enum Mutation {
    //Mode
    SetAquaPathMode(AquaPathV1Mode),

    SetFrontTemperature(f64),
    SetBackTemperature(f64),

    SetFrontFlow(f64),
    SetBackFlow(f64),
}

#[derive(Debug)]
pub struct AquaPathV1Namespace {
    pub namespace: Namespace,
}

impl NamespaceCacheingLogic<AquaPathV1Events> for AquaPathV1Namespace {
    #[instrument(skip_all)]
    fn emit(&mut self, events: AquaPathV1Events) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        self.namespace.emit(event, &buffer_fn);
    }
}

impl AquaPathV1Namespace {
    pub fn new(socket_queue_tx: Sender<(SocketRef, Arc<GenericEvent>)>) -> Self {
        Self {
            namespace: Namespace::new(socket_queue_tx),
        }
    }
}

impl CacheableEvents<AquaPathV1Events> for AquaPathV1Events {
    fn event_value(&self) -> GenericEvent {
        match self {
            AquaPathV1Events::LiveValues(event) => event.into(),
            AquaPathV1Events::State(event) => event.into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        let cache_one_hour = cache_duration(Duration::from_secs(60 * 60), Duration::from_secs(1));
        let cache_first_and_last = cache_first_and_last_event();

        match self {
            AquaPathV1Events::LiveValues(_) => cache_one_hour,
            AquaPathV1Events::State(_) => cache_first_and_last,
        }
    }
}

impl MachineApi for AquaPathV1 {
    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        // there are multiple Modbus Frames that are "prebuilt"
        let control: Mutation = serde_json::from_value(request_body)?;
        match control {
            Mutation::SetAquaPathMode(mode) => self.set_mode_state(mode),
            Mutation::SetBackTemperature(temperature) => {
                self.temp_controller_back
                    .set_target_temperature(ThermodynamicTemperature::new::<degree_celsius>(
                        temperature,
                    ));
            }
            Mutation::SetFrontTemperature(temperature) => {
                self.temp_controller_front
                    .set_target_temperature(ThermodynamicTemperature::new::<degree_celsius>(
                        temperature,
                    ));
            }
            Mutation::SetBackFlow(flow) => self
                .flow_controller_back
                .set_target_flow(VolumeRate::new::<liter_per_minute>(flow)),
            Mutation::SetFrontFlow(flow) => self
                .flow_controller_front
                .set_target_flow(VolumeRate::new::<liter_per_minute>(flow)),
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> &mut Namespace {
        &mut self.namespace.namespace
    }
}
