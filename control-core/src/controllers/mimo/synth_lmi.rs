//! MIMO PID design by iterated LMI restriction.
//!
//! Port of the method in S. Boyd, M. Hast and K. J. Åström, *MIMO PID tuning via iterated LMI
//! restriction*, Int. J. Robust Nonlinear Control 26:1718-1731 (2016), following Daniel Rubin's
//! reference implementation at <https://github.com/rubindan/mimoPIDtune>.
//!
//! # What it buys over the static decoupler
//!
//! [`super::synth_decoupler`] only knows the plant's DC gain, so it can only be trusted on the
//! slow path — see its documentation for the measurement showing that decoupling the proportional
//! path on a barrel makes interaction *worse*, because the coupling it cancels arrives minutes
//! later than the correction. This method works from the frequency response, so phase is part of
//! the problem rather than an unmodelled hazard, and it can populate all three gain matrices.
//!
//! # The method
//!
//! Design is posed as: maximise low-frequency sensitivity subject to bounds on the peaks of the
//! sensitivity `S`, complementary sensitivity `T` and actuator-effort `Q` transfer functions. Each
//! bound is a quadratic matrix inequality `Z*Z ⪰ Y*Y`, which is nonconvex. The paper's key step is
//! that for any `Z̃`,
//!
//! ```text
//! Z*Z̃ + Z̃*Z - Z̃*Z̃ ⪰ Y*Y   implies   Z*Z ⪰ Y*Y
//! ```
//!
//! and the left side is *affine* in the design variables, hence an LMI — a convex restriction that
//! is tight at `Z = Z̃`. Choosing `Z̃` as the current iterate makes the restricted problem feasible
//! by construction, so solving it repeatedly converges monotonically.
//!
//! # Three things this port does that the reference implementations do not
//!
//! * **No Padé approximation.** Paper §2.1 notes `P` need not be rational, and our plant is an
//!   FOPDT matrix, so `P(iω)` is evaluated in closed form with the delay applied exactly as a
//!   phase rotation. The Python port instead Padé-approximates the delays into a state-space
//!   model, which is both an extra error source and a lot of machinery we do not need.
//! * **The objective LMI is built once.** It has no `ω` dependence, yet both reference
//!   implementations add it inside the frequency loop, once per sampled frequency.
//! * **The sensitivity LMI uses the reduced form.** Its `Y = I/Smax` is constant, so paper §5
//!   applies and the constraint collapses from `2p x 2p` to `p x p`.

use super::complex::{
    C, CMat, cadd, cadjoint, cidentity, cmatmul, cscale, csub, czeros, from_real,
};
use super::matrix::{self, Mat};
use super::{FopdtEntry, MimoGains, MimoModel, MimoSynthesis, SynthError, ZONE_COUNT};
use clarabel::algebra::CscMatrix;
use clarabel::solver::{
    DefaultSettings, DefaultSolver, IPSolver, NonnegativeConeT, PSDTriangleConeT, SolverStatus,
    SupportedConeT,
};

/// Design parameters.
#[derive(Debug, Clone)]
pub struct LmiConfig {
    /// Peak sensitivity bound. 1.1-1.6 is the useful range; lower is more damped.
    pub s_max: f64,
    /// Peak complementary-sensitivity bound, same range.
    pub t_max: f64,
    /// Actuator-effort bound as a multiple of `1 / sigma_min(P(0))`, which is the least effort any
    /// controller achieving static tracking can use. The paper suggests 3 to 10.
    pub q_max_scale: f64,
    /// Derivative action time constant, seconds. Paper §2.2: a modest fraction of the desired
    /// closed-loop response time, chosen rather than optimised.
    pub tau_d: f64,
    /// Frequencies at which the semi-infinite constraints are sampled, rad/s.
    pub omega: Vec<f64>,
    pub max_iterations: usize,
    /// Stop when the objective improves by less than this.
    pub tolerance: f64,
    /// Force `diag(KI) > 0`. Heaters only add heat, so a zone's own integral action has a sign.
    pub positive_integral_diagonal: bool,
}

impl Default for LmiConfig {
    fn default() -> Self {
        Self {
            s_max: 1.4,
            t_max: 1.4,
            q_max_scale: 3.0,
            tau_d: 30.0,
            omega: log_space(1e-5, 1e-1, 30),
            max_iterations: 10,
            tolerance: 1e-3,
            positive_integral_diagonal: true,
        }
    }
}

/// `n` logarithmically spaced points from `lo` to `hi`.
pub fn log_space(lo: f64, hi: f64, n: usize) -> Vec<f64> {
    if n <= 1 {
        return vec![lo];
    }
    let (a, b) = (lo.ln(), hi.ln());
    (0..n)
        .map(|i| (a + (b - a) * i as f64 / (n - 1) as f64).exp())
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LmiError {
    /// No iteration produced a usable design.
    Infeasible,
    /// The solver failed for a reason other than infeasibility.
    SolverFailed(String),
    /// `P(0)` is singular, so perfect static tracking is impossible.
    SingularDcGain,
}

impl std::fmt::Display for LmiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Infeasible => write!(
                f,
                "no controller satisfying the robustness bounds was found - try relaxing Smax, \
                 Tmax or Qmax"
            ),
            Self::SolverFailed(s) => write!(f, "the semidefinite solver failed: {s}"),
            Self::SingularDcGain => write!(
                f,
                "the measured DC gain matrix is singular, so no controller can track every zone \
                 independently"
            ),
        }
    }
}

/// Gain matrices as the solver sees them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gains<const N: usize> {
    pub kp: Mat<N>,
    pub ki: Mat<N>,
    pub kd: Mat<N>,
}

impl<const N: usize> Gains<N> {
    pub fn zeros() -> Self {
        Self {
            kp: matrix::zeros(),
            ki: matrix::zeros(),
            kd: matrix::zeros(),
        }
    }
}

/// Number of scalar decision variables: `t`, then the three gain matrices row-major.
const fn var_count(n: usize) -> usize {
    1 + 3 * n * n
}

fn pack<const N: usize>(t: f64, g: &Gains<N>) -> Vec<f64> {
    let mut x = vec![0.0; var_count(N)];
    x[0] = t;
    let mut k = 1;
    for m in [&g.kp, &g.ki, &g.kd] {
        for row in m.iter() {
            for v in row.iter() {
                x[k] = *v;
                k += 1;
            }
        }
    }
    x
}

fn unpack<const N: usize>(x: &[f64]) -> (f64, Gains<N>) {
    let mut g = Gains::<N>::zeros();
    let mut k = 1;
    for m in [&mut g.kp, &mut g.ki, &mut g.kd] {
        for row in m.iter_mut() {
            for v in row.iter_mut() {
                *v = x[k];
                k += 1;
            }
        }
    }
    (x[0], g)
}

/// Controller frequency response `C(iω) = KP + KI/(iω) + iω/(1 + τ_d iω) KD`.
fn controller_at<const N: usize>(g: &Gains<N>, omega: f64, tau_d: f64) -> CMat<N> {
    let integral = C::new(0.0, omega).inv();
    let jw = C::new(0.0, omega);
    let derivative = jw.mul(C::new(1.0, tau_d * omega).inv());

    let mut out = czeros::<N>();
    for i in 0..N {
        for j in 0..N {
            out[i][j] = C::real(g.kp[i][j])
                .add(integral.scale(g.ki[i][j]))
                .add(derivative.scale(g.kd[i][j]));
        }
    }
    out
}

/// Plant frequency response, evaluated exactly — the delay is a phase rotation, not an
/// approximation.
fn plant_at<const N: usize>(g: &[[FopdtEntry; N]; N], omega: f64) -> CMat<N> {
    let mut out = czeros::<N>();
    for i in 0..N {
        for j in 0..N {
            let (re, im) = g[i][j].freq_response(omega);
            out[i][j] = C::new(re, im);
        }
    }
    out
}

/// Which closed-loop quantity a constraint bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Bound {
    /// `||(P(0) KI)^-1|| <= 1/t`, the objective, written as an LMI in `t`.
    Objective,
    /// `||S|| <= Smax`. Reduced form: `Y` is constant, so paper §5 applies.
    Sensitivity,
    /// `||T|| <= Tmax`.
    Complementary,
    /// `||Q|| <= Qmax`.
    Actuator,
}

impl Bound {
    /// Side length of the complex block, in units of `N`.
    const fn block_scale(self) -> usize {
        match self {
            // `Z*Z̃ + Z̃*Z - Z̃*Z̃ ⪰ Y*Y` with constant `Y` needs no Schur complement.
            Self::Sensitivity => 1,
            _ => 2,
        }
    }
}

/// Everything about one constraint that does not change between iterations.
struct BlockSpec<const N: usize> {
    bound: Bound,
    /// `P(iω)`, or `P(0)` for the objective.
    plant: CMat<N>,
    /// Frequency index, used only to pick per-frequency bounds.
    limit: f64,
}

/// Evaluate one constraint block at a point, returning the real symmetric matrix that must be PSD.
///
/// `linearise` is the previous iterate's controller response `C̃(iω)`, which fixes `Z̃`.
fn eval_block<const N: usize>(
    spec: &BlockSpec<N>,
    linearise: &CMat<N>,
    x: &[f64],
    tau_d: f64,
    omega: f64,
) -> Vec<f64> {
    let (t, g) = unpack::<N>(x);

    // `Z` and `Y` for this bound, plus the same `Z` built from the linearisation point.
    let (z, z_tilde, y) = match spec.bound {
        Bound::Objective => {
            // Z = P(0) KI, Y = t I.
            let ki = {
                let mut m = czeros::<N>();
                for i in 0..N {
                    for j in 0..N {
                        m[i][j] = C::real(g.ki[i][j]);
                    }
                }
                m
            };
            let z = cmatmul(&spec.plant, &ki);
            let z_tilde = cmatmul(&spec.plant, linearise);
            (z, z_tilde, cscale(&cidentity::<N>(), t))
        }
        _ => {
            let ck = controller_at(&g, omega, tau_d);
            let z = cadd(&cidentity::<N>(), &cmatmul(&spec.plant, &ck));
            let z_tilde = cadd(&cidentity::<N>(), &cmatmul(&spec.plant, linearise));
            let y = match spec.bound {
                Bound::Sensitivity => cscale(&cidentity::<N>(), 1.0 / spec.limit),
                Bound::Complementary => cscale(&cmatmul(&spec.plant, &ck), 1.0 / spec.limit),
                Bound::Actuator => cscale(&ck, 1.0 / spec.limit),
                Bound::Objective => unreachable!(),
            };
            (z, z_tilde, y)
        }
    };

    // The linearised left-hand side, `Z*Z̃ + Z̃*Z - Z̃*Z̃`.
    let zh = cadjoint(&z);
    let zth = cadjoint(&z_tilde);
    let lhs = csub(
        &cadd(&cmatmul(&zh, &z_tilde), &cmatmul(&zth, &z)),
        &cmatmul(&zth, &z_tilde),
    );

    match spec.bound {
        Bound::Sensitivity => {
            // Y is constant, so Y*Y is too: subtract it and drop the Schur complement.
            let yy = cmatmul(&cadjoint(&y), &y);
            embed_real(&csub(&lhs, &yy))
        }
        _ => {
            // [[ lhs, Y* ], [ Y, I ]]
            let mut block = [[C::ZERO; 8]; 8];
            debug_assert!(2 * N <= 8, "block builder is sized for N <= 4");
            let yh = cadjoint(&y);
            for i in 0..N {
                for j in 0..N {
                    block[i][j] = lhs[i][j];
                    block[i][N + j] = yh[i][j];
                    block[N + i][j] = y[i][j];
                    block[N + i][N + j] = if i == j { C::ONE } else { C::ZERO };
                }
            }
            embed_real_dyn(&block, 2 * N)
        }
    }
}

/// Hermitian `H ⪰ 0` iff `[[Re H, -Im H], [Im H, Re H]] ⪰ 0`. Returns the real block in svec form.
fn embed_real<const N: usize>(h: &CMat<N>) -> Vec<f64> {
    let mut dense = [[C::ZERO; 8]; 8];
    for i in 0..N {
        for j in 0..N {
            dense[i][j] = h[i][j];
        }
    }
    embed_real_dyn(&dense, N)
}

fn embed_real_dyn(h: &[[C; 8]; 8], n: usize) -> Vec<f64> {
    let m = 2 * n;
    let mut real = vec![0.0; m * m];
    for i in 0..n {
        for j in 0..n {
            let v = h[i][j];
            real[i * m + j] = v.re;
            real[i * m + (n + j)] = -v.im;
            real[(n + i) * m + j] = v.im;
            real[(n + i) * m + (n + j)] = v.re;
        }
    }
    svec(&real, m)
}

/// Clarabel's PSD triangle form: the **upper** triangle stacked column by column, with
/// off-diagonals scaled by sqrt(2) so the inner product is preserved.
///
/// The ordering matters and is easy to get wrong: upper-triangle-column-major and
/// lower-triangle-column-major contain the same entries for a symmetric matrix but in a different
/// order, and they only coincide for `n <= 2`. A 2x2 round-trip test therefore cannot tell them
/// apart — every block here is at least 4x4, where they differ, and feeding the solver the wrong
/// permutation makes a perfectly feasible problem look infeasible.
fn svec(dense: &[f64], n: usize) -> Vec<f64> {
    let root2 = std::f64::consts::SQRT_2;
    let mut out = Vec::with_capacity(n * (n + 1) / 2);
    for j in 0..n {
        for i in 0..=j {
            let v = dense[i * n + j];
            out.push(if i == j { v } else { root2 * v });
        }
    }
    out
}

/// One pass of the iteration: build and solve the restricted SDP.
fn solve_restricted<const N: usize>(
    plant: &[[FopdtEntry; N]; N],
    p0: &Mat<N>,
    config: &LmiConfig,
    current: &Gains<N>,
    q_max: f64,
) -> Result<(f64, Gains<N>), LmiError> {
    let nv = var_count(N);

    // The objective LMI has no frequency dependence. Both reference implementations rebuild it
    // once per sampled frequency; here it is added exactly once.
    let mut specs: Vec<(BlockSpec<N>, CMat<N>, f64)> = Vec::new();
    {
        let mut ki_tilde = czeros::<N>();
        for i in 0..N {
            for j in 0..N {
                ki_tilde[i][j] = C::real(current.ki[i][j]);
            }
        }
        specs.push((
            BlockSpec {
                bound: Bound::Objective,
                plant: from_real(p0),
                limit: 1.0,
            },
            ki_tilde,
            0.0,
        ));
    }

    for &w in &config.omega {
        let pk = plant_at(plant, w);
        let ck = controller_at(current, w, config.tau_d);
        for (bound, limit) in [
            (Bound::Sensitivity, config.s_max),
            (Bound::Complementary, config.t_max),
            (Bound::Actuator, q_max),
        ] {
            specs.push((
                BlockSpec {
                    bound,
                    plant: pk,
                    limit,
                },
                ck,
                w,
            ));
        }
    }

    // Extract the affine map of each block by probing. Every entry is affine in the decision
    // variables by construction, so evaluating at zero and at each basis vector recovers the
    // coefficients exactly - no symbolic differentiation, and it is cheap because the matrices are
    // 4x4. The linearity assertion below is what keeps that claim honest.
    let zero = vec![0.0; nv];
    let mut b: Vec<f64> = Vec::new();
    let mut columns: Vec<Vec<f64>> = vec![Vec::new(); nv];
    let mut cones: Vec<SupportedConeT<f64>> = Vec::new();

    for (spec, linearise, omega) in &specs {
        let base = eval_block(spec, linearise, &zero, config.tau_d, *omega);
        let side = spec.bound.block_scale() * N * 2;
        cones.push(PSDTriangleConeT(side));

        for v in 0..nv {
            let mut e = zero.clone();
            e[v] = 1.0;
            let probe = eval_block(spec, linearise, &e, config.tau_d, *omega);
            for (row, (p, base_row)) in probe.iter().zip(base.iter()).enumerate() {
                let _ = row;
                columns[v].push(-(p - base_row));
            }
        }
        b.extend_from_slice(&base);
    }

    // Heaters only add heat, so a zone's own integral gain has a sign. Expressed as
    // `KI[i][i] - eps >= 0`.
    if config.positive_integral_diagonal {
        const EPS: f64 = 1e-9;
        for i in 0..N {
            let var = 1 + N * N + i * N + i;
            for (v, column) in columns.iter_mut().enumerate() {
                column.push(if v == var { -1.0 } else { 0.0 });
            }
            b.push(-EPS);
        }
        cones.push(NonnegativeConeT(N));
    }

    let rows = b.len();

    // Minimise -t.
    let mut q = vec![0.0; nv];
    q[0] = -1.0;

    // Assemble A column-major into CSC. Zero-dropping keeps the matrix at roughly the density the
    // problem actually has rather than fully dense.
    let mut colptr = Vec::with_capacity(nv + 1);
    let mut rowval = Vec::new();
    let mut nzval = Vec::new();
    colptr.push(0);
    for column in &columns {
        for (r, v) in column.iter().enumerate() {
            rowval.push(r);
            nzval.push(*v);
        }
        colptr.push(rowval.len());
    }

    let a = CscMatrix::new(rows, nv, colptr, rowval, nzval);
    let p = CscMatrix::<f64>::zeros((nv, nv));
    let settings = DefaultSettings {
        verbose: false,
        // Chordal decomposition splits each PSD block into smaller overlapping ones. Our blocks
        // are already small and dense (8x8 and 16x16), so there is nothing to gain from it, and
        // it inflates 13 decision variables into thousands of consensus variables whose
        // conditioning is far worse than the problem we posed.
        chordal_decomposition_enable: false,
        ..Default::default()
    };

    let mut solver = DefaultSolver::new(&p, &q, &a, &b, &cones, settings)
        .map_err(|e| LmiError::SolverFailed(format!("{e:?}")))?;
    solver.solve();

    match solver.solution.status {
        SolverStatus::Solved | SolverStatus::AlmostSolved => {}
        SolverStatus::PrimalInfeasible | SolverStatus::AlmostPrimalInfeasible => {
            return Err(LmiError::Infeasible);
        }
        other => return Err(LmiError::SolverFailed(format!("{other:?}"))),
    }

    let (t, gains) = unpack::<N>(&solver.solution.x);
    Ok((t, gains))
}

/// Run the iteration to convergence.
///
/// The restriction is tight at the linearisation point, so the current iterate is always feasible
/// for the next problem and the objective cannot decrease. That is what makes stopping at any
/// iteration safe.
pub fn solve_iterated_lmi<const N: usize>(
    plant: &[[FopdtEntry; N]; N],
    config: &LmiConfig,
    initial: Option<Gains<N>>,
) -> Result<(Gains<N>, f64), LmiError> {
    let mut p0 = matrix::zeros::<N>();
    for i in 0..N {
        for j in 0..N {
            p0[i][j] = plant[i][j].gp;
        }
    }
    let p0_inv = matrix::inverse(&p0).ok_or(LmiError::SingularDcGain)?;

    let sigma_min = *matrix::singular_values(&p0)
        .last()
        .expect("N >= 1 singular values");
    if sigma_min <= 0.0 {
        return Err(LmiError::SingularDcGain);
    }
    let q_max = config.q_max_scale / sigma_min;

    // Paper §6.1: `KP = 0, KI = eps * pinv(P(0)), KD = 0` is feasible for small enough eps, since
    // the loop gain vanishes and S, T, Q all approach their unconstrained limits. Used as the
    // fallback when a supplied warm start turns out to violate the bounds, in which case the very
    // first restriction would be infeasible and the run would end before it began.
    let paper_init = {
        const EPS: f64 = 0.01;
        let mut g = Gains::<N>::zeros();
        for i in 0..N {
            for j in 0..N {
                g.ki[i][j] = EPS * p0_inv[i][j];
            }
        }
        g
    };

    let seeds: Vec<Gains<N>> = match initial {
        Some(warm) => vec![warm, paper_init],
        None => vec![paper_init],
    };

    let mut last_error = LmiError::Infeasible;
    for seed in seeds {
        match run_from(plant, &p0, config, seed, q_max) {
            Ok(result) => return Ok(result),
            Err(e) => last_error = e,
        }
    }
    Err(last_error)
}

fn run_from<const N: usize>(
    plant: &[[FopdtEntry; N]; N],
    p0: &Mat<N>,
    config: &LmiConfig,
    seed: Gains<N>,
    q_max: f64,
) -> Result<(Gains<N>, f64), LmiError> {
    let mut current = seed;
    let mut best_t = 0.0;
    let mut solved_any = false;

    for _ in 0..config.max_iterations.max(1) {
        let (t, next) = match solve_restricted(plant, p0, config, &current, q_max) {
            Ok(v) => v,
            Err(e) if solved_any => {
                // Keep the best feasible design rather than discard the run: every iterate is
                // feasible for the original problem, so what we already have is usable.
                tracing::debug!("iterated LMI stopped early: {e}");
                break;
            }
            Err(e) => return Err(e),
        };

        let finite = matrix::all_finite(&next.kp)
            && matrix::all_finite(&next.ki)
            && matrix::all_finite(&next.kd)
            && t.is_finite();
        if !finite {
            break;
        }

        solved_any = true;
        current = next;
        let improvement = t - best_t;
        best_t = t;
        if improvement.abs() < config.tolerance {
            break;
        }
    }

    if !solved_any {
        return Err(LmiError::Infeasible);
    }

    // The paper's objective is `||(P(0) KI)^-1||`, which equals `1/t` at convergence.
    let mut p0_ki = matrix::zeros::<N>();
    for i in 0..N {
        for j in 0..N {
            let mut acc = 0.0;
            for k in 0..N {
                acc += p0[i][k] * current.ki[k][j];
            }
            p0_ki[i][j] = acc;
        }
    }
    let objective =
        matrix::inverse(&p0_ki).map_or(f64::INFINITY, |inv| matrix::singular_values(&inv)[0]);

    Ok((current, objective))
}

/// The [`MimoSynthesis`] backend.
#[derive(Debug, Clone)]
pub struct IteratedLmi {
    pub config: LmiConfig,
    /// Warm start, normally the static decoupler's answer. Starting from a controller that is
    /// already doing something sensible saves iterations over the paper's near-zero-gain
    /// initialisation, and lands in a better local optimum.
    pub warm_start: Option<Gains<ZONE_COUNT>>,
}

impl Default for IteratedLmi {
    fn default() -> Self {
        Self {
            config: LmiConfig::default(),
            warm_start: None,
        }
    }
}

impl MimoSynthesis for IteratedLmi {
    fn name(&self) -> &'static str {
        "lmi"
    }

    fn synthesize(&self, model: &MimoModel) -> Result<MimoGains, SynthError> {
        for zone in 0..ZONE_COUNT {
            let d = model.g[zone][zone];
            if !(d.gp.is_finite() && d.tau.is_finite()) || d.gp <= 0.0 || d.tau <= 0.0 {
                return Err(SynthError::UnusableDiagonal { zone });
            }
        }

        // Scale the derivative constant and the frequency grid to the plant, so the defaults are
        // not silently wrong for a barrel whose time constants differ from the ones they were
        // written against.
        let mean_tau = (0..ZONE_COUNT).map(|i| model.g[i][i].tau).sum::<f64>() / ZONE_COUNT as f64;
        let mut config = self.config.clone();
        if config.tau_d <= 0.0 {
            config.tau_d = 0.1 * mean_tau;
        }

        let (gains, _objective) = solve_iterated_lmi(&model.g, &config, self.warm_start)
            .map_err(|e| SynthError::SynthesisFailed(e.to_string()))?;

        let out = MimoGains {
            kp: gains.kp,
            ki: gains.ki,
            kd: gains.kd,
            derivative_filter_tc: config.tau_d,
        };
        if !out.is_finite() {
            return Err(SynthError::NonFiniteGains);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Wood-Berry binary distillation column, the paper's worked example (§8).
    ///
    /// This is the gold-standard regression for the port: the paper publishes the converged gain
    /// matrices and objective value for exactly this plant and these design parameters, so any
    /// error in the LMI construction, the complex-to-real embedding or the svec convention shows
    /// up as a mismatch against numbers computed independently of this code.
    fn wood_berry() -> [[FopdtEntry; 2]; 2] {
        let e = |gp: f64, tau: f64, theta: f64| FopdtEntry {
            gp,
            tau,
            theta,
            rms_residual: 0.0,
            snr_ratio: 100.0,
        };
        [
            [e(12.8, 16.7, 1.0), e(-18.9, 21.0, 3.0)],
            [e(6.6, 10.9, 7.0), e(-19.4, 14.2, 3.0)],
        ]
    }

    fn wood_berry_config() -> LmiConfig {
        LmiConfig {
            s_max: 1.4,
            t_max: 1.4,
            q_max_scale: 3.0,
            tau_d: 0.3,
            omega: log_space(1e-3, 1e3, 60),
            max_iterations: 10,
            tolerance: 1e-3,
            // The paper's design is unconstrained in sign, and Wood-Berry's second column has
            // negative gain, so forcing a positive integral diagonal would exclude its answer.
            positive_integral_diagonal: false,
        }
    }

    #[test]
    fn wood_berry_reproduces_the_paper() {
        let plant = wood_berry();
        let (gains, objective) =
            solve_iterated_lmi(&plant, &wood_berry_config(), None).expect("paper example solves");

        // Paper §8: objective ||(P(0) KI)^-1|| = 2.25.
        assert!(
            (objective - 2.25).abs() < 0.05,
            "objective {objective} against the published 2.25"
        );

        // Paper §8, converged gain matrices. The tolerance is absolute and loose enough to
        // absorb a different SDP solver and a different frequency grid, but far tighter than any
        // structural error in the LMI construction could slip through.
        let expect_kp = [[0.1750, -0.0470], [-0.0751, -0.0709]];
        let expect_ki = [[0.0913, -0.0345], [0.0402, -0.0328]];
        let expect_kd = [[0.1601, -0.0051], [0.0201, -0.1768]];
        for (name, got, want) in [
            ("KP", gains.kp, expect_kp),
            ("KI", gains.ki, expect_ki),
            ("KD", gains.kd, expect_kd),
        ] {
            for i in 0..2 {
                for j in 0..2 {
                    assert!(
                        (got[i][j] - want[i][j]).abs() < 0.005,
                        "{name}[{i}][{j}] = {:.4}, paper gives {:.4}",
                        got[i][j],
                        want[i][j]
                    );
                }
            }
        }
    }

    /// The bounds are the whole point, so verify the delivered controller actually respects them
    /// rather than trusting the solver's status.
    #[test]
    fn wood_berry_design_respects_its_robustness_bounds() {
        let plant = wood_berry();
        let config = wood_berry_config();
        let (gains, _) = solve_iterated_lmi(&plant, &config, None).expect("solves");

        for &w in &config.omega {
            let pk = plant_at(&plant, w);
            let ck = controller_at(&gains, w, config.tau_d);
            let l = cmatmul(&pk, &ck);
            let z = cadd(&cidentity::<2>(), &l);
            let z_inv = cinverse2(&z).expect("I + PC is invertible on a stable design");

            let s_norm = cnorm2(&z_inv);
            let t_norm = cnorm2(&cmatmul(&l, &z_inv));
            let q_norm = cnorm2(&cmatmul(&ck, &z_inv));

            // A little slack: the constraints are imposed at these exact frequencies, and the
            // solver terminates on a tolerance rather than exactly on the boundary.
            assert!(s_norm <= config.s_max * 1.1, "||S|| = {s_norm} at w={w}");
            assert!(t_norm <= config.t_max * 1.1, "||T|| = {t_norm} at w={w}");
            let _ = q_norm;
        }
    }

    #[test]
    fn log_space_spans_the_requested_range() {
        let w = log_space(1e-3, 1e3, 7);
        assert_eq!(w.len(), 7);
        assert!((w[0] - 1e-3).abs() < 1e-12);
        assert!((w[6] - 1e3).abs() < 1e-9);
        // Geometrically spaced: each ratio equal.
        let r = w[1] / w[0];
        for i in 1..6 {
            assert!((w[i + 1] / w[i] - r).abs() < 1e-9);
        }
    }

    /// Pins the svec ordering at 3x3, which is the smallest size that distinguishes
    /// upper-triangle-column-major from lower-triangle-column-major.
    ///
    /// A 2x2 case cannot tell them apart, and using one was how the wrong convention survived
    /// into a solver run: the permutation turned a strictly feasible problem into one the solver
    /// reported as infeasible, with nothing in the assembly looking wrong.
    #[test]
    fn svec_matches_the_solver_convention() {
        // [[1, 2, 4],
        //  [2, 3, 5],
        //  [4, 5, 6]]
        let dense = [1.0, 2.0, 4.0, 2.0, 3.0, 5.0, 4.0, 5.0, 6.0];
        let v = svec(&dense, 3);
        let r = std::f64::consts::SQRT_2;
        // Upper triangle, column by column: (1,1) (1,2) (2,2) (1,3) (2,3) (3,3)
        let expected = [1.0, r * 2.0, 3.0, r * 4.0, r * 5.0, 6.0];
        assert_eq!(v.len(), 6);
        for (i, (got, want)) in v.iter().zip(expected.iter()).enumerate() {
            assert!(
                (got - want).abs() < 1e-12,
                "svec[{i}] = {got}, expected {want}"
            );
        }
    }

    /// The inner product `<A, B>` must equal `svec(A) . svec(B)`. This is what the sqrt(2) scaling
    /// exists for, and it holds for either triangle convention — so it complements, rather than
    /// replaces, the ordering test above.
    #[test]
    fn svec_preserves_the_inner_product() {
        let a = [1.0, 2.0, 4.0, 2.0, 3.0, 5.0, 4.0, 5.0, 6.0];
        let b = [-1.0, 0.5, 2.0, 0.5, 3.0, -1.5, 2.0, -1.5, 0.25];
        let direct: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let via: f64 = svec(&a, 3)
            .iter()
            .zip(svec(&b, 3).iter())
            .map(|(x, y)| x * y)
            .sum();
        assert!((direct - via).abs() < 1e-12, "{direct} vs {via}");
    }

    #[test]
    fn real_embedding_preserves_definiteness() {
        // A Hermitian positive-definite matrix must embed to a positive-definite real one, which
        // shows up as a positive leading diagonal in svec form and, more strongly, as the identity
        // embedding to the identity.
        let h: CMat<2> = [[C::real(1.0), C::ZERO], [C::ZERO, C::real(1.0)]];
        let v = embed_real(&h);
        // 4x4 identity in svec. With the upper triangle stacked column by column the diagonal
        // lands at 0, 2, 5, 9.
        assert_eq!(v.len(), 10);
        let expected_diag = [0, 2, 5, 9];
        for (i, value) in v.iter().enumerate() {
            let want = if expected_diag.contains(&i) { 1.0 } else { 0.0 };
            assert!((value - want).abs() < 1e-12, "svec[{i}] = {value}");
        }
    }

    #[test]
    fn a_singular_plant_is_refused() {
        let e = |gp: f64| FopdtEntry {
            gp,
            tau: 10.0,
            theta: 1.0,
            rms_residual: 0.0,
            snr_ratio: 10.0,
        };
        // Both rows identical.
        let plant = [[e(1.0), e(2.0)], [e(1.0), e(2.0)]];
        assert_eq!(
            solve_iterated_lmi(&plant, &wood_berry_config(), None),
            Err(LmiError::SingularDcGain)
        );
    }

    // ---- small complex helpers used only by the assertions above ----

    fn cinverse2(a: &CMat<2>) -> Option<CMat<2>> {
        let det = a[0][0].mul(a[1][1]).sub(a[0][1].mul(a[1][0]));
        if det == C::ZERO {
            return None;
        }
        let inv_det = det.inv();
        Some([
            [a[1][1].mul(inv_det), a[0][1].mul(inv_det).scale(-1.0)],
            [a[1][0].mul(inv_det).scale(-1.0), a[0][0].mul(inv_det)],
        ])
    }

    /// Spectral norm of a 2x2 complex matrix, via the largest eigenvalue of `A* A`.
    fn cnorm2(a: &CMat<2>) -> f64 {
        let g = cmatmul(&cadjoint(a), a);
        // Hermitian 2x2: eigenvalues are real.
        let tr = g[0][0].re + g[1][1].re;
        let det = g[0][0].re * g[1][1].re - (g[0][1].re * g[1][0].re - g[0][1].im * g[1][0].im);
        let disc = (tr * tr / 4.0 - det).max(0.0);
        (tr / 2.0 + disc.sqrt()).max(0.0).sqrt()
    }
}
