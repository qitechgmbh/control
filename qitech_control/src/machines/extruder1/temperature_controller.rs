use std::time::Duration;
use std::time::Instant;

use qitech_control_core::controllers::heating::HeatingStrategy;
use qitech_framework::machine::BuildContext;
use qitech_framework::machine::BuildResult;
use qitech_framework::machine::ConfigProperty;
use qitech_framework::machine::Measurement;
use qitech_framework::machine::StateProperty;
use qitech_lib::ethercat_hal::io::digital_output::DigitalOutputDevice;
use qitech_lib::ethercat_hal::io::temperature_input::TemperatureInputDevice;
use qitech_lib::units::Power;
use qitech_lib::units::ThermodynamicTemperature;
use qitech_lib::units::power::watt;
use qitech_lib::units::thermodynamic_temperature::degree_celsius;

use crate::machines::extruder1::PidGains;
use crate::machines::extruder1::Zone;

/// Fixed hardware limits and tuning of one heating zone, supplied at build time.
pub struct TemperatureControllerConfig {
    /// Over-temperature cutout: above this the relay is held open.
    pub max_temperature: ThermodynamicTemperature,
    /// Highest target an operator may set.
    pub max_target_temperature: ThermodynamicTemperature,
    pub pwm_period: Duration,
    pub heating_element_wattage: f64,
    pub digital_port: usize,
    pub temperature_port: usize,
    /// The control law. It owns its own output clamp, and its gains become the
    /// defaults of the zone's gain properties.
    pub strategy: Box<dyn HeatingStrategy>,
}

pub struct TemperatureController {
    strategy: Box<dyn HeatingStrategy>,
    gains: PidGains,

    // --- config ---
    target_temperature: ConfigProperty<ThermodynamicTemperature>,

    // --- state ---
    wiring_error: StateProperty<bool>,

    // --- measurements ---
    temperature: Measurement<ThermodynamicTemperature>,
    power: Measurement<Power>,

    // --- internals ---
    heating: bool,
    heating_allowed: bool,
    window_start: Instant,
    pwm_period: Duration,
    max_temperature: ThermodynamicTemperature,
    temperature_pid_output: f64,
    heating_element_wattage: f64,
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
        let gains = strategy_gains(config.strategy.as_ref());

        Ok(Self {
            strategy: config.strategy,
            gains: PidGains::init(ctx, paths.gains, gains)?,

            // Defaults to 0 °C, matching the pre-migration behaviour: the old `Heating::default()`
            // started at 0 °C and the 150 °C passed to `TemperatureController::new` was written to
            // a field the control loop never read. Starting at 150 °C would make a freshly built
            // machine heat as soon as it enters Heat mode.
            target_temperature: ctx
                .config::<degree_celsius>(paths.target_temperature)
                .default(0.0)
                .minimum(0.0)
                .maximum(config.max_target_temperature.get::<degree_celsius>())
                .build()?,

            wiring_error: ctx.state::<bool>(paths.wiring_error).build()?,

            temperature: ctx
                .measurement::<degree_celsius>(paths.temperature)
                .build()?,
            power: ctx.measurement::<watt>(paths.power).build()?,

            heating: false,
            heating_allowed: false,
            window_start: Instant::now(),
            pwm_period: config.pwm_period,
            max_temperature: config.max_temperature,
            temperature_pid_output: 0.0,
            heating_element_wattage: config.heating_element_wattage,
            digital_port: config.digital_port,
            temperature_port: config.temperature_port,
        })
    }

    pub fn disable(&mut self, relais: &mut dyn DigitalOutputDevice) {
        self.open_relay(relais);
        self.disallow_heating();
    }

    /// Replace the control law. The new strategy starts with no integral and no
    /// estimate; the cutout, PWM window and relay stay with this controller.
    ///
    /// Its gains replace the zone's gain properties and their defaults: the laws
    /// act on different signals, so the old gains do not carry over.
    pub fn set_strategy(&mut self, strategy: Box<dyn HeatingStrategy>) {
        let (kp, ki, kd) = strategy_gains(strategy.as_ref());
        self.gains.reset_to(kp, ki, kd);
        self.strategy = strategy;
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

    /// Current draw of this zone's heating element.
    pub fn heating_element_wattage(&self) -> Power {
        Power::new::<watt>(self.temperature_pid_output * self.heating_element_wattage)
    }

    pub fn temperature(&self) -> ThermodynamicTemperature {
        self.temperature.get()
    }

    /// Open the relay and record that the zone is not heating.
    fn open_relay(&mut self, relais: &mut dyn DigitalOutputDevice) {
        relais.set_output(self.digital_port, false);
        self.heating = false;
    }

    pub fn update(
        &mut self,
        now: Instant,
        relais: &mut dyn DigitalOutputDevice,
        temperature_sensor: &dyn TemperatureInputDevice,
    ) {
        // Only reconfigure when a gain actually changed — `configure` resets the loop.
        if let Some((ki, kp, kd)) = self.gains.take_change() {
            self.strategy.pid_mut().configure(ki, kp, kd);
        }

        self.temperature_pid_output = 0.0;

        let reading = temperature_sensor.get_input(self.temperature_port);
        let wiring_error = reading.is_err();
        self.wiring_error.set(wiring_error);

        let temperature = match reading {
            Ok(t) => ThermodynamicTemperature::new::<degree_celsius>(t.temperature as f64),
            Err(_) => ThermodynamicTemperature::new::<degree_celsius>(0.0),
        };
        self.temperature.set(temperature);

        // Safety cutoff: if the sensor reports a wiring error or the measured
        // temperature exceeds the configured maximum, open the relay and skip
        // the control update for this tick.
        if wiring_error || temperature > self.max_temperature || !self.heating_allowed {
            self.open_relay(relais);
            self.power.set(self.heating_element_wattage());
            return;
        }

        let duty = self.strategy.update(
            temperature.get::<degree_celsius>(),
            self.target_temperature.get_as::<degree_celsius>(),
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
        self.heating = on;

        self.power.set(self.heating_element_wattage());
    }
}

/// `(kp, ki, kd)` the strategy's outer loop currently runs with.
fn strategy_gains(strategy: &dyn HeatingStrategy) -> (f64, f64, f64) {
    let pid = strategy.pid();
    (pid.get_kp(), pid.get_ki(), pid.get_kd())
}
