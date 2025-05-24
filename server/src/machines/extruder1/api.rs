use std::time::Duration;

use super::{ExtruderV2, ExtruderV2Mode, Heating, HeatingType};
use control_core::{
    machines::api::MachineApi,
    socketio::{
        event::{Event, GenericEvent},
        namespace::{
            CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, NamespaceInterface,
            cache_duration, cache_one_event,
        },
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Debug, Clone)]
pub struct FrequencyEvent {
    frequency: f32,
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
    pub temperature: f32,
    pub heating: bool,
    pub target_temperature: f32,
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
    pub bar: f32,
    pub target_bar: f32,
}

impl PressureStateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("PressureStateEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]

pub struct RpmStateEvent {
    pub rpm: f32,
    pub target_rpm: f32,
}

impl RpmStateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("RpmStateEvent", self.clone())
    }
}

pub enum ExtruderV2Events {
    InverterStateEvent(Event<InverterStatusEvent>),
    InverterModeEvent(Event<OperationModeEvent>),
    InverterErrorEvent(Event<ErrorEvent>),
    InverterFrequencyEvent(Event<FrequencyEvent>),
    InverterSuccessEvent(Event<InverterSuccessEvent>),
    RotationStateEvent(Event<RotationStateEvent>),
    ModeEvent(Event<ModeEvent>),
    RegulationStateEvent(Event<RegulationStateEvent>),
    PressureStateEvent(Event<PressureStateEvent>),
    RpmStateEvent(Event<RpmStateEvent>),
    HeatingStateEvent(Event<HeatingStateEvent>),
}

#[derive(Deserialize, Serialize)]
enum Mutation {
    /// INVERTER
    /// Frequency Control
    /// Set
    SetRunningFrequency(f32),
    SetEepromFrequency(f32),
    SetMinimumFrequency(f32),
    SetMaximumFrequency(f32),

    ///Get
    GetRunningFrequency(),
    GetEepromFrequency(),
    GetMaximumFrequency(),
    GetMinimumFrequency(),

    /// Motor Control
    // true is forward rotation, false reverse rotation
    // Set Rotation also starts the motor
    SetRotation(bool),
    StopMotor(),
    SetTargetPressure(f32),
    SetTargetRpm(f32),
    SetRegulation(bool),

    /// Inverter Control
    SetOperationMode(u8),
    WriteParameter(u16, u16),
    ReadParameter(u16),

    // Clears
    ClearAllParameters(),
    ClearParameter(),
    ClearNonCommunicationParameter(),
    ClearNonCommunicationParameters(),
    // Extruder Control
    //Mode
    SetMode(ExtruderV2Mode),

    //Heating
    SetHeatingFront(Heating),
    SetHeatingBack(Heating),
    SetHeatingMiddle(Heating),
    SetHeatingNozzle(Heating),

    SetFrontHeatingTemperature(f32),
    SetBackHeatingTemperature(f32),
    SetMiddleHeatingTemperature(f32),
    SetNozzleHeatingTemperature(f32),
}

#[derive(Debug)]
pub struct ExtruderV2Namespace(Namespace);

impl NamespaceCacheingLogic<ExtruderV2Events> for ExtruderV2Namespace {
    fn emit_cached(&mut self, events: ExtruderV2Events) {
        let event = match events.event_value() {
            Ok(event) => event,
            Err(err) => {
                log::error!(
                    "[{}::emit_cached] Failed to event.event_value(): {:?}",
                    module_path!(),
                    err
                );
                return;
            }
        };
        let buffer_fn = events.event_cache_fn();
        self.0.emit_cached(&event, &buffer_fn);
    }
}

impl ExtruderV2Namespace {
    pub fn new() -> Self {
        Self(Namespace::new())
    }
}

impl CacheableEvents<ExtruderV2Events> for ExtruderV2Events {
    fn event_value(&self) -> Result<GenericEvent, serde_json::Error> {
        match self {
            ExtruderV2Events::InverterStateEvent(event) => event.try_into(),
            ExtruderV2Events::InverterModeEvent(event) => event.try_into(),
            ExtruderV2Events::InverterErrorEvent(event) => event.try_into(),
            ExtruderV2Events::InverterFrequencyEvent(event) => event.try_into(),
            ExtruderV2Events::InverterSuccessEvent(event) => event.try_into(),
            ExtruderV2Events::RotationStateEvent(event) => event.try_into(),
            ExtruderV2Events::ModeEvent(event) => event.try_into(),
            ExtruderV2Events::RegulationStateEvent(event) => event.try_into(),
            ExtruderV2Events::PressureStateEvent(event) => event.try_into(),
            ExtruderV2Events::RpmStateEvent(event) => event.try_into(),
            ExtruderV2Events::HeatingStateEvent(event) => event.try_into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        let _cache_one_hour = cache_duration(Duration::from_secs(60 * 60), Duration::from_secs(1));
        let _cache_ten_secs = cache_duration(Duration::from_secs(10), Duration::from_secs(1));
        let cache_one = cache_one_event();

        match self {
            ExtruderV2Events::InverterStateEvent(_) => todo!(),
            ExtruderV2Events::InverterModeEvent(_) => todo!(),
            ExtruderV2Events::InverterErrorEvent(_) => todo!(),
            ExtruderV2Events::InverterFrequencyEvent(_) => {
                todo!()
            }
            ExtruderV2Events::InverterSuccessEvent(_) => todo!(),
            ExtruderV2Events::RotationStateEvent(_) => cache_one,
            ExtruderV2Events::ModeEvent(_) => cache_one,
            ExtruderV2Events::RegulationStateEvent(_) => cache_one,
            ExtruderV2Events::PressureStateEvent(_) => cache_one,
            ExtruderV2Events::RpmStateEvent(_) => cache_one,
            ExtruderV2Events::HeatingStateEvent(_) => cache_one,
        }
    }
}

impl MachineApi for ExtruderV2 {
    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        // there are multiple Modbus Frames that are "prebuilt"
        let control: Mutation = serde_json::from_value(request_body)?;
        match control {
            Mutation::SetRunningFrequency(_) => todo!(),
            Mutation::SetEepromFrequency(_) => todo!(),
            Mutation::SetMinimumFrequency(_) => todo!(),
            Mutation::SetMaximumFrequency(_) => todo!(),
            Mutation::GetRunningFrequency() => todo!(),
            Mutation::GetEepromFrequency() => todo!(),
            Mutation::GetMaximumFrequency() => todo!(),
            Mutation::GetMinimumFrequency() => todo!(),
            Mutation::StopMotor() => todo!(),
            Mutation::SetOperationMode(_) => todo!(),
            Mutation::WriteParameter(_, _) => todo!(),
            Mutation::ReadParameter(_) => todo!(),
            Mutation::ClearAllParameters() => todo!(),
            Mutation::ClearParameter() => todo!(),
            Mutation::ClearNonCommunicationParameter() => todo!(),
            Mutation::ClearNonCommunicationParameters() => todo!(),
            Mutation::SetMode(mode) => self.set_mode_state(mode),
            Mutation::SetRotation(forward) => self.set_rotation_state(forward),
            Mutation::SetHeatingFront(heating) => self.set_heating_front(heating),
            Mutation::SetHeatingMiddle(heating) => self.set_heating_middle(heating),
            Mutation::SetHeatingBack(heating) => self.set_heating_back(heating),
            Mutation::SetHeatingNozzle(heating) => self.set_heating_nozzle(heating),
            Mutation::SetRegulation(uses_rpm) => self.set_regulation(uses_rpm),
            Mutation::SetTargetPressure(bar) => self.set_target_pressure(bar),
            Mutation::SetTargetRpm(rpm) => self.set_target_rpm(rpm),
            Mutation::SetFrontHeatingTemperature(temp) => {
                self.set_target_temperature(temp, HeatingType::Front)
            }
            Mutation::SetMiddleHeatingTemperature(temp) => {
                self.set_target_temperature(temp, HeatingType::Middle)
            }
            Mutation::SetBackHeatingTemperature(temp) => {
                self.set_target_temperature(temp, HeatingType::Back)
            }
            Mutation::SetNozzleHeatingTemperature(temp) => {
                self.set_target_temperature(temp, HeatingType::Nozzle)
            }
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> &mut dyn NamespaceInterface {
        &mut self.namespace.0
    }
}
