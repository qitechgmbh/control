use qitech_framework::machine::BuildContext;
use qitech_framework::machine::BuildResult;
use qitech_framework::machine::ConfigProperty;
use qitech_framework::machine::Measurement;
use qitech_framework::machine::StateProperty;
use qitech_lib::ethercat_hal::io::{
    digital_output::DigitalOutputDevice, temperature_input::TemperatureInputDevice,
};
use qitech_lib::units::{ThermodynamicTemperature, thermodynamic_temperature::degree_celsius};
use std::time::{Duration, Instant};

use crate::machines::extruder1::{PidGains, Zone};
use qitech_control_core::controllers::pid::PidController;

/// Fixed hardware limits and tuning of one heating zone, supplied at build time.
pub struct TemperatureControllerConfig {
    pub max_temperature: ThermodynamicTemperature,
    pub pwm_period: Duration,
    pub heating_element_wattage: f64,
    pub max_clamp: f64,
    pub digital_port: usize,
    pub temperature_port: usize,
    /// `(kp, ki, kd)`
    pub gains: (f64, f64, f64),
}

pub struct TemperatureController {
    pid: PidController,
    gains: PidGains,

    // --- config ---
    target_temperature: ConfigProperty<ThermodynamicTemperature>,

    // --- state ---
    wiring_error: StateProperty<bool>,

    // --- measurements ---
    temperature: Measurement<ThermodynamicTemperature>,
    // TODO: `Measurement<Power>` once qitech_lib::units gains a Power quantity (see schema).
    power: Measurement<f64>,

    // --- internals ---
    heating: bool,
    heating_allowed: bool,
    window_start: Instant,
    pwm_period: Duration,
    max_temperature: ThermodynamicTemperature,
    temperature_pid_output: f64,
    heating_element_wattage: f64,
    max_clamp: f64,
    digital_port: usize,
    temperature_port: usize,
}

impl TemperatureController {
    pub fn init(
        ctx: &mut BuildContext,
        zone: Zone,
        config: TemperatureControllerConfig,
    ) -> BuildResult<Self> {
        let paths = zone.paths();
        let (kp, ki, kd) = config.gains;

        Ok(Self {
            pid: PidController::new(kp, ki, kd),
            gains: PidGains::init(ctx, paths.gains, config.gains)?,

            // Defaults to 0 °C, matching the pre-migration behaviour: the old `Heating::default()`
            // started at 0 °C and the 150 °C passed to `TemperatureController::new` was written to
            // a field the control loop never read. Starting at 150 °C would make a freshly built
            // machine heat as soon as it enters Heat mode.
            target_temperature: ctx
                .config::<degree_celsius>(paths.target_temperature)
                .default(0.0)
                .minimum(0.0)
                .maximum(config.max_temperature.get::<degree_celsius>())
                .build()?,

            wiring_error: ctx.state::<bool>(paths.wiring_error).build()?,

            temperature: ctx
                .measurement::<degree_celsius>(paths.temperature)
                .build()?,
            power: ctx.measurement::<f64>(paths.power).build()?,

            heating: false,
            heating_allowed: false,
            window_start: Instant::now(),
            pwm_period: config.pwm_period,
            max_temperature: config.max_temperature,
            temperature_pid_output: 0.0,
            heating_element_wattage: config.heating_element_wattage,
            max_clamp: config.max_clamp,
            digital_port: config.digital_port,
            temperature_port: config.temperature_port,
        })
    }

    pub fn disable(&mut self, relais: &mut dyn DigitalOutputDevice) {
        relais.set_output(self.digital_port, false);
        self.heating = false;
        self.disallow_heating();
    }

    pub const fn disallow_heating(&mut self) {
        self.heating_allowed = false;
    }

    pub const fn allow_heating(&mut self) {
        self.heating_allowed = true;
    }

    /// Current draw of this zone's heating element in watts.
    pub fn heating_element_wattage(&self) -> f64 {
        self.temperature_pid_output * self.heating_element_wattage
    }

    pub fn update(
        &mut self,
        now: Instant,
        relais: &mut dyn DigitalOutputDevice,
        temperature_sensor: &dyn TemperatureInputDevice,
    ) {
        // Only reconfigure when a gain actually changed — `configure` resets the loop.
        if let Some((ki, kp, kd)) = self.gains.take_change() {
            self.pid.configure(ki, kp, kd);
        }

        self.temperature_pid_output = 0.0;

        let reading = temperature_sensor.get_input(self.temperature_port);
        self.wiring_error.set(reading.is_err());

        let temperature = match reading {
            Ok(t) => ThermodynamicTemperature::new::<degree_celsius>(t.temperature as f64),
            Err(_) => ThermodynamicTemperature::new::<degree_celsius>(0.0),
        };
        self.temperature.set(temperature);

        if temperature > self.max_temperature {
            // disable the relais and return
            relais.set_output(self.digital_port, false);
            self.heating = false;
            self.power.set(self.heating_element_wattage());
            return;
        }

        if self.heating_allowed {
            let error: f64 = self.target_temperature.get_as::<degree_celsius>()
                - temperature.get::<degree_celsius>();

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
            self.heating = on;
        }

        self.power.set(self.heating_element_wattage());
    }
}
