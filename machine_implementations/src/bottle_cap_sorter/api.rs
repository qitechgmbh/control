use std::sync::Arc;

use crate::{MachineApi, MachineMessage, MachineValues};

use super::{ColorBoundsHsl, ColorBoundsState, Sorter1, Sorter1Mode};
use control_core::socketio::{event::{Event, GenericEvent}, namespace::{CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_first_and_last_event}};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::instrument;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Mode {
    Standby,
    Running,
}

impl From<Sorter1Mode> for Mode {
    fn from(mode: Sorter1Mode) -> Self {
        match mode {
            Sorter1Mode::Standby => Mode::Standby,
            Sorter1Mode::Running => Mode::Running,
        }
    }
}

impl From<Mode> for Sorter1Mode {
    fn from(mode: Mode) -> Self {
        match mode {
            Mode::Standby => Sorter1Mode::Standby,
            Mode::Running => Sorter1Mode::Running,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ModeState {
    pub mode: Mode,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ConveyorBeltState {
    pub enabled: bool,
    pub target_speed: f64, // m/s
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct StateEvent {
    pub is_default_state: bool,
    pub mode_state: ModeState,
    pub conveyor_belt_state: ConveyorBeltState,
    pub colors_state: Vec<ColorBoundsState>,
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct LiveValuesEvent {
    pub conveyor_belt_speed: f64, // m/s
    pub air_valve_states: [bool; 8],
}

impl LiveValuesEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("LiveValuesEvent", self.clone())
    }
}

pub enum Sorter1Events {
    State(Event<StateEvent>),
    LiveValues(Event<LiveValuesEvent>),
}

impl CacheableEvents<Sorter1Events> for Sorter1Events {
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

#[derive(Debug)]
pub struct Sorter1Namespace {
    pub namespace: Option<Namespace>,
}

pub fn convert_hsl_to_state(colors: [ColorBoundsHsl; 8]) -> Vec<ColorBoundsState> {
    colors
        .into_iter()
        .enumerate()
        .map(|(idx, c)| ColorBoundsState {
            h_min: c.h_min,
            h_max: c.h_max,
            s_min: c.s_min * 100.0,
            s_max: c.s_max * 100.0,
            l_min: c.l_min * 100.0,
            l_max: c.l_max * 100.0,
            valve_index: idx,
        })
        .collect()
}

impl NamespaceCacheingLogic<Sorter1Events> for Sorter1Namespace {
    #[instrument(skip_all)]
    fn emit(&mut self, events: Sorter1Events) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();

        match &mut self.namespace {
            Some(ns) => ns.emit(event, &buffer_fn),
            None => (),
        }
    }
}

#[derive(Deserialize, Serialize)]
/// Mutation for controlling the sorter machine
enum Mutation {
    SetMode(Mode),
    ConveyorBeltSetTargetSpeed(f64),
    ActivateAirValvePulse {
        valve_index: usize,
        duration_ms: u64,
    },
    SetColor(ColorBoundsState),
}

impl MachineApi for Sorter1 {
    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(request_body)?;
        match mutation {
            Mutation::SetMode(mode) => {
                self.set_mode(&mode.into());
            }
            Mutation::ConveyorBeltSetTargetSpeed(target_speed) => {
                self.conveyor_belt_set_target_speed(target_speed);
            }
            Mutation::ActivateAirValvePulse {
                valve_index,
                duration_ms,
            } => {
                self.activate_air_valve_pulse(valve_index, duration_ms);
            }
            Mutation::SetColor(color_bounds_state) => {
                self.assign_colour_bounds_to_valve(color_bounds_state)
            }
        }
        Ok(())
    }

    fn act_machine_message(&mut self, msg: MachineMessage) {
        match msg {
            MachineMessage::SubscribeNamespace(namespace) => {
                self.namespace.namespace = Some(namespace);
                self.emit_state();
                tracing::info!("extruder1 received subscribe");
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
                        live_values: serde_json::to_value(self.get_live_values())
                            .expect("Failed to serialize live values"),
                    })
                    .expect("Failed to send values");
            }
        }
    }

    fn get_api_sender(&self) -> tokio::sync::mpsc::Sender<crate::MachineMessage> {
        self.api_sender.clone()
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }

}
