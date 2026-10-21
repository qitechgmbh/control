//! Duty-computing strategies for a resistively heated zone.
//!
//! A [`HeatingStrategy`] answers one question: given what the sensor reads and
//! where the operator wants the zone, what fraction of full power should the
//! heater get this tick? Reading the sensor, the over-temperature cutout, the
//! slow-PWM window and driving the relay all stay with the caller.

pub mod observer_pi;
pub mod sensor_lag_observer;

use std::time::Instant;

pub use observer_pi::ObserverPi;
pub use observer_pi::ObserverPiParams;
pub use sensor_lag_observer::SensorLagObserver;

use super::pid::PidController;

/// A control law that turns a temperature reading into a duty demand.
pub trait HeatingStrategy: Send {
    /// Duty for this tick, in `0..=1`.
    ///
    /// Called at the caller's loop rate, which may be far faster than the
    /// sensor updates — implementations must tolerate seeing the same
    /// `measured_c` many times in a row.
    fn update(&mut self, measured_c: f64, target_c: f64, now: Instant) -> f64;

    /// Drop all state, so that re-enabling a zone does not resume with a stale
    /// integral or a stale estimate.
    fn reset(&mut self);

    /// The outer-loop PID, so gains stay readable and settable through the
    /// existing API surface whichever strategy is in use.
    fn pid(&self) -> &PidController;

    fn pid_mut(&mut self) -> &mut PidController;
}

/// A PID on the raw reading, clamped to the duty range, with anti-windup.
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
