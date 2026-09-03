use qitech_framework::ScalarValue;

use crate::api::types::MachineInstance;

fn map_enum_to_pascal(category: &str, value: &str) -> String {
    match category {
        "mode" => match value {
            "standby" | "Standby" => "Standby",
            "hold" | "Hold" => "Hold",
            "pull" | "Pull" => "Pull",
            "wind" | "Wind" => "Wind",
            other => other,
        },
        "puller_regulation" => match value {
            "speed" | "direct" | "Speed" | "Direct" => "Speed",
            "diameter" | "adaptive" | "Diameter" | "Adaptive" => "Diameter",
            other => other,
        },
        "gear_ratio" => match value {
            "one_to_one" | "OneToOne" => "OneToOne",
            "one_to_five" | "OneToFive" => "OneToFive",
            "one_to_ten" | "OneToTen" => "OneToTen",
            other => other,
        },
        "spool_regulation" => match value {
            "adaptive" | "Adaptive" => "Adaptive",
            "minmax" | "MinMax" => "MinMax",
            other => other,
        },
        "spool_automatic_action" => match value {
            "no_action" | "NoAction" => "NoAction",
            "pull" | "Pull" => "Pull",
            "hold" | "Hold" => "Hold",
            other => other,
        },
        _ => value,
    }
    .to_string()
}

pub fn init_state_event(
    instance: &MachineInstance,
    is_default_state: bool,
) -> Option<serde_json::Value> {
    let get_state = |name: &'static str| -> Option<ScalarValue> {
        let prop = instance.state_properties.get(name)?;
        match prop.as_ref() {
            Some(info) => Some(info.value.clone()),
            None => Some(ScalarValue::Null),
        }
    };

    let get_config = |name: &'static str| -> Option<ScalarValue> {
        Some(
            instance
                .config_properties
                .get(name)?
                .as_ref()?
                .value
                .clone(),
        )
    };

    let can_go_in = {
        match get_state("traverse.mode").and_then(|v| v.r#enum()) {
            Some(mode) => mode == "standby" || mode == "traverse",
            None => false,
        }
    };

    let can_go_out = {
        match get_state("traverse.mode").and_then(|v| v.r#enum()) {
            Some(mode) => mode == "standby" || mode == "traverse",
            None => false,
        }
    };

    let can_wind = {
        match get_state("mode").and_then(|v| v.r#enum()) {
            Some(mode) => mode == "standby" || mode == "wind",
            None => false,
        }
    };

    let mode_pascal = get_state("mode")
        .and_then(|v| v.r#enum())
        .map(|s| map_enum_to_pascal("mode", &s));

    let puller_regulation = get_config("puller.speed_controller.algorithm")
        .and_then(|v| v.r#enum())
        .map(|s| map_enum_to_pascal("puller_regulation", &s));

    let gear_ratio = get_config("puller.gear_ratio")
        .and_then(|v| v.r#enum())
        .map(|s| map_enum_to_pascal("gear_ratio", &s));

    let spool_regulation = get_config("spool.speed_controller.algorithm")
        .and_then(|v| v.r#enum())
        .map(|s| map_enum_to_pascal("spool_regulation", &s));

    Some(serde_json::json!({
        "is_default_state": is_default_state,

        "traverse_state": {
            "limit_inner": get_config("traverse.limit_inner")
                .and_then(|v| v.float())
                .unwrap_or(0.0),

            "limit_outer": get_config("traverse.limit_outer")
                .and_then(|v| v.float())
                .unwrap_or(0.0),

            "position_in": 0.0,
            "position_out": 0.0,

            "is_going_in": get_state("traverse.state")
                .and_then(|v| v.r#enum())
                .map_or(false, |s| s == "going_in"),

            "is_going_out": get_state("traverse.state")
                .and_then(|v| v.r#enum())
                .map_or(false, |s| s == "going_out"),

            "is_homed": get_state("traverse.homed")
                .and_then(|v| v.boolean())
                .unwrap_or(false),

            "is_going_home": get_state("traverse.state")
                .and_then(|v| v.r#enum())
                .map_or(false, |s| s.starts_with("homing (")),

            "is_traversing": get_state("traverse.state")
                .and_then(|v| v.r#enum())
                .map_or(false, |s| s.starts_with("traversing (")),

            "laserpointer": get_state("laser_pointer.enabled")
                .and_then(|v| v.boolean())
                .unwrap_or(false),

            "step_size": get_config("traverse.step_size")
                .and_then(|v| v.float())
                .unwrap_or(0.0),

            "padding": get_config("traverse.padding")
                .and_then(|v| v.float())
                .unwrap_or(0.0),

            "can_go_in": can_go_in,
            "can_go_out": can_go_out,

            "can_go_home": !get_state("traverse.homed")
                .and_then(|v| v.boolean())
                .unwrap_or(true),
        },

        "puller_state": {
            "regulation": puller_regulation
                .unwrap_or_else(|| "Speed".to_string()),

            "target_speed": get_config("puller.speed_controller.speed_desired")
                .and_then(|v| v.float())
                .unwrap_or(0.0),

            "forward": get_config("puller.direction")
                .and_then(|v| v.r#enum())
                .map_or(false, |s| s == "forward"),

            "gear_ratio": gear_ratio
                .unwrap_or_else(|| "OneToOne".to_string()),

            "adaptive_speed_delta_max": get_config(
                "puller.speed_controller.adaptive.speed_delta_max",
            )
                .and_then(|v| v.float())
                .unwrap_or(0.0),

            "adaptive_adjustment_distance": get_config(
                "puller.speed_controller.adaptive.adjustment_distance",
            )
                .and_then(|v| v.float())
                .unwrap_or(0.0),

            "adaptive_change_per_step": get_config(
                "puller.speed_controller.adaptive.increase_per_step",
            )
                .and_then(|v| v.float())
                .unwrap_or(0.0),

            "allowed_diameter_deviation": get_config(
                "puller.speed_controller.adaptive.tolerance_limit",
            )
                .and_then(|v| v.float())
                .unwrap_or(0.0),

            "adaptive_reference_machine": serde_json::Value::Null,
        },

        "spool_automatic_action_state": {
            "spool_required_meters": 250.0,
            "spool_automatic_action_mode": "NoAction",
        },

        "mode_state": {
            "mode": mode_pascal
                .unwrap_or_else(|| "Standby".to_string()),
            "can_wind": can_wind,
        },

        "tension_arm_state": {
            "zeroed": !matches!(
                get_state("tension_arm.zero").unwrap_or(ScalarValue::Null),
                ScalarValue::Null
            ),
        },

        "spool_speed_controller_state": {
            "regulation_mode": spool_regulation
                .unwrap_or_else(|| "MinMax".to_string()),

            "minmax_min_speed": get_config(
                "spool.speed_controller.speed_min",
            )
                .and_then(|v| v.float())
                .unwrap_or(0.0),

            "minmax_max_speed": get_config(
                "spool.speed_controller.speed_max",
            )
                .and_then(|v| v.float())
                .unwrap_or(0.0),

            "adaptive_tension_target": get_config(
                "spool.speed_controller.adaptive.tension_target",
            )
                .and_then(|v| v.float())
                .unwrap_or(0.0),

            "adaptive_radius_learning_rate": get_config(
                "spool.speed_controller.adaptive.radius_learning_rate",
            )
                .and_then(|v| v.float())
                .unwrap_or(0.0),

            "adaptive_max_speed_multiplier": get_config(
                "spool.speed_controller.adaptive.max_speed_multiplier",
            )
                .and_then(|v| v.float())
                .unwrap_or(0.0),

            "adaptive_acceleration_factor": get_config(
                "spool.speed_controller.adaptive.acceleration_factor",
            )
                .and_then(|v| v.float())
                .unwrap_or(0.0),

            "adaptive_deacceleration_urgency_multiplier": get_config(
                "spool.speed_controller.adaptive.deacceleration_urgency_multiplier",
            )
                .and_then(|v| v.float())
                .unwrap_or(0.0),

            "forward": get_config("spool.direction")
                .and_then(|v| v.r#enum())
                .map_or(false, |s| s == "forward"),
        },

        "puller_reference_machine": serde_json::Value::Null,
    }))
}
