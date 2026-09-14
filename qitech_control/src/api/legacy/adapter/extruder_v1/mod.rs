use crate::api::legacy::MachineLegacyDataAdapter;

mod request;
use request::convert_request;

mod measurements;
use measurements::init_measurements_event;

mod state;
use state::init_state_event;

/// Serves both extruder generations: `ExtruderV1` (machine id 4, the frontend's "extruder2") and
/// `ExtruderV2` (machine id 22, the frontend's "extruder3"). Their schemas are identical apart from
/// the identification block.
pub const ADAPTER: MachineLegacyDataAdapter = MachineLegacyDataAdapter {
    convert_request,
    init_state_event,
    init_measurements_event,
};

#[cfg(test)]
mod tests {
    use qitech_framework::MachineIdentification;
    use qitech_framework::MachineInstanceIdentification;
    use qitech_framework::MachineSchema;
    use qitech_framework::RuntimeRequestKind;
    use qitech_framework::ScalarValue;

    use super::*;
    use crate::api::types::ConfigPropertyInfo;
    use crate::api::types::MachineInstance;
    use crate::api::types::MeasurementInfo;
    use crate::api::types::StatePropertyInfo;

    const SCHEMA: &str = include_str!("../../../../../schemas/extruder_v1.yaml");
    const SCHEMA_V2: &str = include_str!("../../../../../schemas/extruder_v2.yaml");

    /// Distinct values throughout, so a field wired to the wrong property fails the comparison.
    fn config_fixture() -> Vec<(&'static str, ScalarValue)> {
        vec![
            ("screw.direction", ScalarValue::Enum("Forward".into())),
            ("screw.regulation", ScalarValue::Enum("Rpm".into())),
            ("screw.target_rpm", ScalarValue::Float(12.0)),
            ("screw.target_pressure", ScalarValue::Float(34.0)),
            ("pressure.limit", ScalarValue::Float(90.0)),
            ("pressure.limit_enabled", ScalarValue::Boolean(true)),
            ("extrusion.min_temperature", ScalarValue::Float(150.0)),
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
            ..Default::default()
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
            let requests = convert_request(ident(), payload.clone()).expect("should convert");
            assert_eq!(
                requests.len(),
                1,
                "{payload} should map to a single request"
            );

            let RuntimeRequestKind::ExecuteCommand { path, .. } = &requests[0] else {
                panic!("{payload} should map to a command");
            };
            let path = path.clone();

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
                    "min_extrusion_temperature": 150.0,
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
            let requests = convert_request(ident(), payload).expect("should convert");
            assert_eq!(requests.len(), 1);

            let RuntimeRequestKind::SetConfigProperty {
                path: actual_path,
                value,
                ..
            } = &requests[0]
            else {
                panic!("should map to a config write");
            };

            assert_eq!(actual_path, path);
            assert_eq!(*value, ScalarValue::Enum(expected.to_string()));
        }
    }

    /// A registered enum arrives as the variant ident, a written one in the snake_case spelling the
    /// write had to use — both have to read back as the same state. Comparing against the ident
    /// alone left the direction toggle stuck on reverse: once written, a later `forward` write
    /// never matched `"Forward"` again.
    #[test]
    fn enum_reads_accept_both_spellings() {
        // Per property: the payload field it feeds, and every spelling the cache can hold for it —
        // the two variant idents, then the two snake_case spellings a write echoes back.
        let properties = [
            (
                "screw.direction",
                "rotation_state",
                "forward",
                [
                    ("Forward", true),
                    ("Reverse", false),
                    ("forward", true),
                    ("reverse", false),
                ],
            ),
            (
                "screw.regulation",
                "regulation_state",
                "uses_rpm",
                [
                    ("Rpm", true),
                    ("Pressure", false),
                    ("rpm", true),
                    ("pressure", false),
                ],
            ),
        ];

        for (path, field, key, spellings) in properties {
            for (value, expected) in spellings {
                let mut instance = instance();
                instance.config_properties.insert(
                    path.to_string(),
                    Some(ConfigPropertyInfo {
                        value: ScalarValue::Enum(value.to_string()),
                        default: ScalarValue::Enum(value.to_string()),
                        capability: Default::default(),
                        constraints: Default::default(),
                        records: Vec::new(),
                    }),
                );

                let event = init_state_event(&instance, false).expect("should emit");

                assert_eq!(
                    event[field][key], expected,
                    "`{path}` holding `{value}` should read back as {expected}",
                );
            }
        }
    }

    #[test]
    fn scalar_mutations_map_to_config_writes() {
        let requests =
            convert_request(ident(), serde_json::json!({ "SetInverterTargetRpm": 42.0 }))
                .expect("should convert");
        assert_eq!(requests.len(), 1);

        let RuntimeRequestKind::SetConfigProperty { path, value, .. } = &requests[0] else {
            panic!("should map to a config write");
        };

        assert_eq!(path, "screw.target_rpm");
        assert_eq!(*value, ScalarValue::Float(42.0));
    }

    /// Asserts that `request` is a `SetConfigProperty` at `path` with the given float value.
    fn assert_config_write(request: &RuntimeRequestKind, path: &str, value: f64) {
        let RuntimeRequestKind::SetConfigProperty {
            path: actual_path,
            value: actual_value,
            ..
        } = request
        else {
            panic!("expected a config write, got {request:?}");
        };

        assert_eq!(actual_path, path);
        assert_eq!(*actual_value, ScalarValue::Float(value));
    }

    #[test]
    fn set_pressure_pid_settings_produces_three_config_writes() {
        let requests = convert_request(
            ident(),
            serde_json::json!({ "SetPressurePidSettings": { "kp": 1.0, "ki": 2.0, "kd": 3.0 } }),
        )
        .expect("should convert");

        assert_eq!(requests.len(), 3);
        assert_config_write(&requests[0], "pid.pressure.kp", 1.0);
        assert_config_write(&requests[1], "pid.pressure.ki", 2.0);
        assert_config_write(&requests[2], "pid.pressure.kd", 3.0);
    }

    #[test]
    fn set_temperature_pid_settings_targets_the_requested_zone() {
        let requests = convert_request(
            ident(),
            serde_json::json!({
                "SetTemperaturePidSettings": { "kp": 1.0, "ki": 2.0, "kd": 3.0, "zone": "front" }
            }),
        )
        .expect("should convert");

        assert_eq!(requests.len(), 3);
        assert_config_write(&requests[0], "pid.temperature.front.kp", 1.0);
        assert_config_write(&requests[1], "pid.temperature.front.ki", 2.0);
        assert_config_write(&requests[2], "pid.temperature.front.kd", 3.0);
    }

    /// The two config writes must land before the start command fires, since the runtime reads
    /// them synchronously when the command executes.
    #[test]
    fn start_pressure_pid_auto_tune_writes_config_before_issuing_the_command() {
        let requests = convert_request(
            ident(),
            serde_json::json!({
                "StartPressurePidAutoTune": { "tune_delta": 0.5, "frequency_step_hz": 5.0 }
            }),
        )
        .expect("should convert");

        assert_eq!(requests.len(), 3);
        assert_config_write(&requests[0], "pressure.autotune.tune_delta", 0.5);
        assert_config_write(&requests[1], "pressure.autotune.frequency_step", 5.0);

        let RuntimeRequestKind::ExecuteCommand { path, .. } = &requests[2] else {
            panic!("expected the third request to be the start command");
        };
        assert_eq!(path, "pressure.autotune.start");
    }
}
