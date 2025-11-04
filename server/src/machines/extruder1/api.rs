use super::{ExtruderV2Mode, mitsubishi_cs80::MotorStatus};

#[cfg(not(feature = "mock-machine"))]
use super::ExtruderV2;

#[cfg(not(feature = "mock-machine"))]
use crate::machines::extruder1::HeatingType;

#[cfg(not(feature = "mock-machine"))]
use control_core::machines::api::MachineApi;
use control_core::socketio::{
    event::{Event, GenericEvent},
    namespace::{
        CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_duration,
        cache_first_and_last_event,
    },
};
use control_core_derive::BuildEvent;
use serde::{Deserialize, Serialize};
#[cfg(not(feature = "mock-machine"))]
use serde_json::Value;
use smol::lock::Mutex;
use std::{sync::Arc, time::Duration};
use tracing::instrument;
use uom::si::{
    angular_velocity::revolution_per_minute, electric_current::ampere, electric_potential::volt,
    frequency::hertz,
};

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
pub struct RotationState {
    pub forward: bool,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ModeState {
    pub mode: ExtruderV2Mode,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct RegulationState {
    pub uses_rpm: bool,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct PressureState {
    pub target_bar: f64,
    pub wiring_error: bool,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct ScrewState {
    pub target_rpm: f64,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct HeatingStates {
    pub nozzle: HeatingState,
    pub front: HeatingState,
    pub back: HeatingState,
    pub middle: HeatingState,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct HeatingState {
    pub target_temperature: f64,
    pub wiring_error: bool,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct ExtruderSettingsState {
    pub pressure_limit: f64,
    pub pressure_limit_enabled: bool,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct InverterStatusState {
    /// RUN (Inverter running)
    pub running: bool,
    /// Forward running motor spins forward
    pub forward_running: bool,
    /// Reverse running motor spins backwards
    pub reverse_running: bool,
    /// Up to frequency, SU not completely sure what its for
    pub up_to_frequency: bool,
    /// overload warning OL
    pub overload_warning: bool,
    /// No function, its described that way in the datasheet
    pub no_function: bool,
    /// FU Output Frequency Detection
    pub output_frequency_detection: bool,
    /// ABC (Fault)
    pub abc_fault: bool,
    /// is True when a fault occured
    pub fault_occurence: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct PidSettings {
    pub ki: f64,
    pub kp: f64,
    pub kd: f64,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct TemperaturePidStates {
    pub front: TemperaturePid,
    pub middle: TemperaturePid,
    pub back: TemperaturePid,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct TemperaturePid {
    pub ki: f64,
    pub kp: f64,
    pub kd: f64,
    pub zone: String,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct PidSettingsStates {
    pub temperature: TemperaturePidStates,
    pub pressure: PidSettings,
}

pub enum ExtruderV2Events {
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
    SetExtruderMode(ExtruderV2Mode),
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
pub struct ExtruderV2Namespace {
    pub namespace: Arc<Mutex<Namespace>>,
}

impl NamespaceCacheingLogic<ExtruderV2Events> for ExtruderV2Namespace {
    #[instrument(skip_all)]
    fn emit(&mut self, events: ExtruderV2Events) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();

        let mut namespace = self.namespace.lock_blocking();
        namespace.emit(event, &buffer_fn);
    }
}

impl CacheableEvents<Self> for ExtruderV2Events {
    fn event_value(&self) -> GenericEvent {
        match self {
            Self::LiveValues(event) => event.into(),
            Self::State(event) => event.into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        let cache_one_hour = cache_duration(Duration::from_secs(60 * 60), Duration::from_secs(1));
        let cache_first_and_last = cache_first_and_last_event();

        match self {
            Self::LiveValues(_) => cache_one_hour,
            Self::State(_) => cache_first_and_last,
        }
    }
}

#[cfg(not(feature = "mock-machine"))]
impl MachineApi for ExtruderV2 {
    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        // there are multiple Modbus Frames that are "prebuilt"
        let control: Mutation = serde_json::from_value(request_body)?;
        match control {
            Mutation::SetExtruderMode(mode) => self.set_mode_state(mode),
            Mutation::SetInverterRotationDirection(forward) => self.set_rotation_state(forward),
            Mutation::SetInverterRegulation(uses_rpm) => self.set_regulation(uses_rpm),
            Mutation::SetInverterTargetPressure(pressure) => self.set_target_pressure(pressure),
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

    fn api_event_namespace(&mut self) -> Arc<Mutex<Namespace>> {
        self.namespace.namespace.clone()
    }

    fn api_event(
        &mut self,
        events: Option<&control_core::rest::mutation::EventFields>,
    ) -> Result<Value, anyhow::Error> {
        use uom::si::electric_current::ampere;
        use uom::si::electric_potential::volt;
        use uom::si::pressure::bar;
        use uom::si::thermodynamic_temperature::degree_celsius;

        let combined_power = {
            let motor_status = &self.screw_speed_controller.inverter.motor_status;
            let voltage = motor_status.voltage.get::<volt>();
            let current = motor_status.current.get::<ampere>();
            let motor_power = voltage * current;
            motor_power
                + self
                    .temperature_controller_nozzle
                    .get_heating_element_wattage()
                + self
                    .temperature_controller_front
                    .get_heating_element_wattage()
                + self
                    .temperature_controller_back
                    .get_heating_element_wattage()
                + self
                    .temperature_controller_middle
                    .get_heating_element_wattage()
        };

        let live_values = LiveValuesEvent {
            motor_status: self.screw_speed_controller.get_motor_status().into(),
            pressure: self.screw_speed_controller.get_pressure().get::<bar>(),
            nozzle_temperature: self
                .temperature_controller_nozzle
                .heating
                .temperature
                .get::<degree_celsius>(),
            front_temperature: self
                .temperature_controller_front
                .heating
                .temperature
                .get::<degree_celsius>(),
            back_temperature: self
                .temperature_controller_back
                .heating
                .temperature
                .get::<degree_celsius>(),
            middle_temperature: self
                .temperature_controller_middle
                .heating
                .temperature
                .get::<degree_celsius>(),
            nozzle_power: self
                .temperature_controller_nozzle
                .get_heating_element_wattage(),
            front_power: self
                .temperature_controller_front
                .get_heating_element_wattage(),
            back_power: self
                .temperature_controller_back
                .get_heating_element_wattage(),
            middle_power: self
                .temperature_controller_middle
                .get_heating_element_wattage(),
            combined_power,
            total_energy_kwh: self.total_energy_kwh,
        };

        let state = self.build_state_event();

        // Build response with requested events and fields
        let mut result = serde_json::Map::new();

        // Determine which events to include
        let (include_live_values, live_values_fields) = match events {
            None => (true, None), // Include all events with all fields
            Some(ef) => (ef.live_values.is_some(), ef.live_values.as_ref()),
        };

        let (include_state, state_fields) = match events {
            None => (true, None),
            Some(ef) => (ef.state.is_some(), ef.state.as_ref()),
        };

        // Add LiveValues if requested
        if include_live_values {
            let live_values_json = serde_json::to_value(live_values)?;
            let filtered = crate::rest::event_filter::filter_event_fields(
                live_values_json,
                live_values_fields,
            )?;
            if !filtered.is_null() {
                result.insert("LiveValues".to_string(), filtered);
            }
        }

        // Add State if requested
        if include_state {
            let state_json = serde_json::to_value(state)?;
            let filtered =
                crate::rest::event_filter::filter_event_fields(state_json, state_fields)?;
            if !filtered.is_null() {
                result.insert("State".to_string(), filtered);
            }
        }

        Ok(Value::Object(result))
    }
}
