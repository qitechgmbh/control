use std::time::{Duration, Instant};

use qitech_framework::machine::ActResult;
use qitech_framework::machine::BuildContext;
use qitech_framework::machine::BuildResult;
use qitech_framework::machine::ConfigProperty;
use qitech_framework::machine::Measurement;
use qitech_framework::machine::StateProperty;
use qitech_lib::ethercat_hal::io::analog_input::AnalogInputDevice;
use qitech_lib::ethercat_hal::io::analog_input::physical::AnalogInputValue;
use qitech_lib::ethercat_hal::io::serial_interface::SerialInterfaceDevice;
use qitech_lib::units::{
    AngularVelocity, ElectricCurrent, ElectricPotential, Frequency, Pressure,
    angular_velocity::revolution_per_minute, electric_current::ampere,
    electric_current::milliampere, electric_potential::volt, frequency::hertz, pressure::bar,
};

use crate::controllers::clamping_timeagnostic_pid::ClampingTimeagnosticPidController;
use crate::controllers::pid_autotuner::{AutoTuneConfig, PidAutoTuner};
use crate::machines::extruder1::mitsubishi_cs80::{MitsubishiCS80, MitsubishiCS80Status};
use crate::machines::extruder1::{AutoTuneState, Extruder, PidGainPaths, PidGains, Regulation};
use crate::transmission::{Transmission, fixed::FixedTransmission};
use crate::types::RotationDirection;
use crate::utils::interpolation::normalize;

const AUTOTUNE_MAX_DURATION: Duration = Duration::from_secs(30);

const PRESSURE_PID_GAIN_PATHS: PidGainPaths = PidGainPaths {
    kp: "pid.pressure.kp",
    ki: "pid.pressure.ki",
    kd: "pid.pressure.kd",
};

/// The nine inverter status bits, exported as state properties.
struct InverterStatusProperties {
    running: StateProperty<bool>,
    forward_running: StateProperty<bool>,
    reverse_running: StateProperty<bool>,
    up_to_frequency: StateProperty<bool>,
    overload_warning: StateProperty<bool>,
    no_function: StateProperty<bool>,
    output_frequency_detection: StateProperty<bool>,
    abc_fault: StateProperty<bool>,
    fault_occurence: StateProperty<bool>,
}

impl InverterStatusProperties {
    fn init(ctx: &mut BuildContext) -> BuildResult<Self> {
        Ok(Self {
            running: ctx.state::<bool>("inverter.running").build()?,
            forward_running: ctx.state::<bool>("inverter.forward_running").build()?,
            reverse_running: ctx.state::<bool>("inverter.reverse_running").build()?,
            up_to_frequency: ctx.state::<bool>("inverter.up_to_frequency").build()?,
            overload_warning: ctx.state::<bool>("inverter.overload_warning").build()?,
            no_function: ctx.state::<bool>("inverter.no_function").build()?,
            output_frequency_detection: ctx
                .state::<bool>("inverter.output_frequency_detection")
                .build()?,
            abc_fault: ctx.state::<bool>("inverter.abc_fault").build()?,
            fault_occurence: ctx.state::<bool>("inverter.fault_occurence").build()?,
        })
    }

    fn update(&mut self, status: MitsubishiCS80Status) {
        self.running.set(status.running);
        self.forward_running.set(status.forward_running);
        self.reverse_running.set(status.reverse_running);
        self.up_to_frequency.set(status.su);
        self.overload_warning.set(status.ol);
        self.no_function.set(status.no_function);
        self.output_frequency_detection.set(status.fu);
        self.abc_fault.set(status.abc_);
        self.fault_occurence.set(status.fault_occurence);
    }
}

/// Auto-tune progress and outcome, exported so the UI can follow a run.
struct AutoTuneProperties {
    state: StateProperty<AutoTuneState>,
    progress: Measurement<f64>,
    result_kp: StateProperty<Option<f64>>,
    result_ki: StateProperty<Option<f64>>,
    result_kd: StateProperty<Option<f64>>,
}

impl AutoTuneProperties {
    fn init(ctx: &mut BuildContext) -> BuildResult<Self> {
        Ok(Self {
            state: ctx
                .state::<AutoTuneState>("pressure.autotune.state")
                .build()?,
            progress: ctx
                .measurement::<f64>("pressure.autotune_progress")
                .build()?,
            result_kp: ctx
                .state::<Option<f64>>("pressure.autotune.result.kp")
                .build()?,
            result_ki: ctx
                .state::<Option<f64>>("pressure.autotune.result.ki")
                .build()?,
            result_kd: ctx
                .state::<Option<f64>>("pressure.autotune.result.kd")
                .build()?,
        })
    }
}

/// The motor's live electrical and mechanical readings.
struct MotorMeasurements {
    rpm: Measurement<AngularVelocity>,
    frequency: Measurement<Frequency>,
    voltage: Measurement<ElectricPotential>,
    current: Measurement<ElectricCurrent>,
    // TODO: `Measurement<Power>` once qitech_lib::units gains a Power quantity (see schema).
    power: Measurement<f64>,
}

impl MotorMeasurements {
    fn init(ctx: &mut BuildContext) -> BuildResult<Self> {
        Ok(Self {
            rpm: ctx
                .measurement::<revolution_per_minute>("motor.rpm")
                .build()?,
            frequency: ctx.measurement::<hertz>("motor.frequency").build()?,
            voltage: ctx.measurement::<volt>("motor.voltage").build()?,
            current: ctx.measurement::<ampere>("motor.current").build()?,
            power: ctx.measurement::<f64>("motor.power").build()?,
        })
    }
}

pub struct ScrewSpeedController {
    pid: ClampingTimeagnosticPidController,
    gains: PidGains,

    // --- config ---
    direction: ConfigProperty<RotationDirection>,
    regulation: ConfigProperty<Regulation>,
    target_rpm: ConfigProperty<AngularVelocity>,
    target_pressure: ConfigProperty<Pressure>,
    nozzle_pressure_limit: ConfigProperty<Pressure>,
    nozzle_pressure_limit_enabled: ConfigProperty<bool>,
    autotune_tune_delta: ConfigProperty<Pressure>,
    autotune_frequency_step: ConfigProperty<Frequency>,

    /// Last regulation mode acted on, so a change can be detected in the callback.
    regulation_applied: Regulation,

    // --- state ---
    wiring_error: StateProperty<bool>,
    inverter_status: InverterStatusProperties,
    autotune: AutoTuneProperties,

    // --- measurements ---
    pressure: Measurement<Pressure>,
    motor: MotorMeasurements,

    // --- internals ---
    pub inverter: MitsubishiCS80,
    pub motor_poles: usize,
    transmission: FixedTransmission,
    pid_autotuner: Option<PidAutoTuner>,
    frequency: Frequency,
    maximum_frequency: Frequency,
    minimum_frequency: Frequency,
    motor_on: bool,
    autotune_high_frequency: Frequency,
    autotune_low_frequency: Frequency,
    last_update: Instant,
}

impl ScrewSpeedController {
    pub fn init<const VARIANT: usize>(
        ctx: &mut BuildContext,
        inverter: MitsubishiCS80,
        transmission: FixedTransmission,
        motor_poles: usize,
    ) -> BuildResult<Self> {
        let now = Instant::now();

        Ok(Self {
            // needs tuning
            pid: ClampingTimeagnosticPidController::simple_new(0.01, 0.0, 0.02),
            gains: PidGains::init(ctx, PRESSURE_PID_GAIN_PATHS, (0.01, 0.0, 0.02))?,

            direction: ctx
                .config::<RotationDirection>("screw.direction")
                .on_external_changed(|m: &mut Extruder<VARIANT>| {
                    m.screw_speed_controller.on_direction_changed()
                })
                .build()?,

            regulation: ctx
                .config::<Regulation>("screw.regulation")
                .on_external_changed(|m: &mut Extruder<VARIANT>| {
                    m.screw_speed_controller.on_regulation_changed()
                })
                .build()?,

            target_rpm: ctx
                .config::<revolution_per_minute>("screw.target_rpm")
                .default(0.0)
                .minimum(0.0)
                .on_external_changed(|m: &mut Extruder<VARIANT>| {
                    m.screw_speed_controller.on_target_rpm_changed()
                })
                .build()?,

            target_pressure: ctx
                .config::<bar>("screw.target_pressure")
                .default(0.0)
                .minimum(0.0)
                .on_external_changed(|m: &mut Extruder<VARIANT>| {
                    m.screw_speed_controller.on_target_pressure_changed()
                })
                .build()?,

            nozzle_pressure_limit: ctx
                .config::<bar>("pressure.limit")
                .default(100.0)
                .minimum(0.0)
                .build()?,

            nozzle_pressure_limit_enabled: ctx
                .config::<bool>("pressure.limit_enabled")
                .default(true)
                .build()?,

            autotune_tune_delta: ctx
                .config::<bar>("pressure.autotune.tune_delta")
                .default(0.5)
                .minimum(0.0)
                .build()?,

            autotune_frequency_step: ctx
                .config::<hertz>("pressure.autotune.frequency_step")
                .default(5.0)
                .minimum(0.0)
                .build()?,

            regulation_applied: Regulation::default(),

            wiring_error: ctx.state::<bool>("pressure.wiring_error").build()?,
            inverter_status: InverterStatusProperties::init(ctx)?,
            autotune: AutoTuneProperties::init(ctx)?,

            pressure: ctx.measurement::<bar>("pressure.value").build()?,
            motor: MotorMeasurements::init(ctx)?,

            inverter,
            motor_poles,
            transmission,
            pid_autotuner: None,
            frequency: Frequency::new::<hertz>(0.0),
            maximum_frequency: Frequency::new::<hertz>(60.0),
            minimum_frequency: Frequency::new::<hertz>(0.0),
            motor_on: false,
            autotune_high_frequency: Frequency::new::<hertz>(0.0),
            autotune_low_frequency: Frequency::new::<hertz>(0.0),
            last_update: now,
        })
    }

    // --- config callbacks ---

    fn on_direction_changed(&mut self) -> ActResult {
        if self.motor_on {
            self.inverter.set_rotation(self.is_forward());
        }

        Ok(())
    }

    fn on_target_pressure_changed(&mut self) -> ActResult {
        self.reset_pid();
        Ok(())
    }

    fn on_target_rpm_changed(&mut self) -> ActResult {
        self.apply_target_rpm();
        Ok(())
    }

    /// Mirrors the old `set_regulation`: entering rpm mode re-pushes the target frequency, leaving
    /// it hands control back to the pressure PID.
    fn on_regulation_changed(&mut self) -> ActResult {
        let regulation = self.regulation.get();

        if regulation == self.regulation_applied {
            return Ok(());
        }

        self.regulation_applied = regulation;

        match regulation {
            Regulation::Rpm => self.apply_target_rpm(),
            Regulation::Pressure => self.start_pressure_regulation(),
        }

        Ok(())
    }

    // --- accessors ---

    pub fn regulation(&self) -> Regulation {
        self.regulation.get()
    }

    fn is_forward(&self) -> bool {
        self.direction.get() == RotationDirection::Forward
    }

    pub const fn get_motor_enabled(&self) -> bool {
        self.motor_on
    }

    /// Motor power draw in watts.
    pub fn get_motor_power(&self) -> f64 {
        let status = self.inverter.motor_status;
        status.voltage.get::<volt>() * status.current.get::<ampere>()
    }

    pub fn reset_inverter(&mut self) {
        self.inverter.reset_inverter();
    }

    // --- motor ---

    /// Converts the configured screw rpm into an inverter frequency target.
    fn apply_target_rpm(&mut self) {
        let target_motor_rpm = self
            .transmission
            .calculate_angular_velocity_input(self.target_rpm.get());

        let target_frequency = Frequency::new::<hertz>(
            target_motor_rpm.get::<revolution_per_minute>() / 120.0 * self.motor_poles as f64,
        );

        self.inverter.set_frequency_target(target_frequency);
    }

    // Send Motor Turn Off Request to the Inverter
    pub fn turn_motor_off(&mut self) {
        self.inverter.stop_motor();
        self.motor_on = false;
    }

    pub fn turn_motor_on(&mut self) {
        self.inverter.set_rotation(self.is_forward());
        self.motor_on = true;
    }

    /// Screw rpm derived from the inverter's reported output frequency.
    fn screw_rpm(&self) -> AngularVelocity {
        let frequency = self.inverter.motor_status.frequency;
        let motor_rpm = AngularVelocity::new::<revolution_per_minute>(
            frequency.get::<hertz>() * 120.0 / self.motor_poles as f64,
        );

        self.transmission
            .calculate_angular_velocity_output(motor_rpm)
    }

    fn clamp_frequency(frequency: Frequency, min: Frequency, max: Frequency) -> Frequency {
        if frequency < min {
            min
        } else if frequency > max {
            max
        } else {
            frequency
        }
    }

    pub const fn reset_pid(&mut self) {
        self.pid.reset()
    }

    // --- pressure ---

    fn read_pressure(&mut self, pressure_sensor: &dyn AnalogInputDevice) -> Pressure {
        let phys = pressure_sensor.get_input(0);
        let current_result = match phys {
            Ok(phys) => match phys.get_physical(&pressure_sensor.analog_input_range()) {
                AnalogInputValue::Potential(_) => Err(anyhow::anyhow!("Potential is not expected")),
                AnalogInputValue::Current(quantity) => Ok(quantity),
            },
            Err(e) => Err(anyhow::anyhow!("read_pressure failed: {}", e)),
        };

        let current = match current_result {
            Ok(current) => current.get::<milliampere>(),
            Err(_) => {
                // Previously this only logged; the `wiring_error` field existed but was never
                // written, so `pressure.wiring_error` was permanently false.
                self.wiring_error.set(true);
                tracing::error!("cant get pressure sensor reading");
                return Pressure::new::<bar>(0.0);
            }
        };

        self.wiring_error.set(false);

        let normalized = normalize(current, 4.0, 20.0);
        // Our pressure sensor has a range of Up to 350 Bar
        Pressure::new::<bar>(normalized * 350.0)
    }

    // --- act ---

    pub fn update(
        &mut self,
        now: Instant,
        is_extruding: bool,
        serial_interface: &mut dyn SerialInterfaceDevice,
        pressure_sensor: &dyn AnalogInputDevice,
    ) {
        // Only reconfigure when a gain actually changed — `configure` resets the loop.
        if let Some((ki, kp, kd)) = self.gains.take_change() {
            self.pid.configure(ki, kp, kd);
        }

        self.inverter.act(now, serial_interface);
        self.publish_measurements();

        let measured_pressure = self.read_pressure(pressure_sensor);
        self.pressure.set(measured_pressure);

        let uses_rpm = self.regulation.get().uses_rpm();

        if !uses_rpm && !is_extruding && self.motor_on {
            let frequency = Frequency::new::<hertz>(0.0);
            self.inverter.set_frequency_target(frequency);
            self.turn_motor_off();
            self.last_update = now;
            return;
        }

        if (measured_pressure >= self.nozzle_pressure_limit.get())
            && self.nozzle_pressure_limit_enabled.get()
            && self.motor_on
        {
            self.turn_motor_off();
            self.last_update = now;
            return;
        }

        if is_extruding && !self.motor_on {
            self.turn_motor_on();
        }

        if !uses_rpm && is_extruding {
            // --- PID auto-tune active? ---
            let mut tuner_done = false;
            // `publish_autotune` needs all of `self`, so it cannot run while `tuner` borrows
            // `self.pid_autotuner`; the running branch defers the early return until after it.
            let mut tuner_running = false;
            if let Some(ref mut tuner) = self.pid_autotuner {
                if tuner.is_running() {
                    let pressure_bar = measured_pressure.get::<bar>();
                    let duty = tuner.update(pressure_bar, now);

                    // Duty > 0 → drive high; duty == 0 → drive low
                    let target_freq = if duty > 0.0 {
                        self.autotune_high_frequency
                    } else {
                        self.autotune_low_frequency
                    };
                    self.inverter.set_frequency_target(target_freq);
                    self.frequency = target_freq;
                    tuner_running = true;
                } else if tuner.is_completed() {
                    // Apply the computed PID gains and switch back to normal control
                    if let Ok(result) = tuner.result() {
                        let (kp, ki, kd) = (result.kp, result.ki, result.kd);
                        self.pid.configure(ki, kp, kd);
                        // Keep the exported gains in step without triggering a reconfigure.
                        self.gains.adopt(kp, ki, kd);
                        tracing::info!(
                            "Pressure PID auto-tune completed: kp={:.4}, ki={:.4}, kd={:.4}",
                            kp,
                            ki,
                            kd,
                        );
                    }
                    tuner_done = true;
                } else if tuner.is_failed() {
                    tracing::warn!("Pressure PID auto-tune failed");
                    tuner_done = true;
                }
            }

            self.publish_autotune();

            if tuner_running {
                self.last_update = now;
                return;
            }

            if tuner_done {
                self.pid_autotuner = None;
                // Sync frequency to current inverter state before resuming PID control
                self.frequency = self.inverter.motor_status.frequency;
                self.pid.reset();
            }

            // Normal PID pressure regulation
            let error = self.target_pressure.get() - measured_pressure;
            let freq_change = self.pid.update(error.get::<bar>(), now);

            self.frequency += Frequency::new::<hertz>(freq_change);
            self.frequency = Self::clamp_frequency(
                self.frequency,
                self.minimum_frequency,
                self.maximum_frequency,
            );

            self.inverter.set_frequency_target(self.frequency);
        }

        self.last_update = now;
    }

    fn publish_measurements(&mut self) {
        let status = self.inverter.motor_status;
        let inverter_status = self.inverter.status;
        let screw_rpm = self.screw_rpm();
        let motor_power = self.get_motor_power();

        self.motor.rpm.set(screw_rpm);
        self.motor.frequency.set(status.frequency);
        self.motor.voltage.set(status.voltage);
        self.motor.current.set(status.current);
        self.motor.power.set(motor_power);

        self.inverter_status.update(inverter_status);
    }

    fn publish_autotune(&mut self) {
        let (state, progress, result) = match &self.pid_autotuner {
            Some(tuner) => (
                AutoTuneState::from_tuner_state(tuner.state()),
                tuner.get_progress_percent(),
                tuner.result().ok().map(|r| (r.kp, r.ki, r.kd)),
            ),
            None => (AutoTuneState::NotStarted, 0.0, None),
        };

        self.autotune.state.set(state);
        self.autotune.progress.set(progress);
        self.autotune.result_kp.set(result.map(|(kp, _, _)| kp));
        self.autotune.result_ki.set(result.map(|(_, ki, _)| ki));
        self.autotune.result_kd.set(result.map(|(_, _, kd)| kd));
    }

    pub fn start_pressure_regulation(&mut self) {
        self.last_update = Instant::now();
        self.frequency = self.inverter.motor_status.frequency;
        self.pid.reset();
    }

    // --- autotune ---

    pub fn start_pressure_autotune(&mut self, now: Instant) {
        // Snapshot the current inverter frequency as the relay centre point
        let base_hz = self.inverter.motor_status.frequency.get::<hertz>();
        let step_hz = self.autotune_frequency_step.get_as::<hertz>();

        let high = Self::clamp_frequency(
            Frequency::new::<hertz>(base_hz + step_hz),
            self.minimum_frequency,
            self.maximum_frequency,
        );
        let low = Self::clamp_frequency(
            Frequency::new::<hertz>(base_hz - step_hz),
            self.minimum_frequency,
            self.maximum_frequency,
        );

        self.autotune_high_frequency = high;
        self.autotune_low_frequency = low;

        // Use the actual Hz swing as max_power so the Ziegler–Nichols result
        // is in the same units (Hz/bar) that the PID controller expects.
        let hz_swing = (high - low).get::<hertz>().max(0.01); // guard against zero
        let tune_delta = self.autotune_tune_delta.get_as::<bar>();

        let mut tuner = PidAutoTuner::new(AutoTuneConfig {
            tune_delta,
            max_power: hz_swing,
            max_duration: AUTOTUNE_MAX_DURATION,
        });

        let target_pressure = self.target_pressure.get_as::<bar>();
        tuner.start(now, target_pressure);
        self.pid_autotuner = Some(tuner);
        self.pid.reset();
        self.publish_autotune();

        tracing::info!(
            "Pressure PID auto-tune started: target={:.2} bar, delta=±{:.2} bar, \
             relay {:.1}–{:.1} Hz (base {:.1} Hz, step ±{:.1} Hz)",
            target_pressure,
            tune_delta,
            low.get::<hertz>(),
            high.get::<hertz>(),
            base_hz,
            step_hz,
        );
    }

    /// Abort an in-progress auto-tune run
    pub fn stop_autotune(&mut self) {
        if let Some(ref mut tuner) = self.pid_autotuner {
            tuner.stop();
        }

        self.publish_autotune();
    }
}
