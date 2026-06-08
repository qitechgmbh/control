use super::TestMachineStepper;
use crate::{MachineApi, MachineMessage, MachineValues};
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
    pub target_speed: i16,
    pub enabled: bool,
    pub freq: u8,
    pub acc_freq: u8,
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}

pub enum TestMachineStepperEvents {
    State(Event<StateEvent>),
}

#[derive(Deserialize)]
#[serde(tag = "action", content = "value")]
pub enum Mutation {
    SetTargetSpeed { target: i16 },
    SetEnabled { enabled: bool },
    SetFreq { factor: u8 },
    SetAccFreq { factor: u8 },
}

#[derive(Debug, Clone)]
pub struct TestMachineStepperNamespace {
    pub namespace: Option<Namespace>,
}

impl NamespaceCacheingLogic<TestMachineStepperEvents> for TestMachineStepperNamespace {
    fn emit(&mut self, events: TestMachineStepperEvents) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        if let Some(ns) = &mut self.namespace {
            ns.emit(event, &buffer_fn);
        }
    }
}

impl CacheableEvents<TestMachineStepperEvents> for TestMachineStepperEvents {
    fn event_value(&self) -> GenericEvent {
        match self {
            TestMachineStepperEvents::State(event) => event.clone().into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        cache_first_and_last_event()
    }
}

impl MachineApi for TestMachineStepper {
    fn act_machine_message(&mut self, msg: MachineMessage) {
        match msg {
            MachineMessage::SubscribeNamespace(namespace) => {
                self.namespace.namespace = Some(namespace);
                self.emit_state();
            }
            MachineMessage::UnsubscribeNamespace => self.namespace.namespace = None,
            MachineMessage::HttpApiJsonRequest(value) => {
                let _res = self.api_mutate(value);
            }
            MachineMessage::RequestValues(sender) => {
                sender
                    .send(MachineValues {
                        state: serde_json::to_value(self.get_state())
                            .expect("Failed to serialize state"),
                        live_values: serde_json::Value::Null,
                    })
                    .expect("Failed to send values");
            }
        }
    }

    fn get_api_sender(&self) -> Sender<MachineMessage> {
        self.sender.clone()
    }

    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(request_body)?;
        match mutation {
            Mutation::SetTargetSpeed { target } => self.set_target_speed(target),
            Mutation::SetEnabled { enabled } => self.set_enabled(enabled),
            Mutation::SetFreq { factor } => self.set_freq(factor),
            Mutation::SetAccFreq { factor } => self.set_acc_freq(factor),
        }

        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }
}
