use super::{AquaPathV1, AquaPathV1Mode, controller::CoolingMode};
use crate::{MachineApi, MachineMessage, MachineValues};
use control_core_legacy::socketio::{
    event::{Event, GenericEvent},
    namespace::{
        CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_first_and_last_event,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tracing::instrument;

#[derive(Serialize, Debug, Clone, Default)]
pub struct LiveValuesEvent {
    pub left_flow: f64,
    pub right_flow: f64,
    pub left_temperature: f64,
    pub right_temperature: f64,
    pub left_revolutions: f64,
    pub right_revolutions: f64,
    pub left_power: f64,
    pub right_power: f64,
    pub left_heating: bool,
    pub right_heating: bool,
    pub left_cooling_mode: Option<CoolingMode>,
    pub right_cooling_mode: Option<CoolingMode>,
    pub left_pump_cooldown_active: bool,
    pub right_pump_cooldown_active: bool,
    pub left_pump_cooldown_remaining: f64,
    pub right_pump_cooldown_remaining: f64,
    pub left_heating_startup_wait_active: bool,
    pub right_heating_startup_wait_active: bool,
    pub left_heating_startup_wait_remaining: f64,
    pub right_heating_startup_wait_remaining: f64,
    pub left_total_energy: f64,
    pub right_total_energy: f64,
}

impl LiveValuesEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("LiveValuesEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct StateEvent {
    pub is_default_state: bool,
    /// mode state
    pub mode_state: ModeState,
    pub ambient_temperature_calibration: f64,
    pub default_heating_tolerance: f64,
    pub default_cooling_tolerance: f64,
    pub default_pid_kp: f64,
    pub default_pid_ki: f64,
    pub default_pid_kd: f64,
    pub flow_states: FlowStates,
    pub temperature_states: TempStates,
    pub fan_states: FanStates,
    pub cooling_mode_states: CoolingModeStates,
    pub tolerance_states: ToleranceStates,
    pub pid_states: PidStates,
    pub thermal_safety_states: ThermalSafetyStates,
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct NoticeEvent {
    pub title: String,
    pub message: String,
}

impl NoticeEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("NoticeEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct TempStates {
    pub left: TempState,
    pub right: TempState,
}

#[derive(Serialize, Debug, Clone)]
pub struct TempState {
    pub temperature: f64,
    pub target_temperature: f64,
}

#[derive(Serialize, Debug, Clone)]
pub struct ModeState {
    pub mode: AquaPathV1Mode,
}
#[derive(Serialize, Debug, Clone)]
pub struct FlowStates {
    pub left: FlowState,
    pub right: FlowState,
}
#[derive(Serialize, Debug, Clone)]
pub struct FlowState {
    pub flow: f64,
    pub should_flow: bool,
}

#[derive(Serialize, Debug, Clone)]
pub struct FanState {
    pub revolutions: f64,
    pub max_revolutions: f64,
}
#[derive(Serialize, Debug, Clone)]
pub struct FanStates {
    pub left: FanState,
    pub right: FanState,
}

#[derive(Serialize, Debug, Clone)]
pub struct CoolingModeState {
    pub mode: Option<CoolingMode>,
}

#[derive(Serialize, Debug, Clone)]
pub struct CoolingModeStates {
    pub left: CoolingModeState,
    pub right: CoolingModeState,
}

#[derive(Serialize, Debug, Clone)]
pub struct ToleranceState {
    pub heating: f64,
    pub cooling: f64,
}
#[derive(Serialize, Debug, Clone)]
pub struct ToleranceStates {
    pub left: ToleranceState,
    pub right: ToleranceState,
}

#[derive(Serialize, Debug, Clone)]
pub struct PidState {
    pub kp: f64,
    pub ki: f64,
    pub kd: f64,
}

#[derive(Serialize, Debug, Clone)]
pub struct PidStates {
    pub left: PidState,
    pub right: PidState,
}

#[derive(Serialize, Debug, Clone)]
pub struct ThermalSafetyState {
    pub thermal_delay: f64,
    pub cooldown_min_temperature: f64,
}

#[derive(Serialize, Debug, Clone)]
pub struct ThermalSafetyStates {
    pub left: ThermalSafetyState,
    pub right: ThermalSafetyState,
}

pub enum AquaPathV1Events {
    LiveValues(Event<LiveValuesEvent>),
    State(Event<StateEvent>),
    Notice(Event<NoticeEvent>),
}

#[derive(Deserialize, Serialize)]
enum Mutation {
    //Mode
    SetAquaPathMode(AquaPathV1Mode),

    SetLeftTemperature(f64),
    SetRightTemperature(f64),

    SetLeftFlow(bool),
    SetRightFlow(bool),

    SetLeftRevolutions(f64),
    SetRightRevolutions(f64),

    SetLeftHeatingTolerance(f64),
    SetRightHeatingTolerance(f64),
    SetLeftCoolingTolerance(f64),
    SetRightCoolingTolerance(f64),
    SetLeftPidKp(f64),
    SetLeftPidKi(f64),
    SetLeftPidKd(f64),
    SetRightPidKp(f64),
    SetRightPidKi(f64),
    SetRightPidKd(f64),
    SetLeftThermalFlowSettleDuration(f64),
    SetRightThermalFlowSettleDuration(f64),
    SetLeftPumpCooldownMinTemperature(f64),
    SetRightPumpCooldownMinTemperature(f64),
    SetAmbientTemperatureCalibration(f64),
}

#[derive(Debug, Clone)]
pub struct AquaPathV1Namespace {
    pub namespace: Option<Namespace>,
}

impl NamespaceCacheingLogic<AquaPathV1Events> for AquaPathV1Namespace {
    #[instrument(skip_all)]
    fn emit(&mut self, events: AquaPathV1Events) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        match &mut self.namespace {
            Some(ns) => ns.emit(event, &buffer_fn),
            None => (),
        }
    }
}

impl CacheableEvents<AquaPathV1Events> for AquaPathV1Events {
    fn event_value(&self) -> GenericEvent {
        match self {
            AquaPathV1Events::LiveValues(event) => event.into(),
            AquaPathV1Events::State(event) => event.into(),
            AquaPathV1Events::Notice(event) => event.into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        let cache_first_and_last = cache_first_and_last_event();

        match self {
            AquaPathV1Events::LiveValues(_) => cache_first_and_last,
            AquaPathV1Events::State(_) => cache_first_and_last,
            AquaPathV1Events::Notice(_) => Box::new(|_, _| {}),
        }
    }
}

impl MachineApi for AquaPathV1 {
    fn act_machine_message(&mut self, msg: MachineMessage) {
        match msg {
            MachineMessage::SubscribeNamespace(namespace) => {
                self.namespace.namespace = Some(namespace);
                self.emit_state();
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

    fn get_api_sender(&self) -> tokio::sync::mpsc::Sender<MachineMessage> {
        self.api_sender.clone()
    }

    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        let control: Mutation = serde_json::from_value(request_body)?;
        match control {
            Mutation::SetAquaPathMode(mode) => self.set_mode_state(mode),
            Mutation::SetRightTemperature(temperature) => {
                self.set_target_temperature(temperature, super::AquaPathSideType::Right)
            }

            Mutation::SetLeftTemperature(temperature) => {
                self.set_target_temperature(temperature, super::AquaPathSideType::Left)
            }

            Mutation::SetRightFlow(should_pump) => {
                self.set_should_pump(should_pump, super::AquaPathSideType::Right)
            }
            Mutation::SetLeftFlow(should_pump) => {
                self.set_should_pump(should_pump, super::AquaPathSideType::Left)
            }
            Mutation::SetRightRevolutions(revolution) => {
                self.set_max_revolutions(revolution, super::AquaPathSideType::Right)
            }
            Mutation::SetLeftRevolutions(revolutions) => {
                self.set_max_revolutions(revolutions, super::AquaPathSideType::Left)
            }
            Mutation::SetRightHeatingTolerance(tolerance) => {
                self.set_heating_tolerance(tolerance, super::AquaPathSideType::Right)
            }
            Mutation::SetLeftHeatingTolerance(tolerance) => {
                self.set_heating_tolerance(tolerance, super::AquaPathSideType::Left)
            }
            Mutation::SetRightCoolingTolerance(tolerance) => {
                self.set_cooling_tolerance(tolerance, super::AquaPathSideType::Right);
            }
            Mutation::SetLeftCoolingTolerance(tolerance) => {
                self.set_cooling_tolerance(tolerance, super::AquaPathSideType::Left);
            }
            Mutation::SetLeftPidKp(value) => {
                self.set_pid_kp(value, super::AquaPathSideType::Left);
            }
            Mutation::SetLeftPidKi(value) => {
                self.set_pid_ki(value, super::AquaPathSideType::Left);
            }
            Mutation::SetLeftPidKd(value) => {
                self.set_pid_kd(value, super::AquaPathSideType::Left);
            }
            Mutation::SetRightPidKp(value) => {
                self.set_pid_kp(value, super::AquaPathSideType::Right);
            }
            Mutation::SetRightPidKi(value) => {
                self.set_pid_ki(value, super::AquaPathSideType::Right);
            }
            Mutation::SetRightPidKd(value) => {
                self.set_pid_kd(value, super::AquaPathSideType::Right);
            }
            Mutation::SetLeftThermalFlowSettleDuration(value) => {
                self.set_thermal_flow_settle_duration(value, super::AquaPathSideType::Left);
            }
            Mutation::SetRightThermalFlowSettleDuration(value) => {
                self.set_thermal_flow_settle_duration(value, super::AquaPathSideType::Right);
            }
            Mutation::SetLeftPumpCooldownMinTemperature(value) => {
                self.set_pump_cooldown_min_temperature(value, super::AquaPathSideType::Left);
            }
            Mutation::SetRightPumpCooldownMinTemperature(value) => {
                self.set_pump_cooldown_min_temperature(value, super::AquaPathSideType::Right);
            }
            Mutation::SetAmbientTemperatureCalibration(ambient_temp) => {
                self.set_ambient_temperature_calibration(ambient_temp);
            }
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }
}
