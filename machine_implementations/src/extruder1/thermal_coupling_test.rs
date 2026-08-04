//! Automated thermal coupling measurement for the extruder's 4 heating zones.
//!
//! Steps one zone's duty cycle at a time (all 4 zones held open-loop, i.e. driven
//! at a fixed manual duty rather than by their PID) and records how every zone's
//! temperature responds. From the 4 resulting columns it builds a steady-state
//! gain matrix `K[i][j]` (°C response of zone `i` per unit duty-cycle step of zone
//! `j`) and the corresponding Relative Gain Array, which tells you how strongly
//! the zones interact through the shared barrel before you tune each zone's PID.

use super::ExtruderV2Mode;
use super::api::{ThermalCouplingResult, ThermalCouplingTestConfig, ThermalCouplingTestState};
use std::time::Instant;

pub const ZONE_COUNT: usize = 4;
pub const ZONE_NAMES: [&str; ZONE_COUNT] = ["front", "middle", "back", "nozzle"];

#[derive(Debug, Clone)]
pub enum ThermalCouplingPhase {
    Idle,
    /// One-tick transition: captures each zone's current PID duty as the shared
    /// baseline and switches all 4 zones into manual (open-loop) override.
    CaptureBaseline,
    /// Holding all 4 zones at the shared baseline duty, waiting for the previous
    /// step's disturbance (or the initial transient) to settle before recording
    /// the baseline temperatures for `zone_index`.
    Settling { zone_index: usize, since: Instant },
    /// One-tick transition: records baseline temperatures and applies the step.
    RecordBaseline { zone_index: usize },
    /// Holding `zone_index`'s stepped duty, waiting to record the response.
    Stepping { zone_index: usize, since: Instant },
    /// One-tick transition: records the response, fills in one column of the
    /// gain matrix, and reverts `zone_index` back to baseline.
    RecordStep { zone_index: usize },
    Done,
    Aborted(String),
}

#[derive(Debug, Clone)]
pub struct ThermalCouplingTest {
    pub phase: ThermalCouplingPhase,
    config: ThermalCouplingTestConfig,
    baseline_duty: [f64; ZONE_COUNT],
    baseline_temp: [f64; ZONE_COUNT],
    /// gain_matrix[output_zone][input_zone], filled in one column per tested zone
    gain_matrix: [[f64; ZONE_COUNT]; ZONE_COUNT],
    result: Option<ThermalCouplingResult>,
}

impl Default for ThermalCouplingTest {
    fn default() -> Self {
        Self {
            phase: ThermalCouplingPhase::Idle,
            config: ThermalCouplingTestConfig {
                step_size: 0.0,
                settle_duration_secs: 0.0,
                step_duration_secs: 0.0,
            },
            baseline_duty: [0.0; ZONE_COUNT],
            baseline_temp: [0.0; ZONE_COUNT],
            gain_matrix: [[0.0; ZONE_COUNT]; ZONE_COUNT],
            result: None,
        }
    }
}

/// Inverts a 4x4 matrix via Gauss-Jordan elimination with partial pivoting.
/// Returns `None` if the matrix is (numerically) singular.
fn invert4x4(m: [[f64; ZONE_COUNT]; ZONE_COUNT]) -> Option<[[f64; ZONE_COUNT]; ZONE_COUNT]> {
    const N: usize = ZONE_COUNT;
    let mut a = [[0.0; 2 * N]; N];
    for i in 0..N {
        a[i][..N].copy_from_slice(&m[i]);
        a[i][N + i] = 1.0;
    }

    for col in 0..N {
        let mut pivot_row = col;
        let mut pivot_val = a[col][col].abs();
        for row in (col + 1)..N {
            if a[row][col].abs() > pivot_val {
                pivot_val = a[row][col].abs();
                pivot_row = row;
            }
        }
        if pivot_val < 1e-9 {
            return None;
        }
        if pivot_row != col {
            a.swap(col, pivot_row);
        }
        let pivot = a[col][col];
        for j in 0..(2 * N) {
            a[col][j] /= pivot;
        }
        for row in 0..N {
            if row != col {
                let factor = a[row][col];
                if factor != 0.0 {
                    for j in 0..(2 * N) {
                        a[row][j] -= factor * a[col][j];
                    }
                }
            }
        }
    }

    let mut inv = [[0.0; N]; N];
    for i in 0..N {
        inv[i].copy_from_slice(&a[i][N..]);
    }
    Some(inv)
}

/// Builds the result from a completed gain matrix, computing the Relative Gain
/// Array `RGA = K ⊙ (K⁻¹)ᵗ` (element-wise product with the transposed inverse).
fn build_result(gain_matrix: [[f64; ZONE_COUNT]; ZONE_COUNT]) -> ThermalCouplingResult {
    let rga_matrix = match invert4x4(gain_matrix) {
        Some(k_inv) => {
            let mut rga = [[0.0; ZONE_COUNT]; ZONE_COUNT];
            for i in 0..ZONE_COUNT {
                for j in 0..ZONE_COUNT {
                    rga[i][j] = gain_matrix[i][j] * k_inv[j][i];
                }
            }
            rga
        }
        None => [[f64::NAN; ZONE_COUNT]; ZONE_COUNT],
    };
    ThermalCouplingResult {
        zones: ZONE_NAMES.map(String::from),
        gain_matrix,
        rga_matrix,
    }
}

#[cfg(not(feature = "mock-machine"))]
impl super::ExtruderV2 {
    fn temperature_controller_by_index(
        &self,
        i: usize,
    ) -> &super::temperature_controller::TemperatureController {
        match i {
            0 => &self.temperature_controller_front,
            1 => &self.temperature_controller_middle,
            2 => &self.temperature_controller_back,
            _ => &self.temperature_controller_nozzle,
        }
    }

    fn temperature_controller_by_index_mut(
        &mut self,
        i: usize,
    ) -> &mut super::temperature_controller::TemperatureController {
        match i {
            0 => &mut self.temperature_controller_front,
            1 => &mut self.temperature_controller_middle,
            2 => &mut self.temperature_controller_back,
            _ => &mut self.temperature_controller_nozzle,
        }
    }

    fn release_thermal_coupling_overrides(&mut self) {
        for i in 0..ZONE_COUNT {
            self.temperature_controller_by_index_mut(i).set_manual_duty(None);
        }
    }

    /// Starts a thermal coupling test. The extruder must already be in `Heat`
    /// mode (heating on, screw stopped) so that material flow doesn't confound
    /// the purely thermal measurement.
    pub fn start_thermal_coupling_test(&mut self, config: ThermalCouplingTestConfig) {
        if self.mode != ExtruderV2Mode::Heat {
            self.thermal_coupling_test.phase = ThermalCouplingPhase::Aborted(
                "extruder must be in Heat mode (not Standby/Extrude) to run a thermal coupling test"
                    .to_string(),
            );
            self.emit_state();
            return;
        }

        self.thermal_coupling_test = ThermalCouplingTest {
            phase: ThermalCouplingPhase::CaptureBaseline,
            config,
            ..ThermalCouplingTest::default()
        };
        self.emit_state();
    }

    /// Aborts an in-progress test (or clears a finished one) and hands control
    /// of all 4 zones back to their PID.
    pub fn stop_thermal_coupling_test(&mut self) {
        self.release_thermal_coupling_overrides();
        self.thermal_coupling_test.phase = ThermalCouplingPhase::Idle;
        self.emit_state();
    }

    /// Advances the thermal coupling test state machine. Called once per act
    /// tick; no-ops immediately unless a test is running.
    pub fn advance_thermal_coupling_test(&mut self, now: Instant) {
        use qitech_lib::units::thermodynamic_temperature::degree_celsius;

        let is_active = !matches!(
            self.thermal_coupling_test.phase,
            ThermalCouplingPhase::Idle | ThermalCouplingPhase::Done | ThermalCouplingPhase::Aborted(_)
        );
        if is_active && self.mode != ExtruderV2Mode::Heat {
            self.release_thermal_coupling_overrides();
            self.thermal_coupling_test.phase = ThermalCouplingPhase::Aborted(
                "mode changed away from Heat while the test was running".to_string(),
            );
            self.emit_state();
            return;
        }

        let phase = self.thermal_coupling_test.phase.clone();
        match phase {
            ThermalCouplingPhase::Idle
            | ThermalCouplingPhase::Done
            | ThermalCouplingPhase::Aborted(_) => {}

            ThermalCouplingPhase::CaptureBaseline => {
                let wiring_ok =
                    (0..ZONE_COUNT).all(|i| !self.temperature_controller_by_index(i).heating.wiring_error);
                if !wiring_ok {
                    self.thermal_coupling_test.phase = ThermalCouplingPhase::Aborted(
                        "one or more zones report a wiring error".to_string(),
                    );
                    self.emit_state();
                    return;
                }

                for i in 0..ZONE_COUNT {
                    let duty = self.temperature_controller_by_index(i).get_current_duty();
                    self.thermal_coupling_test.baseline_duty[i] = duty;
                    self.temperature_controller_by_index_mut(i)
                        .set_manual_duty(Some(duty));
                }
                self.thermal_coupling_test.phase =
                    ThermalCouplingPhase::Settling { zone_index: 0, since: now };
                self.emit_state();
            }

            ThermalCouplingPhase::Settling { zone_index, since } => {
                if now.duration_since(since).as_secs_f64() >= self.thermal_coupling_test.config.settle_duration_secs
                {
                    self.thermal_coupling_test.phase = ThermalCouplingPhase::RecordBaseline { zone_index };
                }
            }

            ThermalCouplingPhase::RecordBaseline { zone_index } => {
                for i in 0..ZONE_COUNT {
                    self.thermal_coupling_test.baseline_temp[i] = self
                        .temperature_controller_by_index(i)
                        .heating
                        .temperature
                        .get::<degree_celsius>();
                }
                let stepped_duty =
                    self.thermal_coupling_test.baseline_duty[zone_index] + self.thermal_coupling_test.config.step_size;
                self.temperature_controller_by_index_mut(zone_index)
                    .set_manual_duty(Some(stepped_duty.clamp(0.0, 1.0)));
                self.thermal_coupling_test.phase = ThermalCouplingPhase::Stepping { zone_index, since: now };
                self.emit_state();
            }

            ThermalCouplingPhase::Stepping { zone_index, since } => {
                if now.duration_since(since).as_secs_f64() >= self.thermal_coupling_test.config.step_duration_secs {
                    self.thermal_coupling_test.phase = ThermalCouplingPhase::RecordStep { zone_index };
                }
            }

            ThermalCouplingPhase::RecordStep { zone_index } => {
                let step = self.thermal_coupling_test.config.step_size;
                for i in 0..ZONE_COUNT {
                    let stepped_temp = self
                        .temperature_controller_by_index(i)
                        .heating
                        .temperature
                        .get::<degree_celsius>();
                    let delta = stepped_temp - self.thermal_coupling_test.baseline_temp[i];
                    self.thermal_coupling_test.gain_matrix[i][zone_index] = delta / step;
                }
                let baseline_duty = self.thermal_coupling_test.baseline_duty[zone_index];
                self.temperature_controller_by_index_mut(zone_index)
                    .set_manual_duty(Some(baseline_duty));

                if zone_index + 1 < ZONE_COUNT {
                    self.thermal_coupling_test.phase = ThermalCouplingPhase::Settling {
                        zone_index: zone_index + 1,
                        since: now,
                    };
                } else {
                    let result = build_result(self.thermal_coupling_test.gain_matrix);
                    self.thermal_coupling_test.result = Some(result);
                    self.release_thermal_coupling_overrides();
                    self.thermal_coupling_test.phase = ThermalCouplingPhase::Done;
                }
                self.emit_state();
            }
        }
    }

    pub fn get_thermal_coupling_test_state(&self) -> ThermalCouplingTestState {
        let t = &self.thermal_coupling_test;
        let (state, zone_under_test, elapsed_secs, duration_secs) = match &t.phase {
            ThermalCouplingPhase::Idle => ("idle".to_string(), None, 0.0, 0.0),
            ThermalCouplingPhase::CaptureBaseline => ("starting".to_string(), None, 0.0, 0.0),
            ThermalCouplingPhase::Settling { zone_index, since } => (
                "settling".to_string(),
                Some(ZONE_NAMES[*zone_index].to_string()),
                since.elapsed().as_secs_f64(),
                t.config.settle_duration_secs,
            ),
            ThermalCouplingPhase::RecordBaseline { zone_index } => (
                "settling".to_string(),
                Some(ZONE_NAMES[*zone_index].to_string()),
                t.config.settle_duration_secs,
                t.config.settle_duration_secs,
            ),
            ThermalCouplingPhase::Stepping { zone_index, since } => (
                "stepping".to_string(),
                Some(ZONE_NAMES[*zone_index].to_string()),
                since.elapsed().as_secs_f64(),
                t.config.step_duration_secs,
            ),
            ThermalCouplingPhase::RecordStep { zone_index } => (
                "stepping".to_string(),
                Some(ZONE_NAMES[*zone_index].to_string()),
                t.config.step_duration_secs,
                t.config.step_duration_secs,
            ),
            ThermalCouplingPhase::Done => ("done".to_string(), None, 0.0, 0.0),
            ThermalCouplingPhase::Aborted(_) => ("aborted".to_string(), None, 0.0, 0.0),
        };
        let zones_completed = match &t.phase {
            ThermalCouplingPhase::Settling { zone_index, .. }
            | ThermalCouplingPhase::RecordBaseline { zone_index }
            | ThermalCouplingPhase::Stepping { zone_index, .. }
            | ThermalCouplingPhase::RecordStep { zone_index } => *zone_index,
            ThermalCouplingPhase::Done => ZONE_COUNT,
            _ => 0,
        };
        let error = match &t.phase {
            ThermalCouplingPhase::Aborted(reason) => Some(reason.clone()),
            _ => None,
        };
        ThermalCouplingTestState {
            state,
            zone_under_test,
            elapsed_secs,
            duration_secs,
            zones_completed,
            error,
            result: t.result.clone(),
        }
    }
}
