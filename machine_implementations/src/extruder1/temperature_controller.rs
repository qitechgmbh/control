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
    filter: LowPassFilter,
    /// Debug/test only: drive the heating element at a fixed wattage instead of letting the PID
    /// regulate towards the target temperature. The `max_temperature` cutoff still applies.
    power_override_enabled: bool,
    /// Fixed heating power in watts used while `power_override_enabled` is set
    power_override_watts: f64,
    /// Duty commanded by the IMC auto-tuner while a step test owns this zone, as a fraction of
    /// full output. `None` means the zone is under normal control.
    ///
    /// The tuner returns `None` in every non-running state and the caller applies that every tick,
    /// so releasing the zone is structural — there is no restore step an abort path could skip.
    tuner_duty: Option<f64>,
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
            // The integral needs the full output range: at production temperature a zone can need
            // 30-50% duty just to hold setpoint, so a narrower clamp would leave permanent offset,
            // and a non-negative lower bound could never unwind an overshoot. The back-calculation
            // in PidController is what keeps this safe from windup.
            pid: PidController::new(kp, ki, kd, -max_clamp, max_clamp),
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
            filter: LowPassFilter {
                alpha: 0.08,
                state: None,
            },
            power_override_enabled: false,
            power_override_watts: 0.0,
            tuner_duty: None,
        }
    }

    /// Current commanded duty cycle, 0.0 – `max_clamp`. This is what the auto-tuner captures as
    /// the baseline output before it takes over.
    pub const fn get_duty(&self) -> f64 {
        self.temperature_pid_output
    }

    pub const fn get_max_clamp(&self) -> f64 {
        self.max_clamp
    }

    pub fn get_temperature_celsius(&self) -> f64 {
        self.heating.temperature.get::<degree_celsius>()
    }

    pub fn get_target_temperature_celsius(&self) -> f64 {
        self.heating.target_temperature.get::<degree_celsius>()
    }

    pub fn get_max_temperature_celsius(&self) -> f64 {
        self.max_temperature.get::<degree_celsius>()
    }

    /// Hand the output to the auto-tuner (`Some(duty)`) or give it back to the PID (`None`).
    ///
    /// On release the PID is reset, so it restarts without integral and derivative state left over
    /// from before the test.
    pub fn apply_tuner_command(&mut self, cmd: Option<f64>) {
        if self.tuner_duty.is_some() && cmd.is_none() {
            self.pid.reset();
        }
        self.tuner_duty = cmd.map(|duty| duty.clamp(0.0, self.max_clamp));
    }

    pub const fn is_tuner_driving(&self) -> bool {
        self.tuner_duty.is_some()
    }

    /// Apply tuned gains together with the matching derivative filter.
    ///
    /// These are bundled deliberately: a non-zero `kd` without the filter turns the derivative term
    /// into a noise amplifier, since `dt` is the sub-millisecond control-loop period.
    pub const fn configure_pid(&mut self, ki: f64, kp: f64, kd: f64, derivative_filter_tc: f64) {
        self.pid.configure(ki, kp, kd);
        self.pid.set_derivative_filter(derivative_filter_tc);
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

    /// Highest heating power this zone can be driven at, i.e. the element wattage capped by the
    /// duty cycle limit. Used as the upper bound for the debug power override.
    pub fn get_max_heating_power(&self) -> f64 {
        self.heating_element_wattage * self.max_clamp
    }

    pub const fn get_power_override_enabled(&self) -> bool {
        self.power_override_enabled
    }

    pub const fn get_power_override_watts(&self) -> f64 {
        self.power_override_watts
    }

    /// Debug/test measure: pin the heating output to a fixed wattage, bypassing the PID.
    ///
    /// This drives the element open-loop, so the zone can overshoot its target temperature — only
    /// the `max_temperature` cutoff in [`Self::update`] still limits it. The PID is reset on every
    /// change so it starts clean once the override is turned off again.
    pub fn set_power_override(&mut self, enabled: bool, watts: f64) {
        self.power_override_enabled = enabled;
        self.power_override_watts = watts.clamp(0.0, self.get_max_heating_power());
        self.pid.reset();
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
        self.filter
            .update(temperature_celsius.get::<degree_celsius>());
        self.heating.temperature =
            ThermodynamicTemperature::new::<degree_celsius>(self.filter.current_val());
        if self.heating.temperature > self.max_temperature {
            // disable the relais and return
            relais.set_output(self.digital_port, false);
            self.heating.heating = false;
            return;
        }

        if self.heating_allowed {
            let duty = if let Some(tuner_duty) = self.tuner_duty {
                // The auto-tuner owns the output for the duration of a step test. Keep the PID
                // reset so it does not wind up while it is not driving.
                self.pid.reset();
                tuner_duty
            } else if self.power_override_enabled {
                // Debug/test mode: hold a fixed heating power instead of regulating. Keep the PID
                // reset so its integral doesn't wind up while it isn't driving the output.
                self.pid.reset();
                if self.heating_element_wattage > 0.0 {
                    (self.power_override_watts / self.heating_element_wattage)
                        .clamp(0.0, self.max_clamp)
                } else {
                    0.0
                }
            } else {
                let measurement = self.heating.temperature.get::<degree_celsius>();
                let error: f64 =
                    self.heating.target_temperature.get::<degree_celsius>() - measurement;

                // Derivative on measurement, so changing a target temperature does not kick the
                // derivative term.
                let control = self.pid.update_with_measurement(error, measurement, now);
                // Clamp PID output to 0.0 – 1.0 (as duty cycle)
                control.clamp(0.0, self.max_clamp)
            };

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
