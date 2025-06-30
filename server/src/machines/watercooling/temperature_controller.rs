use super::Cooling;
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
    pub pid: PidController,
    temperature_sensor: TemperatureInputGetter,
    relais: DigitalOutputSetter,
    pub cooling: Cooling,
    pub target_temp: ThermodynamicTemperature,
    window_start: Instant,
    cooling_allowed: bool,
    pwm_period: Duration,
    min_temperature: ThermodynamicTemperature,
    temperature_pid_output: f64,
    cooling_element_wattage: f64,
    max_clamp: f64,
}

impl TemperatureController {
    pub fn disable(&mut self) {
        self.relais.set(false);
        self.cooling.cooling = false;
        self.disallow_cooling();
    }

    pub fn new(
        kp: f64,
        ki: f64,
        kd: f64,
        target_temp: ThermodynamicTemperature,
        min_temperature: ThermodynamicTemperature,
        temperature_sensor: TemperatureInputGetter,
        relais: DigitalOutputSetter,
        cooling: Cooling,
        pwm_duration: Duration,
        cooling_element_wattage: f64,
        max_clamp: f64,
    ) -> Self {
        Self {
            pid: PidController::new(kp, ki, kd),
            target_temp,
            window_start: Instant::now(),
            temperature_sensor: temperature_sensor,
            relais: relais,
            cooling: cooling,
            cooling_allowed: false,
            pwm_period: pwm_duration,
            min_temperature: min_temperature,
            temperature_pid_output: 0.0,
            cooling_element_wattage: cooling_element_wattage,
            max_clamp: max_clamp,
        }
    }

    pub fn set_target_temperature(&mut self, temp: ThermodynamicTemperature) {
        self.cooling.target_temperature = temp;
    }

    pub fn disallow_cooling(&mut self) {
        self.cooling_allowed = false;
    }

    pub fn allow_cooling(&mut self) {
        self.cooling_allowed = true;
    }

    pub fn get_cooling_element_wattage(&mut self) -> f64 {
        return self.temperature_pid_output * self.cooling_element_wattage;
    }

    pub async fn update(&mut self, now: Instant) -> () {
        self.temperature_sensor.act(now).await;
        self.temperature_pid_output = 0.0;

        let temperature = ThermodynamicTemperature::new::<degree_celsius>(
            self.temperature_sensor.get_temperature(),
        );

        self.cooling.temperature = temperature;

        if self.cooling.temperature < self.min_temperature {
            // disable the relais and return
            self.relais.set(false);
            self.cooling.cooling = false;
            self.relais.act(now).await;
            return;
        } else {
            self.relais.act(now).await;
        }

        if self.cooling_allowed {
            let error: f64 = self.cooling.target_temperature.get::<degree_celsius>()
                - self.cooling.temperature.get::<degree_celsius>();

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
            self.cooling.cooling = on;
        }
    }
}
