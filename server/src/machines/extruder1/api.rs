use super::{ExtruderV2, ExtruderV2Mode, HeatingType, mitsubishi_cs80::MotorStatus};
use control_core::socketio::event::BuildEvent;
use control_core::{
    machines::api::MachineApi,
    socketio::{
        event::{Event, GenericEvent},
        namespace::{
            CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_duration,
            cache_first_and_last_event,
        },
    },
};
use control_core_derive::BuildEvent;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use smol::channel::Sender;
use socketioxide::extract::SocketRef;
use std::{sync::Arc, time::Duration};
use tracing::instrument;
use uom::si::{
    angular_velocity::revolution_per_minute, electric_current::ampere, electric_potential::volt,
    frequency::hertz,
};

#[derive(Debug, Clone, Default, Serialize)]
pub struct MotorStatusValues {
    screw_rpm: f64, // rpm of motor
    frequency: f64, // frequency of motor
    voltage: f64,   // volt used for motor
    current: f64,   // current used for the motor
    power: f64,     // power in watts
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

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct RotationState {
    pub forward: bool,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct ModeState {
    pub mode: ExtruderV2Mode,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
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

#[derive(Serialize, Debug, Clone, PartialEq)]
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

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct PidSettingsStates {
    pub temperature: PidSettings,
    pub pressure: PidSettings,
}

pub enum ExtruderV2Events {
    LiveValues(Event<LiveValuesEvent>),
    State(Event<StateEvent>),
}

#[derive(Deserialize, Serialize)]
enum Mutation {
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

    // Reset
    ResetInverter(bool),
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
            ExtruderV2Events::LiveValues(event) => event.into(),
            ExtruderV2Events::State(event) => event.into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        let cache_one_hour = cache_duration(Duration::from_secs(60 * 60), Duration::from_secs(1));
        let cache_first_and_last = cache_first_and_last_event();

        match self {
            ExtruderV2Events::LiveValues(_) => cache_one_hour,
            ExtruderV2Events::State(_) => cache_first_and_last,
        }
    }
}

impl MachineApi for ExtruderV2 {
    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        // there are multiple Modbus Frames that are "prebuilt"
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
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> &mut Namespace {
        &mut self.namespace.namespace
    }
}
