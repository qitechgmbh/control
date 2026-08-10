use super::{AquaPathV1Mode, controller::CoolingMode};
use qitech_framework::{machine::{Measurement, StateProperty}};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Clone, Default)]
pub struct Measurements {

    pub left_flow: Measurement<f64>,
    pub right_flow: Measurement<f64>,
    
    pub left_temperature: Measurement<f64>,
    pub right_temperature: Measurement<f64>,

    pub left_revolutions: Measurement<f64>,
    pub right_revolutions: Measurement<f64>,

    pub left_power: Measurement<f64>,
    pub right_power: Measurement<f64>,
    
    pub left_heating: Measurement<bool>,
    pub right_heating: Measurement<bool>,
    
    pub left_cooling_mode: Measurement<Option<CoolingMode>>,
    pub right_cooling_mode: Measurement<Option<CoolingMode>>,
    
    pub left_pump_cooldown_active: Measurement<bool>,
    pub right_pump_cooldown_active: Measurement<bool>,
    
    pub left_pump_cooldown_remaining: Measurement<f64>,
    pub right_pump_cooldown_remaining: Measurement<f64>,
    
    pub left_heating_startup_wait_active: Measurement<bool>,
    pub right_heating_startup_wait_active: Measurement<bool>,
    
    pub left_heating_startup_wait_remaining: Measurement<f64>,
    pub right_heating_startup_wait_remaining: Measurement<f64>,
    
    pub left_total_energy: Measurement<f64>,
    pub right_total_energy: Measurement<f64>,
}





#[derive(Serialize, Debug, Clone)]
pub struct StateProperties {
    pub is_default_state: StateProperty<bool>,
    /// mode state
    pub mode_state: StateProperty<ModeState>,
    pub ambient_temperature_calibration: StateProperty<f64>,
    pub default_heating_tolerance: StateProperty<f64>,
    pub default_cooling_tolerance: StateProperty<f64>,
    pub default_pid_kp: StateProperty<f64>,
    pub default_pid_ki: StateProperty<f64>,
    pub default_pid_kd: StateProperty<f64>,
    pub flow_states: FlowStates,
    pub temperature_states: TempStates,
    pub fan_states: FanStates,
    pub cooling_mode_states: CoolingModeStates,
    pub tolerance_states: ToleranceStates,
    pub pid_states: PidStates,
    pub thermal_safety_states: ThermalSafetyStates,
}


#[derive(Serialize, Debug, Clone)]
pub struct NoticeEvent {
    pub title: String,
    pub message: String,
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