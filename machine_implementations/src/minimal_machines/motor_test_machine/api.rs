use super::MotorTestMachine;
use crate::{MachineApi, MachineMessage};
use control_core::socketio::{
    event::{Event, GenericEvent},
    namespace::{
        CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_first_and_last_event,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

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
// @TODO format diverges, other modules use #[serde(tag = "action", content = "value")]
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

impl MachineApi for MotorTestMachine {
    fn act_machine_message(&mut self, msg: MachineMessage) {
        match msg {
            MachineMessage::SubscribeNamespace(ns) => {
                self.namespace.namespace = Some(ns);
                self.emit_state();
            }
            MachineMessage::UnsubscribeNamespace => {
                self.namespace.namespace = None;
            }
            MachineMessage::HttpApiJsonRequest(value) => {
                let _ = self.api_mutate(value);
            }
            // @TODO: Check whether it's fine that this machine never returns its state. Self is also missing an implementation for get_state
            MachineMessage::RequestValues(_sender) => (),
        }
    }
    fn get_api_sender(&self) -> Sender<MachineMessage> {
        self.sender.clone()
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
            // @TODO motorOff may be redundant
            Mutation::SetMotorOff(_t) => {
                self.turn_motor_off();
            }
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }
}
