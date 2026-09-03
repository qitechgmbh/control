use crate::api::types::MachineInstance;

pub fn init_measurements_event(instance: &MachineInstance) -> Option<serde_json::Value> {
    let get = |name: &'static str| -> Option<Option<f64>> {
        let Some(prop) = instance.measurements.get(name) else {
            panic!("Where is da measurement? {name}");
        };

        prop.as_ref().map(|info| info.value)
    };

    Some(serde_json::json!({
        "traverse_position": get("traverse.position")?
            .expect("Non nullable measurement is null"),

        "puller_speed": get("puller.speed")?
            .expect("Non nullable measurement is null"),

        "spool_rpm": get("spool.rpm")?
            .expect("Non nullable measurement is null"),

        "spool_progress": get("spool.progress")?
            .expect("Non nullable measurement is null"),

        "tension_arm_angle": get("tension_arm.angle")?
            .expect("Non nullable measurement is null"),
    }))
}
