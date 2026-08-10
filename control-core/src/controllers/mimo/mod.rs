//! MIMO (multi-input multi-output) thermal control.
//!
//! The four heating zones of an extruder barrel are coupled by conduction: driving one zone's
//! heater raises its neighbours too. Treating them as four independent SISO loops forces every
//! controller to reject that coupling as unmodelled disturbance, which caps how tight any of them
//! can be tuned.
//!
//! This module measures the coupling and then uses it:
//!
//! 1. [`identify::MimoStepIdentifier`] runs a step-test campaign that identifies the full `N x N`
//!    FOPDT transfer matrix of the barrel — one column per zone stepped, all outputs recorded.
//! 2. A [`MimoSynthesis`] backend turns that model into full `N x N` `KP`/`KI`/`KD` gain matrices.
//! 3. [`controller::MimoPidController`] runs those matrices in the control loop.
//!
//! # Zone ordering
//!
//! Indices follow the *physical* order along the barrel, so `|i - j|` is distance and a
//! well-identified model comes out banded — conduction is a nearest-neighbour effect. The caller
//! owns the mapping from its own zone names onto indices; everything here is index-based.

pub mod complex;
pub mod controller;
pub mod identify;
pub mod matrix;
pub mod synth_decoupler;
#[cfg(feature = "mimo-lmi")]
pub mod synth_lmi;

#[cfg(test)]
mod sim;

use matrix::Mat;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Heating zones on the barrel.
pub const ZONE_COUNT: usize = 4;

/// One entry of the identified transfer matrix: a first-order-plus-dead-time model of the response
/// of one output to a step on one input.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FopdtEntry {
    /// Steady-state gain, process units per unit of actuator command.
    pub gp: f64,
    /// Time constant, seconds.
    pub tau: f64,
    /// Dead time, seconds.
    pub theta: f64,
    /// Root-mean-square fit residual, in process units.
    pub rms_residual: f64,
    /// Response size over peak-to-peak noise. Low values mean this entry is mostly noise, which is
    /// the expected and harmless case for a distant off-diagonal term.
    pub snr_ratio: f64,
}

impl Default for FopdtEntry {
    fn default() -> Self {
        Self {
            gp: 0.0,
            tau: 1.0,
            theta: 0.0,
            rms_residual: 0.0,
            snr_ratio: 0.0,
        }
    }
}

impl FopdtEntry {
    /// Frequency response at `s = i*omega`, returned as `(real, imaginary)`.
    ///
    /// Evaluated in closed form rather than through a Padé approximation of the delay: the LMI
    /// synthesis only ever needs `P(i*omega)` at sampled frequencies, never a rational realisation,
    /// so the delay can be applied exactly as a phase rotation.
    pub fn freq_response(&self, omega: f64) -> (f64, f64) {
        // gp / (1 + i*omega*tau)
        let den = 1.0 + (omega * self.tau).powi(2);
        let (lag_re, lag_im) = (self.gp / den, -self.gp * omega * self.tau / den);
        // multiplied by exp(-i*omega*theta)
        let (s, c) = (-omega * self.theta).sin_cos();
        (lag_re * c - lag_im * s, lag_re * s + lag_im * c)
    }
}

/// The identified plant, plus the interaction diagnostics derived from it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MimoModel {
    /// `g[output][input]`, both indexed in physical zone order.
    pub g: [[FopdtEntry; ZONE_COUNT]; ZONE_COUNT],
    /// Setpoints the campaign ran at. The plant is not linear across its whole range — radiative
    /// loss and melt flow both change the gains — so a model is only valid near the operating
    /// point it was identified at, and that point has to travel with it.
    pub setpoints: [f64; ZONE_COUNT],
    /// Duty each zone was holding when the campaign froze the actuators.
    pub baseline_duty: [f64; ZONE_COUNT],
    /// Relative Gain Array of the DC gain matrix.
    pub rga: Mat<ZONE_COUNT>,
    /// Condition number of the DC gain matrix.
    pub condition_number: f64,
    /// Niederlinski index of the DC gain matrix.
    pub niederlinski: f64,
    pub identified_at: SystemTime,
}

impl MimoModel {
    /// Steady-state gain matrix, `G(0)`.
    pub fn dc_gain(&self) -> Mat<ZONE_COUNT> {
        let mut out = matrix::zeros::<ZONE_COUNT>();
        for i in 0..ZONE_COUNT {
            for j in 0..ZONE_COUNT {
                out[i][j] = self.g[i][j].gp;
            }
        }
        out
    }

    /// Recompute [`Self::rga`], [`Self::condition_number`] and [`Self::niederlinski`] from `g`.
    /// Called once when a campaign completes, and again after deserialising a stored model.
    pub fn refresh_diagnostics(&mut self) {
        let g0 = self.dc_gain();
        self.rga = matrix::rga(&g0).unwrap_or([[f64::NAN; ZONE_COUNT]; ZONE_COUNT]);
        self.condition_number = matrix::condition_number(&g0);
        self.niederlinski = matrix::niederlinski(&g0);
    }

    /// How far the RGA diagonal strays from unity. Zero means the zones are already decoupled and
    /// MIMO control has nothing to add; large values mean the decentralized loops are fighting.
    pub fn max_rga_deviation(&self) -> f64 {
        (0..ZONE_COUNT).fold(0.0_f64, |m, i| m.max((self.rga[i][i] - 1.0).abs()))
    }

    /// Strongest off-diagonal coupling, as a fraction of the driven zone's own gain.
    pub fn max_coupling_ratio(&self) -> f64 {
        let mut worst = 0.0_f64;
        for i in 0..ZONE_COUNT {
            for j in 0..ZONE_COUNT {
                if i == j {
                    continue;
                }
                let own = self.g[j][j].gp.abs();
                if own > 1e-12 {
                    worst = worst.max(self.g[i][j].gp.abs() / own);
                }
            }
        }
        worst
    }
}

/// Full matrix PID gains. `u = KP*e + KI*integral(e) + KD*d(measurement)/dt`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MimoGains {
    pub kp: Mat<ZONE_COUNT>,
    pub ki: Mat<ZONE_COUNT>,
    pub kd: Mat<ZONE_COUNT>,
    /// Derivative filter time constant, seconds. A non-zero `kd` without it turns the derivative
    /// term into a noise amplifier, so the two always travel together.
    pub derivative_filter_tc: f64,
}

impl Default for MimoGains {
    fn default() -> Self {
        Self {
            kp: matrix::zeros(),
            ki: matrix::zeros(),
            kd: matrix::zeros(),
            derivative_filter_tc: 0.0,
        }
    }
}

impl MimoGains {
    pub fn is_finite(&self) -> bool {
        matrix::all_finite(&self.kp)
            && matrix::all_finite(&self.ki)
            && matrix::all_finite(&self.kd)
            && self.derivative_filter_tc.is_finite()
    }
}

/// Why a synthesis backend declined to produce gains.
///
/// Refusing is the useful behaviour here: fragile gains derived from an untrustworthy model would
/// be applied to a live machine, whereas a named diagnostic tells the operator what to fix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SynthError {
    /// The DC gain matrix could not be inverted.
    SingularDcGain,
    /// `condition_number` exceeded the limit; the static inverse would amplify model error.
    IllConditioned { condition_number: i64, limit: i64 },
    /// Negative Niederlinski index: no integral controller can stabilise this pairing.
    StructurallyUnstable,
    /// An RGA diagonal element strayed too far from unity.
    BadPairing { zone: usize },
    /// A diagonal entry of the model is not physically sensible (non-positive gain, or a
    /// non-positive time constant).
    UnusableDiagonal { zone: usize },
    /// The IMC rules rejected the diagonal model.
    GainComputationFailed { zone: usize },
    /// Produced gains contained a non-finite value.
    NonFiniteGains,
    /// A synthesis backend failed for a reason of its own.
    SynthesisFailed(String),
}

impl std::fmt::Display for SynthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SingularDcGain => write!(
                f,
                "the measured DC gain matrix is singular, so the zones cannot be decoupled - \
                 re-run identification with a larger step"
            ),
            Self::IllConditioned {
                condition_number,
                limit,
            } => write!(
                f,
                "the measured DC gain matrix is ill-conditioned ({condition_number} against a \
                 limit of {limit}); decoupling it would amplify model error into the gains"
            ),
            Self::StructurallyUnstable => write!(
                f,
                "the Niederlinski index is negative: this zone pairing cannot be stabilised by \
                 any controller with integral action"
            ),
            Self::BadPairing { zone } => write!(
                f,
                "zone {zone} has an RGA diagonal element far from 1, so it is dominated by its \
                 neighbours rather than by its own heater"
            ),
            Self::UnusableDiagonal { zone } => write!(
                f,
                "zone {zone} has no usable self-response - its own heater did not measurably \
                 raise its own temperature"
            ),
            Self::GainComputationFailed { zone } => {
                write!(f, "the IMC rules rejected the fitted model for zone {zone}")
            }
            Self::NonFiniteGains => write!(f, "synthesis produced a non-finite gain"),
            Self::SynthesisFailed(reason) => write!(f, "{reason}"),
        }
    }
}

impl std::error::Error for SynthError {}

/// Turns an identified model into controller gains.
///
/// Kept behind a trait so a second backend can be slotted in without disturbing the identification
/// or runtime paths: both consume the same [`MimoModel`] and emit the same [`MimoGains`].
pub trait MimoSynthesis {
    fn synthesize(&self, model: &MimoModel) -> Result<MimoGains, SynthError>;
    /// Short identifier stored alongside the gains, so it is always possible to tell which method
    /// produced what is currently running.
    fn name(&self) -> &'static str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn freq_response_at_dc_is_the_process_gain() {
        let e = FopdtEntry {
            gp: 2.5,
            tau: 100.0,
            theta: 20.0,
            ..Default::default()
        };
        let (re, im) = e.freq_response(0.0);
        assert!((re - 2.5).abs() < 1e-12, "re {re}");
        assert!(im.abs() < 1e-12, "im {im}");
    }

    #[test]
    fn freq_response_rolls_off_and_lags() {
        let e = FopdtEntry {
            gp: 1.0,
            tau: 10.0,
            theta: 0.0,
            ..Default::default()
        };
        // At the corner frequency the magnitude is 1/sqrt(2) and the phase is -45 degrees.
        let (re, im) = e.freq_response(0.1);
        let mag = (re * re + im * im).sqrt();
        assert!(
            (mag - std::f64::consts::FRAC_1_SQRT_2).abs() < 1e-9,
            "mag {mag}"
        );
        assert!(
            (im.atan2(re) + std::f64::consts::FRAC_PI_4).abs() < 1e-9,
            "phase {}",
            im.atan2(re)
        );
    }

    #[test]
    fn dead_time_is_pure_phase() {
        // A delay must not change the magnitude at any frequency, only rotate it.
        let no_delay = FopdtEntry {
            gp: 1.5,
            tau: 30.0,
            theta: 0.0,
            ..Default::default()
        };
        let delayed = FopdtEntry {
            theta: 12.0,
            ..no_delay
        };
        for &w in &[1e-4, 1e-3, 1e-2, 1e-1] {
            let (ar, ai) = no_delay.freq_response(w);
            let (br, bi) = delayed.freq_response(w);
            let ma = (ar * ar + ai * ai).sqrt();
            let mb = (br * br + bi * bi).sqrt();
            assert!((ma - mb).abs() < 1e-12, "magnitude changed at w={w}");
            // And the phase must lag by exactly omega*theta.
            let dphase = bi.atan2(br) - ai.atan2(ar);
            let expected = -w * 12.0;
            assert!(
                (dphase - expected).abs() < 1e-9,
                "phase shift at w={w}: {dphase} vs {expected}"
            );
        }
    }

    #[test]
    fn coupling_ratio_is_zero_for_a_decoupled_plant() {
        let mut m = test_model(|i, j| if i == j { 1.0 } else { 0.0 });
        m.refresh_diagnostics();
        assert!(m.max_coupling_ratio() < 1e-12);
        assert!(m.max_rga_deviation() < 1e-12);
    }

    #[test]
    fn coupling_ratio_reports_the_strongest_neighbour() {
        let mut m = test_model(|i, j| {
            if i == j {
                2.0
            } else if i.abs_diff(j) == 1 {
                0.5
            } else {
                0.0
            }
        });
        m.refresh_diagnostics();
        assert!((m.max_coupling_ratio() - 0.25).abs() < 1e-12);
        assert!(m.max_rga_deviation() > 0.0);
    }

    pub(super) fn test_model(gp: impl Fn(usize, usize) -> f64) -> MimoModel {
        let mut g = [[FopdtEntry::default(); ZONE_COUNT]; ZONE_COUNT];
        for i in 0..ZONE_COUNT {
            for j in 0..ZONE_COUNT {
                g[i][j] = FopdtEntry {
                    gp: gp(i, j),
                    tau: 200.0,
                    theta: 20.0,
                    rms_residual: 0.0,
                    snr_ratio: 50.0,
                };
            }
        }
        MimoModel {
            g,
            setpoints: [200.0; ZONE_COUNT],
            baseline_duty: [0.3; ZONE_COUNT],
            rga: matrix::zeros(),
            condition_number: 1.0,
            niederlinski: 1.0,
            identified_at: SystemTime::UNIX_EPOCH,
        }
    }
}
