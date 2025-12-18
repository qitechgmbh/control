use super::BeckhoffMachine;
use crate::{MachineApi, MachineMessage};
use control_core::socketio::{
    event::{Event, GenericEvent},
    namespace::{
        CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_first_and_last_event,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use smol::channel::Sender;
use std::sync::Arc;
use tracing::instrument;

#[derive(Serialize, Debug, Clone)]
pub struct StateEvent {
    pub motor_enabled: bool,
    pub motor_velocity: i32,
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}

pub enum BeckhoffEvents {
    State(Event<StateEvent>),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type", content = "payload")]
enum Mutation {
    SetMotorOn(bool),
    SetMotorVelocity(i32),
    SetMotorOff(bool),
}

#[derive(Debug, Clone)]
pub struct BeckhoffNamespace {
    pub namespace: Option<Namespace>,
}

impl NamespaceCacheingLogic<BeckhoffEvents> for BeckhoffNamespace {
    #[instrument(skip_all)]
    fn emit(&mut self, events: BeckhoffEvents) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        if let Some(ns) = &mut self.namespace {
            ns.emit(event, &buffer_fn);
        }
    }
}

impl CacheableEvents<BeckhoffEvents> for BeckhoffEvents {
    fn event_value(&self) -> GenericEvent {
        match self {
            BeckhoffEvents::State(event) => event.into(),
        }
    }
    fn event_cache_fn(&self) -> CacheFn {
        cache_first_and_last_event()
    }
}

impl MachineApi for BeckhoffMachine {
    fn api_get_sender(&self) -> Sender<MachineMessage> {
        self.api_sender.clone()
    }
    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }
    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(request_body)?;
        match mutation {
            Mutation::SetMotorOn(t) => {
                if t {
                    self.turn_motor_on();
                } else {
                    self.turn_motor_off();
                }
            }
            Mutation::SetMotorVelocity(vel) => {
                self.motor_state.target_velocity = vel;
                self.emit_state();
            }

            _ => {}
        }
        Ok(())
    }
}
