//! IMC (Internal Model Control) PID auto-tuner, a.k.a. Lambda tuning.
//!
//! Runs an open-loop step test on a self-regulating process, fits a first-order-plus-dead-time
//! (FOPDT) model to the recorded response, and derives PI and PID gains from it.
//!
//! The algorithm:
//! 1. Wait for the process to reach steady state while the existing controller still drives it.
//! 2. Freeze the actuator at the controller's current output and wait for it to settle *again*.
//!    This matters: a regulating controller masks drift, so a process that looks flat under
//!    closed-loop control may not be thermally steady. Stepping from a drifting baseline corrupts
//!    the dead-time estimate, and dead time sits in the denominator of the controller gain.
//! 3. Step the actuator by a fixed amount and record until the response settles.
//! 4. Fit `gp`, `tau`, `theta` by least squares over the whole recorded curve.
//! 5. Apply the IMC tuning rules.
//!
//! This tuner is generic over the physical quantity. The caller maps the returned actuator command
//! (in the same units as the configured step) onto real hardware, and is responsible for the
//! machine-level guards — mode, interlocks, sensor faults — which it turns into [`ImcTuner::abort`].
//!
//! # Ownership of the actuator
//!
//! [`ImcTuner::update`] returns `Some(command)` only while the tuner is actively driving, and
//! `None` in every other state including `Completed` and `Failed`. Callers apply that return value
//! every tick, which makes handing control back structural rather than a step that an abort path
//! could skip.

use std::time::{Duration, Instant};

/// Minimum samples in the step response before a fit is attempted.
const MIN_FIT_SAMPLES: usize = 20;
/// Samples the fit is decimated down to. A thermal process with a minutes-long time constant is
/// heavily oversampled at 1 Hz, so this loses nothing and keeps the grid search cheap. Three
/// parameters over 150 points is still a very overdetermined fit.
const FIT_TARGET_SAMPLES: usize = 150;
/// Grid divisions per axis, per refinement pass.
const FIT_GRID: usize = 16;
/// Refinement passes. Each shrinks the search span by `FIT_GRID / 2`, so three passes refine by
/// ~512x — sub-second resolution in dead time from a search that starts spanning minutes.
const FIT_PASSES: usize = 3;
/// Window averaged to obtain the steady-state value at each end of the step, in seconds.
const PLATEAU_WINDOW_SECONDS: f64 = 60.0;
/// Signal-to-noise floor below which a result is flagged low-confidence. From Dataforth an124:
/// the step should move the process variable at least five times the peak-to-peak noise.
pub const MIN_SNR_RATIO: f64 = 5.0;
/// Fit residual, as a percentage of the step size, above which the response is not credibly FOPDT.
///
/// Calibrated against the simulated cases: a clean first-order plant fits to well under 1%, and
/// even two equal lags in series — markedly higher-order — only reach ~1.5%, so this leaves roughly
/// a factor of two of headroom before warning.
///
/// This is a warning, not a rejection, and it is a weak detector for one specific case: a step
/// disturbance arriving *late* in the response is partly absorbed by inflating the fitted
/// amplitude, which biases the process gain more than it raises the residual. The trace overlay in
/// the UI is the real defence there.
pub const MAX_FIT_ERROR_PCT: f64 = 3.0;
/// Divisor applied to the derivative time to obtain a derivative filter time constant.
pub const DERIVATIVE_FILTER_DIVISOR: f64 = 10.0;

/// Configuration for an IMC auto-tune run.
#[derive(Debug, Clone)]
pub struct ImcTunerConfig {
    /// Step in actuator command, signed. Positive steps are the norm; a negative step is valid
    /// when there is no upward headroom, but a process without active cooling will not generally
    /// identify the same model in both directions, so do not mix results.
    pub step_duty: f64,
    /// Upper bound on the actuator command.
    pub max_duty: f64,
    /// Abort if the process variable moves further than this from its baseline.
    pub max_rise_celsius: f64,
    /// Closed-loop time constant as a multiple of the process time constant, i.e. how fast the
    /// tuned loop is asked to respond: a setpoint step reaches 63% at `theta + lambda_factor * tau`.
    /// Lower is faster and less robust. 1.0 is the textbook IMC value; the operator-facing presets
    /// run from 0.15 ("Extremely Aggressive") to 1.0 ("Conservative"), because these zones are
    /// strongly lag-dominant and the textbook value leaves them very sluggish against the
    /// cross-zone disturbances this machine actually has.
    pub lambda_factor: f64,
    /// Interval between recorded samples. Every call to [`ImcTuner::update`] is averaged into the
    /// pending sample, so the record is evenly spaced regardless of how fast the caller ticks.
    pub sample_period: Duration,
    /// Rolling window over which steady state is judged.
    pub steady_window: Duration,
    /// Steady state requires the least-squares slope over the window to be below this.
    pub steady_slope_c_per_min: f64,
    /// Steady state requires the peak-to-peak spread over the window to be below this.
    pub steady_band_celsius: f64,
    /// Steady state additionally requires the process variable to be this close to its setpoint,
    /// before the step only.
    pub setpoint_band_celsius: f64,
    /// Threshold used for the dead-time cross-check that is reported alongside the fitted value.
    pub dead_time_threshold_celsius: f64,
    pub waiting_timeout: Duration,
    pub baseline_timeout: Duration,
    pub step_timeout: Duration,
    pub max_duration: Duration,
}

impl Default for ImcTunerConfig {
    fn default() -> Self {
        Self {
            step_duty: 0.10,
            max_duty: 1.0,
            max_rise_celsius: 30.0,
            lambda_factor: 1.0,
            sample_period: Duration::from_secs(1),
            steady_window: Duration::from_secs(120),
            steady_slope_c_per_min: 0.15,
            steady_band_celsius: 1.0,
            setpoint_band_celsius: 3.0,
            dead_time_threshold_celsius: 1.0,
            waiting_timeout: Duration::from_secs(1200),
            baseline_timeout: Duration::from_secs(1800),
            step_timeout: Duration::from_secs(4800),
            max_duration: Duration::from_secs(7200),
        }
    }
}

/// One tuning candidate, in both IMC and parallel-PID parameterisations.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ImcGains {
    /// Controller gain
    pub kc: f64,
    /// Integral time, seconds
    pub ti: f64,
    /// Derivative time, seconds
    pub td: f64,
    pub kp: f64,
    pub ki: f64,
    pub kd: f64,
}

impl ImcGains {
    /// Derivative filter time constant to use alongside these gains. Applying a non-zero `kd`
    /// without it turns the derivative term into a noise amplifier.
    pub fn derivative_filter_tc(&self) -> f64 {
        if self.td > 0.0 {
            self.td / DERIVATIVE_FILTER_DIVISOR
        } else {
            0.0
        }
    }
}

/// Identified model, fit diagnostics, and both gain candidates.
#[derive(Debug, Clone, PartialEq)]
pub struct ImcTuneResult {
    /// Steady-state gain, process units per unit of actuator command.
    pub process_gain: f64,
    /// Fitted time constant, seconds.
    pub time_constant: f64,
    /// Fitted dead time, seconds.
    pub dead_time: f64,
    /// Classical 63.2% construction, reported as a cross-check only.
    pub tau_63: f64,
    /// Dead time from the first threshold crossing, reported as a cross-check only. Expect it to
    /// read above the fitted value: on a first-order response the first degree of movement takes
    /// `-tau * ln(1 - threshold/delta_pv)` to accumulate after the dead time has already elapsed.
    pub dead_time_threshold: f64,
    /// Root-mean-square fit residual, in process units.
    pub rms_residual: f64,
    /// Residual as a percentage of the step response size.
    pub fit_error_pct: f64,
    pub delta_pv: f64,
    pub delta_u: f64,
    /// Closed-loop time constant actually used.
    pub lambda: f64,
    /// Peak-to-peak noise measured while holding the baseline.
    pub noise_peak_to_peak: f64,
    /// `delta_pv / noise_peak_to_peak`, capped at a large finite value so it stays serialisable.
    pub snr_ratio: f64,
    /// Step that would have hit the target signal-to-noise ratio, to guide a retry.
    pub suggested_step_duty: f64,
    pub pi: ImcGains,
    pub pid: ImcGains,
}

impl ImcTuneResult {
    /// Whether the step moved the process variable far enough clear of the noise.
    pub fn is_confident(&self) -> bool {
        self.snr_ratio >= MIN_SNR_RATIO
    }

    /// Whether an FOPDT model actually describes the recorded response.
    pub fn is_good_fit(&self) -> bool {
        self.fit_error_pct <= MAX_FIT_ERROR_PCT
    }
}

/// One recorded point of the step test.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TraceSample {
    /// Seconds since the run started.
    pub t_seconds: f64,
    pub pv: f64,
    pub duty: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImcTunerPhase {
    Idle,
    WaitingForSteady,
    BaselineHold,
    Step,
    Completed,
    Failed,
}

impl ImcTunerPhase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::WaitingForSteady => "waiting_for_steady",
            Self::BaselineHold => "baseline_hold",
            Self::Step => "step",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }

    const fn is_driving(self) -> bool {
        matches!(self, Self::BaselineHold | Self::Step)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImcTunerError {
    /// The step would exceed the actuator limit. `available` is the remaining headroom.
    NoHeadroom { available: f64 },
    /// The step is zero, not finite, or larger than the actuator range.
    InvalidStep,
    /// A run is already in progress.
    AlreadyRunning,
}

#[derive(Debug)]
pub struct ImcTuner {
    config: ImcTunerConfig,
    phase: ImcTunerPhase,

    setpoint: f64,
    baseline_duty: f64,
    commanded_duty: f64,

    // Sample accumulator: every update() call folds into the pending sample.
    sample_sum: f64,
    sample_count: u32,
    next_sample_at: Instant,

    started_at: Instant,
    phase_started_at: Instant,
    /// Index into `trace` where the current phase began, so steady-state detection never reads
    /// across a phase boundary.
    phase_start_index: usize,
    /// Index into `trace` of the first sample after the step was applied.
    step_start_index: Option<usize>,

    trace: Vec<TraceSample>,

    baseline_pv: f64,
    noise_peak_to_peak: f64,

    result: Option<ImcTuneResult>,
    failure_reason: Option<&'static str>,
    /// Set whenever the phase changes, so the caller can emit state without polling.
    phase_changed: bool,
}

impl ImcTuner {
    pub fn new(config: ImcTunerConfig) -> Self {
        let now = Instant::now();
        Self {
            config,
            phase: ImcTunerPhase::Idle,
            setpoint: 0.0,
            baseline_duty: 0.0,
            commanded_duty: 0.0,
            sample_sum: 0.0,
            sample_count: 0,
            next_sample_at: now,
            started_at: now,
            phase_started_at: now,
            phase_start_index: 0,
            step_start_index: None,
            trace: Vec::new(),
            baseline_pv: 0.0,
            noise_peak_to_peak: 0.0,
            result: None,
            failure_reason: None,
            phase_changed: false,
        }
    }

    pub fn config(&self) -> &ImcTunerConfig {
        &self.config
    }

    /// Begin a run.
    ///
    /// `current_duty` is only used for an early headroom check so the operator gets immediate
    /// feedback; the authoritative baseline is captured later, when the process is confirmed steady.
    pub fn start(
        &mut self,
        now: Instant,
        current_duty: f64,
        setpoint: f64,
    ) -> Result<(), ImcTunerError> {
        if self.is_running() {
            return Err(ImcTunerError::AlreadyRunning);
        }
        let step = self.config.step_duty;
        if !step.is_finite() || step == 0.0 || step.abs() > self.config.max_duty {
            return Err(ImcTunerError::InvalidStep);
        }
        Self::check_headroom(current_duty, step, self.config.max_duty)?;

        self.setpoint = setpoint;
        self.baseline_duty = current_duty;
        self.commanded_duty = current_duty;
        self.sample_sum = 0.0;
        self.sample_count = 0;
        self.next_sample_at = now + self.config.sample_period;
        self.started_at = now;
        self.trace.clear();
        self.step_start_index = None;
        self.baseline_pv = 0.0;
        self.noise_peak_to_peak = 0.0;
        self.result = None;
        self.failure_reason = None;
        self.enter(ImcTunerPhase::WaitingForSteady, now);
        Ok(())
    }

    fn check_headroom(duty: f64, step: f64, max_duty: f64) -> Result<(), ImcTunerError> {
        let target = duty + step;
        if target > max_duty {
            return Err(ImcTunerError::NoHeadroom {
                available: max_duty - duty,
            });
        }
        if target < 0.0 {
            return Err(ImcTunerError::NoHeadroom { available: duty });
        }
        Ok(())
    }

    /// Abort the run. Safe to call in any state; only a running tuner changes phase.
    pub fn abort(&mut self, reason: &'static str) {
        if self.is_running() {
            self.failure_reason = Some(reason);
            let now = Instant::now();
            self.enter(ImcTunerPhase::Failed, now);
        }
    }

    /// Feed one observation and advance the state machine.
    ///
    /// `pid_duty` is the actuator command currently in force from the caller's own controller. It
    /// is recorded during the waiting phase and captured as the baseline when the process settles.
    ///
    /// Returns `Some(command)` while the tuner owns the actuator, `None` otherwise.
    pub fn update(&mut self, pv: f64, pid_duty: f64, now: Instant) -> Option<f64> {
        if !self.is_running() {
            return None;
        }

        if now.duration_since(self.started_at) > self.config.max_duration {
            self.failure_reason = Some("overall timeout");
            self.enter(ImcTunerPhase::Failed, now);
            return None;
        }

        let phase_timeout = match self.phase {
            ImcTunerPhase::WaitingForSteady => self.config.waiting_timeout,
            ImcTunerPhase::BaselineHold => self.config.baseline_timeout,
            _ => self.config.step_timeout,
        };
        if now.duration_since(self.phase_started_at) > phase_timeout {
            self.failure_reason = Some(match self.phase {
                ImcTunerPhase::WaitingForSteady => "process did not reach steady state in time",
                ImcTunerPhase::BaselineHold => "process did not settle at constant output in time",
                _ => "step response did not settle in time",
            });
            self.enter(ImcTunerPhase::Failed, now);
            return None;
        }

        if self.phase == ImcTunerPhase::Step
            && (pv - self.baseline_pv).abs() > self.config.max_rise_celsius
        {
            self.failure_reason = Some("process variable moved past the configured limit");
            self.enter(ImcTunerPhase::Failed, now);
            return None;
        }

        self.sample_sum += pv;
        self.sample_count += 1;

        if now >= self.next_sample_at {
            let mean = self.sample_sum / f64::from(self.sample_count.max(1));
            self.sample_sum = 0.0;
            self.sample_count = 0;

            // Advance on a fixed grid so samples stay evenly spaced; resynchronise if the caller
            // ever falls more than a whole period behind.
            let mut next = self.next_sample_at + self.config.sample_period;
            if next <= now {
                next = now + self.config.sample_period;
            }
            self.next_sample_at = next;

            let duty = if self.phase.is_driving() {
                self.commanded_duty
            } else {
                pid_duty
            };
            self.trace.push(TraceSample {
                t_seconds: now.duration_since(self.started_at).as_secs_f64(),
                pv: mean,
                duty,
            });

            self.on_new_sample(pid_duty, now);
        }

        self.command()
    }

    fn command(&self) -> Option<f64> {
        if self.phase.is_driving() {
            Some(self.commanded_duty)
        } else {
            None
        }
    }

    fn on_new_sample(&mut self, pid_duty: f64, now: Instant) {
        match self.phase {
            ImcTunerPhase::WaitingForSteady => {
                let Some(stats) = self.steady_stats(true) else {
                    return;
                };
                // Capture the authoritative baseline now that the process is confirmed settled.
                if Self::check_headroom(pid_duty, self.config.step_duty, self.config.max_duty)
                    .is_err()
                {
                    self.failure_reason = Some("not enough actuator headroom for the step");
                    self.enter(ImcTunerPhase::Failed, now);
                    return;
                }
                self.baseline_duty = pid_duty;
                self.commanded_duty = pid_duty;
                self.baseline_pv = stats.mean;
                self.enter(ImcTunerPhase::BaselineHold, now);
            }
            ImcTunerPhase::BaselineHold => {
                let Some(stats) = self.steady_stats(false) else {
                    return;
                };
                self.baseline_pv = stats.mean;
                // Peak-to-peak under constant output is the honest noise-and-disturbance figure:
                // no controller is suppressing anything at this point.
                self.noise_peak_to_peak = stats.peak_to_peak;
                self.commanded_duty = self.baseline_duty + self.config.step_duty;
                self.step_start_index = Some(self.trace.len());
                self.enter(ImcTunerPhase::Step, now);
            }
            ImcTunerPhase::Step => {
                if !self.step_response_settled() {
                    return;
                }
                match self.build_result() {
                    Some(result) => {
                        self.result = Some(result);
                        self.enter(ImcTunerPhase::Completed, now);
                    }
                    None => {
                        self.failure_reason = Some("could not fit a first-order model to the step");
                        self.enter(ImcTunerPhase::Failed, now);
                    }
                }
            }
            _ => {}
        }
    }

    /// The step is settled once the response is flat *and* has actually moved. Without the movement
    /// check the flat stretch during the dead time would immediately read as settled.
    fn step_response_settled(&self) -> bool {
        let elapsed = self.trace.len().saturating_sub(self.phase_start_index) as f64
            * self.config.sample_period.as_secs_f64();
        if elapsed < 2.0 * self.config.steady_window.as_secs_f64() {
            return false;
        }
        let Some(stats) = self.steady_stats(false) else {
            return false;
        };
        let moved = (stats.mean - self.baseline_pv).abs();
        moved >= (3.0 * self.noise_peak_to_peak).max(1.0)
    }

    fn steady_window_len(&self) -> usize {
        let n = self.config.steady_window.as_secs_f64() / self.config.sample_period.as_secs_f64();
        (n.round() as usize).max(2)
    }

    /// Statistics over the trailing steady-state window, or `None` if it is not steady yet or there
    /// is not enough data in the current phase.
    fn steady_stats(&self, require_setpoint_band: bool) -> Option<SteadyStats> {
        let n = self.steady_window_len();
        if self.trace.len() < self.phase_start_index + n {
            return None;
        }
        let window = &self.trace[self.trace.len() - n..];

        let mean = window.iter().map(|s| s.pv).sum::<f64>() / n as f64;
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        for s in window {
            min = min.min(s.pv);
            max = max.max(s.pv);
        }
        let peak_to_peak = max - min;

        // Least-squares slope over the window. A first-to-last difference would be dominated by
        // the noise on two individual samples.
        let x_mean = (n as f64 - 1.0) / 2.0;
        let mut num = 0.0;
        let mut den = 0.0;
        for (i, s) in window.iter().enumerate() {
            let dx = i as f64 - x_mean;
            num += dx * (s.pv - mean);
            den += dx * dx;
        }
        let slope_per_sample = if den > 0.0 { num / den } else { 0.0 };
        let slope_per_min = slope_per_sample / self.config.sample_period.as_secs_f64() * 60.0;

        if slope_per_min.abs() > self.config.steady_slope_c_per_min {
            return None;
        }
        if peak_to_peak > self.config.steady_band_celsius {
            return None;
        }
        if require_setpoint_band && (mean - self.setpoint).abs() > self.config.setpoint_band_celsius
        {
            return None;
        }

        Some(SteadyStats { mean, peak_to_peak })
    }

    fn build_result(&self) -> Option<ImcTuneResult> {
        let fit = self.fit_fopdt()?;
        let delta_u = self.config.step_duty;
        let gp = fit.amplitude / delta_u;
        if !gp.is_finite() || gp <= 0.0 || fit.tau <= 0.0 {
            return None;
        }

        let (lambda, pi, pid) = compute_gains(gp, fit.tau, fit.theta, self.config.lambda_factor)?;

        // These ratios must stay finite: serde_json renders a non-finite float as `null`, which
        // would fail schema validation on the client.
        const RATIO_CEILING: f64 = 9999.0;
        let noise = self.noise_peak_to_peak;
        let amplitude = fit.amplitude.abs();
        let snr_ratio = (amplitude / noise.max(1e-9)).min(RATIO_CEILING);
        let suggested_step_duty = if snr_ratio > 0.0 {
            delta_u * MIN_SNR_RATIO / snr_ratio
        } else {
            delta_u
        };
        let fit_error_pct = (100.0 * fit.rms_residual / amplitude.max(1e-9)).min(RATIO_CEILING);

        Some(ImcTuneResult {
            process_gain: gp,
            time_constant: fit.tau,
            dead_time: fit.theta,
            tau_63: fit.tau_63,
            dead_time_threshold: fit.theta_threshold,
            rms_residual: fit.rms_residual,
            fit_error_pct,
            delta_pv: fit.amplitude,
            delta_u,
            lambda,
            noise_peak_to_peak: noise,
            snr_ratio,
            suggested_step_duty,
            pi,
            pid,
        })
    }

    /// Least-squares FOPDT fit over the recorded step response.
    ///
    /// The model is `y(t) = A * (1 - exp(-(t - theta) / tau))` for `t >= theta`, zero before. `A` is
    /// linear given `(theta, tau)`, so only two parameters are searched and the amplitude falls out
    /// in closed form at each grid point.
    ///
    /// Cost is kept off the caller's control loop by decimating first, exploiting the uniform
    /// sample spacing so `exp` is evaluated once per candidate `tau` rather than once per sample,
    /// and refining coarse-to-fine. It runs exactly once, at the end of a run.
    fn fit_fopdt(&self) -> Option<Fopdt> {
        let start = self.step_start_index?;
        let raw: Vec<f64> = self.trace[start..]
            .iter()
            .map(|s| s.pv - self.baseline_pv)
            .collect();
        if raw.len() < MIN_FIT_SAMPLES {
            return None;
        }
        let sample_dt = self.config.sample_period.as_secs_f64();

        let stride = (raw.len() / FIT_TARGET_SAMPLES).max(1);
        let ys: Vec<f64> = raw.iter().step_by(stride).copied().collect();
        let dt = sample_dt * stride as f64;
        let n = ys.len();
        if n < 10 {
            return None;
        }

        // Plateau value, averaged over the tail, used only to bracket the search.
        let tail = ((PLATEAU_WINDOW_SECONDS / dt).round() as usize).clamp(1, n);
        let y_final = ys[n - tail..].iter().sum::<f64>() / tail as f64;
        if y_final.abs() < 1e-9 {
            return None;
        }

        let target = 0.632 * y_final;
        let crossed = ys.iter().position(|&y| {
            if y_final > 0.0 {
                y >= target
            } else {
                y <= target
            }
        });
        let t63 = crossed
            .map(|i| i as f64 * dt)
            .unwrap_or(n as f64 * dt / 2.0)
            .max(dt);

        let sum_yy: f64 = ys.iter().map(|y| y * y).sum();

        let mut theta_lo = 0.0;
        let mut theta_hi = 0.5 * t63;
        let mut tau_lo = 0.2 * t63;
        let mut tau_hi = 3.0 * t63;
        let mut exp_buf = vec![0.0_f64; n];
        let mut best = GridBest::default();

        for _ in 0..FIT_PASSES {
            best = search_grid(
                &ys,
                dt,
                theta_lo,
                theta_hi,
                tau_lo,
                tau_hi,
                FIT_GRID,
                &mut exp_buf,
            )?;
            let dtheta = (theta_hi - theta_lo) / FIT_GRID as f64;
            let dtau = (tau_hi - tau_lo) / FIT_GRID as f64;
            theta_lo = (best.theta - dtheta).max(0.0);
            theta_hi = best.theta + dtheta;
            tau_lo = (best.tau - dtau).max(dt / 10.0);
            tau_hi = best.tau + dtau;
        }

        let err = (sum_yy - best.metric).max(0.0);
        let rms_residual = (err / n as f64).sqrt();

        // Cross-check: dead time from the first threshold crossing, on the undecimated record.
        let threshold = self.config.dead_time_threshold_celsius;
        let theta_threshold = raw
            .iter()
            .position(|&y| {
                if y_final > 0.0 {
                    y >= threshold
                } else {
                    y <= -threshold
                }
            })
            .map_or(0.0, |i| i as f64 * sample_dt);

        Some(Fopdt {
            amplitude: best.amplitude,
            tau: best.tau,
            theta: best.theta,
            tau_63: (t63 - best.theta).max(0.0),
            theta_threshold,
            rms_residual,
        })
    }

    pub fn phase(&self) -> &'static str {
        self.phase.as_str()
    }

    pub const fn phase_enum(&self) -> ImcTunerPhase {
        self.phase
    }

    pub const fn is_running(&self) -> bool {
        matches!(
            self.phase,
            ImcTunerPhase::WaitingForSteady | ImcTunerPhase::BaselineHold | ImcTunerPhase::Step
        )
    }

    pub const fn is_completed(&self) -> bool {
        matches!(self.phase, ImcTunerPhase::Completed)
    }

    pub const fn is_failed(&self) -> bool {
        matches!(self.phase, ImcTunerPhase::Failed)
    }

    pub const fn result(&self) -> Option<&ImcTuneResult> {
        self.result.as_ref()
    }

    pub fn trace(&self) -> &[TraceSample] {
        &self.trace
    }

    pub const fn failure_reason(&self) -> Option<&'static str> {
        self.failure_reason
    }

    pub const fn baseline_duty(&self) -> f64 {
        self.baseline_duty
    }

    pub const fn baseline_pv(&self) -> f64 {
        self.baseline_pv
    }

    pub fn commanded_duty(&self) -> f64 {
        self.command().unwrap_or(0.0)
    }

    pub fn elapsed_seconds(&self, now: Instant) -> f64 {
        if self.phase == ImcTunerPhase::Idle {
            0.0
        } else {
            now.saturating_duration_since(self.started_at).as_secs_f64()
        }
    }

    /// Consume the "phase has changed" edge. Lets the caller emit state on transitions without
    /// tracking the previous phase itself.
    pub const fn take_phase_changed(&mut self) -> bool {
        let changed = self.phase_changed;
        self.phase_changed = false;
        changed
    }

    /// Progress across the whole run, 0-100. Each phase occupies a band and advances by elapsed
    /// time against its own timeout, since none of the phases has a knowable duration up front.
    pub fn progress_percent(&self, now: Instant) -> f64 {
        let fraction = |timeout: Duration| {
            let elapsed = now
                .saturating_duration_since(self.phase_started_at)
                .as_secs_f64();
            (elapsed / timeout.as_secs_f64().max(1e-9)).clamp(0.0, 1.0)
        };
        match self.phase {
            ImcTunerPhase::Idle | ImcTunerPhase::Failed => 0.0,
            ImcTunerPhase::Completed => 100.0,
            ImcTunerPhase::WaitingForSteady => 15.0 * fraction(self.config.waiting_timeout),
            ImcTunerPhase::BaselineHold => 15.0 + 20.0 * fraction(self.config.baseline_timeout),
            ImcTunerPhase::Step => (35.0 + 64.0 * fraction(self.config.step_timeout)).min(99.0),
        }
    }

    fn enter(&mut self, phase: ImcTunerPhase, now: Instant) {
        self.phase = phase;
        self.phase_started_at = now;
        self.phase_start_index = self.trace.len();
        self.phase_changed = true;
        if !phase.is_driving() {
            self.commanded_duty = 0.0;
        }
    }
}

struct SteadyStats {
    mean: f64,
    peak_to_peak: f64,
}

struct Fopdt {
    amplitude: f64,
    tau: f64,
    theta: f64,
    tau_63: f64,
    theta_threshold: f64,
    rms_residual: f64,
}

#[derive(Default, Clone, Copy)]
struct GridBest {
    theta: f64,
    tau: f64,
    amplitude: f64,
    /// `sum_yb^2 / sum_bb`. Maximising this minimises the squared residual, because
    /// `err = sum_yy - sum_yb^2 / sum_bb` once the amplitude is solved in closed form.
    metric: f64,
}

#[allow(clippy::too_many_arguments)]
fn search_grid(
    ys: &[f64],
    dt: f64,
    theta_lo: f64,
    theta_hi: f64,
    tau_lo: f64,
    tau_hi: f64,
    grid: usize,
    exp_buf: &mut [f64],
) -> Option<GridBest> {
    let n = ys.len();
    let mut best: Option<GridBest> = None;

    for j in 0..=grid {
        let tau = tau_lo + (tau_hi - tau_lo) * j as f64 / grid as f64;
        if tau <= 0.0 {
            continue;
        }

        // exp(-t_i / tau) is a geometric sequence for uniformly spaced samples, so one exp() here
        // replaces one per sample.
        let r = (-dt / tau).exp();
        let mut e = 1.0;
        for slot in exp_buf.iter_mut().take(n) {
            *slot = e;
            e *= r;
        }

        for i in 0..=grid {
            let theta = theta_lo + (theta_hi - theta_lo) * i as f64 / grid as f64;
            let ratio = theta / tau;
            if ratio > 50.0 {
                continue; // exp would overflow; such a model cannot fit anyway
            }
            // exp(-(t - theta)/tau) = exp(-t/tau) * exp(theta/tau)
            let g = ratio.exp();

            // Samples before the dead time contribute nothing; skip straight past them rather
            // than testing every one.
            let k0 = (theta / dt).ceil() as usize;
            let mut sum_yb = 0.0;
            let mut sum_bb = 0.0;
            for k in k0..n {
                let b = 1.0 - g * exp_buf[k];
                sum_yb += ys[k] * b;
                sum_bb += b * b;
            }

            if sum_bb > 1e-12 {
                let metric = sum_yb * sum_yb / sum_bb;
                if best.is_none_or(|b| metric > b.metric) {
                    best = Some(GridBest {
                        theta,
                        tau,
                        amplitude: sum_yb / sum_bb,
                        metric,
                    });
                }
            }
        }
    }

    best
}

/// IMC tuning rules for an FOPDT model. Returns the closed-loop time constant actually used plus
/// the PI and PID candidates.
///
/// The closed-loop time constant is `lambda_factor * tau`, floored for robustness at `0.8 * theta`
/// (Rivera) and at `0.15 * tau`.
///
/// The `tau` floor is the lower end of the operator-facing response-speed presets, deliberately:
/// it exists to stop a hand-supplied `lambda_factor` from asking for a closed loop far faster than
/// the plant, not to clip the presets. Raising it above 0.15 would silently collapse the fastest
/// settings onto each other — the same `tau_c`, the same gains, under different labels.
///
/// The PI integral time carries Skogestad's SIMC cap, `Ti = min(tau, 4 * (tau_c + theta))`. Plain
/// IMC sets `Ti = tau`, and on a lag-dominant process that makes disturbance recovery very slow —
/// the one drawback the method is usually charged with. The cap only binds on the faster settings.
pub fn compute_gains(
    gp: f64,
    tau: f64,
    theta: f64,
    lambda_factor: f64,
) -> Option<(f64, ImcGains, ImcGains)> {
    if !(gp.is_finite() && tau.is_finite() && theta.is_finite()) || gp <= 0.0 || tau <= 0.0 {
        return None;
    }
    let theta = theta.max(0.0);
    let tau_c = (lambda_factor * tau).max(0.8 * theta).max(0.15 * tau);

    let kc_pi = tau / (gp * (tau_c + theta));
    let ti_pi = tau.min(4.0 * (tau_c + theta));
    let pi = ImcGains {
        kc: kc_pi,
        ti: ti_pi,
        td: 0.0,
        kp: kc_pi,
        ki: if ti_pi > 0.0 { kc_pi / ti_pi } else { 0.0 },
        kd: 0.0,
    };

    let kc_pid = (2.0 * tau + theta) / (gp * (2.0 * tau_c + theta));
    let ti_pid = tau + theta / 2.0;
    let td_pid = if 2.0 * tau + theta > 0.0 {
        tau * theta / (2.0 * tau + theta)
    } else {
        0.0
    };
    let pid = ImcGains {
        kc: kc_pid,
        ti: ti_pid,
        td: td_pid,
        kp: kc_pid,
        ki: if ti_pid > 0.0 { kc_pid / ti_pid } else { 0.0 },
        kd: kc_pid * td_pid,
    };

    if !(pi.kp.is_finite() && pi.ki.is_finite() && pid.kp.is_finite() && pid.ki.is_finite()) {
        return None;
    }

    Some((tau_c, pi, pid))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Discrete FOPDT plant: a first-order lag behind a transport delay.
    struct Plant {
        gain: f64,
        tau: f64,
        theta: f64,
        dt: f64,
        y: f64,
        delay: Vec<f64>,
        noise: f64,
        seed: u64,
    }

    impl Plant {
        fn new(gain: f64, tau: f64, theta: f64, dt: f64) -> Self {
            let slots = (theta / dt).round().max(0.0) as usize;
            Self {
                gain,
                tau,
                theta,
                dt,
                y: 0.0,
                delay: vec![0.0; slots],
                noise: 0.0,
                seed: 0x2545_F491_4F6C_DD1D,
            }
        }

        fn with_noise(mut self, amplitude: f64) -> Self {
            self.noise = amplitude;
            self
        }

        fn step(&mut self, u: f64) -> f64 {
            let delayed = if self.delay.is_empty() {
                u
            } else {
                self.delay.rotate_right(1);
                let out = self.delay[0];
                self.delay[0] = u;
                out
            };
            let alpha = self.dt / (self.tau + self.dt);
            self.y += alpha * (self.gain * delayed - self.y);
            self.y + self.dither()
        }

        /// Deterministic pseudo-noise, so failures reproduce.
        fn dither(&mut self) -> f64 {
            if self.noise == 0.0 {
                return 0.0;
            }
            self.seed ^= self.seed << 13;
            self.seed ^= self.seed >> 7;
            self.seed ^= self.seed << 17;
            let unit = (self.seed >> 11) as f64 / (1_u64 << 53) as f64; // 0..1
            self.noise * (unit * 2.0 - 1.0)
        }
    }

    fn config(step: f64) -> ImcTunerConfig {
        ImcTunerConfig {
            step_duty: step,
            // Keep the phases short so tests stay fast; the algorithm does not care.
            steady_window: Duration::from_secs(30),
            waiting_timeout: Duration::from_secs(3600),
            baseline_timeout: Duration::from_secs(3600),
            step_timeout: Duration::from_secs(20_000),
            max_duration: Duration::from_secs(40_000),
            max_rise_celsius: 1000.0,
            ..Default::default()
        }
    }

    /// Run a whole tune against a simulated plant held at `setpoint` by a constant baseline duty.
    fn run(
        cfg: ImcTunerConfig,
        plant: &mut Plant,
        baseline_duty: f64,
        max_seconds: u64,
    ) -> ImcTuner {
        let mut tuner = ImcTuner::new(cfg);
        let t0 = Instant::now();
        let setpoint = plant.gain * baseline_duty;
        // Pre-charge the plant to its steady state at the baseline duty.
        plant.y = setpoint;
        for slot in plant.delay.iter_mut() {
            *slot = baseline_duty;
        }

        tuner.start(t0, baseline_duty, setpoint).expect("start");

        let dt = plant.dt;
        let mut u = baseline_duty;
        let steps = (max_seconds as f64 / dt) as u64;
        for i in 0..steps {
            let now = t0 + Duration::from_secs_f64(i as f64 * dt);
            let pv = plant.step(u);
            match tuner.update(pv, baseline_duty, now) {
                Some(cmd) => u = cmd,
                None => {
                    if !tuner.is_running() {
                        break;
                    }
                    u = baseline_duty;
                }
            }
        }
        tuner
    }

    #[test]
    fn identifies_a_lag_dominant_plant() {
        let mut plant = Plant::new(250.0, 300.0, 20.0, 1.0);
        let tuner = run(config(0.10), &mut plant, 0.40, 20_000);

        assert!(tuner.is_completed(), "phase={}", tuner.phase());
        let r = tuner.result().expect("result");
        assert!(
            (r.process_gain - 250.0).abs() / 250.0 < 0.05,
            "gp={}",
            r.process_gain
        );
        assert!(
            (r.time_constant - 300.0).abs() / 300.0 < 0.10,
            "tau={}",
            r.time_constant
        );
        assert!((r.dead_time - 20.0).abs() < 10.0, "theta={}", r.dead_time);
        assert!(r.is_good_fit(), "fit_error_pct={}", r.fit_error_pct);
    }

    #[test]
    fn identifies_a_delay_dominant_plant() {
        let mut plant = Plant::new(200.0, 60.0, 60.0, 1.0);
        let tuner = run(config(0.10), &mut plant, 0.40, 20_000);

        assert!(tuner.is_completed(), "phase={}", tuner.phase());
        let r = tuner.result().expect("result");
        assert!(
            (r.process_gain - 200.0).abs() / 200.0 < 0.05,
            "gp={}",
            r.process_gain
        );
        assert!(
            (r.time_constant - 60.0).abs() / 60.0 < 0.20,
            "tau={}",
            r.time_constant
        );
        assert!((r.dead_time - 60.0).abs() < 15.0, "theta={}", r.dead_time);
    }

    #[test]
    fn survives_measurement_noise() {
        let mut plant = Plant::new(250.0, 300.0, 20.0, 1.0).with_noise(0.2);
        let tuner = run(config(0.10), &mut plant, 0.40, 20_000);

        assert!(tuner.is_completed(), "phase={}", tuner.phase());
        let r = tuner.result().expect("result");
        assert!(
            (r.process_gain - 250.0).abs() / 250.0 < 0.10,
            "gp={}",
            r.process_gain
        );
        assert!(
            (r.time_constant - 300.0).abs() / 300.0 < 0.20,
            "tau={}",
            r.time_constant
        );
        // Noise must not push a legitimate run past the fit-quality warning.
        assert!(r.is_good_fit(), "fit_error_pct={}", r.fit_error_pct);
    }

    /// The property the least-squares fit buys over a threshold method: dead time is a property of
    /// the plant, so it must not move when the step size does.
    #[test]
    fn dead_time_is_not_biased_by_step_size() {
        let mut small_plant = Plant::new(250.0, 300.0, 20.0, 1.0);
        let small = run(config(0.05), &mut small_plant, 0.40, 20_000);
        let mut large_plant = Plant::new(250.0, 300.0, 20.0, 1.0);
        let large = run(config(0.20), &mut large_plant, 0.40, 20_000);

        let a = small.result().expect("small result");
        let b = large.result().expect("large result");
        assert!(
            (a.dead_time - b.dead_time).abs() < 8.0,
            "theta moved with step size: {} vs {}",
            a.dead_time,
            b.dead_time
        );
        // The threshold cross-check, by contrast, is expected to move — and to read high.
        assert!(
            a.dead_time_threshold > a.dead_time,
            "threshold {} should exceed fitted {}",
            a.dead_time_threshold,
            a.dead_time
        );
    }

    /// Two lags in series still fit FOPDT well — the apparent dead time absorbs the second lag.
    /// That is the whole reason FOPDT is the standard model for thermal processes, so this must
    /// *pass* the fit check rather than trip it.
    #[test]
    fn a_mildly_higher_order_plant_still_fits() {
        struct TwoLag {
            a: f64,
            b: f64,
            gain: f64,
            tau: f64,
            dt: f64,
        }
        impl TwoLag {
            fn step(&mut self, u: f64) -> f64 {
                let alpha = self.dt / (self.tau + self.dt);
                self.a += alpha * (self.gain * u - self.a);
                self.b += alpha * (self.a - self.b);
                self.b
            }
        }

        let mut plant = TwoLag {
            a: 0.0,
            b: 0.0,
            gain: 250.0,
            tau: 150.0,
            dt: 1.0,
        };
        let baseline = 0.40;
        plant.a = plant.gain * baseline;
        plant.b = plant.a;
        let setpoint = plant.b;

        let mut tuner = ImcTuner::new(config(0.10));
        let t0 = Instant::now();
        tuner.start(t0, baseline, setpoint).expect("start");
        let mut u = baseline;
        for i in 0..20_000 {
            let now = t0 + Duration::from_secs(i);
            let pv = plant.step(u);
            match tuner.update(pv, baseline, now) {
                Some(cmd) => u = cmd,
                None => {
                    if !tuner.is_running() {
                        break;
                    }
                }
            }
        }

        let r = tuner.result().expect("result");
        assert!(
            r.is_good_fit(),
            "FOPDT should absorb a second lag, fit_error_pct={}",
            r.fit_error_pct
        );
        // The second lag shows up as extra apparent dead time, not as a bad fit.
        assert!(r.dead_time > 20.0, "theta={}", r.dead_time);
    }

    /// What the residual is really for: a disturbance partway through the step — a neighbour zone
    /// moving, say — is not something an FOPDT step response can express, and must be flagged.
    #[test]
    fn fit_error_flags_a_mid_run_disturbance() {
        let mut plant = Plant::new(250.0, 300.0, 20.0, 1.0);
        let baseline = 0.40;
        let setpoint = plant.gain * baseline;
        plant.y = setpoint;
        for slot in plant.delay.iter_mut() {
            *slot = baseline;
        }

        let mut tuner = ImcTuner::new(config(0.10));
        let t0 = Instant::now();
        tuner.start(t0, baseline, setpoint).expect("start");

        let mut u = baseline;
        let mut driving_since: Option<u64> = None;
        for i in 0..20_000_u64 {
            let now = t0 + Duration::from_secs(i);
            let mut pv = plant.step(u);
            // Once the step has been running a while, superimpose a sustained offset.
            if let Some(start) = driving_since {
                if i > start + 400 {
                    pv += 6.0;
                }
            }
            match tuner.update(pv, baseline, now) {
                Some(cmd) => {
                    if cmd != baseline && driving_since.is_none() {
                        driving_since = Some(i);
                    }
                    u = cmd;
                }
                None => {
                    if !tuner.is_running() {
                        break;
                    }
                }
            }
        }

        let r = tuner.result().expect("result");
        assert!(
            !r.is_good_fit(),
            "a mid-run disturbance should trip the fit check, fit_error_pct={}",
            r.fit_error_pct
        );
    }

    #[test]
    fn fit_is_fast_enough_for_the_control_loop() {
        let mut plant = Plant::new(250.0, 300.0, 20.0, 1.0);
        let tuner = run(config(0.10), &mut plant, 0.40, 20_000);
        assert!(tuner.is_completed());
        assert!(
            tuner.trace().len() > 1000,
            "expected a long trace, got {}",
            tuner.trace().len()
        );

        let start = Instant::now();
        let fit = tuner.fit_fopdt().expect("fit");
        let elapsed = start.elapsed();
        assert!(fit.tau > 0.0);
        assert!(
            elapsed < Duration::from_millis(5),
            "fit took {elapsed:?}; it runs inside the control loop"
        );
    }

    #[test]
    fn gains_match_the_closed_form() {
        let (tau, theta, gp) = (300.0, 20.0, 250.0);
        let (lambda, pi, pid) = compute_gains(gp, tau, theta, 1.0).expect("gains");

        assert!((lambda - 300.0).abs() < 1e-9);
        assert!((pi.kc - tau / (gp * (lambda + theta))).abs() < 1e-12);
        assert!((pi.ti - tau).abs() < 1e-9, "SIMC cap should not bind here");
        assert_eq!(pi.td, 0.0);
        assert_eq!(pi.kd, 0.0);

        assert!((pid.kc - (2.0 * tau + theta) / (gp * (2.0 * lambda + theta))).abs() < 1e-12);
        assert!((pid.ti - (tau + theta / 2.0)).abs() < 1e-9);
        assert!((pid.td - tau * theta / (2.0 * tau + theta)).abs() < 1e-9);
        assert!((pid.derivative_filter_tc() - pid.td / 10.0).abs() < 1e-12);
    }

    #[test]
    fn simc_cap_binds_only_on_the_aggressive_setting() {
        let (tau, theta, gp) = (300.0, 5.0, 250.0);
        let (_, aggressive, _) = compute_gains(gp, tau, theta, 0.15).expect("gains");
        let (_, moderate, _) = compute_gains(gp, tau, theta, 1.0).expect("gains");

        assert!(aggressive.ti < tau, "cap should bind, ti={}", aggressive.ti);
        assert!(
            (moderate.ti - tau).abs() < 1e-9,
            "cap should not bind, ti={}",
            moderate.ti
        );
    }

    /// Every response-speed preset the UI offers must land on a distinct, monotonically increasing
    /// `tau_c` for a representative zone. A robustness floor set above the fastest preset would
    /// silently give two differently-labelled settings identical gains, which reads as a broken
    /// control rather than as a floor doing its job.
    #[test]
    fn the_response_speed_presets_are_all_distinct() {
        // Representative extruder zone: lag-dominant, tau/theta = 10.
        let (tau, theta, gp) = (150.0, 15.0, 93.5);
        let presets = [0.15, 0.25, 0.3, 0.5, 1.0];

        let mut previous: Option<(f64, f64)> = None;
        for factor in presets {
            let (lambda, pi, _) = compute_gains(gp, tau, theta, factor).expect("gains");
            assert!(
                (lambda - factor * tau).abs() < 1e-9,
                "lambda_factor {factor} was clipped by a floor: tau_c={lambda}"
            );
            if let Some((prev_lambda, prev_kc)) = previous {
                assert!(lambda > prev_lambda, "tau_c must increase with the factor");
                assert!(pi.kc < prev_kc, "a slower setting must give a smaller Kc");
            }
            previous = Some((lambda, pi.kc));
        }
    }

    #[test]
    fn pid_form_collapses_onto_pi_without_dead_time() {
        let (_, pi, pid) = compute_gains(250.0, 300.0, 0.0, 1.0).expect("gains");
        assert!((pi.kc - pid.kc).abs() < 1e-12);
        assert!((pi.ti - pid.ti).abs() < 1e-9);
        assert_eq!(pid.td, 0.0);
    }

    #[test]
    fn snr_and_suggested_step_are_reported() {
        let mut plant = Plant::new(250.0, 300.0, 20.0, 1.0).with_noise(0.3);
        // A deliberately tiny step: delta_pv lands near the noise floor.
        let tuner = run(config(0.01), &mut plant, 0.40, 30_000);
        let r = tuner.result().expect("result");

        assert!(!r.is_confident(), "snr_ratio={}", r.snr_ratio);
        assert!(
            r.suggested_step_duty > r.delta_u,
            "should suggest a larger step: {} vs {}",
            r.suggested_step_duty,
            r.delta_u
        );
    }

    /// Every reported number must be finite. serde_json renders infinity and NaN as `null`, which
    /// fails schema validation on the client — and a noiseless simulated plant is exactly the case
    /// that produces a division by zero in the signal-to-noise ratio.
    #[test]
    fn all_reported_values_are_finite_without_noise() {
        let mut plant = Plant::new(250.0, 300.0, 20.0, 1.0);
        let tuner = run(config(0.10), &mut plant, 0.40, 20_000);
        let r = tuner.result().expect("result");

        assert_eq!(r.noise_peak_to_peak, 0.0, "precondition: a noiseless plant");
        for (name, value) in [
            ("process_gain", r.process_gain),
            ("time_constant", r.time_constant),
            ("dead_time", r.dead_time),
            ("tau_63", r.tau_63),
            ("dead_time_threshold", r.dead_time_threshold),
            ("rms_residual", r.rms_residual),
            ("fit_error_pct", r.fit_error_pct),
            ("delta_pv", r.delta_pv),
            ("lambda", r.lambda),
            ("snr_ratio", r.snr_ratio),
            ("suggested_step_duty", r.suggested_step_duty),
            ("pi.kp", r.pi.kp),
            ("pi.ki", r.pi.ki),
            ("pi.kd", r.pi.kd),
            ("pid.kp", r.pid.kp),
            ("pid.ki", r.pid.ki),
            ("pid.kd", r.pid.kd),
        ] {
            assert!(value.is_finite(), "{name} was not finite: {value}");
        }
    }

    #[test]
    fn steady_detection_rejects_a_ramp() {
        let cfg = config(0.10);
        let mut tuner = ImcTuner::new(cfg);
        let t0 = Instant::now();
        tuner.start(t0, 0.4, 100.0).expect("start");

        // 0.5 C/min ramp, well above the 0.15 C/min threshold.
        for i in 0..600_u64 {
            let now = t0 + Duration::from_secs(i);
            tuner.update(100.0 + i as f64 * 0.5 / 60.0, 0.4, now);
        }
        assert_eq!(
            tuner.phase(),
            "waiting_for_steady",
            "a ramp must not read as steady"
        );
    }

    #[test]
    fn steady_detection_rejects_an_oscillation() {
        let cfg = config(0.10);
        let mut tuner = ImcTuner::new(cfg);
        let t0 = Instant::now();
        tuner.start(t0, 0.4, 100.0).expect("start");

        // Zero net slope, but a 4 C peak-to-peak swing inside the window. The slope test alone
        // would pass this; the peak-to-peak test is what rejects it.
        //
        // Note the inherent limit of any windowed detector: an oscillation much slower than the
        // window looks locally flat near its turning points and will pass. The production window
        // is 120 s, so periods beyond a few minutes are not caught here — they are caught by the
        // BaselineHold phase, which has to stay steady for a second full window before stepping.
        for i in 0..600_u64 {
            let now = t0 + Duration::from_secs(i);
            let pv = 100.0 + 2.0 * (i as f64 / 3.0).sin();
            tuner.update(pv, 0.4, now);
        }
        assert_eq!(
            tuner.phase(),
            "waiting_for_steady",
            "an oscillation must not read as steady"
        );
    }

    #[test]
    fn start_rejects_a_step_without_headroom() {
        let mut cfg = config(0.10);
        cfg.max_duty = 0.95; // the nozzle's limit
        let mut tuner = ImcTuner::new(cfg);
        let err = tuner.start(Instant::now(), 0.90, 200.0).unwrap_err();
        assert_eq!(
            err,
            ImcTunerError::NoHeadroom {
                available: 0.95 - 0.90
            }
        );
        assert_eq!(tuner.phase(), "idle");
    }

    #[test]
    fn start_rejects_an_invalid_step() {
        let mut tuner = ImcTuner::new(config(0.0));
        assert_eq!(
            tuner.start(Instant::now(), 0.4, 200.0).unwrap_err(),
            ImcTunerError::InvalidStep
        );
    }

    #[test]
    fn start_rejects_a_second_run() {
        let mut tuner = ImcTuner::new(config(0.10));
        let t0 = Instant::now();
        tuner.start(t0, 0.4, 200.0).expect("first start");
        assert_eq!(
            tuner.start(t0, 0.4, 200.0).unwrap_err(),
            ImcTunerError::AlreadyRunning
        );
    }

    #[test]
    fn aborts_when_the_process_variable_runs_away() {
        let mut cfg = config(0.10);
        cfg.max_rise_celsius = 10.0;
        let mut plant = Plant::new(250.0, 300.0, 20.0, 1.0);
        let tuner = run(cfg, &mut plant, 0.40, 20_000);

        assert!(tuner.is_failed(), "phase={}", tuner.phase());
        assert_eq!(
            tuner.failure_reason(),
            Some("process variable moved past the configured limit")
        );
    }

    #[test]
    fn aborts_on_phase_timeout() {
        let mut cfg = config(0.10);
        cfg.waiting_timeout = Duration::from_secs(60);
        let mut tuner = ImcTuner::new(cfg);
        let t0 = Instant::now();
        tuner.start(t0, 0.4, 100.0).expect("start");

        // Never steady, so the waiting phase must time out.
        for i in 0..200_u64 {
            let now = t0 + Duration::from_secs(i);
            tuner.update(100.0 + i as f64, 0.4, now);
        }
        assert!(tuner.is_failed());
        assert_eq!(
            tuner.failure_reason(),
            Some("process did not reach steady state in time")
        );
    }

    #[test]
    fn aborts_on_overall_timeout() {
        let mut cfg = config(0.10);
        cfg.max_duration = Duration::from_secs(30);
        let mut tuner = ImcTuner::new(cfg);
        let t0 = Instant::now();
        tuner.start(t0, 0.4, 100.0).expect("start");
        tuner.update(100.0, 0.4, t0 + Duration::from_secs(31));
        assert!(tuner.is_failed());
        assert_eq!(tuner.failure_reason(), Some("overall timeout"));
    }

    /// The release invariant: whatever happens, a tuner that is not driving must command nothing.
    #[test]
    fn never_commands_the_actuator_outside_a_driving_phase() {
        let t0 = Instant::now();

        // Idle
        let mut tuner = ImcTuner::new(config(0.10));
        assert_eq!(tuner.update(100.0, 0.4, t0), None);

        // Failed, via an explicit abort mid-run
        tuner.start(t0, 0.4, 100.0).expect("start");
        tuner.abort("operator stopped the run");
        assert!(tuner.is_failed());
        assert_eq!(tuner.update(100.0, 0.4, t0 + Duration::from_secs(1)), None);
        assert_eq!(tuner.failure_reason(), Some("operator stopped the run"));

        // Failed, via a timeout
        let mut cfg = config(0.10);
        cfg.max_duration = Duration::from_secs(5);
        let mut tuner = ImcTuner::new(cfg);
        tuner.start(t0, 0.4, 100.0).expect("start");
        assert_eq!(tuner.update(100.0, 0.4, t0 + Duration::from_secs(10)), None);
        for i in 11..20 {
            assert_eq!(tuner.update(100.0, 0.4, t0 + Duration::from_secs(i)), None);
        }

        // Completed
        let mut plant = Plant::new(250.0, 300.0, 20.0, 1.0);
        let mut done = run(config(0.10), &mut plant, 0.40, 20_000);
        assert!(done.is_completed());
        assert_eq!(done.update(100.0, 0.4, Instant::now()), None);
    }

    #[test]
    fn abort_on_an_idle_tuner_is_a_no_op() {
        let mut tuner = ImcTuner::new(config(0.10));
        tuner.abort("nothing to stop");
        assert_eq!(tuner.phase(), "idle");
        assert_eq!(tuner.failure_reason(), None);
    }
}
