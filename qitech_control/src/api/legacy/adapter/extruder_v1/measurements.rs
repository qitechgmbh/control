use crate::api::types::MachineInstance;

pub fn init_measurements_event(instance: &MachineInstance) -> Option<serde_json::Value> {
    let temperature = |zone: &str| measurement(instance, &format!("heating.{zone}.temperature"));
    let power = |zone: &str| measurement(instance, &format!("heating.{zone}.power"));

    Some(serde_json::json!({
        "motor_status": {
            "screw_rpm": measurement(instance, "motor.rpm")?,
            "frequency": measurement(instance, "motor.frequency")?,
            // Unused by the frontend's zod schema, kept for parity with the pre-migration event.
            "voltage":   measurement(instance, "motor.voltage")?,
            "current":   measurement(instance, "motor.current")?,
            "power":     measurement(instance, "motor.power")?,
        },

        "pressure": measurement(instance, "pressure.value")?,

        "nozzle_temperature": temperature("nozzle")?,
        "front_temperature":  temperature("front")?,
        "back_temperature":   temperature("back")?,
        "middle_temperature": temperature("middle")?,

        "nozzle_power": power("nozzle")?,
        "front_power":  power("front")?,
        "back_power":   power("back")?,
        "middle_power": power("middle")?,

        "combined_power":   measurement(instance, "power.combined")?,
        "total_energy_kwh": measurement(instance, "energy.total")?,
    }))
}

fn measurement(instance: &MachineInstance, path: &str) -> Option<f64> {
    instance.measurements.get(path)?.as_ref()?.value
}
