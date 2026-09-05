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

// --- requests ---

fn convert_request(
    ident: MachineInstanceIdentification,
    data: serde_json::Value,
) -> Result<Vec<RuntimeRequestKind>, serde_json::Error> {
    #[derive(Deserialize)]
    enum Mutation {
        SetMixingMotorOn(bool),
        SetHopperAEnabled(bool),
        SetHopperATargetRpm(f64),
        SetHopperAForward(bool),
        SetHopperADosingPercent(f64),
        SetHopperBEnabled(bool),
        SetHopperBTargetRpm(f64),
        SetHopperBForward(bool),
        SetHopperBDosingPercent(f64),
        SetExtruderKgPerRpm(f64),
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

    Ok(vec![match serde_json::from_value(data)? {
        Mutation::SetMixingMotorOn(true) => command("mixing_motor.start"),
        Mutation::SetMixingMotorOn(false) => command("mixing_motor.stop"),

        Mutation::SetHopperAEnabled(true) => command("hopper_a.enable"),
        Mutation::SetHopperAEnabled(false) => command("hopper_a.disable"),
        Mutation::SetHopperATargetRpm(v) => config("hopper_a.target_rpm", ScalarValue::Float(v)),
        Mutation::SetHopperAForward(v) => config("hopper_a.forward", ScalarValue::Boolean(v)),
        Mutation::SetHopperADosingPercent(v) => {
            config("hopper_a.dosing_percent", ScalarValue::Float(v))
        }

        Mutation::SetHopperBEnabled(true) => command("hopper_b.enable"),
        Mutation::SetHopperBEnabled(false) => command("hopper_b.disable"),
        Mutation::SetHopperBTargetRpm(v) => config("hopper_b.target_rpm", ScalarValue::Float(v)),
        Mutation::SetHopperBForward(v) => config("hopper_b.forward", ScalarValue::Boolean(v)),
        Mutation::SetHopperBDosingPercent(v) => {
            config("hopper_b.dosing_percent", ScalarValue::Float(v))
        }

        Mutation::SetExtruderKgPerRpm(v) => config("extruder_kg_per_rpm", ScalarValue::Float(v)),
    }])
}

// --- state event ---

fn init_state_event(
    instance: &MachineInstance,
    is_default_state: bool,
) -> Option<serde_json::Value> {
    Some(serde_json::json!({
        "is_default_state": is_default_state,

        "mixing_motor_state": {
            "on": state_bool(instance, "mixing_motor_on")?,
        },

        "hopper_a_state": {
            "enabled": state_bool(instance, "hopper_a_enabled")?,
            "ready": state_bool(instance, "hopper_a_ready")?,
            "error": state_bool(instance, "hopper_a_error")?,
            "target_rpm": config_float(instance, "hopper_a.target_rpm")?,
            "forward": config_bool(instance, "hopper_a.forward")?,
            "dosing_percent": config_float(instance, "hopper_a.dosing_percent")?,
            "calibration_steps_per_kgh": config_float(instance, "hopper_a.calibration_steps_per_kgh")?,
        },

        "hopper_b_state": {
            "enabled": state_bool(instance, "hopper_b_enabled")?,
            "ready": state_bool(instance, "hopper_b_ready")?,
            "error": state_bool(instance, "hopper_b_error")?,
            "target_rpm": config_float(instance, "hopper_b.target_rpm")?,
            "forward": config_bool(instance, "hopper_b.forward")?,
            "dosing_percent": config_float(instance, "hopper_b.dosing_percent")?,
            "calibration_steps_per_kgh": config_float(instance, "hopper_b.calibration_steps_per_kgh")?,
        },

        "extruder_kg_per_rpm": config_float(instance, "extruder_kg_per_rpm")?,
    }))
}

// --- live values ---

fn init_measurements_event(instance: &MachineInstance) -> Option<serde_json::Value> {
    let get = |path: &str| -> Option<f64> { instance.measurements.get(path)?.as_ref()?.value };

    Some(serde_json::json!({
        "hopper_a_rpm": get("hopper_a.rpm")?,
        "hopper_b_rpm": get("hopper_b.rpm")?,
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

fn config_bool(instance: &MachineInstance, path: &str) -> Option<bool> {
    config_value(instance, path)?.boolean()
}

fn state_bool(instance: &MachineInstance, path: &str) -> Option<bool> {
    state_value(instance, path)?.boolean()
}
