use crate::extruder1::Heating;
use control_core::controllers::pid::PidController;
use ethercat_hal::io::{digital_output::DigitalOutput, temperature_input::TemperatureInput};
use std::time::{Duration, Instant};
use units::f64::*;
use units::thermodynamic_temperature::degree_celsius;

#[derive(Debug)]

pub struct TemperatureController {
    pub pid: PidController,
    temperature_sensor: TemperatureInput,
    relais: DigitalOutput,
    pub heating: Heating,
    pub target_temp: ThermodynamicTemperature,
    window_start: Instant,
    heating_allowed: bool,
    pwm_period: Duration,
    max_temperature: ThermodynamicTemperature,
    temperature_pid_output: f64,
    heating_element_wattage: f64,
    max_clamp: f64,
}

impl TemperatureController {
    pub fn disable(&mut self) {
        self.relais.set(false);
        self.heating.heating = false;
        self.disallow_heating();
    }

    pub fn new(
        kp: f64,
        ki: f64,
        kd: f64,
        target_temp: ThermodynamicTemperature,
        max_temperature: ThermodynamicTemperature,
        temperature_sensor: TemperatureInput,
        relais: DigitalOutput,
        heating: Heating,
        pwm_duration: Duration,
        heating_element_wattage: f64,
        max_clamp: f64,
    ) -> Self {
        Self {
            pid: PidController::new(kp, ki, kd),
            target_temp,
            window_start: Instant::now(),
            temperature_sensor,
            relais,
            heating,
            heating_allowed: false,
            pwm_period: pwm_duration,
            max_temperature,
            temperature_pid_output: 0.0,
            heating_element_wattage,
            max_clamp,
        }
    }

    pub fn set_target_temperature(&mut self, temp: ThermodynamicTemperature) {
        self.heating.target_temperature = temp;
    }

    pub const fn disallow_heating(&mut self) {
        self.heating_allowed = false;
    }

    pub const fn allow_heating(&mut self) {
        self.heating_allowed = true;
    }

    pub fn get_heating_element_wattage(&mut self) -> f64 {
        self.temperature_pid_output * self.heating_element_wattage
    }

    pub fn update(&mut self, now: Instant) {
        self.temperature_pid_output = 0.0;

        let temperature = self.temperature_sensor.get_temperature();
        let temperature_celsius = ThermodynamicTemperature::new::<degree_celsius>(
            temperature.as_ref().unwrap_or(&0.0).to_owned(),
        );

        self.heating.wiring_error = temperature.is_err();
        self.heating.temperature = temperature_celsius;

        if self.heating.temperature > self.max_temperature {
            // disable the relais and return
            self.relais.set(false);
            self.heating.heating = false;
            return;
        }

        if self.heating_allowed {
            let error: f64 = self.heating.target_temperature.get::<degree_celsius>()
                - self.heating.temperature.get::<degree_celsius>();

            let control = self.pid.update(error, now); // PID output
            // Clamp PID output to 0.0 – 1.0 (as duty cycle)
            let duty = control.clamp(0.0, self.max_clamp);

            self.temperature_pid_output = duty;

            let elapsed = now.duration_since(self.window_start);

            // Restart window if needed
            if elapsed >= self.pwm_period {
                self.window_start = now;
            }
            // Compare duty cycle to elapsed time
            let on_time = self.pwm_period.mul_f64(duty);

            // Relay is ON if within duty cycle window
            let on = elapsed < on_time;
            self.relais.set(on);
            self.heating.heating = on;
        }
    }
}
