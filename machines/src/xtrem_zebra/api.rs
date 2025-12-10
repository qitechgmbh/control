use super::XtremZebra;
use crate::{MachineApi, MachineMessage};
use beas_bsl::WeightedItem;
use control_core::socketio::{
    event::{Event, GenericEvent},
    namespace::{
        CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_first_and_last_event,
    },
};
use control_core_derive::BuildEvent;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tracing::instrument;

#[derive(Serialize, Debug, Clone, Default)]
pub struct LiveValuesEvent {
    /// weight measurement in kilograms
    pub total_weight: f64,
    pub current_weight: f64,
    pub plate_counter: u32,
}

impl LiveValuesEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("LiveValuesEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Configuration {
    pub password: Option<String>,
    pub config_string: Option<String>,
}

#[derive(Serialize, Debug, Clone, BuildEvent)]
pub struct StateEvent {
    pub is_default_state: bool,
    pub xtrem_zebra_state: XtremZebraState,
    pub weighted_item: WeightedItem,
    pub configuration: Configuration,
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct XtremZebraState {
    /// tolerance
    pub tolerance: f64,
}

pub enum XtremZebraEvents {
    LiveValues(Event<LiveValuesEvent>),
    State(Event<StateEvent>),
}

#[derive(Debug)]
pub struct XtremZebraNamespace {
    pub namespace: Option<Namespace>,
}

impl CacheableEvents<Self> for XtremZebraEvents {
    fn event_value(&self) -> GenericEvent {
        match self {
            Self::LiveValues(event) => event.into(),
            Self::State(event) => event.into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        let cache_first_and_last = cache_first_and_last_event();

        match self {
            Self::LiveValues(_) => cache_first_and_last,
            Self::State(_) => cache_first_and_last,
        }
    }
}

#[derive(Deserialize, Serialize)]
/// All values in the Mutation enum should be positive.
/// This ensures that the parameters for setting tolerances and target diameter
/// are valid and meaningful within the context of the XtremZebra's operation.
enum Mutation {
    SetTolerance(f64),
    SetTare,
    SetConfigString(String),
    SetPassword(String),
    ZeroCounters,
    ClearLights,
    Start,
}

impl NamespaceCacheingLogic<XtremZebraEvents> for XtremZebraNamespace {
    #[instrument(skip_all)]
    fn emit(&mut self, events: XtremZebraEvents) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();

        match &mut self.namespace {
            Some(ns) => ns.emit(event, &buffer_fn),
            None => (),
        }
    }
}

impl MachineApi for XtremZebra {
    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(request_body)?;
        match mutation {
            Mutation::SetTolerance(tolerance) => {
                self.tolerance = tolerance;
            }
            Mutation::SetTare => {
                self.set_tare();
            }
            Mutation::ZeroCounters => {
                self.zero_counters();
            }
            Mutation::ClearLights => {
                self.clear_lights();
            }
            Mutation::SetConfigString(root) => {
                self.configuration.config_string = Some(root);
                println!("SetConfigString {:?}", self.configuration.config_string);
            }
            Mutation::SetPassword(pw) => {
                self.configuration.password = Some(pw);
                println!("SetPassword {:?}", self.configuration.password);
            }
            Mutation::Start => {
                self.start();
            }
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }

    fn api_get_sender(&self) -> smol::channel::Sender<MachineMessage> {
        self.api_sender.clone()
    }
}
