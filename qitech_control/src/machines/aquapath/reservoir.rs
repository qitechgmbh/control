use std::time::Duration;
use std::time::Instant;

use qitech_framework::machine::BuildContext;
use qitech_framework::machine::BuildResult;
use qitech_framework::machine::ConfigProperty;
use qitech_framework::machine::Measurement;
use qitech_framework::machine::StateProperty;
use qitech_framework::machine_build;
use qitech_lib::units::AngularVelocity;
use qitech_lib::units::ThermodynamicTemperature;
use qitech_lib::units::angular_velocity::revolution_per_minute;
use qitech_lib::units::thermodynamic_temperature::degree_celsius;
use qitech_lib::units::volume_rate::liter_per_minute;

use super::AquapathV1;
use super::controller::Controller;
use super::controller::ControllerNotice;
use super::controller::CoolingMode;

const PID_BOUNDS: (f64, f64) = (0.0, 5.0);
/// °C
const TOLERANCE_BOUNDS: (f64, f64) = (0.0, 10.0);
/// s
const THERMAL_FLOW_SETTLE_DURATION_BOUNDS: (f64, f64) = (0.0, 30.0);
/// °C
const PUMP_COOLDOWN_MIN_TEMPERATURE_BOUNDS: (f64, f64) = (10.0, 80.0);

/// Picks one reservoir out of the machine.
///
/// Framework callbacks must be plain `fn(&mut AquapathV1)` pointers that cannot capture
/// anything, so the side is carried in the type instead:
/// `AquapathV1::on_pid_changed::<Left>` coerces to such a pointer.
pub trait Side {
    fn reservoir(machine: &mut AquapathV1) -> &mut Reservoir;
}

pub enum Left {}

pub enum Right {}

impl Side for Left {
    fn reservoir(machine: &mut AquapathV1) -> &mut Reservoir {
        &mut machine.left
    }
}

impl Side for Right {
    fn reservoir(machine: &mut AquapathV1) -> &mut Reservoir {
        &mut machine.right
    }
}

struct ReservoirMeasurements {
    flow: Measurement<f64>,
    temperature: Measurement<f64>,
    revolutions: Measurement<f64>,
    power: Measurement<f64>,
    total_energy: Measurement<f64>,
    pump_cooldown_remaining: Measurement<f64>,
    heating_startup_wait_remaining: Measurement<f64>,
}

struct ReservoirState {
    heating_startup_wait_active: StateProperty<bool>,
    pump_cooldown_active: StateProperty<bool>,
    should_flow: StateProperty<bool>,
    heating: StateProperty<bool>,
    cooling_mode: StateProperty<Option<CoolingMode>>,
    thermal_delay: StateProperty<f64>,
    cooldown_min_temperature: StateProperty<f64>,
}

struct ReservoirConfig {
    target_temperature: ConfigProperty<f64>,
    fan_max_revolutions: ConfigProperty<f64>,
    heating_tolerance: ConfigProperty<f64>,
    cooling_tolerance: ConfigProperty<f64>,
    pid_kp: ConfigProperty<f64>,
    pid_ki: ConfigProperty<f64>,
    pid_kd: ConfigProperty<f64>,
    thermal_flow_settle_duration: ConfigProperty<f64>,
    pump_cooldown_min_temperature: ConfigProperty<f64>,
}

/// One reservoir loop together with its published measurements, state and config.
pub struct Reservoir {
    label: &'static str,
    controller: Controller,
    measurements: ReservoirMeasurements,
    state: ReservoirState,
    config: ReservoirConfig,
}

impl Reservoir {
    pub fn label(&self) -> &'static str {
        self.label
    }

    pub fn update(&mut self, now: Instant) {
        self.controller.update(now);
    }

    pub fn drain_notices(&mut self) -> Vec<ControllerNotice> {
        self.controller.drain_notices()
    }

    pub fn min_temperature(&self) -> ThermodynamicTemperature {
        self.controller.min_temperature()
    }

    pub fn max_temperature(&self) -> ThermodynamicTemperature {
        self.controller.max_temperature()
    }

    pub fn enter_auto(&mut self) {
        self.controller.allow_cooling();
        self.controller.allow_heating();
        self.controller.allow_pump();
    }

    pub fn enter_standby(&mut self) {
        self.controller.disable_cooling();
        self.controller.disallow_heating();
        self.controller.set_should_pump(false);
    }

    pub fn set_should_pump(&mut self, should_pump: bool) {
        self.controller.set_should_pump(should_pump);
    }

    /// Lifts the target to `minimum` if it currently sits below it.
    pub fn raise_target_to(&mut self, minimum: ThermodynamicTemperature) {
        if self.controller.target_temperature().get::<degree_celsius>()
            < minimum.get::<degree_celsius>()
        {
            self.controller.set_target_temperature(minimum);
        }
    }

    // --- applying config written from outside ---

    pub fn apply_target_temperature(&mut self, min_settable: f64) {
        let max_settable = self.controller.max_temperature().get::<degree_celsius>();
        let target = self
            .config
            .target_temperature
            .get()
            .max(min_settable)
            .min(max_settable);
        self.controller
            .set_target_temperature(ThermodynamicTemperature::new::<degree_celsius>(target));
    }

    pub fn apply_fan_max_revolutions(&mut self) {
        let revolutions = self.config.fan_max_revolutions.get();
        self.controller
            .set_max_revolutions(AngularVelocity::new::<revolution_per_minute>(revolutions));
    }

    pub fn apply_heating_tolerance(&mut self) {
        let current = self.controller.heating_tolerance().get::<degree_celsius>();
        let tolerance = sanitize(
            self.config.heating_tolerance.get(),
            TOLERANCE_BOUNDS,
            current,
        );
        self.controller
            .set_heating_tolerance(ThermodynamicTemperature::new::<degree_celsius>(tolerance));
    }

    pub fn apply_cooling_tolerance(&mut self) {
        let current = self.controller.cooling_tolerance().get::<degree_celsius>();
        let tolerance = sanitize(
            self.config.cooling_tolerance.get(),
            TOLERANCE_BOUNDS,
            current,
        );
        self.controller
            .set_cooling_tolerance(ThermodynamicTemperature::new::<degree_celsius>(tolerance));
    }

    pub fn apply_pid(&mut self) {
        let kp = sanitize(
            self.config.pid_kp.get(),
            PID_BOUNDS,
            self.controller.pid_kp(),
        );
        self.controller.set_pid_kp(kp);
        let ki = sanitize(
            self.config.pid_ki.get(),
            PID_BOUNDS,
            self.controller.pid_ki(),
        );
        self.controller.set_pid_ki(ki);
        let kd = sanitize(
            self.config.pid_kd.get(),
            PID_BOUNDS,
            self.controller.pid_kd(),
        );
        self.controller.set_pid_kd(kd);
    }

    pub fn apply_thermal_flow_settle_duration(&mut self) {
        let current = self.controller.thermal_flow_settle_duration().as_secs_f64();
        let seconds = sanitize(
            self.config.thermal_flow_settle_duration.get(),
            THERMAL_FLOW_SETTLE_DURATION_BOUNDS,
            current,
        );
        self.controller
            .set_thermal_flow_settle_duration(Duration::from_secs_f64(seconds));
    }

    pub fn apply_pump_cooldown_min_temperature(&mut self) {
        let current = self
            .controller
            .pump_cooldown_min_temperature()
            .get::<degree_celsius>();
        let temperature = sanitize(
            self.config.pump_cooldown_min_temperature.get(),
            PUMP_COOLDOWN_MIN_TEMPERATURE_BOUNDS,
            current,
        );
        self.controller
            .set_pump_cooldown_min_temperature(ThermodynamicTemperature::new::<degree_celsius>(
                temperature,
            ));
    }

    // --- publishing ---

    pub fn publish(&mut self, now: Instant) {
        let controller = &self.controller;

        let measurements = &mut self.measurements;
        measurements
            .flow
            .set(controller.flow().get::<liter_per_minute>());
        measurements
            .temperature
            .set(controller.temperature().get::<degree_celsius>());
        measurements
            .revolutions
            .set(controller.revolutions().get::<revolution_per_minute>());
        measurements.power.set(controller.power());
        measurements.total_energy.set(controller.total_energy());
        measurements
            .pump_cooldown_remaining
            .set(controller.pump_cooldown_remaining(now).as_secs_f64());
        measurements
            .heating_startup_wait_remaining
            .set(controller.heating_startup_wait_remaining(now).as_secs_f64());

        let state = &mut self.state;
        state
            .heating_startup_wait_active
            .set(controller.is_heating_startup_wait_active(now));
        state
            .pump_cooldown_active
            .set(controller.is_pump_cooldown_active(now));
        state.should_flow.set(controller.should_pump());
        state.heating.set(controller.is_heating());
        state.cooling_mode.set(controller.cooling_mode());
        state
            .thermal_delay
            .set(controller.thermal_flow_settle_duration().as_secs_f64());
        state.cooldown_min_temperature.set(
            controller
                .pump_cooldown_min_temperature()
                .get::<degree_celsius>(),
        );
    }
}

// The two builders below are deliberately spelled out per side. `#[machine_build]`
// validates every `config()` path against the schema at compile time, and only accepts
// a string literal written directly in the call — so the paths cannot be assembled
// from a side prefix without giving up that check.
impl Reservoir {
    #[machine_build(AquapathV1)]
    pub fn build_left(ctx: &mut BuildContext, controller: Controller) -> BuildResult<Self> {
        Ok(Self {
            label: "Left Reservoir",
            controller,
            measurements: ReservoirMeasurements {
                flow: ctx.measurement::<f64>("left_flow").build()?,
                temperature: ctx.measurement::<f64>("left_temperature").build()?,
                revolutions: ctx.measurement::<f64>("left_revolutions").build()?,
                power: ctx.measurement::<f64>("left_power").build()?,
                total_energy: ctx.measurement::<f64>("left_total_energy").build()?,
                pump_cooldown_remaining: ctx
                    .measurement::<f64>("left_pump_cooldown_remaining")
                    .build()?,
                heating_startup_wait_remaining: ctx
                    .measurement::<f64>("left_heating_startup_wait_remaining")
                    .build()?,
            },
            state: ReservoirState {
                heating_startup_wait_active: ctx
                    .state::<bool>("left_heating_startup_wait_active")
                    .build()?,
                pump_cooldown_active: ctx.state::<bool>("left_pump_cooldown_active").build()?,
                should_flow: ctx.state::<bool>("left_should_flow").build()?,
                heating: ctx.state::<bool>("left_heating").build()?,
                cooling_mode: ctx
                    .state::<Option<CoolingMode>>("left_cooling_mode")
                    .build()?,
                thermal_delay: ctx
                    .state::<f64>("left_thermal_safety_state.thermal_delay")
                    .build()?,
                cooldown_min_temperature: ctx
                    .state::<f64>("left_thermal_safety_state.cooldown_min_temperature")
                    .build()?,
            },
            config: ReservoirConfig {
                target_temperature: ctx
                    .config::<f64>("left_target_temperature")
                    .on_external_changed(AquapathV1::on_target_temperature_changed::<Left>)
                    .default(25.0)
                    .maximum(80.0)
                    .minimum(0.0)
                    .build()?,
                fan_max_revolutions: ctx
                    .config::<f64>("left_fan_max_revolutions")
                    .on_external_changed(AquapathV1::on_fan_max_revolutions_changed::<Left>)
                    .default(100.0)
                    .minimum(0.0)
                    .maximum(100.0)
                    .build()?,
                heating_tolerance: ctx
                    .config::<f64>("left_tolerance_config.heating")
                    .on_external_changed(AquapathV1::on_heating_tolerance_changed::<Left>)
                    .default(0.4)
                    .build()?,
                cooling_tolerance: ctx
                    .config::<f64>("left_tolerance_config.cooling")
                    .on_external_changed(AquapathV1::on_cooling_tolerance_changed::<Left>)
                    .default(0.8)
                    .build()?,
                pid_kp: ctx
                    .config::<f64>("left_pid_config.kp")
                    .on_external_changed(AquapathV1::on_pid_changed::<Left>)
                    .default(AquapathV1::DEFAULT_PID_KP)
                    .build()?,
                pid_ki: ctx
                    .config::<f64>("left_pid_config.ki")
                    .on_external_changed(AquapathV1::on_pid_changed::<Left>)
                    .default(AquapathV1::DEFAULT_PID_KI)
                    .build()?,
                pid_kd: ctx
                    .config::<f64>("left_pid_config.kd")
                    .on_external_changed(AquapathV1::on_pid_changed::<Left>)
                    .default(AquapathV1::DEFAULT_PID_KD)
                    .build()?,
                thermal_flow_settle_duration: ctx
                    .config::<f64>("left_thermal_flow_settle_duration")
                    .on_external_changed(
                        AquapathV1::on_thermal_flow_settle_duration_changed::<Left>,
                    )
                    .default(0.0)
                    .minimum(0.0)
                    .maximum(30.0)
                    .build()?,
                pump_cooldown_min_temperature: ctx
                    .config::<f64>("left_pump_cooldown_min_temperature")
                    .on_external_changed(
                        AquapathV1::on_pump_cooldown_min_temperature_changed::<Left>,
                    )
                    .default(32.0)
                    .minimum(10.0)
                    .maximum(80.0)
                    .build()?,
            },
        })
    }

    #[machine_build(AquapathV1)]
    pub fn build_right(ctx: &mut BuildContext, controller: Controller) -> BuildResult<Self> {
        Ok(Self {
            label: "Right Reservoir",
            controller,
            measurements: ReservoirMeasurements {
                flow: ctx.measurement::<f64>("right_flow").build()?,
                temperature: ctx.measurement::<f64>("right_temperature").build()?,
                revolutions: ctx.measurement::<f64>("right_revolutions").build()?,
                power: ctx.measurement::<f64>("right_power").build()?,
                total_energy: ctx.measurement::<f64>("right_total_energy").build()?,
                pump_cooldown_remaining: ctx
                    .measurement::<f64>("right_pump_cooldown_remaining")
                    .build()?,
                heating_startup_wait_remaining: ctx
                    .measurement::<f64>("right_heating_startup_wait_remaining")
                    .build()?,
            },
            state: ReservoirState {
                heating_startup_wait_active: ctx
                    .state::<bool>("right_heating_startup_wait_active")
                    .build()?,
                pump_cooldown_active: ctx.state::<bool>("right_pump_cooldown_active").build()?,
                should_flow: ctx.state::<bool>("right_should_flow").build()?,
                heating: ctx.state::<bool>("right_heating").build()?,
                cooling_mode: ctx
                    .state::<Option<CoolingMode>>("right_cooling_mode")
                    .build()?,
                thermal_delay: ctx
                    .state::<f64>("right_thermal_safety_state.thermal_delay")
                    .build()?,
                cooldown_min_temperature: ctx
                    .state::<f64>("right_thermal_safety_state.cooldown_min_temperature")
                    .build()?,
            },
            config: ReservoirConfig {
                target_temperature: ctx
                    .config::<f64>("right_target_temperature")
                    .on_external_changed(AquapathV1::on_target_temperature_changed::<Right>)
                    .default(25.0)
                    .maximum(80.0)
                    .minimum(0.0)
                    .build()?,
                fan_max_revolutions: ctx
                    .config::<f64>("right_fan_max_revolutions")
                    .on_external_changed(AquapathV1::on_fan_max_revolutions_changed::<Right>)
                    .default(100.0)
                    .minimum(0.0)
                    .maximum(100.0)
                    .build()?,
                heating_tolerance: ctx
                    .config::<f64>("right_tolerance_config.heating")
                    .on_external_changed(AquapathV1::on_heating_tolerance_changed::<Right>)
                    .default(0.4)
                    .build()?,
                cooling_tolerance: ctx
                    .config::<f64>("right_tolerance_config.cooling")
                    .on_external_changed(AquapathV1::on_cooling_tolerance_changed::<Right>)
                    .default(0.8)
                    .build()?,
                pid_kp: ctx
                    .config::<f64>("right_pid_config.kp")
                    .on_external_changed(AquapathV1::on_pid_changed::<Right>)
                    .default(AquapathV1::DEFAULT_PID_KP)
                    .build()?,
                pid_ki: ctx
                    .config::<f64>("right_pid_config.ki")
                    .on_external_changed(AquapathV1::on_pid_changed::<Right>)
                    .default(AquapathV1::DEFAULT_PID_KI)
                    .build()?,
                pid_kd: ctx
                    .config::<f64>("right_pid_config.kd")
                    .on_external_changed(AquapathV1::on_pid_changed::<Right>)
                    .default(AquapathV1::DEFAULT_PID_KD)
                    .build()?,
                thermal_flow_settle_duration: ctx
                    .config::<f64>("right_thermal_flow_settle_duration")
                    .on_external_changed(
                        AquapathV1::on_thermal_flow_settle_duration_changed::<Right>,
                    )
                    .default(0.0)
                    .minimum(0.0)
                    .maximum(30.0)
                    .build()?,
                pump_cooldown_min_temperature: ctx
                    .config::<f64>("right_pump_cooldown_min_temperature")
                    .on_external_changed(
                        AquapathV1::on_pump_cooldown_min_temperature_changed::<Right>,
                    )
                    .default(32.0)
                    .minimum(10.0)
                    .maximum(80.0)
                    .build()?,
            },
        })
    }
}

/// Rejects non-finite input in favour of `fallback`; clamps everything else.
fn sanitize(value: f64, (min, max): (f64, f64), fallback: f64) -> f64 {
    if !value.is_finite() {
        return fallback;
    }
    value.clamp(min, max)
}
