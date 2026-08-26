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

fn convert_request(
    ident: MachineInstanceIdentification,
    data: serde_json::Value,
) -> Result<RuntimeRequestKind, serde_json::Error> {
    #[derive(Deserialize)]
    #[allow(clippy::enum_variant_names)]
    enum Mutation {
        SetTargetDiameter(f64),
        SetLowerTolerance(f64),
        SetHigherTolerance(f64),
        SetGlobalWarning(bool),
    }

    Ok(match serde_json::from_value(data)? {
        Mutation::SetTargetDiameter(v) => RuntimeRequestKind::SetConfigProperty {
            target: ident,
            path: "diameter.target".to_string(),
            value: ScalarValue::Float(v),
        },
        Mutation::SetLowerTolerance(v) => RuntimeRequestKind::SetConfigProperty {
            target: ident,
            path: "diameter.tolerance.lower".to_string(),
            value: ScalarValue::Float(v),
        },
        Mutation::SetHigherTolerance(v) => RuntimeRequestKind::SetConfigProperty {
            target: ident,
            path: "diameter.tolerance.upper".to_string(),
            value: ScalarValue::Float(v),
        },
        Mutation::SetGlobalWarning(v) => RuntimeRequestKind::SetConfigProperty {
            target: ident,
            path: "out_of_tolerance.active".to_string(),
            value: ScalarValue::Boolean(v),
        },
    })
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
