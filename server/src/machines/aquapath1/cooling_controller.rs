use crate::machines::aquapath1::Cooling;
use control_core::controllers::pid::PidController;
use ethercat_hal::io::{analog_output::AnalogOutput, temperature_input::TemperatureInput};
use std::time::{Duration, Instant};
use uom::si::{f64::ThermodynamicTemperature, thermodynamic_temperature::degree_celsius};

#[derive(Debug)]

pub struct CoolingController {
    pub pid: PidController,
    temperature_pid_output: f64,
    window_start: Instant,
    pwm_period: Duration,

    //pub temperature_sensor: TemperatureInput,
    pub cooling: Cooling,
    pub target_tempetature: ThermodynamicTemperature,
    pub current_tempetature: ThermodynamicTemperature,
    pub min_temperature: ThermodynamicTemperature,

    pub cooling_controller: AnalogOutput,

    pub cooling_allowed: bool,
    max_clamp: f64,
}

impl CoolingController {
    pub fn disable(&mut self) {
        self.cooling.cooling = false;
        self.cooling_controller.set(0.0);
        self.disallow_cooling();
    }

    pub fn new(
        kp: f64,
        ki: f64,
        kd: f64,
        pwm_duration: Duration,

        cooling: Cooling,
        target_tempetature: ThermodynamicTemperature,
        cooling_controller: AnalogOutput,
        max_clamp: f64,
    ) -> Self {
        Self {
            pid: PidController::new(kp, ki, kd),
            temperature_pid_output: 0.0,
            window_start: Instant::now(),
            pwm_period: pwm_duration,
            target_tempetature,
            current_tempetature: ThermodynamicTemperature::new::<degree_celsius>(25.0),
            min_temperature: ThermodynamicTemperature::new::<degree_celsius>(10.0),
            cooling: cooling,
            cooling_controller: cooling_controller,
            cooling_allowed: false,
            max_clamp,
        }
    }

    pub fn set_target_temperature(&mut self, temperature: ThermodynamicTemperature) {
        self.cooling.target_temperature = temperature;
    }

    pub fn disallow_cooling(&mut self) {
        self.cooling_allowed = false;
    }

    pub fn allow_cooling(&mut self) {
        self.cooling_allowed = true;
    }

    pub fn update(&mut self, now: Instant) -> () {
        if self.current_tempetature < self.min_temperature {
            // disable the relais and return
            self.cooling_controller.set(0.0);
            self.cooling.cooling = false;
            return;
        }
        if self.cooling_allowed {
            let error = self.target_tempetature.get::<degree_celsius>()
                - self.current_tempetature.get::<degree_celsius>();
            let control = self.pid.update(error, now);
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
            self.cooling_controller.set(10.0);
            self.cooling.cooling = on;
        }
    }
}
