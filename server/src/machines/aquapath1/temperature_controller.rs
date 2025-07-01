use crate::machines::aquapath1::{Flow, Temperature};
use control_core::controllers::pid::PidController;
use ethercat_hal::io::{
    analog_output::AnalogOutput, digital_output::DigitalOutput, temperature_input::TemperatureInput,
};
use serde::de::value;
use std::{
    sync::{Arc, RwLock},
    thread::sleep,
    time::{Duration, Instant},
};
use uom::si::{f64::ThermodynamicTemperature, thermodynamic_temperature::degree_celsius};

#[derive(Debug)]

pub struct TemperatureController {
    pub pid: PidController,
    temperature_pid_output: f64,
    window_start: Instant,
    pwm_period: Duration,

    //pub temperature_sensor: TemperatureInput,
    pub temp: Temperature,
    pub flow: Arc<RwLock<Flow>>,
    pub target_temperature: ThermodynamicTemperature,
    pub current_temperature: ThermodynamicTemperature,
    pub temp_reservoir: ThermodynamicTemperature,
    pub min_temperature: ThermodynamicTemperature,
    pub max_temperature: ThermodynamicTemperature,

    pub cooling_controller: AnalogOutput,
    pub cooling_relais: DigitalOutput,

    pub heating_relais_1: DigitalOutput,
    pub heating_relais_2: DigitalOutput,

    pub temp_sensor_in: TemperatureInput,
    pub temp_sensor_out: TemperatureInput,

    pub cooling_allowed: bool,
    pub heating_allowed: bool,
}

impl TemperatureController {
    pub fn disable_cooling(&mut self) {
        self.temp.cooling = false;
        self.turn_cooling_off();
        self.disallow_cooling();
    }

    pub fn disable_heating(&mut self) {
        self.temp.heating = false;
        self.turn_heating_off();
        self.disallow_heating();
    }

    pub fn enable_cooling(&mut self) {
        self.temp.cooling = true;
        self.turn_cooling_on();
        self.allow_cooling();
    }

    pub fn enable_heating(&mut self) {
        self.temp.heating = true;
        self.turn_heating_on();
        self.allow_heating();
    }
    pub fn new(
        kp: f64,
        ki: f64,
        kd: f64,
        pwm_duration: Duration,
        temp: Temperature,
        flow: Arc<RwLock<Flow>>,
        target_tempetature: ThermodynamicTemperature,
        cooling_controller: AnalogOutput,
        cooling_relais: DigitalOutput,
        heating_relais_1: DigitalOutput,
        heating_relais_2: DigitalOutput,
        temp_sensor_in: TemperatureInput,
        temp_sensor_out: TemperatureInput,
    ) -> Self {
        Self {
            pid: PidController::new(kp, ki, kd),
            temperature_pid_output: 0.0,
            window_start: Instant::now(),
            pwm_period: pwm_duration,
            target_temperature: target_tempetature,
            current_temperature: ThermodynamicTemperature::new::<degree_celsius>(25.0),
            temp_reservoir: ThermodynamicTemperature::new::<degree_celsius>(25.0),
            min_temperature: ThermodynamicTemperature::new::<degree_celsius>(10.0),
            max_temperature: ThermodynamicTemperature::new::<degree_celsius>(50.0),

            temp: temp,
            flow: flow,
            cooling_controller: cooling_controller,
            cooling_relais: cooling_relais,
            heating_relais_1: heating_relais_1,
            heating_relais_2: heating_relais_2,
            cooling_allowed: false,
            heating_allowed: false,
            temp_sensor_in: temp_sensor_in,
            temp_sensor_out: temp_sensor_out,
        }
    }

    pub fn reset_pid(&mut self) {
        self.pid.reset()
    }
    pub fn set_target_temperature(&mut self, temperature: ThermodynamicTemperature) {
        self.reset_pid();
        self.target_temperature = temperature;
    }

    pub fn get_temp_in(&mut self) -> ThermodynamicTemperature {
        let temp = self.temp_sensor_in.get_temperature();
        match temp {
            Ok(value) => ThermodynamicTemperature::new::<degree_celsius>(value),
            Err(_) => ThermodynamicTemperature::new::<degree_celsius>(0.0),
        }
    }

    pub fn get_temp_out(&mut self) -> ThermodynamicTemperature {
        let temp = self.temp_sensor_out.get_temperature();
        match temp {
            Ok(value) => ThermodynamicTemperature::new::<degree_celsius>(value),
            Err(_) => ThermodynamicTemperature::new::<degree_celsius>(0.0),
        }
    }

    pub fn disallow_cooling(&mut self) {
        self.cooling_allowed = false;
    }

    pub fn allow_cooling(&mut self) {
        self.cooling_allowed = true;
    }

    pub fn disallow_heating(&mut self) {
        self.heating_allowed = false;
    }

    pub fn allow_heating(&mut self) {
        self.heating_allowed = true;
    }

    pub fn turn_cooling_on(&mut self) {
        self.cooling_controller.set(10.0);
        self.cooling_relais.set(true);
        self.temp.cooling = true;
    }

    pub fn turn_cooling_off(&mut self) {
        self.cooling_relais.set(false);
        self.cooling_controller.set(0.0);
        self.temp.cooling = false;
    }

    pub fn turn_heating_on(&mut self) {
        self.heating_relais_1.set(true);
        self.heating_relais_2.set(true);
        self.temp.heating = true;
    }

    pub fn turn_heating_off(&mut self) {
        self.heating_relais_1.set(false);
        self.heating_relais_2.set(false);
        self.temp.heating = false;
    }

    pub fn update(&mut self, now: Instant) -> () {
        self.current_temperature = self.get_temp_in();
        self.temp_reservoir = self.get_temp_out();

        if self.current_temperature < self.min_temperature {
            self.turn_cooling_off();
            return;
        }

        if self.current_temperature > self.max_temperature {
            self.turn_heating_off();
            return;
        }

        // Calculate PID error once
        let error = self.target_temperature.get::<degree_celsius>()
            - self.current_temperature.get::<degree_celsius>();

        let control = self.pid.update(error, now);
        self.temperature_pid_output = control;

        let elapsed = now.duration_since(self.window_start);
        if elapsed >= self.pwm_period {
            self.window_start = now;
        }

        // Get pump status from flow controller
        let is_pump_on = self.flow.read().map(|f| f.pump).unwrap_or(false);

        // Decide whether to heat or cool based on error
        if error > 0.0 {
            // Need heating (current < target)
            self.turn_cooling_off(); // Always turn off cooling when heating is needed

            if self.heating_allowed && is_pump_on {
                // Only start heating if pump is on
                let duty = control.clamp(0.0, 1.0);
                let on_time = self.pwm_period.mul_f64(duty);
                let on = elapsed < on_time;
                self.heating_relais_1.set(on);
                self.heating_relais_2.set(on);
                self.temp.heating = on;
            } else {
                // Pump is off or heating not allowed - don't heat
                self.turn_heating_off();
            }
        } else if error < 0.0 && self.cooling_allowed {
            // Need cooling (current > target)
            self.turn_heating_off(); // Always turn off heating when cooling is needed

            let duty = (-control).clamp(0.0, 1.0);
            let on_time = self.pwm_period.mul_f64(duty);
            let on = elapsed < on_time;
            self.cooling_relais.set(on);

            if on {
                let cooling_speed = duty * 10.0;
                self.cooling_controller.set(cooling_speed as f32);
            } else {
                self.cooling_controller.set(0.0);
            }

            self.temp.cooling = on;
        } else {
            // No heating or cooling needed
            self.turn_heating_off();
            self.turn_cooling_off();
        }
    }
}
