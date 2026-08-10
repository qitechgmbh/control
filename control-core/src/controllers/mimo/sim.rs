//! Simulated plants and end-to-end tests for the MIMO pipeline.
//!
//! A real identification campaign ties up a machine for hours, so the maths has to be debuggable
//! without one. Two plants are modelled here:
//!
//! * [`FopdtBank`] — a superposition of independent first-order-plus-dead-time channels. This is
//!   exactly the model class the identifier fits, so ground truth is known to the last digit and
//!   any error is the estimator's.
//! * [`CoupledBarrel`] — four thermal masses conducting to their neighbours and losing heat to
//!   ambient. Not in the model class (it is a coupled fourth-order system), so it tests whether an
//!   FOPDT approximation of a real barrel is good enough to control from. Its steady-state gain
//!   matrix is still available in closed form, which is what the accuracy assertions use.

use super::controller::MimoPidController;
use super::identify::{MimoIdentifyConfig, MimoStepIdentifier};
use super::matrix::{self, Mat};
use super::synth_decoupler::DecouplerImc;
use super::{MimoModel, MimoSynthesis, ZONE_COUNT};
use std::time::{Duration, Instant};

/// Deterministic pseudo-random noise. A fixed sequence keeps the tests reproducible; `rand` is not
/// a dependency of this crate and a failing test that cannot be re-run is worse than no test.
struct Noise(u64);

impl Noise {
    fn new(seed: u64) -> Self {
        Self(seed | 1)
    }

    /// Uniform in `[-amplitude, amplitude]`.
    fn next(&mut self, amplitude: f64) -> f64 {
        // xorshift64*
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        let x = self.0.wrapping_mul(0x2545_F491_4F6C_DD1D);
        let unit = (x >> 11) as f64 / (1u64 << 53) as f64; // [0, 1)
        (unit * 2.0 - 1.0) * amplitude
    }
}

/// A bank of independent FOPDT channels: `y_i = sum_j gp_ij * fopdt(tau_ij, theta_ij) * u_j`.
pub struct FopdtBank {
    gp: Mat<ZONE_COUNT>,
    tau: Mat<ZONE_COUNT>,
    theta: Mat<ZONE_COUNT>,
    /// Per-channel lag state, `state[output][input]`.
    state: Mat<ZONE_COUNT>,
    /// Delay lines for each input, one per (output, input) pair.
    delay: Vec<Vec<f64>>,
    ambient: f64,
    dt: f64,
}

impl FopdtBank {
    pub fn new(
        gp: Mat<ZONE_COUNT>,
        tau: Mat<ZONE_COUNT>,
        theta: Mat<ZONE_COUNT>,
        ambient: f64,
        dt: f64,
    ) -> Self {
        let mut delay = Vec::new();
        for i in 0..ZONE_COUNT {
            for j in 0..ZONE_COUNT {
                let n = ((theta[i][j] / dt).round() as usize).max(1);
                delay.push(vec![0.0; n]);
            }
        }
        Self {
            gp,
            tau,
            theta,
            state: matrix::zeros(),
            delay,
            ambient,
            dt,
        }
    }

    pub fn step(&mut self, u: &[f64; ZONE_COUNT]) -> [f64; ZONE_COUNT] {
        for i in 0..ZONE_COUNT {
            for j in 0..ZONE_COUNT {
                let line = &mut self.delay[i * ZONE_COUNT + j];
                line.rotate_left(1);
                let n = line.len();
                line[n - 1] = u[j];
                let delayed = line[0];

                let tau = self.tau[i][j].max(self.dt);
                let target = self.gp[i][j] * delayed;
                self.state[i][j] += (target - self.state[i][j]) * self.dt / tau;
            }
        }
        let mut y = [self.ambient; ZONE_COUNT];
        for i in 0..ZONE_COUNT {
            for j in 0..ZONE_COUNT {
                y[i] += self.state[i][j];
            }
        }
        y
    }

    /// Drive to steady state at a constant input.
    pub fn settle(&mut self, u: &[f64; ZONE_COUNT]) -> [f64; ZONE_COUNT] {
        let worst_tau = matrix::max_abs(&self.tau);
        let worst_theta = matrix::max_abs(&self.theta);
        let ticks = (((10.0 * worst_tau + worst_theta) / self.dt).ceil() as usize).max(100);
        let mut y = [0.0; ZONE_COUNT];
        for _ in 0..ticks {
            y = self.step(u);
        }
        y
    }
}

/// Four thermal masses in the physical chain Nozzle-Front-Middle-Back.
///
/// `dT_i/dt = a_i * u_i(t - theta) - loss * (T_i - T_amb) + cond * sum_neighbours (T_j - T_i)`
pub struct CoupledBarrel {
    /// Heating authority, degrees per second at full duty.
    a: [f64; ZONE_COUNT],
    loss: f64,
    cond: f64,
    ambient: f64,
    dt: f64,
    temperature: [f64; ZONE_COUNT],
    /// Transport delay from commanding a duty to heat arriving at the sensor.
    delay: Vec<Vec<f64>>,
    noise: Noise,
    noise_amplitude: f64,
}

impl CoupledBarrel {
    pub fn new(a: [f64; ZONE_COUNT], loss: f64, cond: f64, dead_time: f64, dt: f64) -> Self {
        let ambient = 20.0;
        let n = ((dead_time / dt).round() as usize).max(1);
        Self {
            a,
            loss,
            cond,
            ambient,
            dt,
            temperature: [ambient; ZONE_COUNT],
            delay: (0..ZONE_COUNT).map(|_| vec![0.0; n]).collect(),
            noise: Noise::new(0x5EED),
            noise_amplitude: 0.0,
        }
    }

    pub fn with_noise(mut self, amplitude: f64) -> Self {
        self.noise_amplitude = amplitude;
        self
    }

    /// The conduction/loss matrix `L`, where steady state satisfies `L * (T - T_amb) = A * u`.
    fn loss_matrix(&self) -> Mat<ZONE_COUNT> {
        let mut l = matrix::zeros::<ZONE_COUNT>();
        for i in 0..ZONE_COUNT {
            let neighbours = if i == 0 || i == ZONE_COUNT - 1 { 1 } else { 2 };
            l[i][i] = self.loss + self.cond * neighbours as f64;
            if i > 0 {
                l[i][i - 1] = -self.cond;
            }
            if i + 1 < ZONE_COUNT {
                l[i][i + 1] = -self.cond;
            }
        }
        l
    }

    /// Exact steady-state gain matrix, `G(0) = inv(L) * diag(a)`. This is the ground truth the
    /// identifier is scored against.
    pub fn true_dc_gain(&self) -> Mat<ZONE_COUNT> {
        let l_inv = matrix::inverse(&self.loss_matrix()).expect("L is diagonally dominant");
        let mut g = matrix::zeros::<ZONE_COUNT>();
        for i in 0..ZONE_COUNT {
            for j in 0..ZONE_COUNT {
                g[i][j] = l_inv[i][j] * self.a[j];
            }
        }
        g
    }

    pub fn step(&mut self, u: &[f64; ZONE_COUNT]) -> [f64; ZONE_COUNT] {
        let mut delayed = [0.0; ZONE_COUNT];
        for z in 0..ZONE_COUNT {
            let line = &mut self.delay[z];
            line.rotate_left(1);
            let n = line.len();
            line[n - 1] = u[z];
            delayed[z] = line[0];
        }

        let t = self.temperature;
        for i in 0..ZONE_COUNT {
            let mut conduction = 0.0;
            if i > 0 {
                conduction += self.cond * (t[i - 1] - t[i]);
            }
            if i + 1 < ZONE_COUNT {
                conduction += self.cond * (t[i + 1] - t[i]);
            }
            let d = self.a[i] * delayed[i] - self.loss * (t[i] - self.ambient) + conduction;
            self.temperature[i] += d * self.dt;
        }

        let mut out = self.temperature;
        if self.noise_amplitude > 0.0 {
            for slot in &mut out {
                *slot += self.noise.next(self.noise_amplitude);
            }
        }
        out
    }

    pub fn settle(&mut self, u: &[f64; ZONE_COUNT]) -> [f64; ZONE_COUNT] {
        let mut y = [0.0; ZONE_COUNT];
        for _ in 0..200_000 {
            y = self.step(u);
        }
        y
    }
}

/// Test-sized configuration: same structure as production, shorter windows so a campaign
/// simulates in milliseconds.
fn test_config() -> MimoIdentifyConfig {
    MimoIdentifyConfig {
        step_duty: 0.10,
        max_duty: [1.0; ZONE_COUNT],
        max_rise_celsius: 40.0,
        max_total_rise_celsius: 200.0,
        sample_period: Duration::from_secs(1),
        steady_window: Duration::from_secs(60),
        steady_slope_c_per_min: 0.05,
        steady_band_celsius: 0.5,
        setpoint_band_celsius: 5.0,
        dead_time_threshold_celsius: 0.2,
        waiting_timeout: Duration::from_secs(600),
        baseline_timeout: Duration::from_secs(3600),
        column_timeout: Duration::from_secs(20_000),
        max_duration: Duration::from_secs(200_000),
    }
}

/// Duty the simulated zones hold at their operating point.
///
/// The operating point is derived by settling *from* this duty rather than by asking for a target
/// temperature, which keeps it inside `[0, 1]` by construction. Asking for an arbitrary
/// temperature can demand a duty above full output, and the campaign's step would then clamp into
/// a large negative jump instead of the intended positive one.
const HOLD_DUTY: [f64; ZONE_COUNT] = [0.55; ZONE_COUNT];

/// Heating authority, degrees per second at full duty.
///
/// Sized with the loss terms below so the simulated barrel behaves like the real one: a time
/// constant of a few hundred seconds, a process gain near 300 degrees per unit duty, and a hold
/// duty around half output at production temperature. That matters because the steady-state
/// thresholds in [`MimoIdentifyConfig`] are *absolute* slopes — a plant an order of magnitude
/// slower and weaker than the real machine reads as settled while a third of its response is
/// still to come, and the truncated plateau then mis-brackets the fitter's search for `tau`.
const HEATER_AUTHORITY: f64 = 1.0;

/// Loss and conduction pairs, in units of inverse seconds.
///
/// Conduction is what couples the zones; the ratio to `loss` is what sets how strongly.
const WEAK_COUPLING: (f64, f64) = (0.0030, 0.0002);
const MEDIUM_COUPLING: (f64, f64) = (0.0025, 0.0008);
const STRONG_COUPLING: (f64, f64) = (0.0018, 0.0015);

fn barrel(coupling: (f64, f64)) -> CoupledBarrel {
    CoupledBarrel::new(
        [HEATER_AUTHORITY; ZONE_COUNT],
        coupling.0,
        coupling.1,
        15.0,
        1.0,
    )
}

/// Run a full campaign against `CoupledBarrel`, returning the identified model.
fn run_campaign(plant: &mut CoupledBarrel, config: MimoIdentifyConfig) -> MimoModel {
    let hold = HOLD_DUTY;
    let settled = plant.settle(&hold);

    let mut ident = MimoStepIdentifier::new();
    let t0 = Instant::now();
    ident.start(config, settled, t0).expect("valid config");

    let mut pv = settled;
    for tick in 0..400_000 {
        let t = t0 + Duration::from_secs(tick);
        let cmd = ident.update(pv, hold, t);
        if !ident.is_running() {
            break;
        }
        pv = plant.step(&cmd.unwrap_or(hold));
    }

    assert!(
        ident.is_completed(),
        "campaign ended in {:?}: {:?}",
        ident.phase_enum(),
        ident.failure_reason()
    );
    ident.take_result().expect("completed run has a result")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn banded_gp(diag: f64, near: f64, far: f64) -> Mat<ZONE_COUNT> {
        let mut g = matrix::zeros::<ZONE_COUNT>();
        for i in 0..ZONE_COUNT {
            for j in 0..ZONE_COUNT {
                g[i][j] = match i.abs_diff(j) {
                    0 => diag,
                    1 => near,
                    _ => far,
                };
            }
        }
        g
    }

    fn uniform(v: f64) -> Mat<ZONE_COUNT> {
        [[v; ZONE_COUNT]; ZONE_COUNT]
    }

    /// The estimator against a plant that is exactly in its model class. Any error here is the
    /// fitter's own, with no model mismatch to hide behind.
    #[test]
    fn identifies_synthetic_fopdt_bank() {
        let gp = banded_gp(60.0, 15.0, 3.0);
        let tau = uniform(200.0);
        let theta = uniform(20.0);

        let mut plant = FopdtBank::new(gp, tau, theta, 20.0, 1.0);
        let hold = [0.3; ZONE_COUNT];
        let settled = plant.settle(&hold);

        let mut ident = MimoStepIdentifier::new();
        let t0 = Instant::now();
        ident.start(test_config(), settled, t0).unwrap();

        let mut pv = settled;
        for tick in 0..400_000 {
            let t = t0 + Duration::from_secs(tick);
            let cmd = ident.update(pv, hold, t);
            if !ident.is_running() {
                break;
            }
            pv = plant.step(&cmd.unwrap_or(hold));
        }

        assert!(
            ident.is_completed(),
            "ended in {:?}: {:?}",
            ident.phase_enum(),
            ident.failure_reason()
        );
        let model = ident.result().unwrap();

        for i in 0..ZONE_COUNT {
            for j in 0..ZONE_COUNT {
                let got = model.g[i][j].gp;
                let want = gp[i][j];
                // Tolerance scales to the driven zone's own gain, not to the entry. A small cross
                // term is measured on top of whatever the previous column is still settling out
                // of, so its *absolute* error is what the staircase bounds — and its absolute
                // error relative to the column's dominant gain is also what the decoupler cares
                // about, since that is the ratio it inverts.
                let tol = 0.05 * gp[j][j];
                assert!(
                    (got - want).abs() <= tol,
                    "gp[{i}][{j}] = {got}, expected {want} (tol {tol})"
                );
                // Time constant and dead time are recovered from the same fit; allow more slack
                // since they trade off against each other far more than the gain does.
                let tau_got = model.g[i][j].tau;
                assert!(
                    (tau_got - 200.0).abs() < 60.0,
                    "tau[{i}][{j}] = {tau_got}, expected ~200"
                );
            }
        }
    }

    /// The headline accuracy check: a physically-modelled barrel, scored against its closed-form
    /// steady-state gain matrix.
    #[test]
    fn staircase_recovers_the_true_dc_gain_matrix() {
        let mut plant = barrel(MEDIUM_COUPLING);
        let truth = plant.true_dc_gain();
        let model = run_campaign(&mut plant, test_config());
        let got = model.dc_gain();

        for i in 0..ZONE_COUNT {
            for j in 0..ZONE_COUNT {
                let tol = 0.10 * truth[j][j]; // scaled to the driven zone's own gain
                assert!(
                    (got[i][j] - truth[i][j]).abs() <= tol,
                    "G[{i}][{j}] = {}, expected {} (tol {tol})",
                    got[i][j],
                    truth[i][j]
                );
            }
        }
    }

    /// Superposition check for the staircase shortcut.
    ///
    /// Columns are measured one after another without ever cooling back to the starting state, so
    /// the last column runs on top of three columns' worth of accumulated rise. If that residual
    /// corrupted the measurement, column 3 would be measurably worse than column 0. Requiring it
    /// to be no worse is what justifies skipping the return-to-baseline leg — and roughly half the
    /// campaign duration with it.
    #[test]
    fn staircase_does_not_degrade_later_columns() {
        let mut plant = barrel(MEDIUM_COUPLING);
        let truth = plant.true_dc_gain();
        let model = run_campaign(&mut plant, test_config());
        let got = model.dc_gain();

        let column_error = |j: usize| {
            (0..ZONE_COUNT)
                .map(|i| (got[i][j] - truth[i][j]).abs())
                .fold(0.0_f64, f64::max)
        };

        let first = column_error(0);
        let last = column_error(ZONE_COUNT - 1);
        assert!(
            last <= first.max(0.05 * truth[0][0]) * 1.5,
            "column 3 error {last} is much worse than column 0 error {first}, so staircase \
             residuals are corrupting later columns"
        );
    }

    /// Physical ordering makes the matrix banded. This is a free correctness check on a real
    /// campaign: heat from a distant zone must arrive weaker and later.
    #[test]
    fn identified_matrix_is_banded() {
        let mut plant = barrel(MEDIUM_COUPLING);
        let model = run_campaign(&mut plant, test_config());

        // Compared distance-2-or-more against distance-1 rather than asserting a fully monotone
        // ordering. The two end zones lose heat through one neighbour instead of two, so they run
        // hotter for the same input and can legitimately out-respond an interior zone that sits
        // closer to the driven one. Banding past the immediate neighbour is the part that is
        // genuinely a property of conduction.
        for j in 0..ZONE_COUNT {
            let nearest = (0..ZONE_COUNT)
                .filter(|i| i.abs_diff(j) == 1)
                .map(|i| model.g[i][j].gp)
                .fold(f64::NEG_INFINITY, f64::max);
            for i in (0..ZONE_COUNT).filter(|i| i.abs_diff(j) >= 2) {
                assert!(
                    model.g[i][j].gp < nearest,
                    "column {j}: distant zone {i} responded {}, at least as much as the \
                     strongest immediate neighbour ({nearest})",
                    model.g[i][j].gp
                );
            }
        }

        // Heat from further away must also arrive later.
        //
        // Measured as `theta + tau`, not `theta` alone. Fitting a first-order-plus-dead-time model
        // to the smooth S-curve a distance-3 path produces is genuinely ambiguous between the two
        // parameters — the estimator here puts almost all of that lag into `tau` and leaves
        // `theta` near zero, which describes the curve perfectly well but makes `theta` on its own
        // meaningless to compare. Their sum is the total apparent lag and is well defined however
        // the fit splits it.
        let apparent_lag = |i: usize, j: usize| model.g[i][j].theta + model.g[i][j].tau;
        for j in 0..ZONE_COUNT {
            let near = (0..ZONE_COUNT)
                .filter(|i| i.abs_diff(j) == 1)
                .map(|i| apparent_lag(i, j))
                .fold(f64::INFINITY, f64::min);
            for i in (0..ZONE_COUNT).filter(|i| i.abs_diff(j) >= 2) {
                assert!(
                    apparent_lag(i, j) > near,
                    "column {j}: distant zone {i} responded sooner ({}) than the nearest \
                     neighbour ({near})",
                    apparent_lag(i, j)
                );
            }
        }
    }

    #[test]
    fn rga_tracks_conduction_strength() {
        let weak = run_campaign(&mut barrel(WEAK_COUPLING), test_config());
        let strong = run_campaign(&mut barrel(STRONG_COUPLING), test_config());

        assert!(
            weak.max_rga_deviation() < strong.max_rga_deviation(),
            "weak conduction RGA deviation {} should be below strong {}",
            weak.max_rga_deviation(),
            strong.max_rga_deviation()
        );
        assert!(
            weak.max_coupling_ratio() < strong.max_coupling_ratio(),
            "weak coupling ratio {} should be below strong {}",
            weak.max_coupling_ratio(),
            strong.max_coupling_ratio()
        );
    }

    #[test]
    fn survives_measurement_noise() {
        let mut plant = barrel(MEDIUM_COUPLING).with_noise(0.05);
        let truth = plant.true_dc_gain();
        let mut config = test_config();
        // Noise widens the steady band that can realistically be achieved.
        config.steady_band_celsius = 0.5;
        let model = run_campaign(&mut plant, config);
        let got = model.dc_gain();

        for i in 0..ZONE_COUNT {
            for j in 0..ZONE_COUNT {
                assert!(
                    (got[i][j] - truth[i][j]).abs() <= 0.15 * truth[j][j],
                    "G[{i}][{j}] = {} against {} under noise",
                    got[i][j],
                    truth[i][j]
                );
            }
        }
    }

    /// The actuator-ownership contract: the identifier must never command anything outside a
    /// driving phase, so releasing the zones is structural rather than a step an abort could skip.
    #[test]
    fn identifier_never_commands_outside_driving_phase() {
        let mut plant = barrel(MEDIUM_COUPLING);
        let hold = HOLD_DUTY;
        let settled = plant.settle(&hold);

        let mut ident = MimoStepIdentifier::new();
        let t0 = Instant::now();

        // Before starting.
        assert!(ident.update(settled, hold, t0).is_none());
        assert!(ident.command().is_none());

        ident.start(test_config(), settled, t0).unwrap();
        let mut pv = settled;
        for tick in 0..400_000 {
            let t = t0 + Duration::from_secs(tick);
            let cmd = ident.update(pv, hold, t);
            assert_eq!(
                cmd.is_some(),
                ident.phase_enum().is_driving(),
                "command presence disagrees with phase {:?}",
                ident.phase_enum()
            );
            if !ident.is_running() {
                break;
            }
            pv = plant.step(&cmd.unwrap_or(hold));
        }

        // After completing.
        assert!(ident.command().is_none());
        assert!(ident.update(pv, hold, t0).is_none());
    }

    #[test]
    fn abort_releases_every_zone() {
        let mut plant = barrel(MEDIUM_COUPLING);
        let hold = HOLD_DUTY;
        let settled = plant.settle(&hold);

        let mut ident = MimoStepIdentifier::new();
        let t0 = Instant::now();
        ident.start(test_config(), settled, t0).unwrap();

        let mut pv = settled;
        for tick in 0..2000 {
            let t = t0 + Duration::from_secs(tick);
            let cmd = ident.update(pv, hold, t);
            pv = plant.step(&cmd.unwrap_or(hold));
        }
        ident.abort("operator stopped the run", t0 + Duration::from_secs(2000));

        assert!(ident.is_failed());
        assert!(ident.command().is_none());
        assert_eq!(ident.failure_reason(), Some("operator stopped the run"));
    }

    #[test]
    fn a_runaway_zone_aborts_the_campaign() {
        // One zone with far too much heating authority for the configured rise limit.
        let mut plant = CoupledBarrel::new(
            [
                HEATER_AUTHORITY,
                HEATER_AUTHORITY,
                HEATER_AUTHORITY,
                20.0 * HEATER_AUTHORITY,
            ],
            MEDIUM_COUPLING.0,
            MEDIUM_COUPLING.1,
            15.0,
            1.0,
        );
        let hold = HOLD_DUTY;
        let settled = plant.settle(&hold);

        let mut config = test_config();
        config.max_rise_celsius = 8.0;

        let mut ident = MimoStepIdentifier::new();
        let t0 = Instant::now();
        ident.start(config, settled, t0).unwrap();

        let mut pv = settled;
        for tick in 0..400_000 {
            let t = t0 + Duration::from_secs(tick);
            let cmd = ident.update(pv, hold, t);
            if !ident.is_running() {
                break;
            }
            pv = plant.step(&cmd.unwrap_or(hold));
        }
        assert!(ident.is_failed(), "expected the rise limit to fire");
        assert!(ident.command().is_none());
    }

    #[test]
    fn start_rejects_an_impossible_configuration() {
        let mut ident = MimoStepIdentifier::new();
        let t0 = Instant::now();

        let mut bad = test_config();
        bad.step_duty = 0.0;
        assert!(ident.start(bad, [200.0; ZONE_COUNT], t0).is_err());

        let mut bad = test_config();
        bad.step_duty = 0.5;
        bad.max_duty[3] = 0.2;
        assert!(ident.start(bad, [200.0; ZONE_COUNT], t0).is_err());

        // A valid one now works, and a second start is refused while it runs.
        assert!(ident.start(test_config(), [200.0; ZONE_COUNT], t0).is_ok());
        assert!(ident.start(test_config(), [200.0; ZONE_COUNT], t0).is_err());
    }

    /// Closed-loop payoff: the reason the whole pipeline exists.
    ///
    /// Steps one zone's setpoint and measures how far the *other* zones are dragged off theirs.
    /// The MIMO controller knows the coupling and pre-compensates for it; the decentralized bank
    /// can only react after the disturbance has already arrived.
    #[test]
    fn mimo_beats_decentralized_on_interaction() {
        use crate::controllers::pid::PidController;

        // Strong conduction: the regime the diagnostics flag as needing MIMO. On a weakly coupled
        // barrel the RGA diagonal already sits near 1, decentralized control is close to correct,
        // and there is nothing for a decoupler to win.
        let make_plant = || barrel(STRONG_COUPLING);

        let mut ident_plant = make_plant();
        let model = run_campaign(&mut ident_plant, test_config());
        // Sanity: the plant must actually be coupled enough for this comparison to mean anything.
        assert!(
            model.max_rga_deviation() > 0.25,
            "test plant is too weakly coupled ({}) to distinguish the two schemes",
            model.max_rga_deviation()
        );

        let synth = DecouplerImc {
            lambda_factor: 0.5,
            ..Default::default()
        };
        let gains = synth
            .synthesize(&model)
            .expect("a conducting barrel should be synthesizable");

        // Setpoints are taken from where the plant actually settles at the hold duty, so both
        // controllers start at zero error and the only thing either has to reject is the
        // interaction. Naming target temperatures independently of the plant would start both
        // loops with a large standing offset, and the comparison would measure how they recover
        // from that instead.
        let hold = HOLD_DUTY;
        let hold_target = make_plant().settle(&hold);
        let mut disturbed = hold_target;
        disturbed[0] += 10.0;

        let peak_interaction =
            |mut run: Box<dyn FnMut(&[f64; ZONE_COUNT]) -> [f64; ZONE_COUNT]>| {
                let mut plant = make_plant();
                let mut pv = plant.settle(&hold);
                let mut worst = 0.0_f64;
                for _ in 0..6000 {
                    let u = run(&pv);
                    pv = plant.step(&u);
                    // Only the zones that were *not* commanded to move count as interaction.
                    for z in 1..ZONE_COUNT {
                        worst = worst.max((pv[z] - hold_target[z]).abs());
                    }
                }
                worst
            };

        // --- MIMO ---
        let mut mimo = MimoPidController::<ZONE_COUNT>::from_gains(
            gains.kp,
            gains.ki,
            gains.kd,
            gains.derivative_filter_tc,
            [1.0; ZONE_COUNT],
        );
        mimo.preload_output(&hold);
        let mut t = Instant::now();
        let mimo_worst = peak_interaction(Box::new(move |pv| {
            t += Duration::from_secs(1);
            mimo.update(&disturbed, pv, t)
        }));

        // --- Decentralized, tuned from the same diagonal models and the same lambda ---
        let mut sisos: Vec<PidController> = (0..ZONE_COUNT)
            .map(|z| {
                let d = model.g[z][z];
                let (_, pi, _) =
                    crate::controllers::imc_tuner::compute_gains(d.gp, d.tau, d.theta, 0.5)
                        .unwrap();
                let mut p = PidController::new(pi.kp, pi.ki, 0.0, -1.0, 1.0);
                p.preload_integral(hold[z]);
                p
            })
            .collect();
        let mut t = Instant::now();
        let siso_worst = peak_interaction(Box::new(move |pv| {
            t += Duration::from_secs(1);
            let mut u = [0.0; ZONE_COUNT];
            for z in 0..ZONE_COUNT {
                u[z] = sisos[z]
                    .update_with_measurement(disturbed[z] - pv[z], pv[z], t)
                    .clamp(0.0, 1.0);
            }
            u
        }));

        assert!(
            mimo_worst < siso_worst,
            "MIMO peak interaction {mimo_worst:.3} C should beat decentralized {siso_worst:.3} C"
        );
    }
}
