//! PI on an estimate of the metal, plus a feedforward that already knows what
//! holding that temperature costs.
//!
//! Two changes to a textbook loop, each aimed at one reason it overshoots on a
//! slow thermal plant:
//!
//! - **Regulate the metal, not the probe.** A PID on the raw reading chases a
//!   signal that trails the metal by `tau_probe * rate` — a *constant* error for
//!   as long as the ramp lasts, which no choice of gains removes. On the extruder
//!   that is around 34 K and most of the observed overshoot, so a
//!   [`SensorLagObserver`] hands the loop the metal temperature instead.
//! - **Let feedforward carry the steady state.** Holding a zone costs a
//!   predictable duty, roughly proportional to its lift above ambient. Supplying
//!   it directly leaves the integrator only the residual, so it converges in
//!   minutes rather than tens, cannot wind up much across a saturated ramp, and
//!   answers a setpoint change immediately instead of waiting for an integral to
//!   rebuild.
//!
//! Undoing the probe's lag necessarily amplifies its quantisation: a 0.1 °C
//! step moves the estimate by `tau_sensor_s / tau_filter_s` times that, which
//! `kp` turns into a stuttering duty. So the *output* is low-passed with
//! `tau_duty_s`, kept far below every plant time constant — see
//! [`ObserverPiParams::tau_duty_s`].

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
    /// estimate; several times below `tau_sensor_s` is the useful regime.
    pub tau_filter_s: f64,
    /// Cap on the observer's correction, in K.
    pub lead_max_k: f64,
    /// Steady-state duty needed per K above ambient. Identify it by settling
    /// the zone and reading the duty it holds.
    pub ff_duty_per_k: f64,
    pub ambient_c: f64,
    pub max_clamp: f64,
    /// Time constant of the low-pass on the duty output, in seconds. Sets how
    /// much observer-amplified quantisation reaches the heater, at the cost of
    /// that much extra lag; keep it well under `tau_sensor_s`. `0.0` disables
    /// it.
    ///
    /// It filters the command, not the reading: the PI still sees the
    /// unsmoothed duty, so `kp`, `ki` and the anti-windup rails behave exactly
    /// as without it — the filter only limits how fast the heater follows.
    pub tau_duty_s: f64,
}

/// PI on an observed metal temperature, over a steady-state feedforward.
#[derive(Debug)]
pub struct ObserverPi {
    params: ObserverPiParams,
    observer: SensorLagObserver,
    pid: PidController,
    estimate_c: f64,
    smoothed_duty: Option<f64>,
    last_update: Option<Instant>,
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
            smoothed_duty: None,
            last_update: None,
        }
    }

    /// One-pole low-pass on the duty command.
    ///
    /// The first tick after construction or `reset` passes the duty straight
    /// through, so a cold start commands full power at once instead of ramping
    /// up to it over `tau_duty_s`.
    fn smooth_duty(&mut self, duty: f64, now: Instant) -> f64 {
        let last_update = self.last_update.replace(now);
        if self.params.tau_duty_s <= 0.0 {
            return duty;
        }
        let (Some(previous), Some(last_update)) = (self.smoothed_duty, last_update) else {
            self.smoothed_duty = Some(duty);
            return duty;
        };

        // Exact discretisation, as in the observer: a long gap between ticks
        // converges to the raw duty rather than overshooting it.
        let dt = now.saturating_duration_since(last_update).as_secs_f64();
        let alpha = 1.0 - (-dt / self.params.tau_duty_s).exp();
        // A convex blend of two in-range duties is in range, so no re-clamp.
        let smoothed = alpha.mul_add(duty - previous, previous);
        self.smoothed_duty = Some(smoothed);
        smoothed
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

        let duty = (ff + trim).clamp(0.0, self.params.max_clamp);
        self.smooth_duty(duty, now)
    }

    fn reset(&mut self) {
        self.observer.reset();
        self.pid.reset();
        self.smoothed_duty = None;
        self.last_update = None;
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
            // Off, so the tests below see the control law itself; the
            // smoothing has its own tests at the end.
            tau_duty_s: 0.0,
        }
    }

    fn smoothed_params() -> ObserverPiParams {
        ObserverPiParams {
            tau_duty_s: 5.0,
            ..params()
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

    /// Swing of the duty over the last half of a hold at setpoint, with the
    /// reading toggling by one 0.1 °C quantisation step every sample.
    fn duty_swing_under_quantisation(params: ObserverPiParams) -> f64 {
        let t0 = Instant::now();
        let mut s = ObserverPi::new(params);
        let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
        for i in 0..6_000u64 {
            let measured = if i % 4 < 2 { 180.0 } else { 180.1 };
            let duty = s.update(measured, 180.0, t0 + Duration::from_millis(i * 250));
            if i >= 3_000 {
                lo = lo.min(duty);
                hi = hi.max(duty);
            }
        }
        hi - lo
    }

    /// The reason the smoothing exists: quantisation the observer amplifies
    /// must reach the heater much attenuated.
    #[test]
    fn smoothing_attenuates_quantisation_stutter() {
        let raw = duty_swing_under_quantisation(params());
        let smoothed = duty_swing_under_quantisation(smoothed_params());
        assert!(
            raw > 0.01,
            "test did not produce a stutter to remove: {raw}"
        );
        assert!(
            smoothed < raw / 4.0,
            "smoothing only reduced the duty swing from {raw:.4} to {smoothed:.4}"
        );
    }

    /// A cold start must get full power on its first tick, not ramp into it.
    #[test]
    fn smoothing_passes_the_first_tick_straight_through() {
        let t0 = Instant::now();
        let mut raw = ObserverPi::new(params());
        let mut smoothed = ObserverPi::new(smoothed_params());
        assert_eq!(
            raw.update(22.0, 250.0, t0),
            smoothed.update(22.0, 250.0, t0)
        );
    }

    /// Smoothing adds lag of about `tau_duty_s`, and no more: a step in demand
    /// must be mostly followed after a few time constants.
    #[test]
    fn smoothing_follows_a_step_within_a_few_time_constants() {
        let t0 = Instant::now();
        let run = |params| {
            let mut s = ObserverPi::new(params);
            let mut duty = 0.0;
            for i in 0..2_000u64 {
                duty = s.update(180.0, 180.0, t0 + Duration::from_millis(i * 100));
            }
            let before = duty;
            // 20 s after a 180 -> 190 setpoint step: 4 time constants.
            for i in 1..=200u64 {
                duty = s.update(180.0, 190.0, t0 + Duration::from_millis(200_000 + i * 100));
            }
            duty - before
        };
        let raw_step = run(params());
        let smoothed_step = run(smoothed_params());
        assert!(raw_step > 0.1, "test did not produce a step: {raw_step}");
        // 1 - e^-4 is 98 %; leave a little room for the PI moving meanwhile.
        assert!(
            smoothed_step > 0.95 * raw_step,
            "after 4 tau the smoothed duty covered only {smoothed_step:.4} of a \
             {raw_step:.4} step"
        );
    }

    /// `reset` must drop the smoothed duty too, or re-enabling a zone would
    /// fade in from whatever it was commanding before.
    #[test]
    fn reset_clears_the_smoothed_duty() {
        let t0 = Instant::now();
        let mut s = ObserverPi::new(smoothed_params());
        for i in 0..2_000u64 {
            s.update(180.0, 180.0, t0 + Duration::from_millis(i * 100));
        }
        s.reset();
        let later = t0 + Duration::from_secs(10_000);
        let fresh = ObserverPi::new(smoothed_params()).update(22.0, 250.0, later);
        assert_eq!(s.update(22.0, 250.0, later), fresh);
    }
}
