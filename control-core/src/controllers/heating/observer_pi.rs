//! PI on an estimate of the metal, plus a feedforward that already knows what
//! holding that temperature costs.
//!
//! Two changes to a textbook loop, each aimed at one reason the textbook loop
//! overshoots on a slow thermal plant.
//!
//! # Regulate the metal, not the probe
//!
//! A PID on the raw reading is chasing a signal that trails the metal by
//! `tau_probe * rate` — a *constant* error for as long as the ramp lasts, which
//! no choice of gains removes, because from inside the loop it is
//! indistinguishable from being genuinely that far from setpoint. On the
//! extruder that is around 34 K, and it is most of the observed overshoot. A
//! [`SensorLagObserver`] hands the loop the metal temperature instead.
//!
//! # Let feedforward carry the steady state
//!
//! Holding a zone at temperature costs a predictable duty, roughly proportional
//! to how far above ambient it is being held. Supplying that directly leaves the
//! integrator with only the residual to correct, which has three consequences
//! that all matter here:
//!
//! - it converges in minutes instead of the tens of minutes a small `ki` needs
//!   to build the whole steady-state duty from zero;
//! - it cannot wind up across a long saturated ramp, because it was never
//!   carrying much to begin with; and
//! - **a setpoint change is answered immediately**, since the feedforward moves
//!   the instant the target does rather than waiting for an integral to rebuild.
//!   That is what fixes overshoot on a step up from an already-hot machine,
//!   which is a different failure from the cold-start one.

use std::time::Instant;

use super::{HeatingStrategy, SensorLagObserver};
use crate::controllers::pid::PidController;

/// Configuration for [`ObserverPi`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObserverPiParams {
    pub kp: f64,
    pub ki: f64,
    /// The probe's time constant in seconds — how far it trails the metal.
    pub tau_sensor_s: f64,
    /// Smoothing applied before differentiating the reading, in seconds.
    /// Trades residual lag against how much sensor quantisation reaches the
    /// estimate; an order of magnitude below `tau_sensor_s` is the useful
    /// regime.
    pub tau_filter_s: f64,
    /// Cap on the observer's correction, in K.
    pub lead_max_k: f64,
    /// Steady-state duty needed per K above ambient. Identify it by settling
    /// the zone and reading the duty it holds.
    pub ff_duty_per_k: f64,
    pub ambient_c: f64,
    pub max_clamp: f64,
}

/// PI on an observed metal temperature, over a steady-state feedforward.
#[derive(Debug)]
pub struct ObserverPi {
    params: ObserverPiParams,
    observer: SensorLagObserver,
    pid: PidController,
    estimate_c: f64,
}

impl ObserverPi {
    pub const fn new(params: ObserverPiParams) -> Self {
        Self {
            observer: SensorLagObserver::new(
                params.tau_sensor_s,
                params.tau_filter_s,
                params.lead_max_k,
            ),
            pid: PidController::new(params.kp, params.ki, 0.0),
            params,
            estimate_c: 0.0,
        }
    }

    /// Feedforward duty for a target, before the loop trims it.
    pub fn feedforward(&self, target_c: f64) -> f64 {
        (self.params.ff_duty_per_k * (target_c - self.params.ambient_c))
            .clamp(0.0, self.params.max_clamp)
    }

    /// The most recent estimate of the metal temperature, in °C. Diagnostics.
    pub const fn estimate_c(&self) -> f64 {
        self.estimate_c
    }
}

impl HeatingStrategy for ObserverPi {
    fn update(&mut self, measured_c: f64, target_c: f64, now: Instant) -> f64 {
        self.estimate_c = self.observer.update(measured_c, now);

        let ff = self.feedforward(target_c);

        // The PI only has to supply what the feedforward misses, so its
        // saturation limits are the duty range *shifted by the feedforward*.
        // Passing them this way means the anti-windup freeze triggers on the
        // total output actually reaching a rail, which is the physically real
        // condition — testing the trim alone would freeze the integral while
        // there was still headroom, or let it wind while there was none.
        let trim = self.pid.update_with_antiwindup(
            target_c - self.estimate_c,
            now,
            -ff,
            self.params.max_clamp - ff,
        );

        (ff + trim).clamp(0.0, self.params.max_clamp)
    }

    fn reset(&mut self) {
        self.observer.reset();
        self.pid.reset();
    }

    fn pid(&self) -> &PidController {
        &self.pid
    }

    fn pid_mut(&mut self) -> &mut PidController {
        &mut self.pid
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn params() -> ObserverPiParams {
        ObserverPiParams {
            kp: 0.05,
            ki: 0.0002,
            tau_sensor_s: 150.0,
            tau_filter_s: 15.0,
            lead_max_k: 40.0,
            ff_duty_per_k: 0.004,
            ambient_c: 22.0,
            max_clamp: 1.0,
        }
    }

    #[test]
    fn feedforward_is_proportional_to_the_lift_above_ambient() {
        let s = ObserverPi::new(params());
        assert!((s.feedforward(122.0) - 0.4).abs() < 1e-9);
        assert!((s.feedforward(22.0) - 0.0).abs() < 1e-9);
        // Clamped, not extrapolated past full power.
        assert!((s.feedforward(1000.0) - 1.0).abs() < 1e-9);
    }

    /// The feedforward must answer a setpoint change on the very tick it
    /// happens. This is the mechanism that fixes overshoot on a step up, and it
    /// is what a pure integral cannot do.
    #[test]
    fn a_setpoint_change_moves_the_output_immediately() {
        let t0 = Instant::now();
        let mut s = ObserverPi::new(params());
        // Settle at 180 with the reading right on target.
        let mut last = 0.0;
        for i in 0..2_000u64 {
            last = s.update(180.0, 180.0, t0 + Duration::from_millis(i * 100));
        }
        let after = s.update(180.0, 200.0, t0 + Duration::from_millis(200_000));
        assert!(
            after - last > 0.05,
            "output moved only {:.4} on a 20 K setpoint step",
            after - last
        );
    }

    /// Held below target with the output on the rail, the integral must not
    /// accumulate — otherwise it discharges into an overshoot on arrival.
    #[test]
    fn the_integral_does_not_wind_up_while_saturated() {
        let t0 = Instant::now();
        let mut s = ObserverPi::new(params());
        for i in 0..20_000u64 {
            let duty = s.update(30.0, 250.0, t0 + Duration::from_millis(i * 100));
            assert!((0.0..=1.0).contains(&duty));
        }
        // Now jump to setpoint. A wound-up integral would hold the output high;
        // with anti-windup the demand must collapse to about the feedforward.
        let duty = s.update(250.0, 250.0, t0 + Duration::from_millis(2_000_000));
        let ff = s.feedforward(250.0);
        assert!(
            duty < ff + 0.1,
            "duty {duty:.3} on arrival is far above the {ff:.3} feedforward; \
             the integral wound up"
        );
    }

    /// Output must stay a duty cycle under every input, including a sensor that
    /// reads above target.
    #[test]
    fn output_stays_in_range() {
        let t0 = Instant::now();
        let mut s = ObserverPi::new(params());
        for (i, measured) in [0.0, 22.0, 180.0, 400.0, -50.0]
            .iter()
            .cycle()
            .take(5_000)
            .enumerate()
        {
            let duty = s.update(*measured, 180.0, t0 + Duration::from_millis(i as u64 * 100));
            assert!(
                (0.0..=1.0).contains(&duty),
                "duty {duty} out of range at reading {measured}"
            );
        }
    }

    /// `reset` has to clear the estimator as well as the integral, or
    /// re-enabling heating resumes from a stale picture of a plant that has been
    /// cooling in the meantime.
    #[test]
    fn reset_clears_the_observer_too() {
        let t0 = Instant::now();
        let mut s = ObserverPi::new(params());
        for i in 0..5_000u64 {
            s.update(
                0.1f64.mul_add(i as f64, 22.0),
                250.0,
                t0 + Duration::from_millis(i * 100),
            );
        }
        assert!(s.estimate_c() > 500.0, "test did not build up a lead");

        s.reset();
        let estimate = {
            s.update(180.0, 250.0, t0 + Duration::from_secs(10_000));
            s.estimate_c()
        };
        assert!(
            (estimate - 180.0).abs() < 1e-9,
            "after reset the first estimate should be the reading, was {estimate}"
        );
    }
}
