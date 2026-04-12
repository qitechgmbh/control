use super::{TestMachineMode, WagoTraverseTestMachine};
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
pub struct StateEvent {
    pub enabled: bool,
    pub mode: String,
    pub control_mode: String,
    pub controller_state: String,
    pub is_homed: bool,
    pub speed_mode_ack: bool,
    pub di1: bool,
    pub di2: bool,
    pub switch_output_on: bool,
    pub target_velocity_register: i16,
    pub target_speed_steps_per_second: i32,
    pub actual_velocity_register: i16,
    pub actual_speed_steps_per_second: f64,
    pub actual_speed_mm_per_second: f64,
    pub reference_mode_ack: bool,
    pub reference_ok: bool,
    pub busy: bool,
    pub target_acceleration: u16,
    pub speed_scale: f64,
    pub direction_multiplier: i8,
    pub freq_range_sel: u8,
    pub acc_range_sel: u8,
    pub raw_position_steps: i64,
    pub wrapper_position_steps: i64,
    pub raw_position_mm: f64,
    pub wrapper_position_mm: f64,
    pub controller_position_mm: Option<f64>,
    pub limit_inner_mm: f64,
    pub limit_outer_mm: f64,
    pub manual_speed_mm_per_second: f64,
    pub manual_velocity_register: i16,
    pub control_byte1: u8,
    pub control_byte2: u8,
    pub control_byte3: u8,
    pub status_byte1: u8,
    pub status_byte2: u8,
    pub status_byte3: u8,
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}

pub enum WagoTraverseTestMachineEvents {
    State(Event<StateEvent>),
}

#[derive(Deserialize)]
#[serde(tag = "action", content = "value")]
pub enum Mutation {
    SetEnabled(bool),
    SetMode(String),
    SetSwitchOutput(bool),
    SetManualSpeedMmPerSecond(f64),
    SetManualVelocityRegister(i16),
    Stop,
    GotoHome,
    GotoLimitInner,
    GotoLimitOuter,
    ForceNotHomed,
    SetPositionSteps(i64),
    SetPositionMm(f64),
    SetLimitInnerMm(f64),
    SetLimitOuterMm(f64),
    SetSpeedScale(f64),
    SetDirectionMultiplier(i8),
    SetFreqRangeSel(u8),
    SetAccRangeSel(u8),
    SetAcceleration(u16),
}

#[derive(Debug, Clone)]
pub struct WagoTraverseTestMachineNamespace {
    pub namespace: Option<Namespace>,
}

impl NamespaceCacheingLogic<WagoTraverseTestMachineEvents> for WagoTraverseTestMachineNamespace {
    fn emit(&mut self, events: WagoTraverseTestMachineEvents) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        if let Some(ns) = &mut self.namespace {
            ns.emit(event, &buffer_fn);
        }
    }
}

impl CacheableEvents<WagoTraverseTestMachineEvents> for WagoTraverseTestMachineEvents {
    fn event_value(&self) -> GenericEvent {
        match self {
            WagoTraverseTestMachineEvents::State(event) => event.clone().into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        cache_first_and_last_event()
    }
}

impl MachineApi for WagoTraverseTestMachine {
    fn api_get_sender(&self) -> smol::channel::Sender<MachineMessage> {
        self.api_sender.clone()
    }

    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(request_body)?;
        match mutation {
            Mutation::SetEnabled(enabled) => self.set_enabled(enabled),
            Mutation::SetMode(mode) => {
                let mode = match mode.as_str() {
                    "Hold" => TestMachineMode::Hold,
                    _ => TestMachineMode::Standby,
                };
                self.set_mode(mode);
            }
            Mutation::SetSwitchOutput(on) => self.set_switch_output(on),
            Mutation::SetManualSpeedMmPerSecond(speed) => {
                self.set_manual_speed_mm_per_second(speed)
            }
            Mutation::SetManualVelocityRegister(velocity) => {
                self.set_manual_velocity_register(velocity)
            }
            Mutation::Stop => self.stop(),
            Mutation::GotoHome => self.goto_home(),
            Mutation::GotoLimitInner => self.goto_limit_inner(),
            Mutation::GotoLimitOuter => self.goto_limit_outer(),
            Mutation::ForceNotHomed => self.force_not_homed(),
            Mutation::SetPositionSteps(position) => self.set_position_steps(position),
            Mutation::SetPositionMm(position) => self.set_position_mm(position),
            Mutation::SetLimitInnerMm(limit) => self.set_limit_inner_mm(limit),
            Mutation::SetLimitOuterMm(limit) => self.set_limit_outer_mm(limit),
            Mutation::SetSpeedScale(scale) => self.set_speed_scale(scale),
            Mutation::SetDirectionMultiplier(direction) => self.set_direction_multiplier(direction),
            Mutation::SetFreqRangeSel(factor) => self.set_freq_range_sel(factor),
            Mutation::SetAccRangeSel(factor) => self.set_acc_range_sel(factor),
            Mutation::SetAcceleration(acceleration) => self.set_acceleration(acceleration),
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }
}
