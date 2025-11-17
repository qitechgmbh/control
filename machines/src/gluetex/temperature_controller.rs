use super::Heating;
use control_core::controllers::pid::PidController;
use ethercat_hal::io::{digital_output::DigitalOutput, temperature_input::TemperatureInput};
use std::time::{Duration, Instant};
use units::f64::*;
use units::thermodynamic_temperature::degree_celsius;

#[derive(Debug)]
pub struct TemperatureController {
    pub pid: PidController,
    temperature_sensor: TemperatureInput,
    ssr_output: DigitalOutput,
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
        self.ssr_output.set(false);
        self.heating.heating = false;
        self.disallow_heating();
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        kp: f64,
        ki: f64,
        kd: f64,
        target_temp: ThermodynamicTemperature,
        max_temperature: ThermodynamicTemperature,
        temperature_sensor: TemperatureInput,
        ssr_output: DigitalOutput,
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
            ssr_output,
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

    pub fn get_temperature(
        &self,
    ) -> Result<f64, ethercat_hal::io::temperature_input::TemperatureInputError> {
        self.temperature_sensor.get_temperature()
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
            // disable the SSR and return
            self.ssr_output.set(false);
            self.heating.heating = false;
            return;
        }

        if self.heating_allowed {
            let error: f64 = self.heating.target_temperature.get::<degree_celsius>()
                - self.heating.temperature.get::<degree_celsius>();

            let control = self.pid.update(error, now); // PID output
            // Clamp PID output to 0.0 – 1.0 (as duty cycle)
            let duty_cycle = control.clamp(0.0, self.max_clamp);
            self.temperature_pid_output = duty_cycle;

            // PWM logic
            let elapsed = now.duration_since(self.window_start);
            if elapsed >= self.pwm_period {
                self.window_start = now;
            }

            let on_time = self.pwm_period.mul_f64(duty_cycle);
            if elapsed < on_time {
                self.ssr_output.set(true);
                self.heating.heating = true;
            } else {
                self.ssr_output.set(false);
                self.heating.heating = false;
            }
        } else {
            self.ssr_output.set(false);
            self.heating.heating = false;
        }
    }
}
