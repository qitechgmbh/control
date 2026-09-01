//! A two-state estimator for a band-heated mass with a lagging probe.
//!
//! The machine has one measurement — a probe in the metal, itself a first-order
//! lag — and two things worth knowing that it cannot see:
//!
//! - what the **metal** is actually at right now, rather than what the probe has
//!   caught up to, and
//! - what the **band** is at, which nothing measures at all, and which is where
//!   the energy that causes overshoot is sitting.
//!
//! This runs a small model of both, driven by the electrical power the caller is
//! commanding, and corrects it against the probe reading.
//!
//! ```text
//!   P_elec --> [ band ] --g_bs--> [ metal ] --loss--> ambient
//!                                    |
//!                                 tau_probe
//!                                    v
//!                                 reading
//! ```
//!
//! # Why this and not a Kalman filter
//!
//! The covariance propagation would buy nothing here. The plant coefficients are
//! known only to within a factor of a few — they come from a calibration that
//! provably cannot separate them (band capacity trades almost exactly against
//! probe lag) — so a filter tuned on an optimal gain would be trusting numbers
//! that do not deserve it. Fixed Luenberger gains, chosen for a sane correction
//! time constant and verified across a family of plausible plants, are both
//! honest about that and cheap enough to run in a kHz control loop.

use std::time::Instant;

/// Physical coefficients of one heated zone.
///
/// All of these are order-of-magnitude quantities from geometry and
/// calibration. The observer is expected to work with them wrong by a factor of
/// two; that is what the correction term is for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BandObserverParams {
    /// Heat capacity of the band heater itself, in J/K.
    pub band_capacity_j_per_k: f64,
    /// Heat capacity of the metal the band drives, in J/K.
    pub metal_capacity_j_per_k: f64,
    /// Conductance from band into metal, in W/K. With
    /// [`Self::band_capacity_j_per_k`] this sets both how hard the band leads
    /// the metal and how much energy it has stored when the relay opens.
    pub band_to_metal_w_per_k: f64,
    /// Lumped conductance from the metal to everything that is not the band —
    /// ambient, neighbouring cold steel, the gearbox — in W/K.
    pub metal_loss_w_per_k: f64,
    /// Lumped conductance from the band's own outer skin to ambient, in W/K.
    pub band_loss_w_per_k: f64,
    /// The probe's time constant in seconds.
    pub probe_tau_s: f64,
    /// Ambient temperature in °C.
    pub ambient_c: f64,
    /// Electrical power at full duty, in W.
    pub rated_w: f64,
}

/// Luenberger correction gains, as inverse time constants in 1/s.
///
/// Each is "how fast this state is pulled towards agreeing with the probe".
/// Larger tracks modelling error faster and lets sensor noise further in.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BandObserverGains {
    pub metal: f64,
    pub band: f64,
    pub probe: f64,
}

impl Default for BandObserverGains {
    fn default() -> Self {
        // The metal state is corrected on a ~100 s time constant: fast enough to
        // absorb a wrong initial condition or a drifting loss coefficient within
        // one heat-up, slow enough that a 0.1 °C quantisation step moves it by
        // nothing. The band is corrected more gently still, because the probe
        // says very little about it directly. The modelled probe is pulled
        // hardest, since that is the state the innovation actually measures.
        Self {
            metal: 0.010,
            band: 0.004,
            probe: 0.050,
        }
    }
}

/// Estimates band and metal temperature from a lagging probe and the commanded
/// power.
#[derive(Debug, Clone)]
pub struct BandObserver {
    params: BandObserverParams,
    gains: BandObserverGains,

    band_c: f64,
    metal_c: f64,
    /// The model's own prediction of what the probe should be reading. The
    /// difference between this and reality is the only correction signal there
    /// is.
    probe_c: f64,
    last: Option<Instant>,
}

impl BandObserver {
    pub const fn new(params: BandObserverParams, gains: BandObserverGains) -> Self {
        let start = params.ambient_c;
        Self {
            params,
            gains,
            band_c: start,
            metal_c: start,
            probe_c: start,
            last: None,
        }
    }

    /// Advance the estimate by one tick.
    ///
    /// `duty` is what the caller commanded over the interval just ended, in
    /// `0..=1`; `measured_c` is this tick's probe reading. Returns the estimated
    /// metal temperature in °C.
    pub fn update(&mut self, measured_c: f64, duty: f64, now: Instant) -> f64 {
        let Some(last) = self.last else {
            // Nothing is known about the split yet. A cold machine is uniform,
            // and a hot one is closest to uniform at the reading — either way
            // this is the best available guess, and the correction term earns
            // its keep from here.
            self.band_c = measured_c;
            self.metal_c = measured_c;
            self.probe_c = measured_c;
            self.last = Some(now);
            return measured_c;
        };

        let dt = now.duration_since(last).as_secs_f64();
        if dt <= 0.0 {
            return self.metal_c;
        }
        self.last = Some(now);

        let p = &self.params;

        // ---- predict ----
        let q_band_metal = p.band_to_metal_w_per_k * (self.band_c - self.metal_c);
        let band_loss = p.band_loss_w_per_k * (self.band_c - p.ambient_c);
        let metal_loss = p.metal_loss_w_per_k * (self.metal_c - p.ambient_c);

        let band_next = (dt / p.band_capacity_j_per_k).mul_add(
            duty.mul_add(p.rated_w, -q_band_metal) - band_loss,
            self.band_c,
        );
        let metal_next =
            (dt / p.metal_capacity_j_per_k).mul_add(q_band_metal - metal_loss, self.metal_c);
        let probe_next = (dt / p.probe_tau_s).mul_add(self.metal_c - self.probe_c, self.probe_c);

        // ---- correct ----
        // The innovation is the only thing measured. Spreading it over all
        // three states with different weights is what makes the unmeasured band
        // state observable at all.
        let innovation = measured_c - probe_next;
        self.metal_c = (self.gains.metal * dt).mul_add(innovation, metal_next);
        self.band_c = (self.gains.band * dt).mul_add(innovation, band_next);
        self.probe_c = (self.gains.probe * dt).mul_add(innovation, probe_next);

        // The band can lead the metal by a lot but never trails it far: with no
        // power it decays to the metal, it is never a heat sink. Clamping keeps
        // a bad innovation from driving the estimate somewhere unphysical.
        self.band_c = self.band_c.max(self.metal_c - 20.0);

        self.metal_c
    }

    /// Estimated metal temperature in °C.
    pub const fn metal_c(&self) -> f64 {
        self.metal_c
    }

    /// Estimated band temperature in °C — the state no sensor sees, and the one
    /// that has to be bounded to bound overshoot.
    pub const fn band_c(&self) -> f64 {
        self.band_c
    }

    /// Energy currently stored in the band above the metal, in J. This is what
    /// will still be delivered after the relay opens.
    pub fn stored_energy_j(&self) -> f64 {
        (self.band_c - self.metal_c).max(0.0) * self.params.band_capacity_j_per_k
    }

    /// How far the metal would coast if all power were cut now, in K.
    ///
    /// The band's stored energy dumped into the metal, ignoring losses — so a
    /// deliberate over-estimate, which is the safe direction for deciding when
    /// to stop heating.
    pub fn coast_k(&self) -> f64 {
        self.stored_energy_j() / self.params.metal_capacity_j_per_k
    }

    /// Duty that holds the metal at `metal_c` in steady state.
    ///
    /// At equilibrium every watt into the band leaves through the metal's own
    /// losses plus the band's, so this is the feedforward term: the loop only
    /// has to correct what this misses.
    pub fn steady_state_duty(&self, metal_c: f64) -> f64 {
        let p = &self.params;
        let metal_loss = p.metal_loss_w_per_k * (metal_c - p.ambient_c);
        // The band must sit this far above the metal to push `metal_loss`
        // across the contact, and it loses some of its own from up there.
        let band_c = metal_loss / p.band_to_metal_w_per_k + metal_c;
        let band_loss = p.band_loss_w_per_k * (band_c - p.ambient_c);
        ((metal_loss + band_loss) / p.rated_w).clamp(0.0, 1.0)
    }

    pub const fn params(&self) -> &BandObserverParams {
        &self.params
    }

    pub const fn reset(&mut self) {
        self.last = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// Coefficients of one 700 W barrel zone, from the extruder's calibrated
    /// thermal parameters.
    fn barrel_zone() -> BandObserverParams {
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

    /// A plant for the observer to watch, using the same equations. Truth and
    /// model agreeing exactly is the wrong test — see the mismatch cases below —
    /// but it is the right way to check the algebra.
    struct Plant {
        p: BandObserverParams,
        band_c: f64,
        metal_c: f64,
        probe_c: f64,
    }

    impl Plant {
        fn new(p: BandObserverParams, start_c: f64) -> Self {
            Self {
                p,
                band_c: start_c,
                metal_c: start_c,
                probe_c: start_c,
            }
        }

        fn step(&mut self, duty: f64, dt: f64) {
            let q = self.p.band_to_metal_w_per_k * (self.band_c - self.metal_c);
            let band_loss = self.p.band_loss_w_per_k * (self.band_c - self.p.ambient_c);
            let metal_loss = self.p.metal_loss_w_per_k * (self.metal_c - self.p.ambient_c);
            self.band_c +=
                dt / self.p.band_capacity_j_per_k * (duty * self.p.rated_w - q - band_loss);
            self.metal_c += dt / self.p.metal_capacity_j_per_k * (q - metal_loss);
            self.probe_c += dt / self.p.probe_tau_s * (self.metal_c - self.probe_c);
        }

        fn reading(&self) -> f64 {
            (self.probe_c * 10.0).round() / 10.0
        }
    }

    /// Run truth and observer side by side. `truth` may differ from `model`, to
    /// check behaviour under the mismatch that is guaranteed on a real machine.
    fn converge(
        truth: BandObserverParams,
        model: BandObserverParams,
        duty: f64,
        duration_s: f64,
    ) -> (Plant, BandObserver) {
        let dt = 0.05;
        let t0 = Instant::now();
        let mut plant = Plant::new(truth, truth.ambient_c);
        let mut obs = BandObserver::new(model, BandObserverGains::default());

        for i in 0..((duration_s / dt) as u64) {
            let t = i as f64 * dt;
            plant.step(duty, dt);
            obs.update(
                plant.reading(),
                duty,
                t0 + Duration::from_nanos((t * 1e9) as u64),
            );
        }
        (plant, obs)
    }

    /// The headline: while driven, the band runs far above the metal, and the
    /// observer must see that from a probe that shows none of it.
    #[test]
    fn recovers_the_band_metal_split() {
        let p = barrel_zone();
        let (plant, obs) = converge(p, p, 1.0, 1200.0);

        assert!(
            plant.band_c - plant.metal_c > 100.0,
            "test plant is not exercising the split: band leads by only {:.0} K",
            plant.band_c - plant.metal_c
        );
        assert!(
            (obs.band_c() - plant.band_c).abs() < 15.0,
            "band estimate {:.1} vs truth {:.1}",
            obs.band_c(),
            plant.band_c
        );
        assert!(
            (obs.metal_c() - plant.metal_c).abs() < 3.0,
            "metal estimate {:.1} vs truth {:.1}",
            obs.metal_c(),
            plant.metal_c
        );
    }

    /// The estimate has to beat the raw reading, or there is no reason for any
    /// of this to exist.
    #[test]
    fn the_metal_estimate_beats_the_raw_reading() {
        let p = barrel_zone();
        let (plant, obs) = converge(p, p, 1.0, 1200.0);
        let raw_error = (plant.reading() - plant.metal_c).abs();
        let est_error = (obs.metal_c() - plant.metal_c).abs();
        assert!(
            raw_error > 10.0,
            "probe should be badly behind on a ramp, was {raw_error:.1} K"
        );
        assert!(
            est_error < 0.2 * raw_error,
            "estimate error {est_error:.1} K vs raw {raw_error:.1} K"
        );
    }

    /// The coefficients come from a calibration that cannot separate band
    /// capacity from probe lag, so being wrong by 2x in both, in opposite
    /// directions, is a realistic worst case. The estimate must stay usable.
    #[test]
    fn survives_a_two_fold_parameter_error() {
        let truth = barrel_zone();
        let model = BandObserverParams {
            band_capacity_j_per_k: truth.band_capacity_j_per_k * 2.0,
            probe_tau_s: truth.probe_tau_s * 0.5,
            band_to_metal_w_per_k: truth.band_to_metal_w_per_k * 1.5,
            ..truth
        };
        let (plant, obs) = converge(truth, model, 1.0, 1200.0);

        let raw_error = (plant.reading() - plant.metal_c).abs();
        let est_error = (obs.metal_c() - plant.metal_c).abs();
        assert!(
            est_error < 0.5 * raw_error,
            "with 2x-wrong parameters the estimate ({est_error:.1} K) must still \
             beat the raw reading ({raw_error:.1} K)"
        );
    }

    /// With no power the band decays to the metal and the whole thing settles at
    /// ambient. A drift here means a sign error somewhere.
    #[test]
    fn everything_decays_to_ambient_without_power() {
        let p = barrel_zone();
        let (plant, obs) = converge(p, p, 0.0, 6000.0);
        assert!((plant.metal_c - p.ambient_c).abs() < 0.5);
        assert!(
            (obs.metal_c() - p.ambient_c).abs() < 1.0,
            "estimate drifted to {:.2} with no power",
            obs.metal_c()
        );
        assert!(
            (obs.band_c() - p.ambient_c).abs() < 1.0,
            "band estimate drifted to {:.2} with no power",
            obs.band_c()
        );
    }

    /// `steady_state_duty` is the feedforward term, so it has to be the duty
    /// that genuinely holds the plant still.
    #[test]
    fn steady_state_duty_actually_holds_temperature() {
        let p = barrel_zone();
        let obs = BandObserver::new(p, BandObserverGains::default());
        let duty = obs.steady_state_duty(180.0);

        let mut plant = Plant::new(p, 180.0);
        plant.band_c =
            180.0 + p.metal_loss_w_per_k * (180.0 - p.ambient_c) / p.band_to_metal_w_per_k;
        for _ in 0..200_000 {
            plant.step(duty, 0.05);
        }
        assert!(
            (plant.metal_c - 180.0).abs() < 2.0,
            "held at {:.1} C instead of 180 with the feedforward duty {duty:.3}",
            plant.metal_c
        );
    }

    /// The coast prediction is what a controller uses to decide when to stop,
    /// so it must be in the right ballpark and must never be negative.
    #[test]
    fn coast_prediction_is_plausible() {
        let p = barrel_zone();
        let (_, obs) = converge(p, p, 1.0, 1200.0);
        let coast = obs.coast_k();
        // 41 J/K of band about 175 K above 2500 J/K of metal is roughly 3 K.
        assert!(
            (1.0..8.0).contains(&coast),
            "coast prediction {coast:.2} K is not plausible for this zone"
        );

        let cold = BandObserver::new(p, BandObserverGains::default());
        assert!(cold.coast_k() >= 0.0);
    }
}
