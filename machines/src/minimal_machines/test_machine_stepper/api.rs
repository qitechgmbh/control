use super::TestMachineStepper;
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

#[derive(Serialize, Debug, Clone)]
pub enum MotorState {
    Off,
    Enable,
    SetMode,
    Ready,
    StartPulseStart,
    StartPulseEnd,
    Running,
    ErrorQuit,
    ResetQuit,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum Mode {
    #[default]
    Standby,
    Hold,
    Turn,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct ModeState {
    pub mode: Mode,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum Frequency {
    #[default]
    Default,
    Low,
    Mid,
    High,
}

impl From<u8> for Frequency {
    fn from(val: u8) -> Self {
        match val {
            1 => Frequency::Low,
            2 => Frequency::Mid,
            3 => Frequency::High,
            _ => Frequency::Default,
        }
    }
}

impl From<Frequency> for u8 {
    fn from(freq: Frequency) -> Self {
        match freq {
            Frequency::Default => 0,
            Frequency::Low => 1,
            Frequency::Mid => 2,
            Frequency::High => 3,
        }
    }
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct FrequencyState {
    pub frequency: Frequency,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum AccelerationFactor {
    #[default]
    Default,
    Low,
    Mid,
    High,
}

impl From<u8> for AccelerationFactor {
    fn from(val: u8) -> Self {
        match val {
            1 => AccelerationFactor::Low,
            2 => AccelerationFactor::Mid,
            3 => AccelerationFactor::High,
            _ => AccelerationFactor::Default,
        }
    }
}

impl From<AccelerationFactor> for u8 {
    fn from(factor: AccelerationFactor) -> Self {
        match factor {
            AccelerationFactor::Default => 0,
            AccelerationFactor::Low => 1,
            AccelerationFactor::Mid => 2,
            AccelerationFactor::High => 3,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AccelerationState {
    pub factor: AccelerationFactor,
}

#[derive(Serialize, Debug, Clone)]
pub struct StateEvent {
    pub target_speed: i16,
    pub enabled: bool,
    pub mode_state: ModeState,
    pub frequency_state: FrequencyState,
    pub acceleration_state: AccelerationState,
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
    StartMotor,
    StopMotor,

    // Mode
    SetMode(Mode),
    // Frequency Prescaler
    SetFreq(Frequency),
    // Acceleration Factor
    SetAccFactor(AccelerationFactor),
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
    fn api_get_sender(&self) -> smol::channel::Sender<MachineMessage> {
        self.api_sender.clone()
    }

    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(request_body)?;
        match mutation {
            Mutation::SetTargetSpeed { target } => self.set_target_speed(target),
            Mutation::SetEnabled { enabled } => self.set_enabled(enabled),
            Mutation::StartMotor => self.start_motor(),
            Mutation::StopMotor => self.stop_motor(),
            Mutation::SetMode(mode) => self.set_mode(mode),
            Mutation::SetFreq(freq) => self.set_freq(freq),
            Mutation::SetAccFactor(factor) => self.set_acc_factor(factor),
        }

        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }
}
