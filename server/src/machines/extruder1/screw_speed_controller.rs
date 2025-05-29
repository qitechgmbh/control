use std::time::Instant;

use control_core::{
    actors::{
        Actor,
        analog_input_getter::AnalogInputGetter,
        mitsubishi_inverter_rs485::{MitsubishiControlRequests, MitsubishiInverterRS485Actor},
    },
    controllers::pid::PidController,
    converters::transmission_converter::TransmissionConverter,
};
use uom::si::{
    angular_velocity::revolution_per_minute,
    f64::{AngularVelocity, Frequency, Pressure},
    frequency::{cycle_per_minute, hertz},
    pressure::bar,
};

/// Clampable frequency limits (in Hz)
const MIN_FREQ: f64 = 0.0;
const MAX_FREQ: f64 = 60.0;

#[derive(Debug)]
pub struct ScrewSpeedController {
    pid: PidController,
    pub target_pressure: Pressure,
    pub target_rpm: AngularVelocity,
    pub inverter: MitsubishiInverterRS485Actor,
    pressure_sensor: AnalogInputGetter,
    last_update: Instant,
    uses_rpm: bool,
    forward_rotation: bool,
    transmission_converter: TransmissionConverter,
    motor_on: bool,
}

impl ScrewSpeedController {
    pub fn new(
        inverter: MitsubishiInverterRS485Actor,
        target_pressure: Pressure,
        target_rpm: AngularVelocity,
        pressure_sensor: AnalogInputGetter,
    ) -> Self {
        let now = Instant::now();
        Self {
            inverter: inverter,
            // need to tune
            pid: PidController::new(0.1, 0.1, 0.1),
            last_update: now,
            target_pressure,
            target_rpm,
            pressure_sensor,
            uses_rpm: true,
            forward_rotation: true,
            transmission_converter: TransmissionConverter::new(),
            motor_on: false,
        }
    }

    pub fn get_target_rpm(&mut self) -> AngularVelocity {
        self.target_rpm
    }

    pub fn get_rotation_direction(&mut self) -> bool {
        self.forward_rotation
    }

    pub fn set_rotation_direction(&mut self, forward: bool) {
        self.forward_rotation = forward;
        if self.motor_on {
            if self.forward_rotation {
                self.inverter
                    .add_request(MitsubishiControlRequests::StartReverseRotation.into());
            } else {
                self.inverter
                    .add_request(MitsubishiControlRequests::StartForwardRotation.into());
            }
        }
    }

    pub fn set_target_pressure(&mut self, target_pressure: Pressure) {
        self.target_pressure = target_pressure;
    }

    pub fn set_target_screw_rpm(&mut self, target_rpm: AngularVelocity) {
        // Use uom here and perhaps clamp it
        let target_motor_rpm = self
            .transmission_converter
            .calculate_screw_input_rpm(target_rpm);

        self.target_rpm = target_rpm;
        let target_frequency =
            Frequency::new::<cycle_per_minute>(target_motor_rpm.get::<revolution_per_minute>());

        self.inverter.set_frequency_target(target_frequency);
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
        self.motor_on = false;
    }

    pub fn turn_motor_on(&mut self) {
        if self.forward_rotation {
            self.inverter
                .add_request(MitsubishiControlRequests::StartReverseRotation.into());
        } else {
            self.inverter
                .add_request(MitsubishiControlRequests::StartForwardRotation.into());
        }
        self.motor_on = true;
    }

    pub fn get_screw_rpm(&mut self) -> AngularVelocity {
        let frequency = self.get_frequency();
        let rpm = frequency.get::<cycle_per_minute>();
        self.transmission_converter
            .calculate_screw_output_rpm(AngularVelocity::new::<revolution_per_minute>(rpm))
    }

    pub fn get_frequency(&mut self) -> Frequency {
        self.inverter.frequency
    }

    pub fn get_target_pressure(&self) -> Pressure {
        self.target_pressure
    }

    pub fn get_pressure(&mut self) -> Pressure {
        let normalized = self.pressure_sensor.get_normalized();
        let normalized = match normalized {
            Some(normalized) => normalized,
            None => 0.0,
        };
        // assuming full scale pressure of 350 bar, which is the max range of our sensors
        let pressure: f64 = normalized as f64 * 350.0;
        return Pressure::new::<bar>(pressure);
    }

    pub async fn update(&mut self, now: Instant) {
        self.inverter.act(now).await;
        self.pressure_sensor.act(now).await;

        if !self.uses_rpm {
            let measured_pressure = self.get_pressure();
            let error = self.target_pressure - measured_pressure;

            let freq = self
                .pid
                .update(error.get::<bar>(), now)
                .clamp(MIN_FREQ, MAX_FREQ);
            let frequency = Frequency::new::<hertz>(freq);

            self.last_update = now;
            self.inverter.set_frequency_target(frequency);
        }
    }

    pub fn reset(&mut self) {
        self.pid.reset();
        self.last_update = Instant::now();
    }
}
