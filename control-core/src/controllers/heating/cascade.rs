//! Cascade control for a band-heated mass: bound the stored energy, and the
//! overshoot is bounded with it.
//!
//! # The problem this solves
//!
//! A clamped band heater is not in good thermal contact with the metal it
//! heats. Driven hard it runs *far* above it — on the extruder, around 175 K
//! above — and that gap is stored energy which keeps flowing into the metal
//! after the relay opens. A single loop watching the metal cannot see it
//! coming, so it drives at full power right up to the moment the metal arrives
//! and then coasts past setpoint on energy it can no longer recall.
//!
//! Turning the power down early enough to avoid that is the usual fix, and it
//! is why "no overshoot" normally costs a slow approach.
//!
//! # The idea
//!
//! Control the band, not just the metal.
//!
//! ```text
//!   metal error --> [ outer PI ] --> band setpoint --> [ inner P ] --> duty
//!                        ^                                  ^
//!                   observed metal                     observed band
//! ```
//!
//! The outer loop does not ask for power, it asks for a *band temperature*,
//! clamped to a window around the target. The inner loop delivers that
//! temperature. Because stored energy is `C_band * (band - metal)` and the band
//! setpoint is explicitly clamped, the energy available to coast on is bounded
//! by construction rather than by tuning:
//!
//! - **On the ramp** the outer loop saturates, the band sits at its ceiling, and
//!   the zone heats as fast as the hardware allows.
//! - **On approach** the outer loop walks the band setpoint down through the
//!   target and *below* it, so the inner loop actively dumps the band before the
//!   metal gets there. The coast is spent in advance instead of absorbed.
//!
//! That is what buys fast heating and a clean arrival at the same time, rather
//! than trading one for the other.
//!
//! Neither temperature is measured: both come from a [`BandObserver`] driven by
//! the commanded duty and corrected against the probe.

use std::time::Instant;

use super::{BandObserver, BandObserverGains, BandObserverParams, HeatingStrategy};
use crate::controllers::pid::PidController;

/// Configuration for [`CascadeController`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CascadeParams {
    /// Outer loop: K of band lead demanded per K of metal error. Dimensionless,
    /// and much larger than a duty-producing `kp` — a 20 K error asking for
    /// 200 K of band lead is the normal scale.
    pub kp: f64,
    /// Outer loop integral, in the same units per second. Only has to trim what
    /// the equilibrium-lead feedforward misses.
    pub ki: f64,
    /// How far above its equilibrium lead the band may be driven *over the
    /// metal*, in K.
    ///
    /// **The main knob.** Power into the metal is `g_band_to_metal * lead`, so
    /// this caps the heating rate; stored energy is `C_band * lead`, so it caps
    /// the coast too. Both at once, which is why it is one number and not two.
    /// Set it above the lead the band reaches at full power and it stops binding
    /// — the zone simply ramps at full duty.
    pub band_lead_max_k: f64,
    /// How far *below* the target the band setpoint may be driven, in K, to
    /// dump stored energy ahead of the metal's arrival. Zero disables the
    /// active-dump behaviour and gives up most of the benefit.
    pub band_dump_max_k: f64,
    /// Inner loop: duty per K of band error, on top of the inner feedforward.
    pub k_inner: f64,
    /// Offset applied to the target the outer loop aims the metal at, in K.
    /// Negative deliberately undershoots, for a zone that must never exceed
    /// setpoint at all.
    pub approach_bias_k: f64,
    /// How much of the observer's predicted coast to subtract from the metal
    /// before comparing against the target, in `0..=1`.
    ///
    /// At 1.0 the outer loop regulates *where the metal will end up* if power
    /// stopped now, rather than where it is — so the energy already in the band
    /// is spent against the target instead of on top of it. This is what removes
    /// the last kelvin or two of coast that no choice of gains reaches, because
    /// to a loop watching only the metal, stored energy is invisible until it
    /// arrives.
    ///
    /// Below 1.0 backs the compensation off for a plant whose band capacity is
    /// not trusted; at 0.0 the loop ignores the coast entirely.
    pub coast_compensation: f64,
    pub observer: BandObserverParams,
    pub observer_gains: BandObserverGains,
    pub max_clamp: f64,
}

/// Outer loop on the metal, inner loop on the band.
#[derive(Debug)]
pub struct CascadeController {
    params: CascadeParams,
    observer: BandObserver,
    pid: PidController,
    last_duty: f64,
    band_setpoint_c: f64,
}

impl CascadeController {
    pub const fn new(params: CascadeParams) -> Self {
        Self {
            observer: BandObserver::new(params.observer, params.observer_gains),
            pid: PidController::new(params.kp, params.ki, 0.0),
            params,
            last_duty: 0.0,
            band_setpoint_c: 0.0,
        }
    }

    /// How far the band must lead the metal at equilibrium to keep it at
    /// `metal_c`, in K.
    ///
    /// Every watt the metal loses has to arrive across the band contact, so
    /// this is fixed by physics, not by tuning. Supplying it as feedforward
    /// leaves the outer integral with only the residual, which is what lets a
    /// very small `ki` still settle in minutes.
    fn equilibrium_lead_k(&self, metal_c: f64) -> f64 {
        let p = self.observer.params();
        let metal_loss = p.metal_loss_w_per_k * (metal_c - p.ambient_c);
        (metal_loss / p.band_to_metal_w_per_k).max(0.0)
    }

    /// The band temperature the outer loop is currently asking for, in °C.
    pub const fn band_setpoint_c(&self) -> f64 {
        self.band_setpoint_c
    }

    /// Estimated metal temperature in °C.
    pub const fn metal_c(&self) -> f64 {
        self.observer.metal_c()
    }

    /// Estimated band temperature in °C.
    pub const fn band_c(&self) -> f64 {
        self.observer.band_c()
    }

    /// How far the metal would still coast if power were cut now, in K.
    pub fn coast_k(&self) -> f64 {
        self.observer.coast_k()
    }
}

impl HeatingStrategy for CascadeController {
    fn update(&mut self, measured_c: f64, target_c: f64, now: Instant) -> f64 {
        // The observer is driven by what was actually commanded last tick. At a
        // kHz loop rate against a 500 ms PWM window the commanded duty is the
        // window's mean power, which is what the thermal model wants — better
        // than the instantaneous relay state, which is a square wave the plant
        // never sees as such.
        let metal = self.observer.update(measured_c, self.last_duty, now);
        let band = self.observer.band_c();
        let p = *self.observer.params();

        // ---- outer loop: metal error -> band setpoint ----
        let lead_ff = self.equilibrium_lead_k(target_c);

        // Regulate where the metal is *heading*, not where it is: the band's
        // stored energy is already committed and will land on the metal whatever
        // the relay does from here.
        let predicted_c = self
            .params
            .coast_compensation
            .mul_add(self.observer.coast_k(), metal);
        let error = self.params.approach_bias_k + target_c - predicted_c;

        // The trim's limits are the band window *measured from the feedforward*,
        // so the integral stops accumulating exactly when the band setpoint hits
        // a rail. The lower limit reaches `band_dump_max_k` below the metal:
        // asking for a band colder than the metal is how the loop says "off,
        // now" and discharges the band ahead of arrival.
        let trim = self.pid.update_with_antiwindup(
            error,
            now,
            -(lead_ff + self.params.band_dump_max_k),
            self.params.band_lead_max_k,
        );

        // Referenced to the **metal**, not the target. Stored energy is
        // `C_band * (band - metal)`, so that gap is the quantity the limits have
        // to bound — against the target they would bound nothing during a cold
        // ramp, when the metal is a hundred kelvin below setpoint and the band
        // is free to lead it by the sum of the two.
        self.band_setpoint_c = metal + lead_ff + trim;

        // ---- inner loop: band setpoint -> duty ----
        // Feedforward the power that holds the band where it was asked to be:
        // what crosses into the metal, plus what the band's own skin loses.
        let q_into_metal = p.band_to_metal_w_per_k * (self.band_setpoint_c - metal);
        let band_loss = p.band_loss_w_per_k * (self.band_setpoint_c - p.ambient_c);
        let inner_ff = (q_into_metal + band_loss) / p.rated_w;

        let duty = self
            .params
            .k_inner
            .mul_add(self.band_setpoint_c - band, inner_ff);

        self.last_duty = duty.clamp(0.0, self.params.max_clamp);
        self.last_duty
    }

    fn reset(&mut self) {
        self.observer.reset();
        self.pid.reset();
        self.last_duty = 0.0;
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

    fn observer_params() -> BandObserverParams {
        BandObserverParams {
            band_capacity_j_per_k: 41.0,
            metal_capacity_j_per_k: 2500.0,
            band_to_metal_w_per_k: 4.0,
            metal_loss_w_per_k: 1.6,
            band_loss_w_per_k: 0.35,
            probe_tau_s: 150.0,
            ambient_c: 22.0,
            rated_w: 700.0,
        }
    }

    fn params() -> CascadeParams {
        CascadeParams {
            kp: 12.0,
            ki: 0.002,
            band_lead_max_k: 220.0,
            band_dump_max_k: 30.0,
            k_inner: 0.004,
            approach_bias_k: 0.0,
            coast_compensation: 1.0,
            observer: observer_params(),
            observer_gains: BandObserverGains::default(),
            max_clamp: 1.0,
        }
    }

    /// Closed loop against the same two-state plant the observer models, so the
    /// control law can be checked without dragging in the full extruder rig.
    struct Plant {
        p: BandObserverParams,
        band_c: f64,
        metal_c: f64,
        probe_c: f64,
    }

    impl Plant {
        fn step(&mut self, duty: f64, dt: f64) {
            let q = self.p.band_to_metal_w_per_k * (self.band_c - self.metal_c);
            let band_loss = self.p.band_loss_w_per_k * (self.band_c - self.p.ambient_c);
            let metal_loss = self.p.metal_loss_w_per_k * (self.metal_c - self.p.ambient_c);
            self.band_c +=
                dt / self.p.band_capacity_j_per_k * (duty * self.p.rated_w - q - band_loss);
            self.metal_c += dt / self.p.metal_capacity_j_per_k * (q - metal_loss);
            self.probe_c += dt / self.p.probe_tau_s * (self.metal_c - self.probe_c);
        }
    }

    struct Result {
        peak_metal_c: f64,
        final_metal_c: f64,
        t90_s: Option<f64>,
        peak_band_lead_k: f64,
    }

    fn run(params: CascadeParams, target_c: f64, duration_s: f64) -> Result {
        let dt = 0.05;
        let t0 = Instant::now();
        let mut plant = Plant {
            p: params.observer,
            band_c: 22.0,
            metal_c: 22.0,
            probe_c: 22.0,
        };
        let mut c = CascadeController::new(params);
        let mut out = Result {
            peak_metal_c: f64::NEG_INFINITY,
            final_metal_c: 0.0,
            t90_s: None,
            peak_band_lead_k: 0.0,
        };
        let t90_target = 0.9f64.mul_add(target_c - 22.0, 22.0);

        for i in 0..((duration_s / dt) as u64) {
            let t = i as f64 * dt;
            let duty = c.update(
                (plant.probe_c * 10.0).round() / 10.0,
                target_c,
                t0 + Duration::from_nanos((t * 1e9) as u64),
            );
            plant.step(duty, dt);
            out.peak_metal_c = out.peak_metal_c.max(plant.metal_c);
            out.peak_band_lead_k = out.peak_band_lead_k.max(plant.band_c - plant.metal_c);
            if out.t90_s.is_none() && plant.metal_c >= t90_target {
                out.t90_s = Some(t);
            }
        }
        out.final_metal_c = plant.metal_c;
        out
    }

    /// The whole point: reach setpoint and stay there.
    #[test]
    fn reaches_setpoint_without_overshooting() {
        let r = run(params(), 180.0, 4000.0);
        assert!(
            r.peak_metal_c <= 181.0,
            "peaked at {:.1} C against a 180 C target",
            r.peak_metal_c
        );
        assert!(
            (r.final_metal_c - 180.0).abs() < 1.5,
            "settled at {:.1} C instead of 180",
            r.final_metal_c
        );
    }

    /// And do it quickly — a controller that avoids overshoot by crawling is not
    /// solving the problem.
    #[test]
    fn still_heats_fast() {
        let r = run(params(), 180.0, 4000.0);
        let t90 = r.t90_s.expect("never reached 90 % of setpoint");
        assert!(
            t90 < 900.0,
            "t90 of {t90:.0} s is too slow for a zone that can do it in ~600"
        );
    }

    /// `band_lead_max_k` has to actually bound the band, because that bound is
    /// the entire overshoot argument.
    #[test]
    fn the_band_lead_is_bounded_by_its_limit() {
        let mut p = params();
        p.band_lead_max_k = 60.0;
        let r = run(p, 180.0, 6000.0);

        let equilibrium_lead = 1.6 * (180.0 - 22.0) / 4.0;
        let ceiling = equilibrium_lead + 60.0;
        assert!(
            r.peak_band_lead_k < ceiling + 15.0,
            "band led by {:.0} K against a {ceiling:.0} K ceiling",
            r.peak_band_lead_k
        );
    }

    /// A tighter lead limit must trade speed for gentleness in the expected
    /// direction. If this inverts, the knob does not mean what the docs say.
    #[test]
    fn a_tighter_lead_limit_heats_more_slowly() {
        let fast = run(params(), 180.0, 6000.0);
        let mut slow_params = params();
        slow_params.band_lead_max_k = 40.0;
        let slow = run(slow_params, 180.0, 6000.0);

        assert!(
            slow.t90_s.expect("slow reaches setpoint") > fast.t90_s.expect("fast reaches setpoint"),
            "a 40 K lead limit should be slower than a 220 K one"
        );
    }

    /// Stepping up from an already-hot machine is the second symptom, and it is
    /// a different regime from a cold start: no long saturated ramp to hide
    /// behind.
    #[test]
    fn a_step_up_from_hot_does_not_overshoot() {
        let dt = 0.05;
        let t0 = Instant::now();
        let p = params();
        let mut plant = Plant {
            p: p.observer,
            band_c: 180.0 + 1.6 * (180.0 - 22.0) / 4.0,
            metal_c: 180.0,
            probe_c: 180.0,
        };
        let mut c = CascadeController::new(p);

        let mut peak: f64 = f64::NEG_INFINITY;
        for i in 0..((6000.0 / dt) as u64) {
            let t = i as f64 * dt;
            // Settle at 180 for 500 s, then ask for 200.
            let target = if t < 500.0 { 180.0 } else { 200.0 };
            let duty = c.update(
                (plant.probe_c * 10.0).round() / 10.0,
                target,
                t0 + Duration::from_nanos((t * 1e9) as u64),
            );
            plant.step(duty, dt);
            if t > 500.0 {
                peak = peak.max(plant.metal_c);
            }
        }
        assert!(peak <= 201.0, "step up to 200 peaked at {peak:.1} C");
        assert!(
            (plant.metal_c - 200.0).abs() < 1.5,
            "settled at {:.1} C instead of 200",
            plant.metal_c
        );
    }

    /// A negative bias is the last resort for a zone that must never exceed
    /// setpoint, so it has to move the settling point the way it claims.
    #[test]
    fn approach_bias_shifts_where_the_zone_settles() {
        let mut p = params();
        p.approach_bias_k = -3.0;
        let r = run(p, 180.0, 6000.0);
        assert!(
            r.final_metal_c < 179.0,
            "a -3 K bias should settle below setpoint, settled at {:.1}",
            r.final_metal_c
        );
    }

    #[test]
    fn output_stays_in_range() {
        let t0 = Instant::now();
        let mut c = CascadeController::new(params());
        for (i, measured) in [0.0, 22.0, 180.0, 400.0, -50.0]
            .iter()
            .cycle()
            .take(5_000)
            .enumerate()
        {
            let duty = c.update(*measured, 180.0, t0 + Duration::from_millis(i as u64 * 100));
            assert!(
                (0.0..=1.0).contains(&duty),
                "duty {duty} out of range at reading {measured}"
            );
        }
    }
}
