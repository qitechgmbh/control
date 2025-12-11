use std::time::Instant;

use control_core::{
    controllers::clamping_timeagnostic_pid::ClampingTimeagnosticPidController,
    helpers::interpolation::normalize,
    transmission::{Transmission, fixed::FixedTransmission},
};
use ethercat_hal::io::analog_input::AnalogInput;
use units::angular_velocity::revolution_per_minute;
use units::electric_current::milliampere;
use units::f64::*;
use units::frequency::{cycle_per_minute, hertz};
use units::pressure::bar;

use crate::extruder1::mitsubishi_cs80::MitsubishiCS80Status;

use super::mitsubishi_cs80::{MitsubishiCS80, MotorStatus};

#[derive(Debug)]
pub struct ScrewSpeedController {
    pub pid: ClampingTimeagnosticPidController,
    pub target_pressure: Pressure,
    pub target_rpm: AngularVelocity,
    pub inverter: MitsubishiCS80,
    pressure_sensor: AnalogInput,
    last_update: Instant,
    uses_rpm: bool,
    forward_rotation: bool,
    transmission: FixedTransmission,
    frequency: Frequency,
    maximum_frequency: Frequency,
    minimum_frequency: Frequency,
    motor_on: bool,
    nozzle_pressure_limit: Pressure,
    nozzle_pressure_limit_enabled: bool,
}

impl ScrewSpeedController {
    pub fn new(
        inverter: MitsubishiCS80,
        target_pressure: Pressure,
        target_rpm: AngularVelocity,
        pressure_sensor: AnalogInput,
        transmission: FixedTransmission,
    ) -> Self {
        let now = Instant::now();
        Self {
            inverter,
            // need to tune
            pid: ClampingTimeagnosticPidController::simple_new(0.01, 0.0, 0.02),
            last_update: now,
            target_pressure,
            target_rpm,
            pressure_sensor,
            uses_rpm: true,
            forward_rotation: true,
            transmission: transmission,
            //FixedTransmission::new(1.0 / 34.0),
            motor_on: false,
            nozzle_pressure_limit: Pressure::new::<bar>(100.0),
            nozzle_pressure_limit_enabled: true,
            frequency: Frequency::new::<hertz>(0.0),
            maximum_frequency: Frequency::new::<hertz>(60.0),
            minimum_frequency: Frequency::new::<hertz>(0.0),
        }
    }

    pub const fn get_inverter_status(&mut self) -> MitsubishiCS80Status {
        self.inverter.status
    }

    pub const fn get_motor_enabled(&mut self) -> bool {
        self.motor_on
    }

    pub fn set_nozzle_pressure_limit(&mut self, pressure: Pressure) {
        self.nozzle_pressure_limit = pressure;
    }

    pub fn get_nozzle_pressure_limit(&mut self) -> Pressure {
        self.nozzle_pressure_limit
    }

    pub const fn get_nozzle_pressure_limit_enabled(&mut self) -> bool {
        self.nozzle_pressure_limit_enabled
    }

    pub const fn set_nozzle_pressure_limit_is_enabled(&mut self, enabled: bool) {
        self.nozzle_pressure_limit_enabled = enabled;
    }

    pub fn get_target_rpm(&mut self) -> AngularVelocity {
        self.target_rpm
    }

    pub const fn get_rotation_direction(&mut self) -> bool {
        self.forward_rotation
    }

    pub fn set_rotation_direction(&mut self, forward: bool) {
        self.forward_rotation = forward;
        if self.motor_on {
            self.inverter.set_rotation(self.forward_rotation);
        }
    }

    pub fn set_target_pressure(&mut self, target_pressure: Pressure) {
        self.reset_pid();
        self.target_pressure = target_pressure;
    }

    pub fn set_target_screw_rpm(
        &mut self,
        target_rpm: AngularVelocity,
        _motor_rpm_rating: AngularVelocity,
        motor_poles: usize,
    ) {
        // Use uom here and perhaps clamp it
        let target_motor_rpm = self
            .transmission
            .calculate_angular_velocity_input(target_rpm);

        self.target_rpm = target_rpm;

        let target_frequency: Frequency = Frequency::new::<hertz>(
            target_motor_rpm.get::<revolution_per_minute>() as f64 / 120.0 * motor_poles as f64,
        );

        self.inverter.set_frequency_target(target_frequency);
    }

    pub const fn get_uses_rpm(&mut self) -> bool {
        self.uses_rpm
    }

    pub const fn set_uses_rpm(&mut self, uses_rpm: bool) {
        self.uses_rpm = uses_rpm;
    }

    // Send Motor Turn Off Request to the Inverter
    pub fn turn_motor_off(&mut self) {
        self.inverter.stop_motor();
        self.motor_on = false;
    }

    pub fn turn_motor_on(&mut self) {
        self.inverter.set_rotation(self.forward_rotation);
        self.motor_on = true;
    }

    pub fn get_motor_status(&mut self) -> MotorStatus {
        let frequency = self.inverter.motor_status.frequency;
        let rpm =
            AngularVelocity::new::<revolution_per_minute>(frequency.get::<cycle_per_minute>());

        let screw_rpm = self.transmission.calculate_angular_velocity_output(rpm);

        let mut status = self.inverter.motor_status;
        status.rpm = screw_rpm;

        status
    }

    pub fn get_target_pressure(&self) -> Pressure {
        self.target_pressure
    }

    fn clamp_frequency(frequency: Frequency, min: Frequency, max: Frequency) -> Frequency {
        if frequency < min {
            min
        } else if frequency > max {
            max
        } else {
            frequency
        }
    }

    pub fn get_wiring_error(&self) -> bool {
        self.pressure_sensor.get_wiring_error()
    }

    pub fn get_sensor_current(&self) -> Result<ElectricCurrent, anyhow::Error> {
        let phys: ethercat_hal::io::analog_input::physical::AnalogInputValue =
            self.pressure_sensor.get_physical();

        match phys {
            ethercat_hal::io::analog_input::physical::AnalogInputValue::Potential(_) => {
                Err(anyhow::anyhow!("Potential is not expected"))
            }
            ethercat_hal::io::analog_input::physical::AnalogInputValue::Current(quantity) => {
                Ok(quantity)
            }
        }
    }

    pub const fn reset_pid(&mut self) {
        self.pid.reset()
    }

    pub fn get_pressure(&mut self) -> Pressure {
        let current_result = self.get_sensor_current();
        let current = match current_result {
            Ok(current) => current.get::<milliampere>(),
            Err(_) => {
                tracing::error!("cant get pressure sensor reading");
                return Pressure::new::<bar>(0.0);
            }
        };
        let normalized = normalize(current, 4.0, 20.0);
        // Our pressure sensor has a range of Up to 350 Bar
        let actual_pressure = (normalized) * 350.0;
        Pressure::new::<bar>(actual_pressure)
    }

    pub fn update(&mut self, now: Instant, is_extruding: bool) {
        // TODO: move this logic elsewhere or make non async
        smol::block_on(self.inverter.act(now));
        let measured_pressure = self.get_pressure();
        if !self.uses_rpm && !is_extruding && self.motor_on {
            let frequency = Frequency::new::<hertz>(0.0);
            self.inverter.set_frequency_target(frequency);
            self.turn_motor_off();
            self.last_update = now;
            return;
        }

        if (measured_pressure >= self.nozzle_pressure_limit)
            && self.nozzle_pressure_limit_enabled
            && self.motor_on
        {
            self.turn_motor_off();
            self.last_update = now;
            return;
        }

        if is_extruding && !self.motor_on {
            self.turn_motor_on();
        }

        if !self.uses_rpm && is_extruding {
            let error = self.target_pressure - measured_pressure;
            let freq_change = self.pid.update(error.get::<bar>(), now);

            self.frequency += Frequency::new::<hertz>(freq_change);
            self.frequency = Self::clamp_frequency(
                self.frequency,
                self.minimum_frequency,
                self.maximum_frequency,
            );

            self.inverter.set_frequency_target(self.frequency);
        }
        self.last_update = now;
    }

    pub fn start_pressure_regulation(&mut self) {
        self.last_update = Instant::now();
        self.frequency = self.inverter.motor_status.frequency;
        self.pid.reset();
    }

    pub fn reset(&mut self) {
        self.pid.reset();
        self.last_update = Instant::now();
    }
}
