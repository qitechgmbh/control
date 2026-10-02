use crate::api::types::MachineInstance;

pub fn init_measurements_event(instance: &MachineInstance) -> Option<serde_json::Value> {
    Some(serde_json::json!({
        "traverse_position": measurement(instance, "traverse.position")?,
        "puller_speed": measurement(instance, "puller.speed")?,
        "spool_rpm": measurement(instance, "spool.rpm")?,
        "spool_progress": measurement(instance, "spool.progress")?,
        "tension_arm_angle": measurement(instance, "tension_arm.angle")?,
    }))
}

fn measurement(instance: &MachineInstance, path: &str) -> Option<f64> {
    instance.measurements.get(path)?.as_ref()?.value
}
