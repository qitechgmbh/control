use super::Heating;
use control_core::controllers::pid::PidController;
use qitech_lib::{
    ethercat_hal::io::{
        digital_output::DigitalOutputDevice, temperature_input::TemperatureInputDevice,
    },
    units::{ThermodynamicTemperature, thermodynamic_temperature::degree_celsius},
};
use std::time::{Duration, Instant};

struct LowPassFilter {
    alpha: f64, // Smoothing factor (0.0 < alpha <= 1.0)
    state: Option<f64>,
}

impl LowPassFilter {
    /// Creates a low-pass filter.
    /// - `alpha`: Smoothing coefficient between 0.0 and 1.0.
    ///   - Lower alpha (e.g. 0.05) = stronger noise reduction, slower response (higher phase lag).
    ///   - Higher alpha (e.g. 0.30) = weaker noise reduction, faster response.
    fn new(alpha: f64) -> Self {
        let alpha = alpha.clamp(0.001, 1.0);
        Self { alpha, state: None }
    }

    fn update(&mut self, measurement: f64) -> f64 {
        let input = measurement as f64;

        let filtered_val = match self.state {
            Some(prev) => self.alpha * input + (1.0 - self.alpha) * prev,
            None => input, // Seed filter state with the first reading
        };

        self.state = Some(filtered_val);
        filtered_val
    }

    /// Returns the internal float state of the filter.
    fn current_val(&self) -> f64 {
        self.state.unwrap_or(0.0)
    }
}


pub struct TemperatureController {
    pub pid: PidController,
    pub heating: Heating,
    pub target_temp: ThermodynamicTemperature,
    pub digital_port: usize,
    pub temperature_port: usize,
    window_start: Instant,
    heating_allowed: bool,
    pwm_period: Duration,
    max_temperature: ThermodynamicTemperature,
    temperature_pid_output: f64,
    heating_element_wattage: f64,
    max_clamp: f64,
    target_temp_enabled: bool, // Sets whether the frontend should display a target temperature setter for this temp controller
    filter : LowPassFilter,
}

impl TemperatureController {
    pub fn disable(&mut self, relais: &mut dyn DigitalOutputDevice) {
        relais.set_output(self.digital_port, false);
        self.heating.heating = false;
        self.disallow_heating();
    }

    pub fn new(
        kp: f64,
        ki: f64,
        kd: f64,
        target_temp: ThermodynamicTemperature,
        max_temperature: ThermodynamicTemperature,
        heating: Heating,
        pwm_duration: Duration,
        heating_element_wattage: f64,
        max_clamp: f64,
        digital_port: usize,
        temperature_port: usize,
    ) -> Self {
        Self {
            //the max clamp for integral should be at max 20% of the total output 
            pid: PidController::new( kp, ki, kd,0.0, max_clamp/5.0 ),
            target_temp,
            window_start: Instant::now(),
            heating,
            heating_allowed: false,
            pwm_period: pwm_duration,
            max_temperature,
            temperature_pid_output: 0.0,
            heating_element_wattage,
            max_clamp,
            target_temp_enabled: true,
            digital_port,
            temperature_port,
            filter : LowPassFilter { alpha: 0.08, state: None } 
        }
    }

    pub fn set_target_temperature(&mut self, temp: ThermodynamicTemperature) {
        self.heating.target_temperature = temp;
    }

    pub fn set_temperature_target_enabled(&mut self, enabled: bool) {
        self.target_temp_enabled = enabled;
    }

    pub fn get_temperature_target_enabled(&self) -> bool {
        self.target_temp_enabled
    }

    pub const fn disallow_heating(&mut self) {
        self.heating_allowed = false;
    }

    pub const fn allow_heating(&mut self) {
        self.heating_allowed = true;
    }

    pub fn get_heating_element_wattage(&self) -> f64 {
        self.temperature_pid_output * self.heating_element_wattage
    }

    pub fn update(
        &mut self,
        now: Instant,
        relais: &mut dyn DigitalOutputDevice,
        temperature_sensor: &dyn TemperatureInputDevice,
    ) {
        self.temperature_pid_output = 0.0;
        let temperature = temperature_sensor.get_input(self.temperature_port);
        self.heating.wiring_error = temperature.is_err();
        let temperature_celsius = match temperature {
            Ok(t) => ThermodynamicTemperature::new::<degree_celsius>(t.temperature as f64),
            Err(_e) => ThermodynamicTemperature::new::<degree_celsius>(0.0),
        };
        self.filter.update(temperature_celsius.get::<degree_celsius>());
        self.heating.temperature =  ThermodynamicTemperature::new::<degree_celsius>(self.filter.current_val());
        if self.heating.temperature > self.max_temperature {
            // disable the relais and return
            relais.set_output(self.digital_port, false);
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
            relais.set_output(self.digital_port, on);
            self.heating.heating = on;
        }
    }
}
