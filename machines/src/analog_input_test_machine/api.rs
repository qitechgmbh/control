use std::sync::Arc;

use control_core::socketio::{
    event::{Event, GenericEvent},
    namespace::{
        CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_first_and_last_event,
    },
};
use serde::{Deserialize, Serialize};

use crate::{MachineApi, analog_input_test_machine::AnalogInputTestMachine};

#[derive(Debug, Clone)]
pub struct AnalogInputTestMachineNamespace {
    pub namespace: Option<Namespace>,
}

#[derive(Serialize, Debug, Clone)]
pub enum MeasurementEvent {
    MeasurementRateHz(f64),
    Measurement(f64, String),
}

impl NamespaceCacheingLogic<AnalogInputTestMachineEvents> for AnalogInputTestMachineNamespace {
    fn emit(&mut self, events: AnalogInputTestMachineEvents) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        if let Some(ns) = &mut self.namespace {
            ns.emit(event, &buffer_fn);
        }
    }
}

impl CacheableEvents<AnalogInputTestMachineEvents> for AnalogInputTestMachineEvents {
    fn event_value(&self) -> GenericEvent {
        match self {
            AnalogInputTestMachineEvents::State(event) => event.clone().into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        cache_first_and_last_event()
    }
}

pub enum AnalogInputTestMachineEvents {
    State(Event<MeasurementEvent>),
}

#[derive(Deserialize)]
pub struct Mutation {
    measurement_rate_hz: i32,
}

impl MachineApi for AnalogInputTestMachine {
    fn api_get_sender(&self) -> smol::channel::Sender<crate::MachineMessage> {
        self.api_sender.clone()
    }

    fn api_mutate(&mut self, value: serde_json::Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(value)?;
        self.measurement_rate_hz = f64::from(mutation.measurement_rate_hz);
        self.emit_measurement_rate();
        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<control_core::socketio::namespace::Namespace> {
        self.namespace.namespace.clone()
    }
}
