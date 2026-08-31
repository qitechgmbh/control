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
) -> Result<RuntimeRequestKind, serde_json::Error> {
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
    },
    };

    Ok(res)
}

fn init_state_event(
    instance: &MachineInstance,
    is_default_state: bool,
) -> Option<serde_json::Value> {
    let target_diameter = instance
        .config_properties
        .get("diameter.target")?
        .as_ref()?
        .value
        .clone()
        .float()
        .expect("Cannot be null");

    let higher_tolerance = instance
        .config_properties
        .get("diameter.tolerance.upper")?
        .as_ref()?
        .value
        .clone()
        .float()
        .expect("Cannot be null");

    let lower_tolerance = instance
        .config_properties
        .get("diameter.tolerance.lower")?
        .as_ref()?
        .value
        .clone()
        .float()
        .expect("Cannot be null");

    let in_tolerance = instance
        .state_properties
        .get("in_tolerance")?
        .as_ref()?
        .value
        .clone()
        .boolean()
        .expect("Cannot be null");

    Some(serde_json::json!({
        "is_default_state": is_default_state,
        "laser_state": serde_json::json!({
            "target_diameter":  target_diameter,
            "higher_tolerance": higher_tolerance,
            "lower_tolerance":  lower_tolerance,
            "in_tolerance":     in_tolerance,
            "global_warning":   false,
        })
    }))
}

fn init_measurements_event(instance: &MachineInstance) -> Option<serde_json::Value> {
    let get = |name: &'static str| -> Option<Option<f64>> {
        Some(instance.measurements.get(name)?.as_ref()?.value)
    };

    Some(serde_json::json!({
        "diameter":   get("diameter").expect("Non nullable measurement is null"),
        "x_diameter": get("diameter_x"),
        "y_diameter": get("diameter_y"),
        "roundness":  get("roundness"),
    }))
}
