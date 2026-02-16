use std::sync::Arc;

use crate::extruder1::{
    api::{
        ExtruderSettingsState, HeatingStates, InverterStatusState, PidSettings, PidSettingsStates,
        PressureState, RegulationState, RotationState, ScrewState, TemperaturePid,
    },
    mitsubishi_cs80::MotorStatus,
};
#[cfg(not(feature = "mock-machine"))]
use crate::{MachineApi, MachineMessage};
use control_core::socketio::{
    event::{Event, GenericEvent},
    namespace::{
        CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_first_and_last_event,
    },
};
use control_core_derive::BuildEvent;
use serde::{Deserialize, Serialize};
#[cfg(not(feature = "mock-machine"))]
use serde_json::Value;
#[cfg(not(feature = "mock-machine"))]
use smol::channel::Sender;
use tracing::instrument;
use units::{
    angular_velocity::revolution_per_minute, electric_current::ampere, electric_potential::volt,
    frequency::hertz,
};

#[cfg(not(feature = "mock-machine"))]
use super::ExtruderV3;
use super::ExtruderV3Mode;

#[derive(Debug, Clone, Default, Serialize)]
pub struct MotorStatusValues {
    pub screw_rpm: f64, // rpm of motor
    pub frequency: f64, // frequency of motor
    pub voltage: f64,   // volt used for motor
    pub current: f64,   // current used for the motor
    pub power: f64,     // power in watts
}

impl From<MotorStatus> for MotorStatusValues {
    fn from(status: MotorStatus) -> Self {
        let voltage = status.voltage.get::<volt>();
        let current = status.current.get::<ampere>();

        Self {
            screw_rpm: status.rpm.get::<revolution_per_minute>(),
            frequency: status.frequency.get::<hertz>(),
            voltage,
            current,
            power: voltage * current,
        }
    }
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct LiveValuesEvent {
    /// screw rpm
    pub motor_status: MotorStatusValues,
    /// pressure in bar
    pub pressure: f64,
    /// nozzle temperature in celsius
    pub nozzle_temperature: f64,
    /// front temperature in celsius
    pub front_temperature: f64,
    /// back temperature in celsius
    pub back_temperature: f64,
    /// middle temperature in celsius
    pub middle_temperature: f64,
    /// nozzle heating power in watts
    pub nozzle_power: f64,
    /// front heating power in watts
    pub front_power: f64,
    /// back heating power in watts
    pub back_power: f64,
    /// middle heating power in watts
    pub middle_power: f64,
    /// combined power consumption in watts
    pub combined_power: f64,
    /// total energy consumption in kWh
    pub total_energy_kwh: f64,
}

impl LiveValuesEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("LiveValuesEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone, PartialEq, BuildEvent)]
pub struct StateEvent {
    pub is_default_state: bool,
    /// rotation state
    pub rotation_state: RotationState,
    /// mode state
    pub mode_state: ModeState,
    /// regulation state
    pub regulation_state: RegulationState,
    /// pressure state
    pub pressure_state: PressureState,
    /// screw state
    pub screw_state: ScrewState,
    /// heating states
    pub heating_states: HeatingStates,
    /// extruder settings state
    pub extruder_settings_state: ExtruderSettingsState,
    /// inverter status state
    pub inverter_status_state: InverterStatusState,
    /// pid settings
    pub pid_settings: PidSettingsStates,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ModeState {
    pub mode: ExtruderV3Mode,
}

pub enum ExtruderV3Events {
    LiveValues(Event<LiveValuesEvent>),
    State(Event<StateEvent>),
}

#[derive(Deserialize, Serialize)]
pub enum Mutation {
    /// INVERTER
    /// Frequency Control
    // Set Rotation also starts the motor
    SetInverterRotationDirection(bool),
    SetInverterTargetPressure(f64),
    SetInverterTargetRpm(f64),
    SetInverterRegulation(bool),

    //Mode
    SetExtruderMode(ExtruderV3Mode),
    SetFrontHeatingTargetTemperature(f64),
    SetBackHeatingTargetTemperature(f64),
    SetMiddleHeatingTemperature(f64),
    SetNozzleHeatingTemperature(f64),

    // SetPressure
    SetExtruderPressureLimit(f64),
    SetExtruderPressureLimitIsEnabled(bool),

    // Pid Configure
    SetPressurePidSettings(PidSettings),
    SetTemperaturePidSettings(TemperaturePid),

    // Reset
    ResetInverter(bool),
}

#[derive(Debug)]
pub struct ExtruderV3Namespace {
    pub namespace: Option<Namespace>,
}

impl NamespaceCacheingLogic<ExtruderV3Events> for ExtruderV3Namespace {
    #[instrument(skip_all)]
    fn emit(&mut self, events: ExtruderV3Events) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();

        match &mut self.namespace {
            Some(ns) => ns.emit(event, &buffer_fn),
            None => (),
        }
    }
}

impl CacheableEvents<Self> for ExtruderV3Events {
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

#[cfg(not(feature = "mock-machine"))]
impl MachineApi for ExtruderV3 {
    fn api_get_sender(&self) -> Sender<MachineMessage> {
        self.api_sender.clone()
    }

    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        // there are multiple Modbus Frames that are "prebuilt"
        use crate::extruder1::HeatingType;
        let control: Mutation = serde_json::from_value(request_body)?;
        match control {
            Mutation::SetExtruderMode(mode) => self.set_mode_state(mode),
            Mutation::SetInverterRotationDirection(forward) => self.set_rotation_state(forward),
            Mutation::SetInverterRegulation(uses_rpm) => self.set_regulation(uses_rpm),
            Mutation::SetInverterTargetPressure(bar) => self.set_target_pressure(bar),
            Mutation::SetInverterTargetRpm(rpm) => self.set_target_rpm(rpm),
            Mutation::ResetInverter(_) => self.reset_inverter(),

            Mutation::SetFrontHeatingTargetTemperature(temp) => {
                self.set_target_temperature(temp, HeatingType::Front)
            }
            Mutation::SetMiddleHeatingTemperature(temp) => {
                self.set_target_temperature(temp, HeatingType::Middle)
            }
            Mutation::SetBackHeatingTargetTemperature(temp) => {
                self.set_target_temperature(temp, HeatingType::Back)
            }
            Mutation::SetNozzleHeatingTemperature(temp) => {
                self.set_target_temperature(temp, HeatingType::Nozzle)
            }
            Mutation::SetExtruderPressureLimit(pressure_limit) => {
                self.set_nozzle_pressure_limit(pressure_limit);
            }
            Mutation::SetExtruderPressureLimitIsEnabled(enabled) => {
                self.set_nozzle_pressure_limit_is_enabled(enabled);
            }

            Mutation::SetPressurePidSettings(settings) => {
                self.configure_pressure_pid(settings);
            }

            Mutation::SetTemperaturePidSettings(settings) => {
                self.configure_temperature_pid(settings);
            }
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }
}
