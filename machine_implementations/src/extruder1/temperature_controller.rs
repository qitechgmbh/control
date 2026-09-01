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
/// This owns everything around the control law — reading the RTD, the
/// over-temperature cutout, the slow-PWM window, driving the relay — and
/// delegates the duty decision itself to a [`HeatingStrategy`]. Swapping the
/// control law is then a change of one constructor argument, and the offline
/// simulation in [`crate::extruder1::simulation`] can compare strategies while
/// still driving the shipping code around them.
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
        relais.set_output(self.digital_port, false);
        self.heating.heating = false;
        self.disallow_heating();
    }

    /// A zone driven by a plain PID on the raw reading — the control law that
    /// has always shipped.
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

    /// A zone driven by an arbitrary control law.
    ///
    /// The strategy owns its own output clamp, so there is no `max_clamp` here.
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
        // Drop the integral and any state the estimator built up, so that
        // re-enabling does not resume from a stale picture of a plant that has
        // been cooling in the meantime.
        self.strategy.reset();
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
        self.heating.temperature = temperature_celsius;

        // A failed read decodes to 0 °C, which is maximum error, which would be
        // maximum heat demand — the heater running flat out precisely when
        // nothing can see how hot it is getting. Open the relay instead. The
        // strategy is deliberately not stepped and not reset: a transient fault
        // should not discard a good estimate, and every estimator here is
        // written to tolerate the long `dt` that a sustained one produces.
        if self.heating.wiring_error {
            relais.set_output(self.digital_port, false);
            self.heating.heating = false;
            return;
        }

        if self.heating.temperature > self.max_temperature {
            // disable the relais and return
            relais.set_output(self.digital_port, false);
            self.heating.heating = false;
            return;
        }

        if self.heating_allowed {
            let duty = self.strategy.update(
                self.heating.temperature.get::<degree_celsius>(),
                self.heating.target_temperature.get::<degree_celsius>(),
                now,
            );

            self.temperature_pid_output = duty;

            let mut elapsed = now.duration_since(self.window_start);

            // Restart window if needed. `elapsed` has to be recomputed with it:
            // leaving the old, already-past-the-period value in place made the
            // comparison below false for the first tick of every window, so the
            // relay was held open for one tick per window no matter what duty
            // was asked for.
            if elapsed >= self.pwm_period {
                self.window_start = now;
                elapsed = Duration::ZERO;
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
