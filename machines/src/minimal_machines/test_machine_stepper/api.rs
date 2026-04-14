use super::TestMachineStepper;
use crate::{MachineApi, MachineMessage, minimal_machines::test_machine_stepper::TestMachineMode};
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

impl From<TestMachineMode> for Mode {
    fn from(mode: TestMachineMode) -> Self {
        match mode {
            TestMachineMode::Standby => Self::Standby,
            TestMachineMode::Hold => Self::Hold,
            TestMachineMode::Turn => Self::Turn,
        }
    }
}

impl From<Mode> for TestMachineMode {
    fn from(mode: Mode) -> Self {
        match mode {
            Mode::Standby => Self::Standby,
            Mode::Hold => Self::Hold,
            Mode::Turn => Self::Turn,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum Frequency {
    #[default]
    Default,
    Low,
    Mid,
    High,
}

#[derive(Serialize, Debug, Clone, Default)]


#[derive(Serialize, Debug, Clone)]
pub struct StateEvent {
    pub target_speed: i16,
    pub enabled: bool,
    pub freq: u8,
    pub acc_freq: u8,
    pub mode_state: ModeState,
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
    SetFreq { factor: u8 },
    SetAccFreq { factor: u8 },

    // Mode
    SetMode(Mode),
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
            Mutation::SetFreq { factor } => self.set_freq(factor),
            Mutation::SetAccFreq { factor } => self.set_acc_freq(factor),
            Mutation::SetMode(mode) => self.set_mode(&mode.into()),
        }

        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }
}
