use super::DryerSmartMachine;
use crate::dryer::device::{SmartData, SmartTimerEntry, WeeklySchedule};
use crate::{MachineApi, MachineMessage, MachineValues};
use control_core::socketio::{
    event::{Event, GenericEvent},
    namespace::{CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_first_and_last_event},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tracing::instrument;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct LiveValuesEvent {
    pub status: u16,
    pub temp_process: f64,
    pub temp_safety: f64,
    pub temp_regen_in: f64,
    pub temp_regen_out: f64,
    pub temp_fan_inlet: f64,
    pub temp_return_air: f64,
    pub temp_dew_point: f64,
    pub pwm_fan1: f64,
    pub pwm_fan2: f64,
    pub power_process: f64,
    pub power_regen: f64,
    pub alarm: u16,
    pub warning: u16,
    pub target_temperature: f64,
    pub schedule: WeeklySchedule,
    pub smart_data: SmartData,
}

impl LiveValuesEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("LiveValuesEvent", self.clone())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StateEvent {
    pub is_default_state: bool,
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}

pub enum DryerSmartEvents {
    LiveValues(Event<LiveValuesEvent>),
    State(Event<StateEvent>),
}

#[derive(Debug)]
pub struct DryerSmartMachineNamespace {
    pub namespace: Option<Namespace>,
}

impl CacheableEvents<Self> for DryerSmartEvents {
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

impl NamespaceCacheingLogic<DryerSmartEvents> for DryerSmartMachineNamespace {
    #[instrument(skip_all)]
    fn emit(&mut self, events: DryerSmartEvents) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        if let Some(ns) = &mut self.namespace {
            ns.emit(event, &buffer_fn);
        }
    }
}

#[derive(Deserialize, Serialize)]
enum Mutation {
    SetStartStop(bool),
    SetTargetTemperature(f64),
    SetSchedule(WeeklySchedule),
    ApplyMaterialPreset {
        abbrev: String,
        throughput_kg_per_h: f64,
    },
    SyncClock,
    SetTimerEnabled(bool),
    WriteTimerEntry { index: u8, entry: SmartTimerEntry },
    WriteNewTimerEntry { entry: SmartTimerEntry },
    DeleteTimerEntry { index: u8 },
}

impl MachineApi for DryerSmartMachine {
    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(request_body)?;
        match mutation {
            Mutation::SetStartStop(_) => self.set_start_stop(),
            Mutation::SetTargetTemperature(temp) => self.set_target_temperature(temp),
            Mutation::SetSchedule(schedule) => self.set_schedule(schedule),
            Mutation::ApplyMaterialPreset {
                abbrev,
                throughput_kg_per_h,
            } => self.apply_material_preset(&abbrev, throughput_kg_per_h),
            Mutation::SyncClock => self.sync_system_clock(),
            Mutation::SetTimerEnabled(enabled) => self.set_timer_enabled(enabled),
            Mutation::WriteTimerEntry { index, entry } => self.write_timer_entry(index, entry),
            Mutation::WriteNewTimerEntry { entry } => self.write_new_timer_entry(entry),
            Mutation::DeleteTimerEntry { index } => self.delete_timer_entry(index),
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }

    fn get_api_sender(&self) -> Sender<MachineMessage> {
        self.api_sender.clone()
    }

    fn act_machine_message(&mut self, msg: MachineMessage) {
        match msg {
            MachineMessage::SubscribeNamespace(namespace) => {
                self.namespace.namespace = Some(namespace);
                self.emit_live_values();
            }
            MachineMessage::UnsubscribeNamespace => {
                self.namespace.namespace = None;
            }
            MachineMessage::HttpApiJsonRequest(value) => {
                let _res = self.api_mutate(value);
            }
            MachineMessage::RequestValues(sender) => {
                sender
                    .send(MachineValues {
                        state: serde_json::to_value(self.get_state()).expect("Failed to serialize state"),
                        live_values: serde_json::to_value(self.get_live_values())
                            .expect("Failed to serialize live values"),
                    })
                    .expect("Failed to send values");
            }
        }
    }
}
