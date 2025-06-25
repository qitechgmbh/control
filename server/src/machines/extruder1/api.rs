use std::{sync::Arc, time::Duration};

use super::{ExtruderV2, ExtruderV2Mode, HeatingType};
use control_core::{
    machines::api::MachineApi,
    socketio::{
        event::{Event, GenericEvent},
        namespace::{
            CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_duration,
            cache_one_event,
        },
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use smol::channel::Sender;
use socketioxide::extract::SocketRef;
use tracing::instrument;

#[derive(Serialize, Debug, Clone)]
pub struct FrequencyEvent {
    frequency: f64,
    // is this the Frequency in the eeprom or the one in memory(running)
    is_ram: bool,
}

#[derive(Serialize, Debug, Clone)]
pub struct MotorStateEvent {
    start: bool,
    forward_rotation: bool,
}

#[derive(Serialize, Debug, Clone)]
pub struct HeatingStateEvent {
    pub temperature: f64,
    pub target_temperature: f64,
    pub wiring_error: bool,
}

impl HeatingStateEvent {
    pub fn build(&self, heating_type: HeatingType) -> Event<Self> {
        let event = match heating_type {
            HeatingType::Nozzle => Event::new("NozzleHeatingStateEvent", self.clone()),
            HeatingType::Front => Event::new("FrontHeatingStateEvent", self.clone()),
            HeatingType::Back => Event::new("BackHeatingStateEvent", self.clone()),
            HeatingType::Middle => Event::new("MiddleHeatingStateEvent", self.clone()),
        };
        return event;
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct RotationStateEvent {
    pub forward: bool,
}

impl RotationStateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("RotationStateEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct OperationModeEvent {
    operation_mode: u8,
    mode_name: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct ModeEvent {
    pub mode: ExtruderV2Mode,
}

impl ModeEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("ModeStateEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct RegulationStateEvent {
    pub uses_rpm: bool,
}

impl RegulationStateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("RegulationStateEvent", self.clone())
    }
}

/// Inverter status Register 40009
// bit 8-14 is unused
#[derive(Serialize, Debug, Clone)]
pub struct InverterStatusEvent {
    /// RUN (Inverter running)
    running: bool,
    /// Forward running motor spins forward
    forward_running: bool,
    /// Reverse running motor spins backwards
    reverse_running: bool,
    /// Up to frequency, SU not completely sure what its for
    up_to_frequency: bool,
    /// overload warning OL
    overload_warning: bool,
    /// No function, its described that way in the datasheet
    no_function: bool,
    /// FU Output Frequency Detection
    output_frequency_detection: bool,
    /// ABC (Fault)
    abc_fault: bool,
    /// is True when a fault occured
    fault_occurence: bool,
}

/// This is used when we just need a simple confirmation, that what we did, didnt cause errors
#[derive(Serialize, Debug, Clone)]
pub struct InverterSuccessEvent {
    success: bool,
}

#[derive(Serialize, Debug, Clone)]
pub struct ErrorEvent {
    message: String,
    fault_code: u16,
}
#[derive(Serialize, Debug, Clone)]

pub struct PressureStateEvent {
    pub bar: f64,
    pub target_bar: f64,
}

impl PressureStateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("PressureStateEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]

pub struct ScrewStateEvent {
    pub rpm: f64,
    pub target_rpm: f64,
}

impl ScrewStateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("ScrewStateEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]

pub struct ExtruderSettingsStateEvent {
    pub pressure_limit: f64,
    pub pressure_limit_enabled: bool,
}

impl ExtruderSettingsStateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("ExtruderSettingsStateEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct HeatingPowerEvent {
    pub wattage: f64,
}

impl HeatingPowerEvent {
    pub fn build(&self, heating_type: HeatingType) -> Event<Self> {
        match heating_type {
            HeatingType::Nozzle => Event::new("NozzleHeatingPowerEvent", self.clone()),
            HeatingType::Front => Event::new("FrontHeatingPowerEvent", self.clone()),
            HeatingType::Back => Event::new("BackHeatingPowerEvent", self.clone()),
            HeatingType::Middle => Event::new("MiddleHeatingPowerEvent", self.clone()),
        }
    }
}

pub enum ExtruderV2Events {
    RotationStateEvent(Event<RotationStateEvent>),
    ModeEvent(Event<ModeEvent>),
    RegulationStateEvent(Event<RegulationStateEvent>),
    PressureStateEvent(Event<PressureStateEvent>),
    ScrewStateEvent(Event<ScrewStateEvent>),
    HeatingStateEvent(Event<HeatingStateEvent>),
    ExtruderSettingsStateEvent(Event<ExtruderSettingsStateEvent>),
    HeatingPowerEvent(Event<HeatingPowerEvent>),
}

#[derive(Deserialize, Serialize)]
enum Mutation {
    /// INVERTER
    /// Frequency Control
    // Set Rotation also starts the motor
    InverterRotationSetDirection(bool),
    InverterSetTargetPressure(f64),
    InverterSetTargetRpm(f64),
    InverterSetRegulation(bool),

    //Mode
    ExtruderSetMode(ExtruderV2Mode),
    FrontHeatingSetTargetTemperature(f64),
    BackHeatingSetTargetTemperature(f64),
    MiddleSetHeatingTemperature(f64),
    NozzleSetHeatingTemperature(f64),

    // SetPressure
    ExtruderSetPressureLimit(f64),
    ExtruderSetPressureLimitIsEnabled(bool),
}

#[derive(Debug)]
pub struct ExtruderV2Namespace {
    pub namespace: Namespace,
}

impl NamespaceCacheingLogic<ExtruderV2Events> for ExtruderV2Namespace {
    #[instrument(skip_all)]
    fn emit(&mut self, events: ExtruderV2Events) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        self.namespace.emit(event, &buffer_fn);
    }
}

impl ExtruderV2Namespace {
    pub fn new(socket_queue_tx: Sender<(SocketRef, Arc<GenericEvent>)>) -> Self {
        Self {
            namespace: Namespace::new(socket_queue_tx),
        }
    }
}

impl CacheableEvents<ExtruderV2Events> for ExtruderV2Events {
    fn event_value(&self) -> GenericEvent {
        match self {
            ExtruderV2Events::RotationStateEvent(event) => event.into(),
            ExtruderV2Events::ModeEvent(event) => event.into(),
            ExtruderV2Events::RegulationStateEvent(event) => event.into(),
            ExtruderV2Events::PressureStateEvent(event) => event.into(),
            ExtruderV2Events::ScrewStateEvent(event) => event.into(),
            ExtruderV2Events::HeatingStateEvent(event) => event.into(),
            ExtruderV2Events::ExtruderSettingsStateEvent(event) => event.into(),
            ExtruderV2Events::HeatingPowerEvent(event) => event.into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        let cache_one_hour = cache_duration(Duration::from_secs(60 * 60), Duration::from_secs(1));
        let cache_one = cache_one_event();

        match self {
            // State events (keep only the latest)
            ExtruderV2Events::RotationStateEvent(_) => cache_one,
            ExtruderV2Events::ModeEvent(_) => cache_one,
            ExtruderV2Events::RegulationStateEvent(_) => cache_one,
            ExtruderV2Events::ExtruderSettingsStateEvent(_) => cache_one,
            ExtruderV2Events::HeatingStateEvent(_) => cache_one,

            // Time series events (keep for 1 hour)
            ExtruderV2Events::PressureStateEvent(_) => cache_one_hour,
            ExtruderV2Events::ScrewStateEvent(_) => cache_one_hour,
            ExtruderV2Events::HeatingPowerEvent(_) => cache_one_hour,
        }
    }
}

impl MachineApi for ExtruderV2 {
    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        // there are multiple Modbus Frames that are "prebuilt"
        let control: Mutation = serde_json::from_value(request_body)?;
        match control {
            Mutation::ExtruderSetMode(mode) => self.set_mode_state(mode),
            Mutation::InverterRotationSetDirection(forward) => self.set_rotation_state(forward),
            Mutation::InverterSetRegulation(uses_rpm) => self.set_regulation(uses_rpm),
            Mutation::InverterSetTargetPressure(bar) => self.set_target_pressure(bar),
            Mutation::InverterSetTargetRpm(rpm) => self.set_target_rpm(rpm),

            Mutation::FrontHeatingSetTargetTemperature(temp) => {
                self.set_target_temperature(temp, HeatingType::Front)
            }
            Mutation::MiddleSetHeatingTemperature(temp) => {
                self.set_target_temperature(temp, HeatingType::Middle)
            }
            Mutation::BackHeatingSetTargetTemperature(temp) => {
                self.set_target_temperature(temp, HeatingType::Back)
            }
            Mutation::NozzleSetHeatingTemperature(temp) => {
                self.set_target_temperature(temp, HeatingType::Nozzle)
            }
            Mutation::ExtruderSetPressureLimit(pressure_limit) => {
                self.set_nozzle_pressure_limit(pressure_limit);
            }
            Mutation::ExtruderSetPressureLimitIsEnabled(enabled) => {
                self.set_nozzle_pressure_limit_is_enabled(enabled);
            }
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> &mut Namespace {
        &mut self.namespace.namespace
    }
}
