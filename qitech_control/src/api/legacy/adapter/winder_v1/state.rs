use qitech_framework::ScalarValue;

use crate::api::types::MachineInstance;

// --- small helper functions ---
/// Maps runtime puller algorithm enum to frontend regulation name.
/// Runtime uses "Direct"/"Adaptive", frontend expects "Speed"/"Diameter".
fn map_puller_regulation(value: &str) -> &'static str {
    match value {
        "speed" | "direct" | "Speed" | "Direct" => "Speed",
        "diameter" | "adaptive" | "Diameter" | "Adaptive" => "Diameter",
        _ => "Speed",
    }
}

fn map_spool_action(value: &str) -> &'static str {
    match value {
        "no_action" | "NoAction" => "NoAction",
        "pull" | "Pull" => "Pull",
        "hold" | "Hold" => "Hold",
        _ => "NoAction",
    }
}

fn config_enum(instance: &MachineInstance, path: &str) -> Option<String> {
    config_value(instance, path)?.r#enum()
}

pub fn init_state_event(
    instance: &MachineInstance,
    is_default_state: bool,
) -> Option<serde_json::Value> {
    let traverse_mode = state_enum(instance, "traverse.mode")?;
    let traverse_state = state_enum(instance, "traverse.state")?;
    let mode = state_enum(instance, "mode")?;
    let is_homed = state_bool(instance, "traverse.homed")?;

    let can_traverse = traverse_mode == "Standby" || traverse_mode == "Traverse";

    let tension_arm_zeroed = state_value(instance, "tension_arm.zero")
    .map_or(false, |v| !matches!(v, ScalarValue::Null));
    let is_homing = traverse_state.starts_with("Homing");
    let can_wind = tension_arm_zeroed && is_homed && !is_homing;

    Some(serde_json::json!({
        "is_default_state": is_default_state,

        "traverse_state": {
            "limit_inner": config_float(instance, "traverse.limit_inner")?,
            "limit_outer": config_float(instance, "traverse.limit_outer")?,
            "position_in": 0.0,
            "position_out": 0.0,
            "is_going_in": traverse_state == "GoingIn",
            "is_going_out": traverse_state == "GoingOut",
            "is_homed": is_homed,
            "is_going_home": traverse_state.starts_with("Homing"),
            "is_traversing": traverse_state.starts_with("Traversing"),
            "laserpointer": state_bool(instance, "laser_pointer.enabled")?,
            "step_size": config_float(instance, "traverse.step_size")?,
            "padding": config_float(instance, "traverse.padding")?,
            "can_go_in": can_traverse,
            "can_go_out": can_traverse,
            "can_go_home": !is_homed,
        },

        "puller_state": {
            "regulation": config_enum(instance, "puller.speed_controller.algorithm")
                .as_deref()
                .map(map_puller_regulation)
                .unwrap_or("Speed"),
            "target_speed": config_float(instance, "puller.speed_controller.speed_desired")?,
            "forward": config_enum(instance, "puller.direction")
                .map_or(false, |s| s == "Forward" || s == "forward"),
            "gear_ratio": config_enum(instance, "puller.gear_ratio")?,
            "adaptive_speed_delta_max": config_float(instance, "puller.speed_controller.adaptive.speed_delta_max")?,
            "adaptive_adjustment_distance": config_float(instance, "puller.speed_controller.adaptive.adjustment_distance")?,
            "adaptive_change_per_step": config_float(instance, "puller.speed_controller.adaptive.increase_per_step")?,
            "allowed_diameter_deviation": config_float(instance, "puller.speed_controller.adaptive.tolerance_limit")?,
            "adaptive_reference_machine": serde_json::Value::Null,
        },

        "spool_automatic_action_state": {
            "spool_required_meters": config_float(instance, "spool_automatic.required_meters")?,
            "spool_automatic_action_mode": config_enum(instance, "spool_automatic.action")
                .as_deref()
                .map(map_spool_action)
                .unwrap_or("NoAction"),
        },

        "mode_state": {
            "mode": mode,
            "can_wind": can_wind,
        },

        "tension_arm_state": {
            "zeroed": state_value(instance, "tension_arm.zero")
                .map_or(false, |v| !matches!(v, ScalarValue::Null)),
        },

        "spool_speed_controller_state": {
            "regulation_mode": config_enum(instance, "spool.speed_controller.algorithm")?,
            "minmax_min_speed": config_float(instance, "spool.speed_controller.speed_min")?,
            "minmax_max_speed": config_float(instance, "spool.speed_controller.speed_max")?,
            "adaptive_tension_target": config_float(instance, "spool.speed_controller.adaptive.tension_target")?,
            "adaptive_radius_learning_rate": config_float(instance, "spool.speed_controller.adaptive.radius_learning_rate")?,
            "adaptive_max_speed_multiplier": config_float(instance, "spool.speed_controller.adaptive.max_speed_multiplier")?,
            "adaptive_acceleration_factor": config_float(instance, "spool.speed_controller.adaptive.acceleration_factor")?,
            "adaptive_deacceleration_urgency_multiplier": config_float(instance, "spool.speed_controller.adaptive.deacceleration_urgency_multiplier")?,
            "forward": config_enum(instance, "spool.direction")
                .map_or(false, |s| s == "Forward" || s == "forward"),
        },

        "puller_reference_machine": serde_json::Value::Null,
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

fn state_bool(instance: &MachineInstance, path: &str) -> Option<bool> {
    state_value(instance, path)?.boolean()
}

fn state_enum(instance: &MachineInstance, path: &str) -> Option<String> {
    state_value(instance, path)?.r#enum()
}
