use std::time::Instant;

use control_core::{
    actors::{
        Actor,
        analog_input_getter::AnalogInputGetter,
        mitsubishi_inverter_rs485::{
            MitsubishiControlRequests, MitsubishiInverterRS485Actor, MitsubishiModbusRequest,
        },
    },
    controllers::pid::PidController,
};

/// Clampable frequency limits (in Hz)
const MIN_FREQ: f64 = 0.0;
const MAX_FREQ: f64 = 60.0;
const TRANSMISSION_RATIO: f32 = 34.0;

#[derive(Debug)]
pub struct ScrewSpeedController {
    pid: PidController,
    pub target_pressure: f32,
    pub target_rpm: f32,
    pub inverter: MitsubishiInverterRS485Actor,
    pressure_sensor: AnalogInputGetter,
    last_update: Instant,
    uses_rpm: bool,
    forward_rotation: bool,
}

impl ScrewSpeedController {
    pub fn new(
        inverter: MitsubishiInverterRS485Actor,
        kp: f64,
        ki: f64,
        kd: f64,
        target_pressure: f32,
        target_rpm: f32,
        pressure_sensor: AnalogInputGetter,
    ) -> Self {
        let now = Instant::now();
        Self {
            inverter: inverter,
            pid: PidController::new(kp, ki, kd),
            last_update: now,
            target_pressure,
            target_rpm,
            pressure_sensor,
            uses_rpm: true,
            forward_rotation: true,
        }
    }

    pub fn get_rotation_direction(&mut self) -> bool {
        self.forward_rotation
    }

    pub fn set_rotation_direction(&mut self, forward: bool) {
        let req: MitsubishiModbusRequest = match forward {
            // Our gearbox is inverted!!!
            true => MitsubishiControlRequests::StartReverseRotation.into(),
            false => MitsubishiControlRequests::StartForwardRotation.into(),
        };
        self.inverter.add_request(req);
    }

    pub fn set_target_pressure(&mut self, target_pressure: f32) {
        self.target_pressure = target_pressure;
    }

    pub fn set_target_screw_rpm(&mut self, target_rpm: f32) {
        self.target_rpm = target_rpm;
        // Use uom here and perhaps clamp it

        self.inverter
            .set_frequency_target((target_rpm / 60.0) * TRANSMISSION_RATIO);
    }

    pub fn get_uses_rpm(&mut self) -> bool {
        self.uses_rpm
    }

    pub fn set_uses_rpm(&mut self, uses_rpm: bool) {
        self.uses_rpm = uses_rpm;
    }

    // Send Motor Turn Off Request to the Inverter
    pub fn turn_motor_off(&mut self) {
        self.inverter
            .add_request(MitsubishiControlRequests::StopMotor.into());
    }

    pub fn turn_motor_on(&mut self) {
        if self.inverter.forward_rotation {
            self.inverter
                .add_request(MitsubishiControlRequests::StartForwardRotation.into());
        } else {
            self.inverter
                .add_request(MitsubishiControlRequests::StartReverseRotation.into());
        }
    }

    pub fn get_rpm(&mut self) -> f32 {
        let rpm = self.inverter.frequency / 60.0;
        self.calculate_transmission(rpm)
    }

    pub fn get_frequency(&mut self) -> f32 {
        self.inverter.frequency
    }

    pub fn get_pressure(&mut self) -> f32 {
        let normalized = self.pressure_sensor.get_normalized();
        let normalized = match normalized {
            Some(normalized) => normalized,
            None => 0.0,
        };
        // assuming full scale pressure of 10 bar
        let bar = normalized * 10.0;
        bar
    }

    pub fn calculate_transmission(&self, rpm: f32) -> f32 {
        rpm / TRANSMISSION_RATIO
    }

    pub async fn update(&mut self, now: Instant) {
        self.inverter.act(now).await;
        if !self.uses_rpm {
            let measured_pressure: f32 = self.get_pressure();
            let error = self.target_pressure - measured_pressure;
            let freq = self.pid.update(error.into(), now).clamp(MIN_FREQ, MAX_FREQ);
            self.last_update = now;
            self.inverter.set_frequency_target(freq as f32);
        }
    }

    pub fn reset(&mut self) {
        self.pid.reset();
        self.last_update = Instant::now();
    }
}
