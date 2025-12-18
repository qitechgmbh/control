use crate::aquapath1::VolumeRate;
use crate::aquapath1::{Flow, Temperature};
use control_core::controllers::pid::PidController;
use ethercat_hal::io::encoder_input::EncoderInput;
use ethercat_hal::io::{
    analog_output::AnalogOutput, digital_output::DigitalOutput, temperature_input::TemperatureInput,
};
use std::time::{Duration, Instant};
use units::f64::ThermodynamicTemperature;
use units::thermodynamic_temperature::degree_celsius;
use units::volume_rate::liter_per_minute;
#[derive(Debug)]

pub struct Controller {
    pub pid: PidController,
    temperature_pid_output: f64,
    window_start: Instant,
    pwm_period: Duration,

    pub temperature: Temperature,
    pub target_temperature: ThermodynamicTemperature,
    pub current_temperature: ThermodynamicTemperature,
    pub temp_reservoir: ThermodynamicTemperature,
    pub min_temperature: ThermodynamicTemperature,
    pub max_temperature: ThermodynamicTemperature,

    pub cooling_controller: AnalogOutput,
    pub cooling_relais: DigitalOutput,

    pub heating_relais_1: DigitalOutput,
    pub temperature_sensor_in: TemperatureInput,
    pub temperature_sensor_out: TemperatureInput,

    pub cooling_allowed: bool,
    pub heating_allowed: bool,

    pub flow: Flow,
    pump_relais: DigitalOutput,
    pub should_pump: bool,
    pub flow_sensor: EncoderInput,
    pub pump_allowed: bool,
    pub current_flow: VolumeRate,
    pub max_flow: VolumeRate,
}

impl Controller {
    pub fn new(
        kp: f64,
        ki: f64,
        kd: f64,
        pwm_duration: Duration,
        temp: Temperature,
        target_tempetature: ThermodynamicTemperature,
        cooling_controller: AnalogOutput,
        cooling_relais: DigitalOutput,
        heating_relais_1: DigitalOutput,
        temp_sensor_in: TemperatureInput,
        temp_sensor_out: TemperatureInput,

        flow: Flow,
        pump_relais: DigitalOutput,
        flow_sensor: EncoderInput,
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

            temperature: temp,
            cooling_controller: cooling_controller,
            cooling_relais: cooling_relais,
            heating_relais_1: heating_relais_1,
            cooling_allowed: false,
            heating_allowed: false,
            temperature_sensor_in: temp_sensor_in,
            temperature_sensor_out: temp_sensor_out,

            flow: flow,
            pump_relais: pump_relais,
            flow_sensor: flow_sensor,
            should_pump: false,
            current_flow: VolumeRate::new::<liter_per_minute>(0.0),
            pump_allowed: false,
            max_flow: VolumeRate::new::<liter_per_minute>(10.0),
        }
    }

    pub fn turn_pump_off(&mut self) {
        self.flow.pump = false;
        self.pump_relais.set(false);
    }

    pub fn turn_pump_on(&mut self) {
        self.flow.pump = true;
        self.pump_relais.set(true);
    }

    pub fn disallow_pump(&mut self) {
        self.pump_allowed = false;
    }

    pub fn allow_pump(&mut self) {
        self.pump_allowed = true;
    }

    pub fn get_pump(&mut self) -> bool {
        self.pump_allowed
    }

    pub fn disable_cooling(&mut self) {
        self.turn_cooling_off();
        self.disallow_cooling();
    }

    pub fn disable_heating(&mut self) {
        self.turn_heating_off();
        self.disallow_heating();
    }

    pub fn enable_cooling(&mut self) {
        self.turn_cooling_on();
        self.allow_cooling();
    }

    pub fn enable_heating(&mut self) {
        self.turn_heating_on();
        self.allow_heating();
    }
    pub fn reset_pid(&mut self) {
        self.pid.reset()
    }
    pub fn set_target_temperature(&mut self, temperature: ThermodynamicTemperature) {
        self.reset_pid();
        self.target_temperature = temperature;
    }

    pub fn get_temp_in(&mut self) -> ThermodynamicTemperature {
        let temp = self.temperature_sensor_in.get_temperature();
        match temp {
            Ok(value) => ThermodynamicTemperature::new::<degree_celsius>(value),
            Err(_) => ThermodynamicTemperature::new::<degree_celsius>(0.0),
        }
    }

    pub fn get_temp_out(&mut self) -> ThermodynamicTemperature {
        let temp = self.temperature_sensor_out.get_temperature();
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
        self.cooling_relais.set(true);
        self.cooling_controller.set(10.0);
        self.temperature.cooling = true;
    }

    pub fn turn_cooling_off(&mut self) {
        self.cooling_relais.set(false);
        self.temperature.cooling = false;
    }

    pub fn turn_heating_on(&mut self) {
        self.heating_relais_1.set(true);
        self.temperature.heating = true;
    }

    pub fn turn_heating_off(&mut self) {
        self.heating_relais_1.set(false);
        self.temperature.heating = false;
    }

    pub fn set_should_pump(&mut self, should_pump: bool) {
        self.should_pump = should_pump;
    }

    pub fn get_should_pump(&mut self) -> bool {
        self.should_pump
    }

    pub fn get_flow(&mut self) -> VolumeRate {
        let value = match self.flow_sensor.get_frequency_value() {
            Ok(val) => val,
            Err(_e) => {
                return VolumeRate::new::<liter_per_minute>(0.0);
            }
        };

        match value {
            Some(val) => {
                if val == 0 {
                    return VolumeRate::new::<liter_per_minute>(0.0);
                }
                // Formula: f = 8.1*q - 3, so q = (f + 3) / 8.1
                let actual_flow = ((val / 100) as f32 + 3.0) / 8.1;
                VolumeRate::new::<liter_per_minute>(actual_flow.into())
            }
            None => {
                return VolumeRate::new::<liter_per_minute>(0.0);
            }
        }
    }

    pub fn update(&mut self, now: Instant) -> () {
        let current_flow = self.get_flow();
        self.current_flow = current_flow;
        self.flow.flow = current_flow;

        let should_flow = self.get_should_pump();
        self.flow.should_pump = should_flow;

        if !self.flow.pump && self.get_pump() && should_flow {
            self.turn_pump_on();
        } else if self.flow.pump && (!self.get_pump() || !should_flow) {
            self.turn_pump_off();
        }

        self.current_temperature = self.get_temp_in();
        self.temp_reservoir = self.get_temp_out();

        if self.current_temperature < self.min_temperature && self.temperature.cooling {
            self.turn_cooling_off();
        } else if self.current_temperature > self.max_temperature && self.temperature.heating {
            self.turn_heating_off();
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

        // Decide whether to heat or cool based on error
        if error > 0.0 {
            // Need heating (current < target)
            if self.temperature.cooling {
                self.turn_cooling_off();
            }
            if self.heating_allowed && current_flow > VolumeRate::new::<liter_per_minute>(0.0) {
                // Only start heating if pump is on
                let duty = control.clamp(0.0, 1.0);
                let on_time = self.pwm_period.mul_f64(duty);
                let on = elapsed < on_time;
                if on && !self.temperature.heating {
                    self.turn_heating_on();
                } else if !on && self.temperature.heating {
                    self.turn_heating_off();
                }
            } else {
                // Pump is off or heating not allowed - don't heat
                if self.temperature.heating {
                    self.turn_heating_off();
                }
            }
        } else {
            // Need cooling (current > target)
            if self.temperature.heating {
                self.turn_heating_off();
            }
            if self.cooling_allowed && !self.temperature.cooling {
                self.turn_cooling_on();
            }
        }
    }
}
