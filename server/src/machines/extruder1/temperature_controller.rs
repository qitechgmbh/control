use super::Heating;
use control_core::{
    actors::{
        Actor, digital_output_setter::DigitalOutputSetter,
        temperature_input_getter::TemperatureInputGetter,
    },
    controllers::pid::PidController,
};
use std::time::{Duration, Instant};
use uom::si::{f64::ThermodynamicTemperature, thermodynamic_temperature::degree_celsius};

#[derive(Debug)]

pub struct TemperatureController {
    pid: PidController,
    temperature_sensor: TemperatureInputGetter,
    relais: DigitalOutputSetter,
    pub heating: Heating,
    pub target_temp: ThermodynamicTemperature,
    window_start: Instant,
    heating_allowed: bool,
    pwm_period: Duration,
    max_temperature: ThermodynamicTemperature,
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
        temperature_sensor: TemperatureInputGetter,
        relais: DigitalOutputSetter,
        heating: Heating,
        pwm_duration: Duration,
    ) -> Self {
        Self {
            pid: PidController::new(kp, ki, kd),
            target_temp,
            window_start: Instant::now(),
            temperature_sensor: temperature_sensor,
            relais: relais,
            heating: heating,
            heating_allowed: false,
            pwm_period: pwm_duration,
            max_temperature: max_temperature,
        }
    }

    pub fn set_target_temperature(&mut self, temp: ThermodynamicTemperature) {
        // heating needs to be deserializable and serializable!!, so we cant use uom on it
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
        let temperature = ThermodynamicTemperature::new::<degree_celsius>(
            self.temperature_sensor.get_temperature(),
        );
        self.heating.temperature = temperature;
        self.heating.wiring_error = self.temperature_sensor.get_wiring_error();
        if self.heating.temperature > self.max_temperature {
            // disable the relais and return
            self.relais.set(false);
            self.heating.heating = false;
            self.relais.act(now).await;
            return;
        } else {
            self.relais.act(now).await;
        }

        if self.heating_allowed {
            let error: f64 = self.heating.target_temperature.get::<degree_celsius>()
                - self.heating.temperature.get::<degree_celsius>();
            let control = self.pid.update(error, now); // PID output
            // Clamp PID output to 0.0 – 1.0 (as duty cycle)
            let duty = control.clamp(0.0, 1.0);
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
