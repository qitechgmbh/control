use crate::api::legacy::MachineLegacyDataAdapter;
use crate::api::types::MachineInstance;
use crate::machines::aquapath::AquaPathV1Mode;
use qitech_framework::MachineInstanceIdentification;
use qitech_framework::RuntimeRequestKind;
use qitech_framework::ScalarValue;
use serde::Deserialize;

pub const ADAPTER: MachineLegacyDataAdapter = MachineLegacyDataAdapter {
    convert_request,
    init_state_event,
    init_measurements_event,
};

/// The two cooling loops, in the order the legacy payload lists them.
const SIDES: [&str; 2] = ["left", "right"];

fn convert_request(
    ident: MachineInstanceIdentification,
    data: serde_json::Value,
) -> Result<RuntimeRequestKind, serde_json::Error> {
    #[derive(Deserialize)]
    #[allow(clippy::enum_variant_names)]
    enum Mutation {
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

    let config = |path: &str, value: ScalarValue| RuntimeRequestKind::SetConfigProperty {
        target: ident,
        path: path.to_string(),
        value,
    };

    let command = |path: &str| RuntimeRequestKind::ExecuteCommand {
        target: ident,
        path: path.to_string(),
    };

    Ok(match serde_json::from_value(data)? {
        // Mode is driven by commands, not a config property: see `init_commands` in `new.rs`.
        Mutation::SetAquaPathMode(v) => command(match v {
            AquaPathV1Mode::Standby => "state.set_standby",
            AquaPathV1Mode::Auto => "state.set_auto",
        }),

        Mutation::SetLeftTemperature(v) => config("left_target_temperature", ScalarValue::Float(v)),
        Mutation::SetRightTemperature(v) => {
            config("right_target_temperature", ScalarValue::Float(v))
        }

        // Flow is requested by starting/stopping the pump, not a config property.
        Mutation::SetLeftFlow(v) => command(if v {
            "pump.start_left_pump"
        } else {
            "pump.stop_left_pump"
        }),
        Mutation::SetRightFlow(v) => command(if v {
            "pump.start_right_pump"
        } else {
            "pump.stop_right_pump"
        }),

        Mutation::SetLeftRevolutions(v) => {
            config("left_fan_max_revolutions", ScalarValue::Float(v))
        }
        Mutation::SetRightRevolutions(v) => {
            config("right_fan_max_revolutions", ScalarValue::Float(v))
        }

        Mutation::SetLeftHeatingTolerance(v) => {
            config("left_tolerance_config.heating", ScalarValue::Float(v))
        }
        Mutation::SetRightHeatingTolerance(v) => {
            config("right_tolerance_config.heating", ScalarValue::Float(v))
        }
        Mutation::SetLeftCoolingTolerance(v) => {
            config("left_tolerance_config.cooling", ScalarValue::Float(v))
        }
        Mutation::SetRightCoolingTolerance(v) => {
            config("right_tolerance_config.cooling", ScalarValue::Float(v))
        }

        Mutation::SetLeftPidKp(v) => config("left_pid_config.kp", ScalarValue::Float(v)),
        Mutation::SetLeftPidKi(v) => config("left_pid_config.ki", ScalarValue::Float(v)),
        Mutation::SetLeftPidKd(v) => config("left_pid_config.kd", ScalarValue::Float(v)),
        Mutation::SetRightPidKp(v) => config("right_pid_config.kp", ScalarValue::Float(v)),
        Mutation::SetRightPidKi(v) => config("right_pid_config.ki", ScalarValue::Float(v)),
        Mutation::SetRightPidKd(v) => config("right_pid_config.kd", ScalarValue::Float(v)),

        Mutation::SetLeftThermalFlowSettleDuration(v) => {
            config("left_thermal_flow_settle_duration", ScalarValue::Float(v))
        }
        Mutation::SetRightThermalFlowSettleDuration(v) => {
            config("right_thermal_flow_settle_duration", ScalarValue::Float(v))
        }

        Mutation::SetLeftPumpCooldownMinTemperature(v) => {
            config("left_pump_cooldown_min_temperature", ScalarValue::Float(v))
        }
        Mutation::SetRightPumpCooldownMinTemperature(v) => {
            config("right_pump_cooldown_min_temperature", ScalarValue::Float(v))
        }

        Mutation::SetAmbientTemperatureCalibration(v) => {
            config("ambient_temperature_calibration", ScalarValue::Float(v))
        }
    })
}

// --- state event ---

fn init_state_event(
    instance: &MachineInstance,
    is_default_state: bool,
) -> Option<serde_json::Value> {
    let flow_states = side_states(|side| {
        Some(serde_json::json!({
            "flow": measurement(instance, &format!("{side}_flow"))?,
            "should_flow": state_bool(instance, &format!("{side}_should_flow"))?,
        }))
    })?;

    let temperature_states = side_states(|side| {
        Some(serde_json::json!({
            "temperature": measurement(instance, &format!("{side}_temperature"))?,
            "target_temperature": config_float(instance, &format!("{side}_target_temperature"))?,
        }))
    })?;

    let fan_states = side_states(|side| {
        Some(serde_json::json!({
            "revolutions": measurement(instance, &format!("{side}_revolutions"))?,
            "max_revolutions": config_float(instance, &format!("{side}_fan_max_revolutions"))?,
        }))
    })?;

    let cooling_mode_states = side_states(|side| {
        Some(serde_json::json!({
            "mode": state_nullable_enum(instance, &format!("{side}_cooling_mode"))?,
        }))
    })?;

    let tolerance_states = side_states(|side| {
        Some(serde_json::json!({
            "heating": config_float(instance, &format!("{side}_tolerance_config.heating"))?,
            "cooling": config_float(instance, &format!("{side}_tolerance_config.cooling"))?,
        }))
    })?;

    let pid_states = side_states(|side| {
        Some(serde_json::json!({
            "kp": config_float(instance, &format!("{side}_pid_config.kp"))?,
            "ki": config_float(instance, &format!("{side}_pid_config.ki"))?,
            "kd": config_float(instance, &format!("{side}_pid_config.kd"))?,
        }))
    })?;

    let thermal_safety_states = side_states(|side| {
        Some(serde_json::json!({
            "thermal_delay": state_float(instance, &format!("{side}_thermal_safety_state.thermal_delay"))?,
            "cooldown_min_temperature": state_float(instance, &format!("{side}_thermal_safety_state.cooldown_min_temperature"))?,
        }))
    })?;

    Some(serde_json::json!({
        "is_default_state": is_default_state,

        "mode_state": {
            "mode": state_enum(instance, "mode_state.mode")?,
        },

        "ambient_temperature_calibration": config_float(instance, "ambient_temperature_calibration")?,

        "flow_states": flow_states,
        "temperature_states": temperature_states,
        "fan_states": fan_states,
        "cooling_mode_states": cooling_mode_states,
        "tolerance_states": tolerance_states,
        "pid_states": pid_states,
        "thermal_safety_states": thermal_safety_states,
    }))
}

/// Builds `{ left: .., right: .. }`, yielding `None` as soon as one side cannot be rendered yet.
fn side_states<F>(mut render: F) -> Option<serde_json::Value>
where
    F: FnMut(&str) -> Option<serde_json::Value>,
{
    let mut map = serde_json::Map::with_capacity(SIDES.len());

    for side in SIDES {
        map.insert(side.to_string(), render(side)?);
    }

    Some(serde_json::Value::Object(map))
}

// --- live values ---

fn init_measurements_event(instance: &MachineInstance) -> Option<serde_json::Value> {
    let get_state = |name: &'static str| -> Option<ScalarValue> {
        Some(instance.state_properties.get(name)?.as_ref()?.value.clone())
    };

    Some(serde_json::json!({
        "left_flow": measurement(instance, "left_flow").expect("Non nullable measurement is null"),
        "right_flow": measurement(instance, "right_flow").expect("Non nullable measurement is null"),
        "left_temperature": measurement(instance, "left_temperature").expect("Non nullable measurement is null"),
        "right_temperature": measurement(instance, "right_temperature").expect("Non nullable measurement is null"),
        "left_revolutions": measurement(instance, "left_revolutions").expect("Non nullable measurement is null"),
        "right_revolutions": measurement(instance, "right_revolutions").expect("Non nullable measurement is null"),
        "left_power": measurement(instance, "left_power").expect("Non nullable measurement is null"),
        "right_power": measurement(instance, "right_power").expect("Non nullable measurement is null"),
        "left_total_energy": measurement(instance, "left_total_energy").expect("Non nullable measurement is null"),
        "right_total_energy": measurement(instance, "right_total_energy").expect("Non nullable measurement is null"),
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

// --- property lookups ---
//
// Each returns `None` while the runtime has not registered the property yet, which propagates out
// of the event builders so a partial payload is never emitted.

fn config_value(instance: &MachineInstance, path: &str) -> Option<ScalarValue> {
    Some(
        instance
            .config_properties
            .get(path)?
            .as_ref()?
            .value
            .clone(),
    )
}

fn state_value(instance: &MachineInstance, path: &str) -> Option<ScalarValue> {
    Some(instance.state_properties.get(path)?.as_ref()?.value.clone())
}

fn measurement(instance: &MachineInstance, path: &str) -> Option<f64> {
    instance.measurements.get(path)?.as_ref()?.value
}

fn config_float(instance: &MachineInstance, path: &str) -> Option<f64> {
    config_value(instance, path)?.float()
}

fn state_bool(instance: &MachineInstance, path: &str) -> Option<bool> {
    state_value(instance, path)?.boolean()
}

fn state_float(instance: &MachineInstance, path: &str) -> Option<f64> {
    state_value(instance, path)?.float()
}

fn state_enum(instance: &MachineInstance, path: &str) -> Option<String> {
    state_value(instance, path)?.r#enum()
}

/// `Some(None)` when the property is registered but currently null (e.g. no cooling mode active),
/// unlike the other lookups where `None` only ever means "not registered yet".
fn state_nullable_enum(instance: &MachineInstance, path: &str) -> Option<Option<String>> {
    Some(state_value(instance, path)?.r#enum())
}