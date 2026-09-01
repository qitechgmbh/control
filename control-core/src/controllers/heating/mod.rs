//! Duty-computing strategies for a resistively heated zone.
//!
//! A [`HeatingStrategy`] answers one question: given what the sensor reads and
//! where the operator wants the zone, what fraction of full power should the
//! heater get this tick? Everything around that — reading the sensor, the
//! over-temperature cutout, the slow-PWM window, driving the relay — stays with
//! the caller.
//!
//! Splitting it out this way lets several control laws be compared against the
//! same plant without reimplementing any of them for the test rig; see
//! `machine_implementations::extruder1::simulation`.
//!
//! # Why more than a PID
//!
//! A band-heated barrel is a hard plant for a textbook PID, for two reasons that
//! are properties of the hardware rather than of the gains:
//!
//! - **The sensor lags.** An RTD in a pocket trails the steel by its own time
//!   constant. On a ramp that is a *constant* error of `tau * rate`, so the loop
//!   shuts off that far past setpoint no matter how it is tuned.
//! - **The band stores energy.** A clamped band runs far above the steel it
//!   heats and keeps discharging after the relay opens.
//!
//! Both are predictable, and both are addressed by estimating what cannot be
//! measured: [`SensorLagObserver`] undoes the first, [`BandObserver`] the
//! second.

pub mod band_observer;
pub mod cascade;
pub mod observer_pi;
pub mod sensor_lag_observer;

use std::time::Instant;

use super::pid::PidController;

pub use band_observer::{BandObserver, BandObserverGains, BandObserverParams};
pub use cascade::{CascadeController, CascadeParams};
pub use observer_pi::{ObserverPi, ObserverPiParams};
pub use sensor_lag_observer::SensorLagObserver;

/// A control law that turns a temperature reading into a duty demand.
pub trait HeatingStrategy: Send {
    /// Duty for this tick, in `0..=1`.
    ///
    /// Called at the caller's loop rate, which may be far faster than the
    /// sensor updates — implementations must tolerate seeing the same
    /// `measured_c` many times in a row.
    fn update(&mut self, measured_c: f64, target_c: f64, now: Instant) -> f64;

    /// Drop all state. Called when heating is disabled, so that re-enabling
    /// does not resume with a stale integral or a stale estimate.
    fn reset(&mut self);

    /// The outer-loop PID, so gains stay readable and settable through the
    /// existing API surface whichever strategy is in use.
    fn pid(&self) -> &PidController;

    /// Mutable access to the outer-loop PID, for `SetTemperaturePidSettings`.
    fn pid_mut(&mut self) -> &mut PidController;
}

/// The control law that has always shipped: a PID on the raw reading, clamped
/// to the duty range, with conditional-integration anti-windup.
///
/// Kept as a reference point to measure the others against, and as the fallback
/// for hardware whose thermal behaviour has not been modelled.
#[derive(Debug)]
pub struct PidBaseline {
    pid: PidController,
    max_clamp: f64,
}

impl PidBaseline {
    pub const fn new(kp: f64, ki: f64, kd: f64, max_clamp: f64) -> Self {
        Self {
            pid: PidController::new(kp, ki, kd),
            max_clamp,
        }
    }
}

impl HeatingStrategy for PidBaseline {
    fn update(&mut self, measured_c: f64, target_c: f64, now: Instant) -> f64 {
        self.pid
            .update_with_antiwindup(target_c - measured_c, now, 0.0, self.max_clamp)
    }

    fn reset(&mut self) {
        self.pid.reset();
    }

    fn pid(&self) -> &PidController {
        &self.pid
    }

    fn pid_mut(&mut self) -> &mut PidController {
        &mut self.pid
    }
}
