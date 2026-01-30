use crate::gluetex::Heating;
use control_core::controllers::pid::PidController;
use control_core::controllers::pid_autotuner::{AutoTuneConfig, PidAutoTuner};
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

    // Auto-tuning support
    pub autotuner: Option<PidAutoTuner>,
    autotuning_active: bool,
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
            autotuner: None,
            autotuning_active: false,
        }
    }

    pub fn set_target_temperature(&mut self, temp: ThermodynamicTemperature) {
        self.heating.target_temperature = temp;
    }

    pub fn disallow_heating(&mut self) {
        self.heating_allowed = false;
    }

    pub fn allow_heating(&mut self) {
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

    /// Start PID auto-tuning for this heater
    pub fn start_autotuning(&mut self, target_temp: ThermodynamicTemperature) {
        let config = AutoTuneConfig {
            tune_delta: 5.0,           // ±5°C oscillation around target (like Klipper)
            max_power: 1.0,            // Full power heating
            max_duration_secs: 3600.0, // 1 hour timeout
        };

        let target_celsius = target_temp.get::<degree_celsius>();
        let mut tuner = PidAutoTuner::new(config);
        tuner.start(Instant::now(), target_celsius);

        self.autotuner = Some(tuner);
        self.autotuning_active = true;
        self.set_target_temperature(target_temp);
        self.allow_heating();
    }

    /// Stop auto-tuning
    pub fn stop_autotuning(&mut self) {
        if let Some(tuner) = &mut self.autotuner {
            tuner.stop();
        }
        self.autotuning_active = false;
    }

    /// Check if auto-tuning is active
    pub fn is_autotuning(&self) -> bool {
        self.autotuning_active
    }

    /// Get auto-tuning progress (0-100%)
    pub fn get_autotuning_progress(&self) -> f64 {
        if let Some(tuner) = &self.autotuner {
            tuner.get_progress_percent()
        } else {
            0.0
        }
    }

    /// Check if auto-tuning completed successfully and get results
    pub fn get_autotuning_result(&mut self) -> Option<(f64, f64, f64)> {
        if let Some(tuner) = &mut self.autotuner {
            if tuner.is_completed() {
                if let Some(result) = tuner.result.take() {
                    let kp = result.kp;
                    let ki = result.ki;
                    let kd = result.kd;

                    // Apply the tuned parameters
                    self.pid.configure(ki, kp, kd);
                    self.autotuning_active = false;
                    self.autotuner = None;

                    return Some((kp, ki, kd));
                }
            } else if tuner.is_failed() {
                self.autotuning_active = false;
                self.autotuner = None;
            }
        }
        None
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

        // Check if auto-tuning is active
        if self.autotuning_active {
            if let Some(tuner) = &mut self.autotuner {
                let current_temp = temperature_celsius.get::<degree_celsius>();
                let (duty_cycle, completed) = tuner.update(current_temp, now);

                if completed {
                    // Auto-tuning finished (either success or failure)
                    // The result will be checked by get_autotuning_result()
                    self.autotuning_active = false;
                }

                // Use the auto-tuner's output
                self.temperature_pid_output = duty_cycle.clamp(0.0, self.max_clamp);

                // PWM logic
                let elapsed = now.duration_since(self.window_start);
                if elapsed >= self.pwm_period {
                    self.window_start = now;
                }

                let on_time = self.pwm_period.mul_f64(self.temperature_pid_output);
                if elapsed < on_time {
                    self.ssr_output.set(true);
                    self.heating.heating = true;
                } else {
                    self.ssr_output.set(false);
                    self.heating.heating = false;
                }
                return;
            }
        }

        // Normal PID control
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
