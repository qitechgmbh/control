use qitech_framework::MachineInstanceIdentification;
use qitech_framework::RuntimeRequestKind;
use qitech_framework::ScalarValue;
use serde::Deserialize;

use crate::api::legacy::MachineLegacyDataAdapter;
use crate::api::types::MachineInstance;

pub const ADAPTER: MachineLegacyDataAdapter = MachineLegacyDataAdapter {
    convert_request,
    init_state_event,
    init_measurements_event,
};

const DAYS: [&str; 7] = [
    "monday",
    "tuesday",
    "wednesday",
    "thursday",
    "friday",
    "saturday",
    "sunday",
];

// --- requests ---

#[derive(Deserialize)]
struct ScheduleDayInput {
    start_minutes: i64,
    stop_minutes: i64,
}

fn convert_request(
    ident: MachineInstanceIdentification,
    data: serde_json::Value,
) -> Result<Vec<RuntimeRequestKind>, serde_json::Error> {
    let config = |path: String, value: ScalarValue| RuntimeRequestKind::SetConfigProperty {
        target: ident,
        path,
        value,
    };

    let command = |path: &str| RuntimeRequestKind::ExecuteCommand {
        target: ident,
        path: path.to_string(),
    };

    /// Sets the whole 7-day weekly schedule at once - the Schedule page always sends every
    /// day back, even when only one was edited, since the config properties are per-day.
    #[derive(Deserialize)]
    enum CompoundMutation {
        SetSchedule { schedule: [ScheduleDayInput; 7] },
    }

    if let Ok(mutation) = serde_json::from_value::<CompoundMutation>(data.clone()) {
        return Ok(match mutation {
            CompoundMutation::SetSchedule { schedule } => schedule
                .iter()
                .zip(DAYS)
                .flat_map(|(day, name)| {
                    [
                        config(
                            format!("schedule.{name}.start_minutes"),
                            ScalarValue::Integer(day.start_minutes),
                        ),
                        config(
                            format!("schedule.{name}.stop_minutes"),
                            ScalarValue::Integer(day.stop_minutes),
                        ),
                    ]
                })
                .collect(),
        });
    }

    #[derive(Deserialize)]
    enum Mutation {
        SetRunning(bool),
        SetTargetTemperature(f64),
        SetAirVolume(i64),
        SetDryingTimerMinutes(i64),
    }

    Ok(vec![match serde_json::from_value(data)? {
        Mutation::SetRunning(true) => command("start"),
        Mutation::SetRunning(false) => command("stop"),
        Mutation::SetTargetTemperature(v) => {
            config("target_temperature".to_string(), ScalarValue::Float(v))
        }
        Mutation::SetAirVolume(v) => config("air_volume".to_string(), ScalarValue::Integer(v)),
        Mutation::SetDryingTimerMinutes(v) => config(
            "drying_timer_minutes".to_string(),
            ScalarValue::Integer(v),
        ),
    }])
}

// --- state event ---

fn init_state_event(
    instance: &MachineInstance,
    is_default_state: bool,
) -> Option<serde_json::Value> {
    let schedule: Vec<serde_json::Value> = DAYS
        .iter()
        .map(|name| {
            Some(serde_json::json!({
                "start_minutes": config_integer(instance, &format!("schedule.{name}.start_minutes"))?,
                "stop_minutes": config_integer(instance, &format!("schedule.{name}.stop_minutes"))?,
            }))
        })
        .collect::<Option<Vec<_>>>()?;

    Some(serde_json::json!({
        "is_default_state": is_default_state,

        "status": state_integer(instance, "status")?,
        "alarm": state_integer(instance, "alarm")?,
        "warning": state_integer(instance, "warning")?,
        "is_smart": state_bool(instance, "is_smart")?,

        "target_temperature": config_float(instance, "target_temperature")?,
        "air_volume": config_integer(instance, "air_volume")?,
        "drying_timer_minutes": config_integer(instance, "drying_timer_minutes")?,

        "schedule": schedule,
    }))
}

// --- live values ---

fn init_measurements_event(instance: &MachineInstance) -> Option<serde_json::Value> {
    let get = |path: &str| -> Option<f64> { instance.measurements.get(path)?.as_ref()?.value };

    Some(serde_json::json!({
        "temp_process": get("temp_process")?,
        "temp_safety": get("temp_safety")?,
        "temp_regen_in": get("temp_regen_in")?,
        "temp_regen_out": get("temp_regen_out")?,
        "temp_fan_inlet": get("temp_fan_inlet")?,
        "temp_return_air": get("temp_return_air")?,
        "temp_dew_point": get("temp_dew_point")?,
        "pwm_fan1": get("pwm_fan1")?,
        "pwm_fan2": get("pwm_fan2")?,
        "power_process": get("power_process")?,
        "power_regen": get("power_regen")?,
        "remaining_seconds": get("remaining_seconds"),
    }))
}

// --- property lookups ---

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

fn config_float(instance: &MachineInstance, path: &str) -> Option<f64> {
    config_value(instance, path)?.float()
}

fn config_integer(instance: &MachineInstance, path: &str) -> Option<i64> {
    config_value(instance, path)?.integer()
}

fn state_integer(instance: &MachineInstance, path: &str) -> Option<i64> {
    state_value(instance, path)?.integer()
}

fn state_bool(instance: &MachineInstance, path: &str) -> Option<bool> {
    state_value(instance, path)?.boolean()
}
