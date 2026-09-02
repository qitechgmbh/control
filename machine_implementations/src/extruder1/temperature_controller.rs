use super::Heating;
use control_core::controllers::heating::{HeatingStrategy, PidBaseline};
use control_core::controllers::pid::PidController;
use qitech_lib::{
    ethercat_hal::io::{
        digital_output::DigitalOutputDevice, temperature_input::TemperatureInputDevice,
    },
    units::{ThermodynamicTemperature, thermodynamic_temperature::degree_celsius},
};
use std::time::{Duration, Instant};

/// One heating zone: sensor in, relay out.
///
/// Owns everything around the control law — reading the RTD, the
/// over-temperature cutout, the slow-PWM window, driving the relay — and
/// delegates the duty decision to a [`HeatingStrategy`], so swapping the control
/// law is a change of one constructor argument.
pub struct TemperatureController {
    strategy: Box<dyn HeatingStrategy>,
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
    target_temp_enabled: bool, // Sets whether the frontend should display a target temperature setter for this temp controller
}

impl TemperatureController {
    pub fn disable(&mut self, relais: &mut dyn DigitalOutputDevice) {
        self.open_relay(relais);
        self.disallow_heating();
    }

    /// A zone driven by a plain PID on the raw reading.
    #[allow(clippy::too_many_arguments)]
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
        Self::with_strategy(
            Box::new(PidBaseline::new(kp, ki, kd, max_clamp)),
            target_temp,
            max_temperature,
            heating,
            pwm_duration,
            heating_element_wattage,
            digital_port,
            temperature_port,
        )
    }

    /// A zone driven by an arbitrary control law. The strategy owns its own
    /// output clamp, so there is no `max_clamp` here.
    #[allow(clippy::too_many_arguments)]
    pub fn with_strategy(
        strategy: Box<dyn HeatingStrategy>,
        target_temp: ThermodynamicTemperature,
        max_temperature: ThermodynamicTemperature,
        heating: Heating,
        pwm_duration: Duration,
        heating_element_wattage: f64,
        digital_port: usize,
        temperature_port: usize,
    ) -> Self {
        Self {
            strategy,
            target_temp,
            window_start: Instant::now(),
            heating,
            heating_allowed: false,
            pwm_period: pwm_duration,
            max_temperature,
            temperature_pid_output: 0.0,
            heating_element_wattage,
            target_temp_enabled: true,
            digital_port,
            temperature_port,
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

    /// The outer-loop PID, whichever strategy is in use, so gains stay readable
    /// and settable through the existing API.
    pub fn pid(&self) -> &PidController {
        self.strategy.pid()
    }

    pub fn pid_mut(&mut self) -> &mut PidController {
        self.strategy.pid_mut()
    }

    pub fn disallow_heating(&mut self) {
        self.heating_allowed = false;
        // Drop the integral and the estimator's state, so re-enabling does not
        // resume from a stale picture of a plant that has been cooling.
        self.strategy.reset();
    }

    pub const fn allow_heating(&mut self) {
        self.heating_allowed = true;
    }

    /// The duty the control law last asked for, in `0..=1`.
    pub const fn duty(&self) -> f64 {
        self.temperature_pid_output
    }

    pub fn get_heating_element_wattage(&self) -> f64 {
        self.temperature_pid_output * self.heating_element_wattage
    }

    /// Open the relay and record that the zone is not heating.
    fn open_relay(&mut self, relais: &mut dyn DigitalOutputDevice) {
        relais.set_output(self.digital_port, false);
        self.heating.heating = false;
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
        self.heating.temperature = temperature_celsius;

        // A failed read decodes to 0 °C, which is maximum error, which would be
        // maximum heat demand — the heater running flat out precisely when
        // nothing can see how hot it is getting. The strategy is deliberately
        // neither stepped nor reset: a transient fault should not discard a good
        // estimate, and every estimator here tolerates a long `dt`.
        if self.heating.wiring_error || self.heating.temperature > self.max_temperature {
            self.open_relay(relais);
            return;
        }

        if !self.heating_allowed {
            self.open_relay(relais);
            return;
        }

        let duty = self.strategy.update(
            self.heating.temperature.get::<degree_celsius>(),
            self.heating.target_temperature.get::<degree_celsius>(),
            now,
        );
        self.temperature_pid_output = duty;

        let mut elapsed = now.duration_since(self.window_start);
        // `elapsed` has to be reset along with the window: leaving the old,
        // already-past-the-period value in place made the comparison below false
        // for the first tick of every window, holding the relay open for one tick
        // per window whatever duty was asked for.
        if elapsed >= self.pwm_period {
            self.window_start = now;
            elapsed = Duration::ZERO;
        }

        let on = elapsed < self.pwm_period.mul_f64(duty);
        relais.set_output(self.digital_port, on);
        self.heating.heating = on;
    }
}
