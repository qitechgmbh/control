use qitech_framework::MachineInstanceIdentification;
use qitech_framework::RuntimeRequestKind;
use qitech_framework::ScalarValue;
use serde::Deserialize;
use crate::api::legacy::MachineLegacyDataAdapter;
use crate::api::types::MachineInstance;
use crate::machines::aquapath::AquaPathV1Mode;

pub const ADAPTER: MachineLegacyDataAdapter = MachineLegacyDataAdapter {
    convert_request,
    init_state_event,
    init_measurements_event,
};

fn convert_request(
    ident: MachineInstanceIdentification,
    data: serde_json::Value,
) -> Result<Vec<RuntimeRequestKind>, serde_json::Error> {
    #[derive(Deserialize)]
    #[allow(clippy::enum_variant_names)]
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
    
    let res = match serde_json::from_value(data)? {
    Mutation::SetAquaPathMode(v) => RuntimeRequestKind::SetConfigProperty {
        target: ident,
        path: "mode".to_string(),
        value: ScalarValue::Enum(v.to_string()), // Or ScalarValue::AquaPathV1Mode(v) depending on your type
    },
    Mutation::SetLeftTemperature(v) => RuntimeRequestKind::SetConfigProperty {
        target: ident,
        path: "left.temperature".to_string(),
        value: ScalarValue::Float(v),
    },
    Mutation::SetRightTemperature(v) => RuntimeRequestKind::SetConfigProperty {
        target: ident,
        path: "right.temperature".to_string(),
        value: ScalarValue::Float(v),
    },
    Mutation::SetLeftFlow(v) => RuntimeRequestKind::SetConfigProperty {
        target: ident,
        path: "left.flow".to_string(),
        value: ScalarValue::Boolean(v),
    },
    Mutation::SetRightFlow(v) => RuntimeRequestKind::SetConfigProperty {
        target: ident,
        path: "right.flow".to_string(),
        value: ScalarValue::Boolean(v),
    },
    Mutation::SetLeftRevolutions(v) => RuntimeRequestKind::SetConfigProperty {
        target: ident,
        path: "left.revolutions".to_string(),
        value: ScalarValue::Float(v),
    },
    Mutation::SetRightRevolutions(v) => RuntimeRequestKind::SetConfigProperty {
        target: ident,
        path: "right.revolutions".to_string(),
        value: ScalarValue::Float(v),
    },
    Mutation::SetLeftHeatingTolerance(v) => RuntimeRequestKind::SetConfigProperty {
        target: ident,
        path: "left.heating_tolerance".to_string(),
        value: ScalarValue::Float(v),
    },
    Mutation::SetRightHeatingTolerance(v) => RuntimeRequestKind::SetConfigProperty {
        target: ident,
        path: "right.heating_tolerance".to_string(),
        value: ScalarValue::Float(v),
    },
    Mutation::SetLeftCoolingTolerance(v) => RuntimeRequestKind::SetConfigProperty {
        target: ident,
        path: "left.cooling_tolerance".to_string(),
        value: ScalarValue::Float(v),
    },
    Mutation::SetRightCoolingTolerance(v) => RuntimeRequestKind::SetConfigProperty {
        target: ident,
        path: "right.cooling_tolerance".to_string(),
        value: ScalarValue::Float(v),
    },
    Mutation::SetLeftPidKp(v) => RuntimeRequestKind::SetConfigProperty {
        target: ident,
        path: "left.pid.kp".to_string(),
        value: ScalarValue::Float(v),
    },
    Mutation::SetLeftPidKi(v) => RuntimeRequestKind::SetConfigProperty {
        target: ident,
        path: "left.pid.ki".to_string(),
        value: ScalarValue::Float(v),
    },
    Mutation::SetLeftPidKd(v) => RuntimeRequestKind::SetConfigProperty {
        target: ident,
        path: "left.pid.kd".to_string(),
        value: ScalarValue::Float(v),
    },
    Mutation::SetRightPidKp(v) => RuntimeRequestKind::SetConfigProperty {
        target: ident,
        path: "right.pid.kp".to_string(),
        value: ScalarValue::Float(v),
    },
    Mutation::SetRightPidKi(v) => RuntimeRequestKind::SetConfigProperty {
        target: ident,
        path: "right.pid.ki".to_string(),
        value: ScalarValue::Float(v),
    },
    Mutation::SetRightPidKd(v) => RuntimeRequestKind::SetConfigProperty {
        target: ident,
        path: "right.pid.kd".to_string(),
        value: ScalarValue::Float(v),
    },
    Mutation::SetLeftThermalFlowSettleDuration(v) => RuntimeRequestKind::SetConfigProperty {
        target: ident,
        path: "left.thermal_flow_settle_duration".to_string(),
        value: ScalarValue::Float(v),
    },
    Mutation::SetRightThermalFlowSettleDuration(v) => RuntimeRequestKind::SetConfigProperty {
        target: ident,
        path: "right.thermal_flow_settle_duration".to_string(),
        value: ScalarValue::Float(v),
    },
    Mutation::SetLeftPumpCooldownMinTemperature(v) => RuntimeRequestKind::SetConfigProperty {
        target: ident,
        path: "left.pump_cooldown_min_temperature".to_string(),
        value: ScalarValue::Float(v),
    },
    Mutation::SetRightPumpCooldownMinTemperature(v) => RuntimeRequestKind::SetConfigProperty {
        target: ident,
        path: "right.pump_cooldown_min_temperature".to_string(),
        value: ScalarValue::Float(v),
    },
    Mutation::SetAmbientTemperatureCalibration(v) => RuntimeRequestKind::SetConfigProperty {
        target: ident,
        path: "ambient_temperature_calibration".to_string(),
        value: ScalarValue::Float(v),
    }};
    Ok(vec![res])
}

fn init_state_event(
    instance: &MachineInstance,
    _is_default_state: bool,
) -> Option<serde_json::Value> {
    let get_state = |name: &'static str| -> Option<ScalarValue> {
        Some(instance.state_properties.get(name)?.as_ref()?.value.clone())
    };

    let get_config = |name: &'static str| -> Option<ScalarValue> {
        Some(instance.config_properties.get(name)?.as_ref()?.value.clone())
    };

   Some(serde_json::json!({
"mode": get_state("mode")?
    .r#enum()
    .expect("Cannot be null"),

"is_default_state": get_state("is_default_state")?
    .boolean()
    .expect("Cannot be null"),

"left_should_flow": get_state("left_should_flow")?
    .boolean()
    .expect("Cannot be null"),

"right_should_flow": get_state("right_should_flow")?
    .boolean()
    .expect("Cannot be null"),

"left_has_flow": get_state("left_has_flow")?
    .boolean()
    .expect("Cannot be null"),

"right_has_flow": get_state("right_has_flow")?
    .boolean()
    .expect("Cannot be null"),

"left_thermal_safety_state": {
    "thermal_delay": get_state("left_thermal_safety_state.thermal_delay")?
        .float()
        .expect("Cannot be null"),

    "cooldown_min_temperature": get_state("left_thermal_safety_state.cooldown_min_temperature")?
        .float()
        .expect("Cannot be null"),
},

"right_thermal_safety_state": {
    "thermal_delay": get_state("right_thermal_safety_state.thermal_delay")?
        .float()
        .expect("Cannot be null"),

    "cooldown_min_temperature": get_state("right_thermal_safety_state.cooldown_min_temperature")?
        .float()
        .expect("Cannot be null"),
},

"left_target_temperature": get_config("left_target_temperature")?
    .float()
    .expect("Cannot be null"),

"right_target_temperature": get_config("right_target_temperature")?
    .float()
    .expect("Cannot be null"),

"ambient_temperature_calibration": get_config("ambient_temperature_calibration")?
    .float()
    .expect("Cannot be null"),

"left_fan_max_revolutions": get_config("left_fan_max_revolutions")?
    .float()
    .expect("Cannot be null"),

"right_fan_max_revolutions": get_config("right_fan_max_revolutions")?
    .float()
    .expect("Cannot be null"),

"left_tolerance_config": {
    "heating": get_config("left_tolerance_config.heating")?
        .float()
        .expect("Cannot be null"),

    "cooling": get_config("left_tolerance_config.cooling")?
        .float()
        .expect("Cannot be null"),
},

"right_tolerance_config": {
    "heating": get_config("right_tolerance_config.heating")?
        .float()
        .expect("Cannot be null"),

    "cooling": get_config("right_tolerance_config.cooling")?
        .float()
        .expect("Cannot be null"),
},

"left_pid_config": {
    "kp": get_config("left_pid_config.kp")?
        .float()
        .expect("Cannot be null"),

    "ki": get_config("left_pid_config.ki")?
        .float()
        .expect("Cannot be null"),

    "kd": get_config("left_pid_config.kd")?
        .float()
        .expect("Cannot be null"),
},

"right_pid_config": {
    "kp": get_config("right_pid_config.kp")?
        .float()
        .expect("Cannot be null"),

    "ki": get_config("right_pid_config.ki")?
        .float()
        .expect("Cannot be null"),

    "kd": get_config("right_pid_config.kd")?
        .float()
        .expect("Cannot be null"),
},

"left_thermal_flow_settle_duration": get_config("left_thermal_flow_settle_duration")?
    .float()
    .expect("Cannot be null"),

"right_thermal_flow_settle_duration": get_config("right_thermal_flow_settle_duration")?
    .float()
    .expect("Cannot be null"),

"left_pump_cooldown_min_temperature": get_config("left_pump_cooldown_min_temperature")?
    .float()
    .expect("Cannot be null"),

"right_pump_cooldown_min_temperature": get_config("right_pump_cooldown_min_temperature")?
    .float()
    .expect("Cannot be null"),
}))
}

fn init_measurements_event(instance: &MachineInstance) -> Option<serde_json::Value> {
    let get = |name: &'static str| -> Option<Option<f64>> {
        Some(instance.measurements.get(name)?.as_ref()?.value)
    };

    let get_state = |name: &'static str| -> Option<ScalarValue> {
        Some(instance.state_properties.get(name)?.as_ref()?.value.clone())
    };

    Some(serde_json::json!({
        "left_flow": get("left_flow").expect("Non nullable measurement is null"),
        "right_flow": get("right_flow").expect("Non nullable measurement is null"),
        "left_temperature": get("left_temperature").expect("Non nullable measurement is null"),
        "right_temperature": get("right_temperature").expect("Non nullable measurement is null"),
        "left_revolutions": get("left_revolutions").expect("Non nullable measurement is null"),
        "right_revolutions": get("right_revolutions").expect("Non nullable measurement is null"),
        "left_power": get("left_power").expect("Non nullable measurement is null"),
        "right_power": get("right_power").expect("Non nullable measurement is null"),
        "left_total_energy": get("left_total_energy").expect("Non nullable measurement is null"),
        "right_total_energy": get("right_total_energy").expect("Non nullable measurement is null"),
        "left_heating_startup_wait_active": get_state("left_heating_startup_wait_active")?
            .boolean()
            .expect("Cannot be null"),
        "right_heating_startup_wait_active": get_state("right_heating_startup_wait_active")?
            .boolean()
            .expect("Cannot be null"),
        "left_pump_cooldown_active": get_state("left_pump_cooldown_active")?
            .boolean()
            .expect("Cannot be null"),
        "right_pump_cooldown_active": get_state("right_pump_cooldown_active")?
            .boolean()
            .expect("Cannot be null"),
        "left_heating": get_state("left_heating")?
            .boolean()
            .expect("Cannot be null"),
        "right_heating": get_state("right_heating")?
            .boolean()
            .expect("Cannot be null"),
        "left_pump_cooldown_remaining": get_state("left_pump_cooldown_remaining")?
            .float()
            .expect("Cannot be null"),
        "right_pump_cooldown_remaining": get_state("right_pump_cooldown_remaining")?
            .float()
            .expect("Cannot be null"),
        "left_heating_startup_wait_remaining": get_state("left_heating_startup_wait_remaining")?
            .float()
            .expect("Cannot be null"),
        "right_heating_startup_wait_remaining": get_state("right_heating_startup_wait_remaining")?
            .float()
            .expect("Cannot be null"),
        "left_cooling_mode": get_state("left_cooling_mode")
            .and_then(|v| v.r#enum()),
        "right_cooling_mode": get_state("right_cooling_mode")
            .and_then(|v| v.r#enum()),
    }))
}
