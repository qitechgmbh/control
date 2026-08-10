//! Sequential staircase identification of the coupling matrix.
//!
//! Steps one zone at a time and records *every* zone's response, so each step fills a whole column
//! of the transfer matrix. Four steps therefore identify all `N x N` entries.
//!
//! # Why the identifier owns every actuator, not just the stepped one
//!
//! This is the critical difference from the single-zone tuner. If the other zones' controllers
//! keep regulating during a step, they cut their own duty to reject the heat arriving from the
//! stepped zone — and what gets recorded is the closed-loop disturbance response, not the plant.
//! The measured coupling would come out near zero no matter how strongly the barrel actually
//! conducts. So all `N` duties are frozen at their captured baseline for the whole campaign and
//! only the stepped column moves.
//!
//! # Why the campaign does not return to baseline between columns
//!
//! Cooling a barrel back to its starting state is uncontrolled and slow — there is no active
//! cooling — and doing it three times would roughly double an already multi-hour campaign. Instead
//! each column re-baselines from wherever the previous one settled. Superposition makes this
//! valid: the plant is being treated as linear around the operating point either way, and that
//! assumption is exactly what the identified model asserts.
//!
//! # Ownership of the actuators
//!
//! [`MimoStepIdentifier::update`] returns `Some(duties)` only while actively driving, and `None`
//! in every other state including `Completed` and `Failed`. Callers apply that return value every
//! tick, which makes handing control back structural rather than a step an abort path could skip.

use super::{FopdtEntry, MimoModel, ZONE_COUNT};
use crate::controllers::imc_tuner::fit_fopdt_series;
use std::time::{Duration, Instant, SystemTime};

/// Configuration for an identification campaign.
#[derive(Debug, Clone)]
pub struct MimoIdentifyConfig {
    /// Duty step applied to one zone at a time, as a fraction of full output.
    pub step_duty: f64,
    /// Upper bound on any commanded duty, per zone.
    pub max_duty: [f64; ZONE_COUNT],
    /// Abort if any zone moves further than this from the baseline of the *current column*.
    ///
    /// Per column, not per campaign: a staircase deliberately leaves each zone hotter than it
    /// found it, so measuring against the campaign start would trip this guard on the later
    /// columns purely from accumulated rise. This bound is what catches a genuinely runaway
    /// heater, which shows up as excess rise within a single column.
    pub max_rise_celsius: f64,
    /// Abort if any zone ends up further than this from where the campaign started.
    ///
    /// The companion to `max_rise_celsius`: that one bounds each step, this one bounds the total
    /// drift a four-column staircase is allowed to accumulate before it is judged to have walked
    /// the barrel somewhere it should not be.
    pub max_total_rise_celsius: f64,
    pub sample_period: Duration,
    /// Rolling window over which steady state is judged.
    pub steady_window: Duration,
    pub steady_slope_c_per_min: f64,
    pub steady_band_celsius: f64,
    /// Before the campaign starts, every zone must additionally be this close to its setpoint.
    pub setpoint_band_celsius: f64,
    /// Crossing level for the reported dead-time cross-check.
    pub dead_time_threshold_celsius: f64,
    pub waiting_timeout: Duration,
    pub baseline_timeout: Duration,
    /// Timeout for one column, not the whole run.
    pub column_timeout: Duration,
    pub max_duration: Duration,
}

impl Default for MimoIdentifyConfig {
    fn default() -> Self {
        Self {
            step_duty: 0.10,
            max_duty: [1.0; ZONE_COUNT],
            max_rise_celsius: 15.0,
            max_total_rise_celsius: 40.0,
            sample_period: Duration::from_secs(1),
            steady_window: Duration::from_secs(120),
            steady_slope_c_per_min: 0.15,
            steady_band_celsius: 1.0,
            setpoint_band_celsius: 3.0,
            dead_time_threshold_celsius: 0.5,
            waiting_timeout: Duration::from_secs(1200),
            baseline_timeout: Duration::from_secs(1800),
            column_timeout: Duration::from_secs(4800),
            // Four columns plus the approach, with headroom.
            max_duration: Duration::from_secs(28_800),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MimoIdentPhase {
    Idle,
    WaitingForSteady,
    BaselineHold,
    /// Stepping zone `column`, recording all outputs.
    Step {
        column: usize,
    },
    Completed,
    Failed,
}

impl MimoIdentPhase {
    /// Whether the identifier is commanding the actuators in this phase.
    pub const fn is_driving(self) -> bool {
        matches!(self, Self::BaselineHold | Self::Step { .. })
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::WaitingForSteady => "waiting_for_steady",
            Self::BaselineHold => "baseline_hold",
            Self::Step { .. } => "step",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}

/// One recorded tick of the campaign: every zone's temperature and commanded duty.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MimoTraceSample {
    pub t_seconds: f64,
    pub pv: [f64; ZONE_COUNT],
    pub duty: [f64; ZONE_COUNT],
}

/// Per-column bookkeeping captured when the column starts.
#[derive(Debug, Clone, Copy)]
struct ColumnStart {
    /// Index into the trace where this column's step began.
    trace_index: usize,
    /// Temperature each zone had settled at before the step.
    baseline_pv: [f64; ZONE_COUNT],
    /// Peak-to-peak noise per zone, measured while holding constant duty.
    noise_pp: [f64; ZONE_COUNT],
}

pub struct MimoStepIdentifier {
    config: MimoIdentifyConfig,
    phase: MimoIdentPhase,
    phase_changed: bool,

    started_at: Instant,
    phase_started_at: Instant,
    phase_start_index: usize,

    /// Duty commanded to every zone. Frozen at baseline except for the stepped column.
    commanded: [f64; ZONE_COUNT],
    baseline_duty: [f64; ZONE_COUNT],
    /// Temperature at the very start of the campaign, for the runaway check.
    campaign_start_pv: [f64; ZONE_COUNT],
    setpoints: [f64; ZONE_COUNT],

    trace: Vec<MimoTraceSample>,
    /// Accumulator for the current sample period, so the record is evenly spaced regardless of how
    /// fast the caller ticks.
    pending: Option<PendingSample>,

    column_start: Option<ColumnStart>,
    /// `g[output][input]`, filled one column at a time.
    entries: [[FopdtEntry; ZONE_COUNT]; ZONE_COUNT],
    columns_done: usize,

    result: Option<MimoModel>,
    failure_reason: Option<&'static str>,
}

struct PendingSample {
    sum_pv: [f64; ZONE_COUNT],
    count: u32,
    next_at: Instant,
}

impl MimoStepIdentifier {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            config: MimoIdentifyConfig::default(),
            phase: MimoIdentPhase::Idle,
            phase_changed: false,
            started_at: now,
            phase_started_at: now,
            phase_start_index: 0,
            commanded: [0.0; ZONE_COUNT],
            baseline_duty: [0.0; ZONE_COUNT],
            campaign_start_pv: [0.0; ZONE_COUNT],
            setpoints: [0.0; ZONE_COUNT],
            trace: Vec::new(),
            pending: None,
            column_start: None,
            entries: [[FopdtEntry::default(); ZONE_COUNT]; ZONE_COUNT],
            columns_done: 0,
            result: None,
            failure_reason: None,
        }
    }

    /// Begin a campaign. Returns an error string if the configuration cannot work.
    pub fn start(
        &mut self,
        config: MimoIdentifyConfig,
        setpoints: [f64; ZONE_COUNT],
        now: Instant,
    ) -> Result<(), &'static str> {
        if self.is_running() {
            return Err("an identification campaign is already running");
        }
        if !config.step_duty.is_finite() || config.step_duty <= 0.0 {
            return Err("step duty must be positive");
        }
        if config.max_rise_celsius <= 0.0 {
            return Err("maximum rise must be positive");
        }
        // Every zone must have headroom, because every zone gets stepped in turn.
        for z in 0..ZONE_COUNT {
            if config.step_duty > config.max_duty[z] {
                return Err("step duty exceeds a zone's maximum duty");
            }
        }

        *self = Self {
            config,
            phase: MimoIdentPhase::WaitingForSteady,
            phase_changed: true,
            started_at: now,
            phase_started_at: now,
            phase_start_index: 0,
            commanded: [0.0; ZONE_COUNT],
            baseline_duty: [0.0; ZONE_COUNT],
            campaign_start_pv: [0.0; ZONE_COUNT],
            setpoints,
            trace: Vec::new(),
            pending: None,
            column_start: None,
            entries: [[FopdtEntry::default(); ZONE_COUNT]; ZONE_COUNT],
            columns_done: 0,
            result: None,
            failure_reason: None,
        };
        Ok(())
    }

    /// Advance the campaign.
    ///
    /// `pv` is every zone's current temperature and `pid_duty` what the normal controllers are
    /// commanding right now — the latter is captured as the baseline when the actuators are frozen.
    /// Returns the duties to apply, or `None` when the identifier is not driving.
    pub fn update(
        &mut self,
        pv: [f64; ZONE_COUNT],
        pid_duty: [f64; ZONE_COUNT],
        now: Instant,
    ) -> Option<[f64; ZONE_COUNT]> {
        if !self.is_running() {
            return None;
        }

        if now.saturating_duration_since(self.started_at) > self.config.max_duration {
            self.fail("the campaign exceeded its overall time limit", now);
            return None;
        }

        if self.phase == MimoIdentPhase::WaitingForSteady {
            // Still under normal control: track what the controllers are doing, so freezing the
            // actuators is a continuation rather than a jump.
            self.baseline_duty = pid_duty;
            self.campaign_start_pv = pv;
        }

        self.record(pv, now);
        self.check_runaway(pv, now);
        if !self.is_running() {
            return None;
        }
        self.check_phase_timeout(now);
        if !self.is_running() {
            return None;
        }

        self.advance(now);

        self.command()
    }

    /// Duties currently commanded, or `None` when not driving.
    pub fn command(&self) -> Option<[f64; ZONE_COUNT]> {
        self.phase.is_driving().then_some(self.commanded)
    }

    /// Give up on the run. Safe to call in any state.
    pub fn abort(&mut self, reason: &'static str, now: Instant) {
        if self.is_running() {
            self.fail(reason, now);
        }
    }

    fn fail(&mut self, reason: &'static str, now: Instant) {
        self.failure_reason = Some(reason);
        self.enter(MimoIdentPhase::Failed, now);
    }

    fn record(&mut self, pv: [f64; ZONE_COUNT], now: Instant) {
        let period = self.config.sample_period;
        match &mut self.pending {
            Some(p) => {
                for z in 0..ZONE_COUNT {
                    p.sum_pv[z] += pv[z];
                }
                p.count += 1;
                if now >= p.next_at {
                    let count = f64::from(p.count);
                    let mut mean = [0.0; ZONE_COUNT];
                    for z in 0..ZONE_COUNT {
                        mean[z] = p.sum_pv[z] / count;
                    }
                    let next_at = p.next_at + period;
                    self.pending = Some(PendingSample {
                        sum_pv: [0.0; ZONE_COUNT],
                        count: 0,
                        next_at,
                    });
                    self.trace.push(MimoTraceSample {
                        t_seconds: now.saturating_duration_since(self.started_at).as_secs_f64(),
                        pv: mean,
                        duty: self.commanded,
                    });
                }
            }
            None => {
                self.pending = Some(PendingSample {
                    sum_pv: pv,
                    count: 1,
                    next_at: now + period,
                });
                self.trace.push(MimoTraceSample {
                    t_seconds: 0.0,
                    pv,
                    duty: self.commanded,
                });
            }
        }
    }

    fn check_runaway(&mut self, pv: [f64; ZONE_COUNT], now: Instant) {
        if !self.phase.is_driving() {
            return;
        }
        // Per-column bound. A staircase leaves every zone hotter than it found it, so this has to
        // be measured against the current column's baseline; against the campaign start it would
        // fire on the later columns from accumulated rise rather than from anything going wrong.
        let reference = match self.column_start {
            Some(start) => start.baseline_pv,
            None => self.campaign_start_pv,
        };
        for z in 0..ZONE_COUNT {
            if (pv[z] - reference[z]).abs() > self.config.max_rise_celsius {
                self.fail("a zone moved further than the per-step limit", now);
                return;
            }
            if (pv[z] - self.campaign_start_pv[z]).abs() > self.config.max_total_rise_celsius {
                self.fail("the campaign drifted the barrel past its total limit", now);
                return;
            }
        }
    }

    fn check_phase_timeout(&mut self, now: Instant) {
        let elapsed = now.saturating_duration_since(self.phase_started_at);
        let timeout = match self.phase {
            MimoIdentPhase::WaitingForSteady => self.config.waiting_timeout,
            MimoIdentPhase::BaselineHold => self.config.baseline_timeout,
            MimoIdentPhase::Step { .. } => self.config.column_timeout,
            _ => return,
        };
        if elapsed > timeout {
            self.fail("a phase timed out before the zones settled", now);
        }
    }

    fn advance(&mut self, now: Instant) {
        match self.phase {
            MimoIdentPhase::WaitingForSteady => {
                if self.all_steady(true).is_none() {
                    return;
                }
                // Freeze every actuator where the controllers had them. From here to the end of
                // the campaign no zone is regulating.
                self.commanded = self.baseline_duty;
                self.enter(MimoIdentPhase::BaselineHold, now);
            }
            MimoIdentPhase::BaselineHold => {
                let Some(stats) = self.all_steady(false) else {
                    return;
                };
                self.begin_column(0, &stats, now);
            }
            MimoIdentPhase::Step { column } => {
                if !self.column_settled() {
                    return;
                }
                if let Err(reason) = self.fit_column(column) {
                    self.fail(reason, now);
                    return;
                }
                self.columns_done = column + 1;

                if column + 1 < ZONE_COUNT {
                    // Staircase: the settled state of this column is the next one's baseline.
                    let Some(stats) = self.all_steady(false) else {
                        // Settled enough to fit but not to re-baseline; wait for it to firm up.
                        return;
                    };
                    self.begin_column(column + 1, &stats, now);
                } else {
                    self.finish(now);
                }
            }
            _ => {}
        }
    }

    fn begin_column(&mut self, column: usize, stats: &SteadyStats, now: Instant) {
        self.column_start = Some(ColumnStart {
            trace_index: self.trace.len(),
            baseline_pv: stats.mean,
            noise_pp: stats.peak_to_peak,
        });
        // Freeze everything, step one.
        self.commanded[column] =
            (self.commanded[column] + self.config.step_duty).min(self.config.max_duty[column]);
        self.enter(MimoIdentPhase::Step { column }, now);
    }

    fn finish(&mut self, now: Instant) {
        let mut model = MimoModel {
            g: self.entries,
            setpoints: self.setpoints,
            baseline_duty: self.baseline_duty,
            rga: super::matrix::zeros(),
            condition_number: f64::INFINITY,
            niederlinski: f64::NAN,
            identified_at: SystemTime::now(),
        };
        model.refresh_diagnostics();
        self.result = Some(model);
        self.enter(MimoIdentPhase::Completed, now);
    }

    /// A column is settled once every zone is flat *and* the stepped zone has actually moved.
    /// Without the movement check, the flat stretch during dead time reads as settled immediately.
    fn column_settled(&self) -> bool {
        let MimoIdentPhase::Step { column } = self.phase else {
            return false;
        };
        let Some(start) = self.column_start else {
            return false;
        };

        let elapsed = self.trace.len().saturating_sub(self.phase_start_index) as f64
            * self.config.sample_period.as_secs_f64();
        if elapsed < 2.0 * self.config.steady_window.as_secs_f64() {
            return false;
        }
        let Some(stats) = self.all_steady(false) else {
            return false;
        };
        let moved = (stats.mean[column] - start.baseline_pv[column]).abs();
        moved >= (3.0 * start.noise_pp[column]).max(1.0)
    }

    fn steady_window_len(&self) -> usize {
        let n = self.config.steady_window.as_secs_f64() / self.config.sample_period.as_secs_f64();
        (n.round() as usize).max(2)
    }

    /// Per-zone statistics over the trailing window, or `None` unless **every** zone is steady.
    fn all_steady(&self, require_setpoint_band: bool) -> Option<SteadyStats> {
        let n = self.steady_window_len();
        if self.trace.len() < self.phase_start_index + n {
            return None;
        }
        let window = &self.trace[self.trace.len() - n..];

        let mut out = SteadyStats {
            mean: [0.0; ZONE_COUNT],
            peak_to_peak: [0.0; ZONE_COUNT],
        };

        for z in 0..ZONE_COUNT {
            let mean = window.iter().map(|s| s.pv[z]).sum::<f64>() / n as f64;
            let mut min = f64::INFINITY;
            let mut max = f64::NEG_INFINITY;
            for s in window {
                min = min.min(s.pv[z]);
                max = max.max(s.pv[z]);
            }
            let peak_to_peak = max - min;

            // Least-squares slope: a first-to-last difference would be dominated by the noise on
            // two individual samples.
            let x_mean = (n as f64 - 1.0) / 2.0;
            let mut num = 0.0;
            let mut den = 0.0;
            for (i, s) in window.iter().enumerate() {
                let dx = i as f64 - x_mean;
                num += dx * (s.pv[z] - mean);
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
            if require_setpoint_band
                && (mean - self.setpoints[z]).abs() > self.config.setpoint_band_celsius
            {
                return None;
            }

            out.mean[z] = mean;
            out.peak_to_peak[z] = peak_to_peak;
        }

        Some(out)
    }

    /// Fit all `ZONE_COUNT` entries of one column from the recorded segment.
    fn fit_column(&mut self, column: usize) -> Result<(), &'static str> {
        let start = self
            .column_start
            .ok_or("internal: column started without bookkeeping")?;
        let segment = &self.trace[start.trace_index..];
        let sample_dt = self.config.sample_period.as_secs_f64();
        let delta_u = self.config.step_duty;

        // The diagonal entry is the one the whole column depends on: it sets the zone's own model
        // and is what the decoupler normalises against. Off-diagonal entries are allowed to come
        // back as "no measurable response", which is the expected result for a distant zone.
        let mut fitted = [FopdtEntry::default(); ZONE_COUNT];
        for output in 0..ZONE_COUNT {
            let series: Vec<f64> = segment
                .iter()
                .map(|s| s.pv[output] - start.baseline_pv[output])
                .collect();

            let noise = start.noise_pp[output];
            match fit_fopdt_series(&series, sample_dt, self.config.dead_time_threshold_celsius) {
                Some(fit) if fit.tau > 0.0 && fit.amplitude.is_finite() => {
                    fitted[output] = FopdtEntry {
                        gp: fit.amplitude / delta_u,
                        tau: fit.tau,
                        theta: fit.theta,
                        rms_residual: fit.rms_residual,
                        snr_ratio: (fit.amplitude.abs() / noise.max(1e-9)).min(9999.0),
                    };
                }
                _ => {
                    if output == column {
                        return Err("the stepped zone's own response could not be fitted");
                    }
                    // No measurable coupling on this path. Zero gain is the right model, and a
                    // unit time constant keeps it harmless in any later arithmetic.
                    fitted[output] = FopdtEntry {
                        gp: 0.0,
                        tau: 1.0,
                        theta: 0.0,
                        rms_residual: 0.0,
                        snr_ratio: 0.0,
                    };
                }
            }
        }

        for output in 0..ZONE_COUNT {
            self.entries[output][column] = fitted[output];
        }
        Ok(())
    }

    fn enter(&mut self, phase: MimoIdentPhase, now: Instant) {
        self.phase = phase;
        self.phase_started_at = now;
        self.phase_start_index = self.trace.len();
        self.phase_changed = true;
        if !phase.is_driving() {
            self.commanded = [0.0; ZONE_COUNT];
        }
    }

    pub const fn phase_enum(&self) -> MimoIdentPhase {
        self.phase
    }

    pub const fn phase(&self) -> &'static str {
        self.phase.as_str()
    }

    pub const fn is_running(&self) -> bool {
        matches!(
            self.phase,
            MimoIdentPhase::WaitingForSteady
                | MimoIdentPhase::BaselineHold
                | MimoIdentPhase::Step { .. }
        )
    }

    pub const fn is_completed(&self) -> bool {
        matches!(self.phase, MimoIdentPhase::Completed)
    }

    pub const fn is_failed(&self) -> bool {
        matches!(self.phase, MimoIdentPhase::Failed)
    }

    pub const fn result(&self) -> Option<&MimoModel> {
        self.result.as_ref()
    }

    pub fn take_result(&mut self) -> Option<MimoModel> {
        self.result.take()
    }

    pub fn trace(&self) -> &[MimoTraceSample] {
        &self.trace
    }

    pub const fn failure_reason(&self) -> Option<&'static str> {
        self.failure_reason
    }

    pub const fn columns_done(&self) -> usize {
        self.columns_done
    }

    pub const fn baseline_duty(&self) -> &[f64; ZONE_COUNT] {
        &self.baseline_duty
    }

    /// Consume the "phase has changed" edge, so a caller can emit state on transitions without
    /// tracking the previous phase itself.
    pub const fn take_phase_changed(&mut self) -> bool {
        let changed = self.phase_changed;
        self.phase_changed = false;
        changed
    }

    pub fn elapsed_seconds(&self, now: Instant) -> f64 {
        if self.phase == MimoIdentPhase::Idle {
            0.0
        } else {
            now.saturating_duration_since(self.started_at).as_secs_f64()
        }
    }

    /// Progress across the whole campaign, 0-100. The approach occupies the first band and each
    /// column an equal share of the rest, advancing by elapsed time against its own timeout since
    /// no phase has a knowable duration up front.
    pub fn progress_percent(&self, now: Instant) -> f64 {
        let fraction = |timeout: Duration| {
            let elapsed = now
                .saturating_duration_since(self.phase_started_at)
                .as_secs_f64();
            (elapsed / timeout.as_secs_f64().max(1e-9)).clamp(0.0, 1.0)
        };
        const APPROACH: f64 = 20.0;
        let per_column = (100.0 - APPROACH) / ZONE_COUNT as f64;
        match self.phase {
            MimoIdentPhase::Idle | MimoIdentPhase::Failed => 0.0,
            MimoIdentPhase::Completed => 100.0,
            MimoIdentPhase::WaitingForSteady => 8.0 * fraction(self.config.waiting_timeout),
            MimoIdentPhase::BaselineHold => 8.0 + 12.0 * fraction(self.config.baseline_timeout),
            MimoIdentPhase::Step { column } => (APPROACH
                + per_column * (column as f64 + fraction(self.config.column_timeout)))
            .min(99.0),
        }
    }
}

impl Default for MimoStepIdentifier {
    fn default() -> Self {
        Self::new()
    }
}

struct SteadyStats {
    mean: [f64; ZONE_COUNT],
    peak_to_peak: [f64; ZONE_COUNT],
}
