use std::sync::Arc;

use super::{ExtruderV2Mode, mitsubishi_cs80::MotorStatus};

#[cfg(not(feature = "mock-machine"))]
use super::ExtruderV2;

#[cfg(not(feature = "mock-machine"))]
use crate::{MachineMessage, extruder1::HeatingType};

#[cfg(not(feature = "mock-machine"))]
use crate::{MachineApi, MachineValues};
use control_core::socketio::{
    event::{Event, GenericEvent},
    namespace::{
        CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_first_and_last_event,
    },
};
use qitech_lib::units::angular_velocity::revolution_per_minute;
use qitech_lib::units::electric_current::ampere;
use qitech_lib::units::electric_potential::volt;
use qitech_lib::units::frequency::hertz;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::instrument;

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

#[derive(Serialize, Debug, Clone, PartialEq)]
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
    /// fixed heating power overrides (debug/test)
    pub heating_power_override_states: HeatingPowerOverrideStates,
    /// extruder settings state
    pub extruder_settings_state: ExtruderSettingsState,
    /// inverter status state
    pub inverter_status_state: InverterStatusState,
    /// pid settings
    pub pid_settings: PidSettingsStates,
    /// pressure PID auto-tuner state
    pub pid_autotune_state: PidAutoTuneState,
    /// temperature (IMC step-test) auto-tuner state
    pub temperature_autotune_state: TemperatureAutoTuneState,
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
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

/// Debug/test override that pins one heating zone to a fixed output power instead of letting the
/// temperature PID regulate it.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct HeatingPowerOverrideState {
    /// Whether the zone is currently driven at a fixed power
    pub enabled: bool,
    /// Fixed heating power in watts
    pub watts: f64,
    /// Highest power the zone can be driven at, i.e. element wattage capped by the duty limit
    pub max_watts: f64,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct HeatingPowerOverrideStates {
    pub nozzle: HeatingPowerOverrideState,
    pub front: HeatingPowerOverrideState,
    pub back: HeatingPowerOverrideState,
    pub middle: HeatingPowerOverrideState,
}

/// Payload for setting the fixed-power debug override on a single heating zone.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct HeatingPowerOverride {
    /// One of `"front"`, `"middle"`, `"back"`, `"nozzle"`
    pub zone: String,
    pub enabled: bool,
    pub watts: f64,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct ExtruderSettingsState {
    pub pressure_limit: f64,
    pub pressure_limit_enabled: bool,
    pub nozzle_temperature_target_enabled: bool,
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
    pub nozzle: TemperaturePid,
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

/// Parameters for starting a pressure PID auto-tune run.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct PressureAutoTuneConfig {
    /// Pressure oscillation half-amplitude in bar (e.g. `0.5` for ±0.5 bar).
    /// Typical starting value: 0.5 – 2 bar depending on operating pressure.
    pub tune_delta: f64,
    /// Frequency step in Hz.  The inverter will oscillate between
    /// `(current_freq − frequency_step_hz)` and `(current_freq + frequency_step_hz)`.
    /// Keep this small relative to the steady-state operating frequency to avoid
    /// over-pressing the machine (e.g. 3 – 8 Hz for a typical extruder).
    pub frequency_step_hz: f64,
}

/// Live state of the pressure PID auto-tuner, broadcast as part of the machine
/// state.  The `result` field is populated once `state == "completed"`.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct PidAutoTuneState {
    /// One of: `"not_started"`, `"running"`, `"completed"`, `"failed"`
    pub state: String,
    /// Progress percentage in the range 0 – 100
    pub progress: f64,
    /// Computed PID parameters – only present after a successful run
    pub result: Option<PidSettings>,
}

impl Default for PidAutoTuneState {
    fn default() -> Self {
        Self {
            state: "not_started".to_string(),
            progress: 0.0,
            result: None,
        }
    }
}

/// Parameters for starting a temperature PID auto-tune run on one heating zone.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct TemperatureAutoTuneConfig {
    /// Zone to tune: `"nozzle"`, `"front"`, `"back"` or `"middle"`.
    pub zone: String,
    /// Step in duty cycle, as a fraction of full output (e.g. `0.10` for +10 percentage points).
    /// Signed; negative is allowed when there is no upward headroom.
    pub step_duty: f64,
    /// Abort if the zone moves further than this from its baseline temperature.
    pub max_rise_celsius: f64,
    /// Closed-loop time constant as a multiple of the identified process time constant.
    /// 0.5 aggressive, 1.0 moderate, 2.0 conservative.
    pub lambda_factor: f64,
}

/// One tuning candidate, in both IMC and parallel-PID parameterisations.
#[derive(Serialize, Debug, Clone, PartialEq, Default)]
pub struct ImcGainsState {
    pub kc: f64,
    pub ti: f64,
    pub td: f64,
    pub kp: f64,
    pub ki: f64,
    pub kd: f64,
}

impl From<control_core::controllers::imc_tuner::ImcGains> for ImcGainsState {
    fn from(g: control_core::controllers::imc_tuner::ImcGains) -> Self {
        Self {
            kc: g.kc,
            ti: g.ti,
            td: g.td,
            kp: g.kp,
            ki: g.ki,
            kd: g.kd,
        }
    }
}

/// Identified model, fit diagnostics and both gain candidates from a completed run.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct TemperatureAutoTuneResultState {
    /// Steady-state gain in °C per unit of duty cycle.
    pub process_gain: f64,
    /// Fitted time constant in seconds.
    pub time_constant: f64,
    /// Fitted dead time in seconds.
    pub dead_time: f64,
    /// Classical 63.2% construction, shown as a cross-check.
    pub tau_63: f64,
    /// Dead time from the first threshold crossing, shown as a cross-check. Expect it above the
    /// fitted value.
    pub dead_time_threshold: f64,
    pub rms_residual: f64,
    pub fit_error_pct: f64,
    pub is_good_fit: bool,
    pub delta_pv: f64,
    pub delta_u: f64,
    pub lambda: f64,
    pub noise_peak_to_peak: f64,
    pub snr_ratio: f64,
    pub is_confident: bool,
    /// Step that would have reached the target signal-to-noise ratio, to guide a retry.
    pub suggested_step_duty: f64,
    pub pi: ImcGainsState,
    pub pid: ImcGainsState,
}

/// Live state of the temperature auto-tuner, broadcast as part of the machine state.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct TemperatureAutoTuneState {
    /// Zone being tuned, if a run has been started.
    pub zone: Option<String>,
    /// One of `"idle"`, `"waiting_for_steady"`, `"baseline_hold"`, `"step"`, `"completed"`,
    /// `"failed"`.
    pub phase: String,
    pub progress: f64,
    pub elapsed_seconds: f64,
    pub baseline_duty: f64,
    pub baseline_temperature: f64,
    pub current_duty: f64,
    pub result: Option<TemperatureAutoTuneResultState>,
    pub failure_reason: Option<String>,
}

impl Default for TemperatureAutoTuneState {
    fn default() -> Self {
        Self {
            zone: None,
            phase: "idle".to_string(),
            progress: 0.0,
            elapsed_seconds: 0.0,
            baseline_duty: 0.0,
            baseline_temperature: 0.0,
            current_duty: 0.0,
            result: None,
            failure_reason: None,
        }
    }
}

/// One recorded point of a step test.
#[derive(Serialize, Debug, Clone, Copy, PartialEq)]
pub struct TemperatureAutoTuneSample {
    pub t_seconds: f64,
    pub temperature: f64,
    pub duty: f64,
}

/// The recorded step-test curve.
///
/// Kept out of [`StateEvent`], which is rebuilt and re-emitted on every mutation — a 30-minute run
/// is ~1800 samples. Sent whole rather than incrementally so the frontend stays stateless and the
/// curve survives a page reload.
#[derive(Serialize, Debug, Clone, Default)]
pub struct TemperatureAutoTuneTraceEvent {
    pub zone: Option<String>,
    pub phase: String,
    pub samples: Vec<TemperatureAutoTuneSample>,
}

impl TemperatureAutoTuneTraceEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("TemperatureAutoTuneTraceEvent", self.clone())
    }
}

pub enum ExtruderV2Events {
    LiveValues(Event<LiveValuesEvent>),
    State(Event<StateEvent>),
    TuneTrace(Event<TemperatureAutoTuneTraceEvent>),
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

    // Pressure PID Auto-Tune
    /// Start pressure PID auto-tuning with bounded frequency excitation.
    StartPressurePidAutoTune(PressureAutoTuneConfig),
    StopPressurePidAutoTune {},

    // Reset
    ResetInverter(bool),

    // Toggle nozzle temperature target
    SetNozzleTemperatureTargetEnabled(bool),

    // Debug/test: drive one heating zone at a fixed wattage instead of regulating it
    SetHeatingPowerOverride(HeatingPowerOverride),

    // Temperature PID Auto-Tune (IMC step test), one zone at a time
    /// Start an IMC step test on one heating zone. Requires `Heat` mode with the screw stopped.
    StartTemperaturePidAutoTune(TemperatureAutoTuneConfig),
    StopTemperaturePidAutoTune {},
    /// Push a completed run's gains into the tuned zone's PID.
    /// `form` selects the candidate: `"pi"` or `"pid"`.
    ApplyTemperatureAutoTuneResult {
        form: String,
    },
}

#[derive(Debug)]
pub struct ExtruderV2Namespace {
    pub namespace: Option<Namespace>,
}

impl NamespaceCacheingLogic<ExtruderV2Events> for ExtruderV2Namespace {
    #[instrument(skip_all)]
    fn emit(&mut self, events: ExtruderV2Events) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();

        match &mut self.namespace {
            Some(ns) => ns.emit(event, &buffer_fn),
            None => (),
        }
    }
}

impl CacheableEvents<Self> for ExtruderV2Events {
    fn event_value(&self) -> GenericEvent {
        match self {
            Self::LiveValues(event) => event.into(),
            Self::State(event) => event.into(),
            Self::TuneTrace(event) => event.into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        let cache_first_and_last = cache_first_and_last_event();
        match self {
            Self::LiveValues(_) => cache_first_and_last,
            Self::State(_) => cache_first_and_last,
            Self::TuneTrace(_) => cache_first_and_last,
        }
    }
}

#[cfg(not(feature = "mock-machine"))]
impl MachineApi for ExtruderV2 {
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

    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        // there are multiple Modbus Frames that are "prebuilt"
        let control: Mutation = serde_json::from_value(request_body)?;
        match control {
            Mutation::SetExtruderMode(mode) => {
                // This might look like an inconvenient borrow, however remember that the machines
                // Have full control over when the api_mutate is actually executed!
                let relais_out = self.get_relais();
                self.set_mode_state(mode, &mut *relais_out.borrow_mut());
            }
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
            Mutation::SetNozzleTemperatureTargetEnabled(enabled) => {
                self.set_nozzle_temperature_target_is_enabled(enabled);
            }
            Mutation::SetHeatingPowerOverride(override_settings) => {
                self.set_heating_power_override(override_settings);
            }
            Mutation::StartTemperaturePidAutoTune(config) => {
                self.start_temperature_pid_autotune(config);
            }
            Mutation::StopTemperaturePidAutoTune {} => {
                self.stop_temperature_pid_autotune();
            }
            Mutation::ApplyTemperatureAutoTuneResult { form } => {
                self.apply_temperature_autotune_result(&form);
            }
            Mutation::StartPressurePidAutoTune(config) => {
                self.start_pressure_pid_autotune(config);
            }
            Mutation::StopPressurePidAutoTune {} => {
                self.stop_pressure_pid_autotune();
            }
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }

    fn get_api_sender(&self) -> tokio::sync::mpsc::Sender<MachineMessage> {
        self.api_sender.clone()
    }
}
