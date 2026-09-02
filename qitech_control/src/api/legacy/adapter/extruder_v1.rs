// TODO: SetPressurePidSettings, SetTemperaturePidSettings and StartPressurePidAutoTune each expand
// into several RuntimeRequestKind values (3 config writes; 2 config writes + 1 command). They stay
// unimplemented until MachineLegacyDataAdapter::convert_request can return more than one request.

use qitech_framework::MachineInstanceIdentification;
use qitech_framework::RuntimeRequestKind;
use qitech_framework::ScalarValue;
use serde::Deserialize;

use crate::api::legacy::MachineLegacyDataAdapter;
use crate::api::types::MachineInstance;

/// Serves both extruder generations: `ExtruderV1` (machine id 4, the frontend's "extruder2") and
/// `ExtruderV2` (machine id 22, the frontend's "extruder3"). Their schemas are identical apart from
/// the identification block.
pub const ADAPTER: MachineLegacyDataAdapter = MachineLegacyDataAdapter {
    convert_request,
    init_state_event,
    init_measurements_event,
};

/// The four heating zones, in the order the legacy payload lists them.
const ZONES: [&str; 4] = ["nozzle", "front", "back", "middle"];

// --- requests ---

fn convert_request(
    ident: MachineInstanceIdentification,
    data: serde_json::Value,
) -> Result<RuntimeRequestKind, serde_json::Error> {
    /// Mirrors the payloads emitted by `useExtruder.ts`.
    #[derive(Deserialize)]
    enum Mutation {
        SetInverterRotationDirection(bool),
        SetExtruderMode(Mode),
        SetInverterRegulation(bool),
        SetInverterTargetRpm(f64),
        SetInverterTargetPressure(f64),
        SetNozzleHeatingTemperature(f64),
        SetFrontHeatingTargetTemperature(f64),
        SetMiddleHeatingTemperature(f64),
        SetBackHeatingTargetTemperature(f64),
        SetExtruderPressureLimit(f64),
        SetExtruderPressureLimitIsEnabled(bool),
        SetNozzleTemperatureTargetEnabled(bool),
        /// The frontend always sends `true` here; the flag carries no meaning, only the request does.
        ResetInverter(#[allow(dead_code)] bool),
        StopPressurePidAutoTune {},
    }

    #[derive(Deserialize)]
    enum Mode {
        Standby,
        Heat,
        Extrude,
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
        // `EnumProperty::from_scalar` only accepts the snake_case spelling of a variant, even
        // though it reads back as the variant ident itself. See `init_state_event`.
        Mutation::SetInverterRotationDirection(forward) => config(
            "screw.direction",
            ScalarValue::Enum(if forward { "forward" } else { "reverse" }.to_string()),
        ),

        Mutation::SetInverterRegulation(uses_rpm) => config(
            "screw.regulation",
            ScalarValue::Enum(if uses_rpm { "rpm" } else { "pressure" }.to_string()),
        ),

        Mutation::SetExtruderMode(mode) => command(match mode {
            Mode::Standby => "mode.standby",
            Mode::Heat => "mode.heat",
            Mode::Extrude => "mode.extrude",
        }),

        Mutation::SetInverterTargetRpm(v) => config("screw.target_rpm", ScalarValue::Float(v)),

        Mutation::SetInverterTargetPressure(v) => {
            config("screw.target_pressure", ScalarValue::Float(v))
        }

        Mutation::SetNozzleHeatingTemperature(v) => {
            config("heating.nozzle.target_temperature", ScalarValue::Float(v))
        }

        Mutation::SetFrontHeatingTargetTemperature(v) => {
            config("heating.front.target_temperature", ScalarValue::Float(v))
        }

        Mutation::SetMiddleHeatingTemperature(v) => {
            config("heating.middle.target_temperature", ScalarValue::Float(v))
        }

        Mutation::SetBackHeatingTargetTemperature(v) => {
            config("heating.back.target_temperature", ScalarValue::Float(v))
        }

        Mutation::SetExtruderPressureLimit(v) => config("pressure.limit", ScalarValue::Float(v)),

        Mutation::SetExtruderPressureLimitIsEnabled(v) => {
            config("pressure.limit_enabled", ScalarValue::Boolean(v))
        }

        Mutation::SetNozzleTemperatureTargetEnabled(v) => {
            config("heating.nozzle.target_enabled", ScalarValue::Boolean(v))
        }

        Mutation::ResetInverter(_) => command("inverter.reset"),

        Mutation::StopPressurePidAutoTune {} => command("pressure.autotune.stop"),
    })
}

// --- state event ---

fn init_state_event(
    instance: &MachineInstance,
    is_default_state: bool,
) -> Option<serde_json::Value> {
    let heating_states = zone_map(|zone| {
        Some(serde_json::json!({
            "target_temperature": config_float(instance, &format!("heating.{zone}.target_temperature"))?,
            "wiring_error": state_bool(instance, &format!("heating.{zone}.wiring_error"))?,
        }))
    })?;

    let temperature_pids = zone_map(|zone| {
        Some(serde_json::json!({
            "kp": config_float(instance, &format!("pid.temperature.{zone}.kp"))?,
            "ki": config_float(instance, &format!("pid.temperature.{zone}.ki"))?,
            "kd": config_float(instance, &format!("pid.temperature.{zone}.kd"))?,
            "zone": zone,
        }))
    })?;

    Some(serde_json::json!({
        "is_default_state": is_default_state,

        // `EnumProperty::into_scalar` writes the variant ident verbatim, so these compare against
        // the PascalCase spelling — unlike the snake_case one writes take.
        "rotation_state": {
            "forward": config_enum(instance, "screw.direction")? == "Forward",
        },

        "mode_state": {
            "mode": state_enum(instance, "mode")?,
        },

        "regulation_state": {
            "uses_rpm": config_enum(instance, "screw.regulation")? == "Rpm",
        },

        "pressure_state": {
            "target_bar": config_float(instance, "screw.target_pressure")?,
            "wiring_error": state_bool(instance, "pressure.wiring_error")?,
        },

        "screw_state": {
            "target_rpm": config_float(instance, "screw.target_rpm")?,
        },

        "heating_states": heating_states,

        "extruder_settings_state": {
            "pressure_limit": config_float(instance, "pressure.limit")?,
            "pressure_limit_enabled": config_bool(instance, "pressure.limit_enabled")?,
            "nozzle_temperature_target_enabled": config_bool(instance, "heating.nozzle.target_enabled")?,
        },

        "inverter_status_state": {
            "running": state_bool(instance, "inverter.running")?,
            "forward_running": state_bool(instance, "inverter.forward_running")?,
            "reverse_running": state_bool(instance, "inverter.reverse_running")?,
            "up_to_frequency": state_bool(instance, "inverter.up_to_frequency")?,
            "overload_warning": state_bool(instance, "inverter.overload_warning")?,
            "no_function": state_bool(instance, "inverter.no_function")?,
            "output_frequency_detection": state_bool(instance, "inverter.output_frequency_detection")?,
            "abc_fault": state_bool(instance, "inverter.abc_fault")?,
            "fault_occurence": state_bool(instance, "inverter.fault_occurence")?,
        },

        "pid_settings": {
            "temperature": temperature_pids,
            "pressure": {
                "kp": config_float(instance, "pid.pressure.kp")?,
                "ki": config_float(instance, "pid.pressure.ki")?,
                "kd": config_float(instance, "pid.pressure.kd")?,
            },
        },

        "pid_autotune_state": {
            // The frontend compares this against "running" / "not_started", so the variant ident
            // has to be folded back to the legacy snake_case spelling.
            "state": match state_enum(instance, "pressure.autotune.state")?.as_str() {
                "Running" => "running",
                "Completed" => "completed",
                "Failed" => "failed",
                _ => "not_started",
            },
            "progress": autotune_progress(instance),
            "result": autotune_result(instance)?,
        },
    }))
}

/// Builds `{ nozzle: .., front: .., back: .., middle: .. }`, yielding `None` as soon as one zone
/// cannot be rendered yet.
fn zone_map<F>(mut render: F) -> Option<serde_json::Value>
where
    F: FnMut(&str) -> Option<serde_json::Value>,
{
    let mut map = serde_json::Map::with_capacity(ZONES.len());

    for zone in ZONES {
        map.insert(zone.to_string(), render(zone)?);
    }

    Some(serde_json::Value::Object(map))
}

/// Never blocks the state event: the measurement is absent until the first snapshot arrives, and
/// the legacy payload reported 0 % in that case.
fn autotune_progress(instance: &MachineInstance) -> f64 {
    instance
        .measurements
        .get("pressure.autotune_progress")
        .and_then(|info| info.as_ref())
        .and_then(|info| info.value)
        .unwrap_or(0.0)
}

/// The three result gains are nullable and only populated after a completed run — all or nothing.
fn autotune_result(instance: &MachineInstance) -> Option<serde_json::Value> {
    let gains = (
        state_nullable_float(instance, "pressure.autotune.result.kp")?,
        state_nullable_float(instance, "pressure.autotune.result.ki")?,
        state_nullable_float(instance, "pressure.autotune.result.kd")?,
    );

    Some(match gains {
        (Some(kp), Some(ki), Some(kd)) => serde_json::json!({
            "kp": kp,
            "ki": ki,
            "kd": kd,
        }),
        _ => serde_json::Value::Null,
    })
}

// --- live values ---

fn init_measurements_event(instance: &MachineInstance) -> Option<serde_json::Value> {
    let get = |path: &str| -> Option<f64> { instance.measurements.get(path)?.as_ref()?.value };

    let temperature = |zone: &str| get(&format!("heating.{zone}.temperature"));
    let power = |zone: &str| get(&format!("heating.{zone}.power"));

    Some(serde_json::json!({
        "motor_status": {
            "screw_rpm": get("motor.rpm")?,
            "frequency": get("motor.frequency")?,
            // Unused by the frontend's zod schema, kept for parity with the pre-migration event.
            "voltage":   get("motor.voltage")?,
            "current":   get("motor.current")?,
            "power":     get("motor.power")?,
        },

        "pressure": get("pressure.value")?,

        "nozzle_temperature": temperature("nozzle")?,
        "front_temperature":  temperature("front")?,
        "back_temperature":   temperature("back")?,
        "middle_temperature": temperature("middle")?,

        "nozzle_power": power("nozzle")?,
        "front_power":  power("front")?,
        "back_power":   power("back")?,
        "middle_power": power("middle")?,

        "combined_power":   get("power.combined")?,
        "total_energy_kwh": get("energy.total")?,
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

fn config_float(instance: &MachineInstance, path: &str) -> Option<f64> {
    config_value(instance, path)?.float()
}

fn config_bool(instance: &MachineInstance, path: &str) -> Option<bool> {
    config_value(instance, path)?.boolean()
}

fn config_enum(instance: &MachineInstance, path: &str) -> Option<String> {
    config_value(instance, path)?.r#enum()
}

fn state_bool(instance: &MachineInstance, path: &str) -> Option<bool> {
    state_value(instance, path)?.boolean()
}

fn state_enum(instance: &MachineInstance, path: &str) -> Option<String> {
    state_value(instance, path)?.r#enum()
}

/// `Some(None)` when the property is registered but currently null, unlike the other lookups where
/// `None` only ever means "not registered yet".
fn state_nullable_float(instance: &MachineInstance, path: &str) -> Option<Option<f64>> {
    Some(state_value(instance, path)?.float())
}

#[cfg(test)]
mod tests {
    use qitech_framework::MachineIdentification;
    use qitech_framework::MachineSchema;

    use super::*;
    use crate::api::types::ConfigPropertyInfo;
    use crate::api::types::MeasurementInfo;
    use crate::api::types::StatePropertyInfo;

    const SCHEMA: &str = include_str!("../../../../schemas/extruder_v1.yaml");
    const SCHEMA_V2: &str = include_str!("../../../../schemas/extruder_v2.yaml");

    /// Distinct values throughout, so a field wired to the wrong property fails the comparison.
    fn config_fixture() -> Vec<(&'static str, ScalarValue)> {
        vec![
            ("screw.direction", ScalarValue::Enum("Forward".into())),
            ("screw.regulation", ScalarValue::Enum("Rpm".into())),
            ("screw.target_rpm", ScalarValue::Float(12.0)),
            ("screw.target_pressure", ScalarValue::Float(34.0)),
            ("pressure.limit", ScalarValue::Float(90.0)),
            ("pressure.limit_enabled", ScalarValue::Boolean(true)),
            ("pressure.autotune.tune_delta", ScalarValue::Float(0.5)),
            ("pressure.autotune.frequency_step", ScalarValue::Float(5.0)),
            (
                "heating.nozzle.target_temperature",
                ScalarValue::Float(210.0),
            ),
            ("heating.nozzle.target_enabled", ScalarValue::Boolean(true)),
            (
                "heating.front.target_temperature",
                ScalarValue::Float(200.0),
            ),
            (
                "heating.middle.target_temperature",
                ScalarValue::Float(190.0),
            ),
            ("heating.back.target_temperature", ScalarValue::Float(180.0)),
            ("pid.pressure.kp", ScalarValue::Float(1.0)),
            ("pid.pressure.ki", ScalarValue::Float(2.0)),
            ("pid.pressure.kd", ScalarValue::Float(3.0)),
            ("pid.temperature.nozzle.kp", ScalarValue::Float(11.0)),
            ("pid.temperature.nozzle.ki", ScalarValue::Float(12.0)),
            ("pid.temperature.nozzle.kd", ScalarValue::Float(13.0)),
            ("pid.temperature.front.kp", ScalarValue::Float(21.0)),
            ("pid.temperature.front.ki", ScalarValue::Float(22.0)),
            ("pid.temperature.front.kd", ScalarValue::Float(23.0)),
            ("pid.temperature.middle.kp", ScalarValue::Float(31.0)),
            ("pid.temperature.middle.ki", ScalarValue::Float(32.0)),
            ("pid.temperature.middle.kd", ScalarValue::Float(33.0)),
            ("pid.temperature.back.kp", ScalarValue::Float(41.0)),
            ("pid.temperature.back.ki", ScalarValue::Float(42.0)),
            ("pid.temperature.back.kd", ScalarValue::Float(43.0)),
        ]
    }

    fn state_fixture() -> Vec<(&'static str, ScalarValue)> {
        vec![
            ("mode", ScalarValue::Enum("Heat".into())),
            ("pressure.wiring_error", ScalarValue::Boolean(false)),
            (
                "pressure.autotune.state",
                ScalarValue::Enum("Running".into()),
            ),
            ("pressure.autotune.result.kp", ScalarValue::Null),
            ("pressure.autotune.result.ki", ScalarValue::Null),
            ("pressure.autotune.result.kd", ScalarValue::Null),
            ("heating.nozzle.wiring_error", ScalarValue::Boolean(false)),
            ("heating.front.wiring_error", ScalarValue::Boolean(true)),
            ("heating.middle.wiring_error", ScalarValue::Boolean(false)),
            ("heating.back.wiring_error", ScalarValue::Boolean(false)),
            ("inverter.running", ScalarValue::Boolean(true)),
            ("inverter.forward_running", ScalarValue::Boolean(true)),
            ("inverter.reverse_running", ScalarValue::Boolean(false)),
            ("inverter.up_to_frequency", ScalarValue::Boolean(true)),
            ("inverter.overload_warning", ScalarValue::Boolean(false)),
            ("inverter.no_function", ScalarValue::Boolean(false)),
            (
                "inverter.output_frequency_detection",
                ScalarValue::Boolean(true),
            ),
            ("inverter.abc_fault", ScalarValue::Boolean(false)),
            ("inverter.fault_occurence", ScalarValue::Boolean(false)),
        ]
    }

    fn measurement_fixture() -> Vec<(&'static str, f64)> {
        vec![
            ("motor.rpm", 100.0),
            ("motor.frequency", 50.0),
            ("motor.voltage", 230.0),
            ("motor.current", 2.0),
            ("motor.power", 460.0),
            ("pressure.value", 55.0),
            ("pressure.autotune_progress", 42.0),
            ("heating.nozzle.temperature", 209.0),
            ("heating.front.temperature", 199.0),
            ("heating.middle.temperature", 189.0),
            ("heating.back.temperature", 179.0),
            ("heating.nozzle.power", 100.0),
            ("heating.front.power", 200.0),
            ("heating.middle.power", 300.0),
            ("heating.back.power", 400.0),
            ("power.combined", 1000.0),
            ("energy.total", 7.5),
        ]
    }

    fn instance() -> MachineInstance {
        let config_properties = config_fixture()
            .into_iter()
            .map(|(path, value)| {
                let info = ConfigPropertyInfo {
                    default: value.clone(),
                    value,
                    capability: Default::default(),
                    constraints: Default::default(),
                    records: Vec::new(),
                };

                (path.to_string(), Some(info))
            })
            .collect();

        let state_properties = state_fixture()
            .into_iter()
            .map(|(path, value)| {
                let info = StatePropertyInfo {
                    value,
                    records: Vec::new(),
                };

                (path.to_string(), Some(info))
            })
            .collect();

        let measurements = measurement_fixture()
            .into_iter()
            .map(|(path, value)| {
                (
                    path.to_string(),
                    Some(MeasurementInfo { value: Some(value) }),
                )
            })
            .collect();

        MachineInstance {
            config_properties,
            state_properties,
            measurements,
        }
    }

    fn ident() -> MachineInstanceIdentification {
        MachineInstanceIdentification {
            machine: MachineIdentification {
                vendor_id: 1,
                machine_id: 4,
            },
            serial: 1,
        }
    }

    /// Guards the fixtures against schema drift: if a property is renamed in the YAML, the fixture
    /// stops matching and every payload assertion below would otherwise silently pass on stale
    /// paths.
    #[test]
    fn fixture_covers_exactly_the_schema() {
        let schema = MachineSchema::parse_str(SCHEMA).expect("schema should parse");

        let sorted = |mut v: Vec<String>| {
            v.sort();
            v
        };

        assert_eq!(
            sorted(
                config_fixture()
                    .iter()
                    .map(|(p, _)| p.to_string())
                    .collect()
            ),
            sorted(schema.config_properties.keys().cloned().collect()),
        );

        assert_eq!(
            sorted(state_fixture().iter().map(|(p, _)| p.to_string()).collect()),
            sorted(schema.state_properties.keys().cloned().collect()),
        );

        assert_eq!(
            sorted(
                measurement_fixture()
                    .iter()
                    .map(|(p, _)| p.to_string())
                    .collect()
            ),
            sorted(schema.measurements.keys().cloned().collect()),
        );
    }

    /// `adapter::get` hands both extruder generations this one adapter, which only holds while the
    /// two schemas expose the same resource paths.
    #[test]
    fn both_extruder_generations_share_the_same_paths() {
        let v1 = MachineSchema::parse_str(SCHEMA).expect("schema should parse");
        let v2 = MachineSchema::parse_str(SCHEMA_V2).expect("schema should parse");

        let sorted = |mut v: Vec<String>| {
            v.sort();
            v
        };

        assert_eq!(
            sorted(v1.config_properties.keys().cloned().collect()),
            sorted(v2.config_properties.keys().cloned().collect()),
        );

        assert_eq!(
            sorted(v1.state_properties.keys().cloned().collect()),
            sorted(v2.state_properties.keys().cloned().collect()),
        );

        assert_eq!(
            sorted(v1.measurements.keys().cloned().collect()),
            sorted(v2.measurements.keys().cloned().collect()),
        );

        assert_eq!(
            sorted(v1.commands.keys().cloned().collect()),
            sorted(v2.commands.keys().cloned().collect()),
        );
    }

    /// Every command path `convert_request` can emit must exist in the schema.
    #[test]
    fn emitted_commands_exist_in_the_schema() {
        let schema = MachineSchema::parse_str(SCHEMA).expect("schema should parse");

        for payload in [
            serde_json::json!({ "SetExtruderMode": "Standby" }),
            serde_json::json!({ "SetExtruderMode": "Heat" }),
            serde_json::json!({ "SetExtruderMode": "Extrude" }),
            serde_json::json!({ "ResetInverter": true }),
            serde_json::json!({ "StopPressurePidAutoTune": {} }),
        ] {
            let RuntimeRequestKind::ExecuteCommand { path, .. } =
                convert_request(ident(), payload.clone()).expect("should convert")
            else {
                panic!("{payload} should map to a command");
            };

            assert!(
                schema.commands.contains_key(&path),
                "command `{path}` is not in the schema",
            );
        }
    }

    /// Matches `stateEventDataSchema` in the frontend's `extruder2Namespace.ts`.
    #[test]
    fn state_event_matches_frontend_contract() {
        let event = init_state_event(&instance(), true).expect("all properties are registered");

        assert_eq!(
            event,
            serde_json::json!({
                "is_default_state": true,
                "rotation_state": { "forward": true },
                "mode_state": { "mode": "Heat" },
                "regulation_state": { "uses_rpm": true },
                "pressure_state": { "target_bar": 34.0, "wiring_error": false },
                "screw_state": { "target_rpm": 12.0 },
                "heating_states": {
                    "nozzle": { "target_temperature": 210.0, "wiring_error": false },
                    "front":  { "target_temperature": 200.0, "wiring_error": true },
                    "back":   { "target_temperature": 180.0, "wiring_error": false },
                    "middle": { "target_temperature": 190.0, "wiring_error": false },
                },
                "extruder_settings_state": {
                    "pressure_limit": 90.0,
                    "pressure_limit_enabled": true,
                    "nozzle_temperature_target_enabled": true,
                },
                "inverter_status_state": {
                    "running": true,
                    "forward_running": true,
                    "reverse_running": false,
                    "up_to_frequency": true,
                    "overload_warning": false,
                    "no_function": false,
                    "output_frequency_detection": true,
                    "abc_fault": false,
                    "fault_occurence": false,
                },
                "pid_settings": {
                    "temperature": {
                        "nozzle": { "kp": 11.0, "ki": 12.0, "kd": 13.0, "zone": "nozzle" },
                        "front":  { "kp": 21.0, "ki": 22.0, "kd": 23.0, "zone": "front" },
                        "back":   { "kp": 41.0, "ki": 42.0, "kd": 43.0, "zone": "back" },
                        "middle": { "kp": 31.0, "ki": 32.0, "kd": 33.0, "zone": "middle" },
                    },
                    "pressure": { "kp": 1.0, "ki": 2.0, "kd": 3.0 },
                },
                "pid_autotune_state": {
                    "state": "running",
                    "progress": 42.0,
                    "result": serde_json::Value::Null,
                },
            })
        );
    }

    /// Matches `liveValuesEventDataSchema` in the frontend's `extruder2Namespace.ts`.
    #[test]
    fn measurements_event_matches_frontend_contract() {
        let event = init_measurements_event(&instance()).expect("all measurements are sampled");

        assert_eq!(
            event,
            serde_json::json!({
                "motor_status": {
                    "screw_rpm": 100.0,
                    "frequency": 50.0,
                    "voltage": 230.0,
                    "current": 2.0,
                    "power": 460.0,
                },
                "pressure": 55.0,
                "nozzle_temperature": 209.0,
                "front_temperature": 199.0,
                "back_temperature": 179.0,
                "middle_temperature": 189.0,
                "nozzle_power": 100.0,
                "front_power": 200.0,
                "back_power": 400.0,
                "middle_power": 300.0,
                "combined_power": 1000.0,
                "total_energy_kwh": 7.5,
            })
        );
    }

    /// A partial payload would fail the frontend's zod parse, taking the whole page down.
    #[test]
    fn events_are_withheld_until_every_property_is_registered() {
        let mut partial = instance();
        partial
            .state_properties
            .insert("inverter.abc_fault".to_string(), None);

        assert!(init_state_event(&partial, true).is_none());

        let mut partial = instance();
        partial.measurements.insert("motor.rpm".to_string(), None);

        assert!(init_measurements_event(&partial).is_none());
    }

    /// The auto-tune progress measurement is unsampled until a run starts, and must not block the
    /// state event the way the other lookups do.
    #[test]
    fn unsampled_autotune_progress_reports_zero() {
        let mut instance = instance();
        instance
            .measurements
            .insert("pressure.autotune_progress".to_string(), None);

        let event = init_state_event(&instance, false).expect("should still emit");

        assert_eq!(event["pid_autotune_state"]["progress"], 0.0);
    }

    #[test]
    fn autotune_result_is_emitted_once_every_gain_is_set() {
        let mut instance = instance();

        for (path, value) in [
            ("pressure.autotune.result.kp", 0.7),
            ("pressure.autotune.result.ki", 0.8),
            ("pressure.autotune.result.kd", 0.9),
        ] {
            instance.state_properties.insert(
                path.to_string(),
                Some(StatePropertyInfo {
                    value: ScalarValue::Float(value),
                    records: Vec::new(),
                }),
            );
        }

        let event = init_state_event(&instance, false).expect("should emit");

        assert_eq!(
            event["pid_autotune_state"]["result"],
            serde_json::json!({ "kp": 0.7, "ki": 0.8, "kd": 0.9 })
        );
    }

    #[test]
    fn autotune_state_is_folded_to_the_legacy_spelling() {
        for (scalar, expected) in [
            ("NotStarted", "not_started"),
            ("Running", "running"),
            ("Completed", "completed"),
            ("Failed", "failed"),
        ] {
            let mut instance = instance();
            instance.state_properties.insert(
                "pressure.autotune.state".to_string(),
                Some(StatePropertyInfo {
                    value: ScalarValue::Enum(scalar.to_string()),
                    records: Vec::new(),
                }),
            );

            let event = init_state_event(&instance, false).expect("should emit");

            assert_eq!(event["pid_autotune_state"]["state"], expected);
        }
    }

    /// `EnumProperty::from_scalar` only accepts snake_case, so a PascalCase write would be rejected
    /// by the runtime.
    #[test]
    fn enum_writes_use_the_snake_case_spelling() {
        for (payload, path, expected) in [
            (
                serde_json::json!({ "SetInverterRotationDirection": false }),
                "screw.direction",
                "reverse",
            ),
            (
                serde_json::json!({ "SetInverterRegulation": false }),
                "screw.regulation",
                "pressure",
            ),
        ] {
            let RuntimeRequestKind::SetConfigProperty {
                path: actual_path,
                value,
                ..
            } = convert_request(ident(), payload).expect("should convert")
            else {
                panic!("should map to a config write");
            };

            assert_eq!(actual_path, path);
            assert_eq!(value, ScalarValue::Enum(expected.to_string()));
        }
    }

    #[test]
    fn scalar_mutations_map_to_config_writes() {
        let RuntimeRequestKind::SetConfigProperty { path, value, .. } =
            convert_request(ident(), serde_json::json!({ "SetInverterTargetRpm": 42.0 }))
                .expect("should convert")
        else {
            panic!("should map to a config write");
        };

        assert_eq!(path, "screw.target_rpm");
        assert_eq!(value, ScalarValue::Float(42.0));
    }

    /// Until `convert_request` can return several requests these have to fail cleanly rather than
    /// panic or silently drop the user's input. See the TODO at the top of this file.
    #[test]
    fn compound_mutations_are_rejected() {
        for payload in [
            serde_json::json!({ "SetPressurePidSettings": { "kp": 1.0, "ki": 0.0, "kd": 0.0 } }),
            serde_json::json!({
                "SetTemperaturePidSettings": { "kp": 1.0, "ki": 0.0, "kd": 0.0, "zone": "front" }
            }),
            serde_json::json!({
                "StartPressurePidAutoTune": { "tune_delta": 0.5, "frequency_step_hz": 5.0 }
            }),
        ] {
            assert!(
                convert_request(ident(), payload.clone()).is_err(),
                "{payload} should be rejected",
            );
        }
    }
}
