//! Static decoupling plus per-channel IMC tuning.
//!
//! The idea is to make the coupled plant look diagonal to the controller, then tune the resulting
//! independent channels with the rules the machine already uses. Given the measured DC gain matrix
//! `G(0)`, the decoupler
//!
//! ```text
//! D = inv(G(0)) * diag(g_ii)
//! ```
//!
//! satisfies `G(0) * D = diag(g_ii)`: a unit of decoupled command `v_j` produces a steady change
//! only in output `j`, with the same DC gain that zone's own heater already had. A diagonal
//! controller `C_d(s)` on `v` then becomes `C(s) = C_d(s) * D` on the real actuators — full
//! matrices, but derived from rules whose behaviour is already understood on this machine.
//!
//! The decoupling is exact only at DC. That is the deliberate trade: a dynamic decoupler would
//! need to invert the delays, which is not causal, and inverting a measured model that far is
//! fragile.
//!
//! # Only the integral path is decoupled
//!
//! Because the design is DC-only, it is applied only where DC reasoning holds. `KI` is decoupled;
//! `KP` and `KD` stay diagonal.
//!
//! The reason is dead time. A static decoupler issues its full steady-state correction the instant
//! an error appears, but the coupling that correction cancels is still in transit — on the
//! simulated barrel the off-diagonal paths carry 96-256 s of dead time against zone time constants
//! of 350-630 s. Routing the proportional path through the decoupler therefore *pre-cools* a
//! neighbour minutes before any extra heat reaches it, and the neighbour is driven off setpoint by
//! the correction rather than by the disturbance. Measured on `CoupledBarrel` with strong
//! conduction, peak interaction after a 10 degree setpoint step:
//!
//! ```text
//! decentralized PI                  0.769 C
//! decoupling P, I and D             1.727 C   <- worse than doing nothing
//! decoupling I only                 0.550 C
//! ```
//!
//! The integral path does not have this problem: it acts on accumulated error, so it is already
//! slow relative to the transport delays, and it is what determines where the actuators finally
//! settle — which is exactly the steady-state interaction a DC decoupler is entitled to fix.
//!
//! Getting the fast paths right needs a design that knows about phase, not just DC gain. That is
//! what a frequency-domain synthesis backend is for; this one deliberately does not pretend to it.

use super::matrix::{self, Mat};
use super::{MimoGains, MimoModel, MimoSynthesis, SynthError, ZONE_COUNT};
use crate::controllers::imc_tuner::compute_gains;

/// Condition number above which the static inverse is refused.
///
/// `inv(G(0))` amplifies relative model error by roughly the condition number, so at 20 a 5%
/// error in the identified gains is already a 100% error in the decoupler. Past that the honest
/// answer is that the campaign did not resolve the zones well enough to decouple them.
pub const MAX_CONDITION_NUMBER: f64 = 20.0;

/// Acceptable range for the RGA diagonal.
///
/// Outside it, output `i` is dominated by something other than heater `i`, and pairing them is the
/// wrong structure regardless of gains.
pub const MIN_RGA_DIAGONAL: f64 = 0.3;
pub const MAX_RGA_DIAGONAL: f64 = 3.0;

/// Static-decoupling synthesis.
#[derive(Debug, Clone, Copy)]
pub struct DecouplerImc {
    /// Closed-loop time constant as a multiple of the process time constant, with exactly the
    /// meaning it has in the single-zone IMC tuner — the operator-facing response-speed presets
    /// carry over unchanged.
    pub lambda_factor: f64,
    /// Use the PID candidate rather than PI. PI is the safer default on a noisy thermal loop.
    pub use_pid: bool,
    /// Also route the proportional and derivative paths through the decoupler.
    ///
    /// Off by default, and measurably so: on the simulated barrel this roughly *triples* the peak
    /// interaction it is supposed to remove. See the type-level documentation.
    pub decouple_fast_paths: bool,
}

impl Default for DecouplerImc {
    fn default() -> Self {
        Self {
            lambda_factor: 1.0,
            use_pid: false,
            decouple_fast_paths: false,
        }
    }
}

impl MimoSynthesis for DecouplerImc {
    fn name(&self) -> &'static str {
        "decoupler"
    }

    fn synthesize(&self, model: &MimoModel) -> Result<MimoGains, SynthError> {
        let g0 = model.dc_gain();

        // Refuse before inverting, so the operator gets the reason rather than a fragile answer.
        for zone in 0..ZONE_COUNT {
            let d = model.g[zone][zone];
            if !(d.gp.is_finite() && d.tau.is_finite()) || d.gp <= 0.0 || d.tau <= 0.0 {
                return Err(SynthError::UnusableDiagonal { zone });
            }
        }

        let cond = matrix::condition_number(&g0);
        if !cond.is_finite() {
            return Err(SynthError::SingularDcGain);
        }
        if cond > MAX_CONDITION_NUMBER {
            return Err(SynthError::IllConditioned {
                condition_number: cond.round() as i64,
                limit: MAX_CONDITION_NUMBER as i64,
            });
        }

        if matrix::niederlinski(&g0) < 0.0 {
            return Err(SynthError::StructurallyUnstable);
        }

        let rga = matrix::rga(&g0).ok_or(SynthError::SingularDcGain)?;
        for zone in 0..ZONE_COUNT {
            let d = rga[zone][zone];
            if !d.is_finite() || !(MIN_RGA_DIAGONAL..=MAX_RGA_DIAGONAL).contains(&d) {
                return Err(SynthError::BadPairing { zone });
            }
        }

        let g_inv = matrix::inverse(&g0).ok_or(SynthError::SingularDcGain)?;
        let mut own_gain = [0.0; ZONE_COUNT];
        for (j, slot) in own_gain.iter_mut().enumerate() {
            *slot = model.g[j][j].gp;
        }
        let decoupler = matmul_diag_right(&g_inv, &own_gain);

        // Tune each decoupled channel with the existing IMC rules.
        let mut kp_d = [0.0; ZONE_COUNT];
        let mut ki_d = [0.0; ZONE_COUNT];
        let mut kd_d = [0.0; ZONE_COUNT];
        let mut worst_td = 0.0_f64;

        for j in 0..ZONE_COUNT {
            let entry = model.g[j][j];
            let theta_eff = effective_dead_time(model, &decoupler, j);

            let (_, pi, pid) = compute_gains(entry.gp, entry.tau, theta_eff, self.lambda_factor)
                .ok_or(SynthError::GainComputationFailed { zone: j })?;
            let chosen = if self.use_pid { pid } else { pi };

            kp_d[j] = chosen.kp;
            ki_d[j] = chosen.ki;
            kd_d[j] = chosen.kd;
            worst_td = worst_td.max(chosen.td);
        }

        // The integral path is always decoupled; the fast paths are not, unless asked for.
        let fast = |d: &[f64; ZONE_COUNT]| {
            if self.decouple_fast_paths {
                matmul_diag_left(d, &decoupler)
            } else {
                matrix::diag(d)
            }
        };

        let gains = MimoGains {
            kp: fast(&kp_d),
            ki: matmul_diag_left(&ki_d, &decoupler),
            kd: fast(&kd_d),
            // One filter for the whole controller, set from the fastest derivative action present,
            // for the same reason the SISO path bundles the two: a non-zero kd without a filter
            // differentiates sensor noise over a sub-millisecond loop period.
            derivative_filter_tc: if worst_td > 0.0 {
                worst_td / crate::controllers::imc_tuner::DERIVATIVE_FILTER_DIVISOR
            } else {
                0.0
            },
        };

        if !gains.is_finite() {
            return Err(SynthError::NonFiniteGains);
        }
        Ok(gains)
    }
}

/// Fraction of a decoupled channel's total throughput a path must carry before its dead time is
/// allowed to detune that channel.
const PATH_SIGNIFICANCE: f64 = 0.1;

/// Dead time of decoupled channel `j`.
///
/// Under the decoupler, a unit of command on channel `j` moves *every* actuator: actuator `i` by
/// `D[i][j]`. Output `j` therefore receives heat over several paths at once, path `i` carrying
/// `G[j][i] * D[i][j]` and arriving after `theta[j][i]`. The channel is no faster than the
/// slowest path that carries real weight.
///
/// Both halves of that matter. Taking the plain maximum over the whole column lets the weakest,
/// worst-resolved entry in the matrix set the tuning — a distant zone's dead time is fitted from a
/// response barely above the noise, and feeding that into the IMC rules (where dead time sits in
/// the denominator of the gain) detunes a perfectly good channel several-fold for no reason. So
/// paths below [`PATH_SIGNIFICANCE`] of the channel's throughput are excluded: the decoupler is
/// not meaningfully relying on them, and their dead-time estimates are the least trustworthy
/// numbers in the model.
fn effective_dead_time(model: &MimoModel, decoupler: &Mat<ZONE_COUNT>, j: usize) -> f64 {
    let weight = |i: usize| (model.g[j][i].gp * decoupler[i][j]).abs();
    let total: f64 = (0..ZONE_COUNT).map(weight).sum();

    // Degenerate throughput: fall back to the zone's own dead time rather than inventing one.
    if !(total > 0.0) {
        return model.g[j][j].theta;
    }

    (0..ZONE_COUNT)
        .filter(|&i| weight(i) >= PATH_SIGNIFICANCE * total)
        .map(|i| model.g[j][i].theta)
        .fold(model.g[j][j].theta, f64::max)
}

/// `diag(d) * m`
fn matmul_diag_left<const N: usize>(d: &[f64; N], m: &Mat<N>) -> Mat<N> {
    let mut out = matrix::zeros::<N>();
    for i in 0..N {
        for j in 0..N {
            out[i][j] = d[i] * m[i][j];
        }
    }
    out
}

/// `m * diag(d)`
fn matmul_diag_right<const N: usize>(m: &Mat<N>, d: &[f64; N]) -> Mat<N> {
    let mut out = matrix::zeros::<N>();
    for i in 0..N {
        for j in 0..N {
            out[i][j] = m[i][j] * d[j];
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controllers::mimo::{FopdtEntry, MimoModel};
    use std::time::SystemTime;

    fn model_from(gp: [[f64; 4]; 4], tau: f64, theta: f64) -> MimoModel {
        let mut g = [[FopdtEntry::default(); 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                g[i][j] = FopdtEntry {
                    gp: gp[i][j],
                    tau,
                    theta,
                    rms_residual: 0.0,
                    snr_ratio: 50.0,
                };
            }
        }
        let mut m = MimoModel {
            g,
            setpoints: [200.0; 4],
            baseline_duty: [0.3; 4],
            rga: matrix::zeros(),
            condition_number: 1.0,
            niederlinski: 1.0,
            identified_at: SystemTime::UNIX_EPOCH,
        };
        m.refresh_diagnostics();
        m
    }

    fn banded(diag: f64, near: f64) -> [[f64; 4]; 4] {
        let mut g = [[0.0; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                g[i][j] = match i.abs_diff(j) {
                    0 => diag,
                    1 => near,
                    _ => 0.0,
                };
            }
        }
        g
    }

    /// With no coupling the decoupler is the identity, so the matrix gains must reduce exactly to
    /// what the existing single-zone tuner would have produced. This is what ties the new path back
    /// to behaviour that is already trusted on the machine.
    #[test]
    fn decoupler_matches_siso_imc_when_diagonal() {
        let gp = 250.0;
        let tau = 300.0;
        let theta = 25.0;
        let model = model_from(banded(gp, 0.0), tau, theta);

        let synth = DecouplerImc {
            lambda_factor: 0.5,
            ..Default::default()
        };
        let gains = synth.synthesize(&model).expect("diagonal plant is fine");

        let (_, pi, _) = compute_gains(gp, tau, theta, 0.5).unwrap();
        for i in 0..4 {
            for j in 0..4 {
                let expect_kp = if i == j { pi.kp } else { 0.0 };
                let expect_ki = if i == j { pi.ki } else { 0.0 };
                assert!(
                    (gains.kp[i][j] - expect_kp).abs() < 1e-12,
                    "kp[{i}][{j}] = {} expected {expect_kp}",
                    gains.kp[i][j]
                );
                assert!(
                    (gains.ki[i][j] - expect_ki).abs() < 1e-12,
                    "ki[{i}][{j}] = {} expected {expect_ki}",
                    gains.ki[i][j]
                );
            }
        }
    }

    /// The defining property of the design: `G(0) * D = diag(g_ii)`.
    #[test]
    fn decoupler_diagonalises_the_dc_gain() {
        let model = model_from(banded(250.0, 60.0), 300.0, 25.0);
        let g0 = model.dc_gain();
        let g_inv = matrix::inverse(&g0).unwrap();
        let own: [f64; 4] = std::array::from_fn(|j| model.g[j][j].gp);
        let d = matmul_diag_right(&g_inv, &own);

        let product = matrix::matmul(&g0, &d);
        for i in 0..4 {
            for j in 0..4 {
                let expect = if i == j { own[i] } else { 0.0 };
                assert!(
                    (product[i][j] - expect).abs() < 1e-8,
                    "G*D[{i}][{j}] = {} expected {expect}",
                    product[i][j]
                );
            }
        }
    }

    #[test]
    fn coupled_plant_produces_nonzero_off_diagonal_gains() {
        let model = model_from(banded(250.0, 60.0), 300.0, 25.0);
        let gains = DecouplerImc::default()
            .synthesize(&model)
            .expect("moderately coupled plant is synthesizable");

        let off_diagonal_energy: f64 = (0..4)
            .flat_map(|i| (0..4).map(move |j| (i, j)))
            .filter(|(i, j)| i != j)
            .map(|(i, j)| gains.ki[i][j].abs())
            .sum();
        assert!(
            off_diagonal_energy > 0.0,
            "a coupled plant must yield cross terms in the integral path"
        );

        // Neighbour correction opposes the neighbour's own action: holding zone 1 while zone 0
        // runs hotter calls for backing zone 1 off, since some of its heat now arrives from zone 0.
        assert!(
            gains.ki[1][0] < 0.0,
            "expected a negative cross term, got {}",
            gains.ki[1][0]
        );

        // The fast paths stay diagonal by default.
        for i in 0..4 {
            for j in 0..4 {
                if i != j {
                    assert_eq!(gains.kp[i][j], 0.0, "kp[{i}][{j}] should not be decoupled");
                    assert_eq!(gains.kd[i][j], 0.0, "kd[{i}][{j}] should not be decoupled");
                }
            }
        }
    }

    #[test]
    fn opting_into_fast_path_decoupling_populates_kp() {
        let model = model_from(banded(250.0, 60.0), 300.0, 25.0);
        let gains = DecouplerImc {
            decouple_fast_paths: true,
            ..Default::default()
        }
        .synthesize(&model)
        .unwrap();
        assert!(gains.kp[1][0] < 0.0, "expected a cross term in kp");
    }

    #[test]
    fn refuses_a_singular_plant() {
        // Two zones with identical response rows: no way to tell their heaters apart.
        let mut gp = banded(250.0, 60.0);
        gp[2] = gp[1];
        let model = model_from(gp, 300.0, 25.0);
        let err = DecouplerImc::default().synthesize(&model).unwrap_err();
        assert!(
            matches!(
                err,
                SynthError::SingularDcGain | SynthError::IllConditioned { .. }
            ),
            "got {err:?}"
        );
    }

    #[test]
    fn refuses_an_ill_conditioned_plant() {
        // Near-parallel rows: invertible on paper, useless in practice.
        let mut gp = banded(250.0, 60.0);
        gp[2] = [gp[1][0] * 1.001, gp[1][1] * 1.001, gp[1][2], gp[1][3]];
        let model = model_from(gp, 300.0, 25.0);
        let err = DecouplerImc::default().synthesize(&model).unwrap_err();
        assert!(
            matches!(
                err,
                SynthError::IllConditioned { .. } | SynthError::BadPairing { .. }
            ),
            "got {err:?}"
        );
    }

    #[test]
    fn refuses_a_zone_that_does_not_heat_itself() {
        let mut gp = banded(250.0, 60.0);
        gp[2][2] = 0.0;
        let model = model_from(gp, 300.0, 25.0);
        assert_eq!(
            DecouplerImc::default().synthesize(&model).unwrap_err(),
            SynthError::UnusableDiagonal { zone: 2 }
        );
    }

    #[test]
    fn refuses_a_plant_dominated_by_its_neighbours() {
        // Off-diagonal larger than the diagonal: the pairing itself is wrong.
        let model = model_from(banded(60.0, 250.0), 300.0, 25.0);
        let err = DecouplerImc::default().synthesize(&model).unwrap_err();
        assert!(
            matches!(
                err,
                SynthError::BadPairing { .. }
                    | SynthError::StructurallyUnstable
                    | SynthError::IllConditioned { .. }
            ),
            "got {err:?}"
        );
    }

    #[test]
    fn refusal_messages_name_the_problem() {
        // These strings surface to an operator deciding whether to re-run a multi-hour campaign,
        // so they must not be opaque.
        let msg = SynthError::BadPairing { zone: 2 }.to_string();
        assert!(msg.contains("zone 2"), "{msg}");
        let msg = SynthError::IllConditioned {
            condition_number: 47,
            limit: 20,
        }
        .to_string();
        assert!(msg.contains("47") && msg.contains("20"), "{msg}");
    }

    /// A slow path that the decoupler actually leans on must detune the channel it feeds.
    #[test]
    fn a_slow_significant_path_detunes_its_channel() {
        // Strongly coupled, so the neighbour path carries enough of zone 0's throughput to clear
        // the significance threshold. At the milder 60/250 ratio it contributes only a few percent
        // and is correctly ignored — that is the point of the companion test below.
        // At 100/250 the neighbour path carries ~17% of zone 0's throughput, comfortably over the
        // significance threshold, while the matrix stays well conditioned (~4.7).
        let baseline = model_from(banded(250.0, 100.0), 300.0, 20.0);

        // Make the path the decoupler relies on slow.
        let mut slowed = baseline.clone();
        slowed.g[0][1].theta = 200.0;

        let base_gains = DecouplerImc::default().synthesize(&baseline).unwrap();
        let slow_gains = DecouplerImc::default().synthesize(&slowed).unwrap();

        assert!(
            slow_gains.kp[0][0] < base_gains.kp[0][0],
            "a slow path feeding zone 0 must detune it: {} vs {}",
            slow_gains.kp[0][0],
            base_gains.kp[0][0]
        );
    }

    /// ...but a negligible path must not.
    ///
    /// The regression this guards against is real and was measured: taking a plain maximum over
    /// the column let the most distant entry — whose dead time is fitted from a response barely
    /// above the noise — set the tuning for a perfectly well-resolved zone. On the simulated
    /// barrel that detuned zone 0 by roughly threefold against its neighbours for no physical
    /// reason.
    #[test]
    fn a_slow_negligible_path_does_not_detune() {
        let mut gp = banded(250.0, 60.0);
        // Zone 3 barely reaches zone 0 at all.
        gp[0][3] = 0.5;
        gp[3][0] = 0.5;
        let baseline = model_from(gp, 300.0, 20.0);

        let mut slowed = baseline.clone();
        slowed.g[0][3].theta = 900.0;

        let base_gains = DecouplerImc::default().synthesize(&baseline).unwrap();
        let slow_gains = DecouplerImc::default().synthesize(&slowed).unwrap();

        assert!(
            (slow_gains.kp[0][0] - base_gains.kp[0][0]).abs() < 1e-12,
            "a negligible path must not change the tuning: {} vs {}",
            slow_gains.kp[0][0],
            base_gains.kp[0][0]
        );
    }

    #[test]
    fn lambda_factor_trades_speed_for_gain() {
        let model = model_from(banded(250.0, 60.0), 300.0, 25.0);
        let fast = DecouplerImc {
            lambda_factor: 0.2,
            ..Default::default()
        }
        .synthesize(&model)
        .unwrap();
        let slow = DecouplerImc {
            lambda_factor: 1.0,
            ..Default::default()
        }
        .synthesize(&model)
        .unwrap();
        assert!(
            fast.kp[0][0] > slow.kp[0][0],
            "a faster setting must give more gain: {} vs {}",
            fast.kp[0][0],
            slow.kp[0][0]
        );
    }

    #[test]
    fn pid_form_carries_a_derivative_filter() {
        let model = model_from(banded(250.0, 60.0), 300.0, 25.0);
        let pid = DecouplerImc {
            lambda_factor: 1.0,
            use_pid: true,
            ..Default::default()
        }
        .synthesize(&model)
        .unwrap();
        assert!(pid.derivative_filter_tc > 0.0, "kd without a filter");

        let pi = DecouplerImc {
            lambda_factor: 1.0,
            ..Default::default()
        }
        .synthesize(&model)
        .unwrap();
        assert_eq!(pi.derivative_filter_tc, 0.0);
        assert!(pi.kd.iter().flat_map(|r| r.iter()).all(|v| *v == 0.0));
    }
}
