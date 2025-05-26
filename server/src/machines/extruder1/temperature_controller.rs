use control_core::{
    actors::{
        Actor, digital_output_setter::DigitalOutputSetter,
        temperature_input_getter::TemperatureInputGetter,
    },
    controllers::pid::PidController,
};
use std::time::{Duration, Instant};

use super::Heating;

const PWM_PERIOD: Duration = Duration::from_millis(100);

#[derive(Debug)]

pub struct TemperatureController {
    pid: PidController,
    temperature_sensor: TemperatureInputGetter,
    relais: DigitalOutputSetter,
    pub heating: Heating,
    pub target_temp: f64,
    window_start: Instant,
    heating_allowed: bool,
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
        target_temp: f64,
        temperature_sensor: TemperatureInputGetter,
        relais: DigitalOutputSetter,
        heating: Heating,
    ) -> Self {
        Self {
            pid: PidController::new(kp, ki, kd),
            target_temp,
            window_start: Instant::now(),
            temperature_sensor: temperature_sensor,
            relais: relais,
            heating: heating,
            heating_allowed: false,
        }
    }

    pub fn set_target_temperature(&mut self, temp: f32) {
        self.heating.target_temperature = temp;
    }

    pub fn disallow_heating(&mut self) {
        self.heating_allowed = false;
    }

    pub fn allow_heating(&mut self) {
        self.heating_allowed = true;
    }

    pub async fn update(&mut self, now: Instant) -> () {
        self.temperature_sensor.act(now).await;
        self.heating.temperature = self.temperature_sensor.get_temperature();
        self.heating.wiring_error = self.temperature_sensor.get_wiring_error();
        self.relais.act(now).await;

        if self.heating_allowed {
            let error = (self.heating.target_temperature - self.heating.temperature) as f64;
            let control = self.pid.update(error, now); // PID output
            // Clamp PID output to 0.0 – 1.0 (as duty cycle)
            let duty = control.clamp(0.0, 1.0);
            let elapsed = now.duration_since(self.window_start);

            // Restart window if needed
            if elapsed >= PWM_PERIOD {
                self.window_start = now;
            }

            // Compare duty cycle to elapsed time
            let on_time = PWM_PERIOD.mul_f64(duty);

            // Relay is ON if within duty cycle window
            let on = elapsed < on_time;
            self.relais.set(on);
            self.heating.heating = on;
        }
    }
}
