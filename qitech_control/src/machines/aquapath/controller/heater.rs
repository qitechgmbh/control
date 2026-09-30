use std::time::Duration;
use std::time::Instant;

use qitech_control_core::controllers::pid::PidController;
use qitech_lib::units::thermodynamic_temperature::degree_celsius;

use super::config::ControllerConfig;
use super::config::PidGains;
use super::io::DwellRelay;
use super::timing::Timer;

/// Owns the heating element: its relay, the PID driving it, and the PWM window the
/// duty cycle is sliced out of.
pub struct HeaterController {
    relay: DwellRelay,
    pid: PidController,
    duty: f64,
    window_start: Instant,
    elapsed_in_window: Duration,
    last_active: Timer,
    total_energy: f64,
}

impl HeaterController {
    pub fn new(relay: DwellRelay, gains: PidGains, now: Instant) -> Self {
        Self {
            relay,
            pid: PidController::new(gains.kp, gains.ki, gains.kd),
            duty: 0.0,
            window_start: now,
            elapsed_in_window: Duration::ZERO,
            last_active: Timer::default(),
            total_energy: 0.0,
        }
    }

    pub fn is_on(&self) -> bool {
        self.relay.is_on()
    }

    pub fn power(&self, config: &ControllerConfig) -> f64 {
        if self.relay.is_on() {
            config.heating_element_power
        } else {
            0.0
        }
    }

    pub fn total_energy(&self) -> f64 {
        self.total_energy
    }

    pub fn kp(&self) -> f64 {
        self.pid.get_kp()
    }

    pub fn ki(&self) -> f64 {
        self.pid.get_ki()
    }

    pub fn kd(&self) -> f64 {
        self.pid.get_kd()
    }

    pub fn configure_pid(&mut self, ki: f64, kp: f64, kd: f64) {
        self.pid.configure(ki, kp, kd);
    }

    /// Drops PID memory and restarts the PWM window. Every change that invalidates the
    /// accumulated control history goes through here.
    pub fn reset_control(&mut self, now: Instant) {
        self.pid.reset();
        self.duty = 0.0;
        self.window_start = now;
    }

    /// Heat was delivered recently enough that the loop still holds residual heat.
    pub fn was_recently_active(&self, now: Instant, config: &ControllerConfig) -> bool {
        self.is_on()
            || self
                .last_active
                .started_within(now, config.thermal_flow_settle_duration)
    }

    /// Advances the PID and the PWM window. Runs every cycle whichever way the control
    /// decision goes, so the integral term stays continuous.
    pub fn tick(&mut self, error: f64, now: Instant, config: &ControllerConfig) {
        let control = self.pid.update(error, now);
        self.duty = if control.is_finite() { control } else { 0.0 };

        self.elapsed_in_window = now.duration_since(self.window_start);
        if self.elapsed_in_window >= config.heating_pwm_period {
            self.window_start = now;
            self.elapsed_in_window = Duration::ZERO;
        }
    }

    /// Drives the element for this cycle's slice of the PWM window and books the energy.
    pub fn drive(&mut self, error: f64, now: Instant, dt: f64, config: &ControllerConfig) {
        if error >= config.heating_full_power_error.get::<degree_celsius>() {
            // Far below target: skip the duty cycle and heat continuously.
            self.request(true, now, config);
        } else {
            let duty = self.duty.clamp(0.0, 1.0);
            let on_time = config.heating_pwm_period.mul_f64(duty);
            self.request(duty > 0.0 && self.elapsed_in_window < on_time, now, config);
        }

        if self.is_on() {
            self.last_active.restart(now);
            self.total_energy += self.power(config) * dt / 3600.0;
        }
    }

    /// Normal, dwell-limited switching for ordinary demand changes.
    pub fn request(&mut self, on: bool, now: Instant, config: &ControllerConfig) {
        self.relay
            .request(on, now, config.relay_min_on_time, config.relay_min_off_time);
    }

    /// Cuts the element immediately, ignoring the dwell.
    ///
    /// Used whenever the flow interlock or a hard limit says heating must stop: the
    /// element may never stay energized past the cycle in which flow is lost, so this
    /// path cannot be subject to `relay_min_on_time` the way duty cycling is.
    pub fn force_off(&mut self, now: Instant) {
        self.relay.force_off(now);
    }

    /// Inside the deadband: park the element and drop PID memory, so the integral term
    /// cannot wind up while no correction is needed.
    pub fn settle(&mut self, now: Instant, config: &ControllerConfig) {
        self.request(false, now, config);
        self.duty = 0.0;
        self.pid.reset();
    }
}
