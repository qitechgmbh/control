use qitech_framework::ScalarValue;

use crate::api::types::MachineInstance;

/// The four heating zones, in the order the legacy payload lists them.
const ZONES: [&str; 4] = ["nozzle", "front", "back", "middle"];

pub fn init_state_event(
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

        "rotation_state": {
            "forward": config_enum_is(instance, "screw.direction", "forward")?,
        },

        // The only enum handed to the frontend verbatim: its zod schema spells the modes
        // PascalCase. `mode` is a state property, so it is always the variant ident.
        "mode_state": {
            "mode": state_enum(instance, "mode")?,
        },

        "regulation_state": {
            "uses_rpm": config_enum_is(instance, "screw.regulation", "rpm")?,
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
            "min_extrusion_temperature": config_float(instance, "extrusion.min_temperature")?,
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
            "state": snake_case(&state_enum(instance, "pressure.autotune.state")?),
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

/// Folds a variant ident to its snake_case spelling, leaving an already snake_case value unchanged.
fn snake_case(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 4);

    for (i, ch) in value.char_indices() {
        if ch.is_ascii_uppercase() {
            if i != 0 {
                out.push('_');
            }

            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch);
        }
    }

    out
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

/// Whether the enum config property at `path` currently holds `variant`, named in the schema's
/// snake_case spelling.
///
/// Enum config properties round-trip asymmetrically through the framework: the registered default
/// arrives as the variant ident (`EnumProperty::into_scalar` writes `Forward`) while an external
/// write is echoed back in the snake_case spelling the write itself had to use
/// (`EnumProperty::from_scalar` only accepts `forward`). Comparing against either spelling alone
/// silently stops matching as soon as the property is written once — which is how the direction
/// toggle got stuck on reverse — so fold to snake_case first and accept both.
fn config_enum_is(instance: &MachineInstance, path: &str, variant: &str) -> Option<bool> {
    Some(snake_case(&config_enum(instance, path)?) == variant)
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
    use super::*;

    #[test]
    fn snake_case_folds_idents_and_passes_snake_case_through() {
        assert_eq!(snake_case("Forward"), "forward");
        assert_eq!(snake_case("forward"), "forward");
        assert_eq!(snake_case("NotStarted"), "not_started");
        assert_eq!(snake_case("not_started"), "not_started");
    }
}
