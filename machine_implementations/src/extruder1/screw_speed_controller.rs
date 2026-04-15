use std::time::{Duration, Instant};

use control_core::{
    controllers::{clamping_timeagnostic_pid::ClampingTimeagnosticPidController, pid_autotuner::{AutoTuneConfig, PidAutoTuner}},
    helpers::interpolation::normalize,
    transmission::{Transmission, fixed::FixedTransmission},
};
use qitech_lib::{ethercat_hal::{self, io::{analog_input::AnalogInputDevice, serial_interface::SerialInterfaceDevice}}, units::{AngularVelocity, Frequency, Pressure, angular_velocity::revolution_per_minute, electric_current::milliampere, frequency::hertz, pressure::bar}};
use crate::extruder1::mitsubishi_cs80::MitsubishiCS80Status;
use super::{api::{PidSettings, PressureAutoTuneConfig}, mitsubishi_cs80::{MitsubishiCS80, MotorStatus}};
const AUTOTUNE_MAX_DURATION: Duration = Duration::from_secs(30);

#[derive(Debug)]
pub struct ScrewSpeedController {
    pub pid: ClampingTimeagnosticPidController,
    pub target_pressure: Pressure,
    pub target_rpm: AngularVelocity,
    pub motor_poles: usize,
    pub inverter: MitsubishiCS80,
    last_update: Instant,
    uses_rpm: bool,
    forward_rotation: bool,
    transmission: FixedTransmission,
    pid_autotuner: Option<PidAutoTuner>,
    frequency: Frequency,
    pub pressure: Pressure,
    maximum_frequency: Frequency,
    minimum_frequency: Frequency,
    motor_on: bool,
    nozzle_pressure_limit: Pressure,
    nozzle_pressure_limit_enabled: bool,
    autotune_high_frequency: Frequency,
    autotune_low_frequency: Frequency,
    pub wiring_error : bool,
}

impl ScrewSpeedController {
    pub fn new(
        inverter: MitsubishiCS80,
        target_pressure: Pressure,
        target_rpm: AngularVelocity,
        transmission: FixedTransmission,
        motor_poles: usize,
    ) -> Self {
        let now = Instant::now();
        Self {
            inverter,
            // need to tune
            pid: ClampingTimeagnosticPidController::simple_new(0.01, 0.0, 0.02),
            last_update: now,
            target_pressure,
            target_rpm,
            uses_rpm: true,
            forward_rotation: true,
            transmission: transmission,
            //FixedTransmission::new(1.0 / 34.0),
            motor_on: false,
            nozzle_pressure_limit: Pressure::new::<bar>(100.0),
            nozzle_pressure_limit_enabled: true,
            frequency: Frequency::new::<hertz>(0.0),
            maximum_frequency: Frequency::new::<hertz>(60.0),
            minimum_frequency: Frequency::new::<hertz>(0.0),
            motor_poles,
            wiring_error: false,
            pressure: Pressure::new::<bar>(0.0),
            pid_autotuner: None,
            autotune_high_frequency: Frequency::new::<hertz>(0.0),
            autotune_low_frequency: Frequency::new::<hertz>(0.0),
        }
    }

    pub const fn get_inverter_status(&mut self) -> MitsubishiCS80Status {
        self.inverter.status
    }

    pub const fn get_motor_enabled(&mut self) -> bool {
        self.motor_on
    }

    pub fn set_nozzle_pressure_limit(&mut self, pressure: Pressure) {
        self.nozzle_pressure_limit = pressure;
    }

    pub fn get_nozzle_pressure_limit(&self) -> Pressure {
        self.nozzle_pressure_limit
    }

    pub const fn get_nozzle_pressure_limit_enabled(&self) -> bool {
        self.nozzle_pressure_limit_enabled
    }

    pub const fn set_nozzle_pressure_limit_is_enabled(&mut self, enabled: bool) {
        self.nozzle_pressure_limit_enabled = enabled;
    }

    pub fn get_target_rpm(&self) -> AngularVelocity {
        self.target_rpm
    }

    pub const fn get_rotation_direction(&self) -> bool {
        self.forward_rotation
    }

    pub fn set_rotation_direction(&mut self, forward: bool) {
        self.forward_rotation = forward;
        if self.motor_on {
            self.inverter.set_rotation(self.forward_rotation);
        }
    }

    pub fn set_target_pressure(&mut self, target_pressure: Pressure) {
        self.reset_pid();
        self.target_pressure = target_pressure;
    }

    pub fn set_target_screw_rpm(
        &mut self,
        target_rpm: AngularVelocity,
        _motor_rpm_rating: AngularVelocity,
        motor_poles: usize,
    ) {
        // Use uom here and perhaps clamp it
        let target_motor_rpm = self
            .transmission
            .calculate_angular_velocity_input(target_rpm);

        self.target_rpm = target_rpm;

        let target_frequency: Frequency = Frequency::new::<hertz>(
            target_motor_rpm.get::<revolution_per_minute>() as f64 / 120.0 * motor_poles as f64,
        );

        self.inverter.set_frequency_target(target_frequency);
    }

    pub const fn get_uses_rpm(&self) -> bool {
        self.uses_rpm
    }

    pub const fn set_uses_rpm(&mut self, uses_rpm: bool) {
        self.uses_rpm = uses_rpm;
    }

    // Send Motor Turn Off Request to the Inverter
    pub fn turn_motor_off(&mut self) {
        self.inverter.stop_motor();
        self.motor_on = false;
    }

    pub fn turn_motor_on(&mut self) {
        self.inverter.set_rotation(self.forward_rotation);
        self.motor_on = true;
    }

    pub fn get_motor_status(&self) -> MotorStatus {
        let frequency = self.inverter.motor_status.frequency;
        let motor_rpm = AngularVelocity::new::<revolution_per_minute>(
            frequency.get::<hertz>() * 120.0 / self.motor_poles as f64,
        );

        let screw_rpm = self
            .transmission
            .calculate_angular_velocity_output(motor_rpm);

        let mut status = self.inverter.motor_status;
        status.rpm = screw_rpm;

        status
    }

    pub fn get_target_pressure(&self) -> Pressure {
        self.target_pressure
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
/*
    pub fn get_wiring_error(&self) -> bool {
        self.pressure_sensor.get_wiring_error()
    }
*//*
    pub fn get_sensor_current(&self) -> Result<ElectricCurrent, anyhow::Error> {

        let phys: ethercat_hal::io::analog_input::physical::AnalogInputValue =
            self.pressure_sensor.get_physical();

        match phys {
            ethercat_hal::io::analog_input::physical::AnalogInputValue::Potential(_) => {
                Err(anyhow::anyhow!("Potential is not expected"))
            }
            ethercat_hal::io::analog_input::physical::AnalogInputValue::Current(quantity) => {
                Ok(quantity)
            }
        }
    }
*/
    pub const fn reset_pid(&mut self) {
        self.pid.reset()
    }

    pub fn get_pressure(&self, pressure_sensor : &dyn AnalogInputDevice) -> Pressure {
        let phys = pressure_sensor.get_input(0);
        let current_result = match phys {
            Ok(phys) => {
                match phys.get_physical(&pressure_sensor.analog_input_range()) {
                    ethercat_hal::io::analog_input::physical::AnalogInputValue::Potential(_) => {
                        Err(anyhow::anyhow!("Potential is not expected"))
                    }
                    ethercat_hal::io::analog_input::physical::AnalogInputValue::Current(quantity) => {
                        Ok(quantity)
                    }
                }

            },
            Err(e) => {
                Err(anyhow::anyhow!("get_pressure failed: {}",e))
            }
        };

        let current = match current_result {
            Ok(current) => current.get::<milliampere>(),
            Err(_) => {
                tracing::error!("cant get pressure sensor reading");
                return Pressure::new::<bar>(0.0);
            }
        };
        let normalized = normalize(current, 4.0, 20.0);
        // Our pressure sensor has a range of Up to 350 Bar
        let actual_pressure = (normalized) * 350.0;
        Pressure::new::<bar>(actual_pressure)
    }

    pub fn update(&mut self, now: Instant, is_extruding: bool,serial_interface : &mut dyn SerialInterfaceDevice,pressure_sensor : &dyn AnalogInputDevice) {
        // TODO: move this logic elsewhere or make non async
        self.inverter.act(now, serial_interface);
        let measured_pressure = self.get_pressure(pressure_sensor);
        if !self.uses_rpm && !is_extruding && self.motor_on {
            let frequency = Frequency::new::<hertz>(0.0);
            self.inverter.set_frequency_target(frequency);
            self.turn_motor_off();
            self.last_update = now;
            return;
        }

        if (measured_pressure >= self.nozzle_pressure_limit)
            && self.nozzle_pressure_limit_enabled
            && self.motor_on
        {
            self.turn_motor_off();
            self.last_update = now;
            return;
        }

        if is_extruding && !self.motor_on {
            self.turn_motor_on();
        }

        if !self.uses_rpm && is_extruding {
            let error = self.target_pressure - measured_pressure;
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

    pub fn start_pressure_regulation(&mut self) {
        self.last_update = Instant::now();
        self.frequency = self.inverter.motor_status.frequency;
        self.pid.reset();
    }

    pub fn reset(&mut self) {
        self.pid.reset();
        self.last_update = Instant::now();
    }


      pub fn start_pressure_autotune(&mut self, now: Instant, config: PressureAutoTuneConfig) {
        // Snapshot the current inverter frequency as the relay centre point
        let base_hz = self.inverter.motor_status.frequency.get::<hertz>();
        let step_hz = config.frequency_step_hz;

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

        let auto_config = AutoTuneConfig {
            tune_delta: config.tune_delta,
            max_power: hz_swing,
            max_duration: AUTOTUNE_MAX_DURATION,
        };
        let mut tuner = PidAutoTuner::new(auto_config);
        let target_pressure = self.target_pressure.get::<bar>();
        tuner.start(now, target_pressure);
        self.pid_autotuner = Some(tuner);
        self.pid.reset();

        tracing::info!(
            "Pressure PID auto-tune started: target={:.2} bar, delta=±{:.2} bar, \
             relay {:.1}–{:.1} Hz (base {:.1} Hz, step ±{:.1} Hz)",
            target_pressure,
            config.tune_delta,
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
    }

    /// Current auto-tuner state as a string slice
    pub fn get_autotune_state(&self) -> &str {
        match &self.pid_autotuner {
            Some(tuner) => tuner.state(),
            None => "not_started",
        }
    }

    /// Auto-tune progress as a percentage (0 – 100)
    pub fn get_autotune_progress(&self) -> f64 {
        match &self.pid_autotuner {
            Some(tuner) => tuner.get_progress_percent(),
            None => 0.0,
        }
    }

    /// Returns PID values from the last completed auto-tune run, if any
    pub fn get_autotune_result(&self) -> Option<PidSettings> {
        self.pid_autotuner
            .as_ref()
            .and_then(|t| t.result().ok())
            .map(|result| PidSettings {
                ki: result.ki,
                kp: result.kp,
                kd: result.kd,
            })
    }

}
