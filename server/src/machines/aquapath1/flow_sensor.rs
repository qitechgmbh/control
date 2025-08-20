use std::time::Instant;

use control_core::{
    controllers::clamping_timeagnostic_pid::ClampingTimeagnosticPidController,
    helpers::interpolation::normalize,
};
use ethercat_hal::io::analog_input::AnalogInput;
use uom::si::{
    electric_potential::volt,
    f64::{ElectricPotential, VolumeRate},
    volume_rate::liter_per_minute,
};

#[derive(Debug)]
pub struct FlowSensor {
    pub pid: ClampingTimeagnosticPidController,
    flow_sensor: AnalogInput,
    target_flow: f64,
    last_update: Instant,
    pub current_flow: VolumeRate,
}

impl FlowSensor {
    pub fn new(temperature_sensor: AnalogInput, target_temperature: f64) -> Self {
        let now = Instant::now();
        Self {
            // need to tune
            pid: ClampingTimeagnosticPidController::simple_new(0.01, 0.0, 0.02),
            last_update: now,
            flow_sensor: temperature_sensor,
            target_flow: target_temperature,
            current_flow: VolumeRate::new::<liter_per_minute>(0.0),
        }
    }

    pub fn set_target_temperature(&mut self, target_flow: f64) {
        self.reset_pid();
        self.target_flow = target_flow;
    }

    pub fn get_target_temperature(&self) -> f64 {
        self.target_flow
    }

    pub fn get_sensor_potential(&self) -> Result<ElectricPotential, anyhow::Error> {
        let phys: ethercat_hal::io::analog_input::physical::AnalogInputValue =
            self.flow_sensor.get_physical();

        match phys {
            ethercat_hal::io::analog_input::physical::AnalogInputValue::Potential(quantity) => {
                Ok(quantity)
            }
            ethercat_hal::io::analog_input::physical::AnalogInputValue::Current(_) => {
                Err(anyhow::anyhow!("Current is not expected"))
            }
        }
    }

    pub fn reset_pid(&mut self) {
        self.pid.reset()
    }

    pub fn get_flow(&mut self) -> VolumeRate {
        let potential_result = self.get_sensor_potential();
        let potential = match potential_result {
            Ok(potential) => potential.get::<volt>(),
            Err(_) => {
                tracing::error!("cant get pressure sensor reading");
                return VolumeRate::new::<liter_per_minute>(0.0);
            }
        };
        let normalized = normalize(potential, 3.0, 24.0);
        // Assuming the sensor range is 0-100 L/min, adjust these values according to your sensor specifications
        let actual_flow = normalized * 100.0;

        return VolumeRate::new::<liter_per_minute>(potential);
    }

    pub fn update(&mut self, now: Instant) {
        self.current_flow = self.get_flow();
        self.last_update = now;
    }

    pub fn reset(&mut self) {
        self.pid.reset();
        self.last_update = Instant::now();
    }
}
