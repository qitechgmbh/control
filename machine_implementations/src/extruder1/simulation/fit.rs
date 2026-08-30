//! Calibrating the model against a recorded run.
//!
//! Geometry fixes the thermal masses, but contact conductances, insulation
//! quality and convection cannot be read off a CAD model. Those are fitted here.
//!
//! # Method
//!
//! The recorded run is replayed **open loop**: the model is driven by the duty
//! cycles the real controller commanded, and the simulated sensor temperatures
//! are compared against the recorded ones. Taking the controller out of the loop
//! is deliberate — a closed-loop fit would let plant errors hide behind the
//! controller's own correction, and would confound "the model is wrong" with
//! "the gains are wrong".
//!
//! Parameters are optimised in log space relative to their starting values, so
//! coefficients that differ by four orders of magnitude (`0.07` for insulation
//! conductivity, `1500` for the flange contact) get comparable step sizes.

use super::geometry::Zone;
use super::harness::{SimConfig, ThermalSim};
use super::params::ExtruderThermalParams;

/// The reference heat-up shipped in `data/heatup_2026-02-24.csv`.
const REFERENCE_CSV: &str = include_str!("data/heatup_2026-02-24.csv");

/// A recorded run: temperatures and commanded duties over time.
///
/// All 4-element arrays are indexed by [`Zone::port`].
#[derive(Debug, Clone)]
pub struct RecordedRun {
    pub t_s: Vec<f64>,
    pub temperature_c: Vec<[f64; 4]>,
    pub duty: Vec<[f64; 4]>,
    /// [`Self::duty`] passed through a centred moving average — see
    /// [`DUTY_SMOOTHING_HALF_WIDTH`].
    pub duty_smoothed: Vec<[f64; 4]>,
}

/// Half-width, in samples, of the moving average applied to the recorded duty.
///
/// The log is 1 Hz and captures the PID's *instantaneous* duty at the sample
/// moment. While a zone is ramping that is unambiguous — the duty is pinned at
/// 1.0 or 0.0. Once it is near setpoint the duty chatters (largely from the
/// derivative term firing on sensor quantisation), and 1 Hz sampling of that
/// aliases badly: consecutive samples read 144 W, 0 W, 101 W, 0 W.
///
/// Energy delivered is what the plant integrates, so for replay the mean duty
/// over a few tens of seconds is a far better reconstruction than the raw
/// samples. Without this, an open-loop fit chases sampling noise and reports
/// parameters that contradict the machine's actual behaviour.
pub const DUTY_SMOOTHING_HALF_WIDTH: usize = 15;

/// Centred moving average over `2 * half_width + 1` samples, clamped at the ends.
fn smooth(series: &[[f64; 4]], half_width: usize) -> Vec<[f64; 4]> {
    (0..series.len())
        .map(|i| {
            let lo = i.saturating_sub(half_width);
            let hi = (i + half_width + 1).min(series.len());
            let n = (hi - lo) as f64;
            let mut acc = [0.0; 4];
            for row in &series[lo..hi] {
                for (a, v) in acc.iter_mut().zip(row) {
                    *a += v / n;
                }
            }
            acc
        })
        .collect()
}

impl RecordedRun {
    /// Parse the CSV format written by `scripts/record-extruder.mjs`:
    /// `t_s,T_front,T_middle,T_back,T_nozzle,duty_front,duty_middle,duty_back,duty_nozzle`,
    /// with `#` comment lines allowed anywhere.
    pub fn parse(csv: &str) -> Result<Self, String> {
        let mut t_s = Vec::new();
        let mut temperature_c = Vec::new();
        let mut duty = Vec::new();
        let mut seen_header = false;

        for (lineno, raw) in csv.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if !seen_header {
                seen_header = true;
                if line.starts_with("t_s") {
                    continue;
                }
                return Err(format!("line {}: expected a t_s header row", lineno + 1));
            }
            let f: Vec<f64> = line
                .split(',')
                .map(|v| {
                    v.trim()
                        .parse::<f64>()
                        .map_err(|e| format!("line {}: {e}", lineno + 1))
                })
                .collect::<Result<_, _>>()?;
            if f.len() < 9 {
                return Err(format!(
                    "line {}: expected 9 columns, got {}",
                    lineno + 1,
                    f.len()
                ));
            }
            t_s.push(f[0]);
            temperature_c.push([f[1], f[2], f[3], f[4]]);
            duty.push([f[5], f[6], f[7], f[8]]);
        }

        if t_s.len() < 2 {
            return Err("recording has fewer than two samples".to_owned());
        }
        let duty_smoothed = smooth(&duty, DUTY_SMOOTHING_HALF_WIDTH);
        Ok(Self {
            t_s,
            temperature_c,
            duty,
            duty_smoothed,
        })
    }

    /// The reference run committed alongside this module.
    pub fn reference() -> Self {
        Self::parse(REFERENCE_CSV).expect("the committed reference CSV must parse")
    }

    pub fn duration_s(&self) -> f64 {
        self.t_s.last().copied().unwrap_or(0.0) - self.t_s.first().copied().unwrap_or(0.0)
    }

    /// Starting temperature, averaged over the zones.
    pub fn initial_c(&self) -> f64 {
        self.temperature_c[0].iter().sum::<f64>() / 4.0
    }

    /// Raw commanded duty at time `t`, held from the most recent sample.
    pub fn duty_at(&self, t: f64) -> [f64; 4] {
        self.duty[self.index_at(t)]
    }

    /// Smoothed duty at time `t` — what [`residual`] drives the plant with.
    ///
    /// See [`DUTY_SMOOTHING_HALF_WIDTH`] for why the raw samples are not used.
    pub fn duty_smoothed_at(&self, t: f64) -> [f64; 4] {
        self.duty_smoothed[self.index_at(t)]
    }

    fn index_at(&self, t: f64) -> usize {
        let i = match self.t_s.binary_search_by(|p| p.total_cmp(&t)) {
            Ok(i) => i,
            Err(0) => 0,
            Err(i) => i - 1,
        };
        i.min(self.t_s.len() - 1)
    }

    /// Recorded temperature at time `t`, linearly interpolated.
    pub fn temperature_at(&self, t: f64) -> [f64; 4] {
        let i = match self.t_s.binary_search_by(|p| p.total_cmp(&t)) {
            Ok(i) => return self.temperature_c[i],
            Err(0) => return self.temperature_c[0],
            Err(i) if i >= self.t_s.len() => return self.temperature_c[self.t_s.len() - 1],
            Err(i) => i - 1,
        };
        let (t0, t1) = (self.t_s[i], self.t_s[i + 1]);
        let f = if (t1 - t0).abs() < 1e-12 {
            0.0
        } else {
            (t - t0) / (t1 - t0)
        };
        let (a, b) = (self.temperature_c[i], self.temperature_c[i + 1]);
        [
            f.mul_add(b[0] - a[0], a[0]),
            f.mul_add(b[1] - a[1], a[1]),
            f.mul_add(b[2] - a[2], a[2]),
            f.mul_add(b[3] - a[3], a[3]),
        ]
    }
}

/// Per-zone and overall RMS temperature error of a replay, in K.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FitResidual {
    /// RMS error per zone, indexed by [`Zone::port`].
    pub per_zone_k: [f64; 4],
    /// RMS over all zones and samples.
    pub overall_k: f64,
}

/// Plant integration step used while fitting, in seconds.
///
/// The replay has no PWM to resolve — the duty comes from a 1 Hz log — so a much
/// coarser step than [`super::harness::DT_PLANT_S`] is accurate here, and it makes
/// each optimiser evaluation cheap. Still far inside the network's stability
/// limit.
pub const FIT_DT_S: f64 = 0.25;

/// Replay `run` open loop through a model built from `params` and score it.
pub fn residual(params: &ExtruderThermalParams, run: &RecordedRun) -> FitResidual {
    let config = SimConfig {
        dt_plant_s: FIT_DT_S,
        record_period_s: 1.0,
        ..SimConfig::default()
    };
    let mut sim = ThermalSim::new(params.clone(), config);
    let trace = sim.run_open_loop(run.initial_c(), run.duration_s(), &|t| {
        run.duty_smoothed_at(t)
    });

    let mut sq = [0.0f64; 4];
    let mut n = 0usize;
    for s in &trace.samples {
        let measured = run.temperature_at(s.t_s);
        for zone in Zone::ALL {
            let p = zone.port();
            let d = s.sensor_c[p] - measured[p];
            sq[p] += d * d;
        }
        n += 1;
    }
    let n = n.max(1) as f64;
    let per_zone_k = [
        (sq[0] / n).sqrt(),
        (sq[1] / n).sqrt(),
        (sq[2] / n).sqrt(),
        (sq[3] / n).sqrt(),
    ];
    FitResidual {
        overall_k: (sq.iter().sum::<f64>() / (n * 4.0)).sqrt(),
        per_zone_k,
    }
}

/// Outcome of a calibration.
#[derive(Debug, Clone)]
pub struct FitOutcome {
    pub params: ExtruderThermalParams,
    pub residual: FitResidual,
    pub starting_residual: FitResidual,
    pub evaluations: usize,
}

/// Fit the free coefficients of `start` to `run` with Nelder-Mead.
///
/// `max_evaluations` bounds the work; a few hundred is usually enough for eight
/// parameters. Returns the best parameters found, which is never worse than
/// `start`.
pub fn fit(run: &RecordedRun, start: &ExtruderThermalParams, max_evaluations: usize) -> FitOutcome {
    let base = start.to_vector();
    let n = base.len();
    let mut evaluations = 0usize;

    // Optimise log-multipliers so every coefficient gets a comparable step.
    let to_params = |x: &[f64]| {
        let mut p = start.clone();
        let v: Vec<f64> = base.iter().zip(x).map(|(b, xi)| b * xi.exp()).collect();
        p.apply_vector(&v);
        p
    };

    let objective = |x: &[f64], evaluations: &mut usize| {
        *evaluations += 1;
        residual(&to_params(x), run).overall_k
    };

    // A simplex around `centre`, one step per axis. The step is in log space, so
    // 0.6 is a factor of ~1.8 on each coefficient — deliberately coarse, because
    // the starting guess can be an order of magnitude off.
    let build_simplex = |centre: &[f64], step: f64| {
        let mut s = Vec::with_capacity(n + 1);
        s.push(centre.to_vec());
        for i in 0..n {
            let mut v = centre.to_vec();
            v[i] += step;
            s.push(v);
        }
        s
    };

    let mut step = 0.6;
    let mut simplex = build_simplex(&vec![0.0; n], step);
    let mut values: Vec<f64> = simplex
        .iter()
        .map(|s| objective(s, &mut evaluations))
        .collect();
    let mut best_ever = simplex[0].clone();
    let mut best_ever_value = f64::INFINITY;

    let (alpha, gamma, rho, sigma) = (1.0_f64, 2.0_f64, 0.5_f64, 0.5_f64);

    while evaluations < max_evaluations {
        // Order by objective value.
        let mut order: Vec<usize> = (0..simplex.len()).collect();
        order.sort_by(|&a, &b| values[a].total_cmp(&values[b]));
        simplex = order.iter().map(|&i| simplex[i].clone()).collect();
        values = order.iter().map(|&i| values[i]).collect();

        if values[0] < best_ever_value {
            best_ever_value = values[0];
            best_ever.clone_from(&simplex[0]);
        }

        // When the simplex collapses, restart it around the best point with a
        // smaller step instead of stopping. A single Nelder-Mead descent in nine
        // correlated dimensions reliably parks in a corner; restarting is what
        // makes the difference between a degenerate answer and a usable one.
        if (values[values.len() - 1] - values[0]).abs() < 1e-3 * values[0].max(1e-6) {
            if step < 0.05 {
                break;
            }
            step *= 0.4;
            simplex = build_simplex(&best_ever.clone(), step);
            values = simplex
                .iter()
                .map(|s| objective(s, &mut evaluations))
                .collect();
            continue;
        }

        // Centroid of everything but the worst point.
        let worst = simplex.len() - 1;
        let mut centroid = vec![0.0; n];
        for s in &simplex[..worst] {
            for (c, v) in centroid.iter_mut().zip(s) {
                *c += v / worst as f64;
            }
        }

        let reflect: Vec<f64> = centroid
            .iter()
            .zip(&simplex[worst])
            .map(|(c, w)| alpha.mul_add(c - w, *c))
            .collect();
        let f_reflect = objective(&reflect, &mut evaluations);

        if f_reflect < values[0] {
            let expand: Vec<f64> = centroid
                .iter()
                .zip(&reflect)
                .map(|(c, r)| gamma.mul_add(r - c, *c))
                .collect();
            let f_expand = objective(&expand, &mut evaluations);
            if f_expand < f_reflect {
                simplex[worst] = expand;
                values[worst] = f_expand;
            } else {
                simplex[worst] = reflect;
                values[worst] = f_reflect;
            }
        } else if f_reflect < values[worst - 1] {
            simplex[worst] = reflect;
            values[worst] = f_reflect;
        } else {
            let contract: Vec<f64> = centroid
                .iter()
                .zip(&simplex[worst])
                .map(|(c, w)| rho.mul_add(w - c, *c))
                .collect();
            let f_contract = objective(&contract, &mut evaluations);
            if f_contract < values[worst] {
                simplex[worst] = contract;
                values[worst] = f_contract;
            } else {
                // Shrink towards the best vertex.
                let best = simplex[0].clone();
                for i in 1..simplex.len() {
                    simplex[i] = best
                        .iter()
                        .zip(&simplex[i])
                        .map(|(b, s)| sigma.mul_add(s - b, *b))
                        .collect();
                    values[i] = objective(&simplex[i], &mut evaluations);
                }
            }
        }
    }

    let best = simplex
        .iter()
        .zip(&values)
        .min_by(|a, b| a.1.total_cmp(b.1))
        .filter(|(_, v)| **v <= best_ever_value)
        .map_or(best_ever, |(s, _)| s.clone());

    let params = to_params(&best);
    FitOutcome {
        residual: residual(&params, run),
        starting_residual: residual(start, run),
        params,
        evaluations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_recording_parses() {
        let run = RecordedRun::reference();
        assert!(run.t_s.len() > 3000, "got {} samples", run.t_s.len());
        assert!((run.duration_s() - 3263.0).abs() < 5.0);
        // Starts cold, all four zones near room temperature.
        for v in run.temperature_c[0] {
            assert!((20.0..25.0).contains(&v), "cold start reads {v}");
        }
        // Opens at full duty on the barrel zones and the nozzle's 0.95 clamp.
        assert!((run.duty[0][Zone::Front.port()] - 1.0).abs() < 1e-6);
        assert!((run.duty[0][Zone::Nozzle.port()] - 0.95).abs() < 1e-6);
    }

    #[test]
    fn recorded_run_reproduces_the_documented_symptoms() {
        let run = RecordedRun::reference();
        let peak = |p: usize| {
            run.temperature_c
                .iter()
                .map(|r| r[p])
                .fold(f64::NEG_INFINITY, f64::max)
        };
        let (front, middle, back, nozzle) = (
            peak(Zone::Front.port()),
            peak(Zone::Middle.port()),
            peak(Zone::Back.port()),
            peak(Zone::Nozzle.port()),
        );
        // Middle overshoots hardest, by a wide margin.
        assert!(
            middle > front && middle > back,
            "middle {middle} not highest"
        );
        assert!(
            middle - 180.0 > 25.0,
            "middle overshoot only {}",
            middle - 180.0
        );
        // The nozzle never reaches its 175 C setpoint.
        assert!(nozzle < 175.0, "nozzle reached {nozzle}");
    }

    #[test]
    fn duty_lookup_holds_between_samples() {
        let run = RecordedRun::reference();
        let a = run.duty_at(100.0);
        let b = run.duty_at(100.4);
        assert_eq!(a, b, "duty must be held, not interpolated");
    }
}
