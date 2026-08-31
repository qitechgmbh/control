//! Named simulation runs.
//!
//! Setpoint arrays are indexed by [`Zone::port`], i.e.
//! `[front, middle, back, nozzle]`.

use super::geometry::Zone;

/// A simulation run: where the machine starts, what it is asked to do, for how
/// long.
#[derive(Debug, Clone)]
pub struct Scenario {
    pub name: String,
    /// Uniform starting temperature in °C.
    pub initial_c: f64,
    pub duration_s: f64,
    /// Setpoints in °C from the start of the run.
    pub setpoints_c: [f64; 4],
    /// When `allow_heating` is called, in seconds. Use `f64::INFINITY` to keep
    /// heating off for the whole run.
    pub heating_enabled_at_s: f64,
    /// Later setpoint changes as `(at_seconds, setpoints)`, applied in order.
    pub changes: Vec<(f64, [f64; 4])>,
}

impl Scenario {
    fn base(name: &str, setpoints_c: [f64; 4], duration_s: f64) -> Self {
        Self {
            name: name.to_owned(),
            initial_c: 22.0,
            duration_s,
            setpoints_c,
            heating_enabled_at_s: 0.0,
            changes: Vec::new(),
        }
    }

    /// The run recorded in `data/heatup_2026-02-24.csv`, for direct comparison.
    ///
    /// Setpoints were front 180, middle 180, back 170, nozzle 175 °C, from a
    /// 22 °C cold start, for just under an hour.
    pub fn recorded_heatup() -> Self {
        Self::base("recorded-heatup", [180.0, 180.0, 170.0, 175.0], 3270.0)
    }

    pub fn normal_production() -> Self {
        Self::base("normal-production", [180.0, 160.0, 150.0, 180.0], 3600.0)
    }

    /// Plain cold start with every zone asked for the same temperature. The
    /// cleanest way to see the zones diverge.
    pub fn cold_start() -> Self {
        Self::base("cold-start", [180.0; 4], 3600.0)
    }

    /// Nozzle alone, with the barrel zones off — isolates the slowest loop.
    pub fn nozzle_only() -> Self {
        Self::base("nozzle-only", [0.0, 0.0, 0.0, 175.0], 5400.0)
    }

    /// Heat to 180, settle, then step every zone up by 20 K. Shows small-signal
    /// behaviour without the cold-start saturation that dominates a full ramp.
    pub fn step_up() -> Self {
        let mut s = Self::base("step-up", [180.0; 4], 9000.0);
        s.changes.push((5400.0, [200.0; 4]));
        s
    }

    /// One barrel zone alone from cold. Run this per zone to separate the
    /// zone-to-zone coupling from the loss terms during calibration.
    pub fn single_zone(zone: Zone) -> Self {
        let mut sp = [0.0; 4];
        sp[zone.port()] = 180.0;
        Self::base(&format!("single-{}", zone.name()), sp, 5400.0)
    }

    /// Look up a scenario by its CLI name.
    pub fn by_name(name: &str) -> Option<Self> {
        Some(match name {
            "recorded-heatup" => Self::recorded_heatup(),
            "cold-start" => Self::cold_start(),
            "normal-production" => Self::normal_production(),
            "nozzle-only" => Self::nozzle_only(),
            "step-up" => Self::step_up(),
            "single-front" => Self::single_zone(Zone::Front),
            "single-middle" => Self::single_zone(Zone::Middle),
            "single-back" => Self::single_zone(Zone::Back),
            "single-nozzle" => Self::single_zone(Zone::Nozzle),
            _ => return None,
        })
    }

    /// Every scenario name [`Self::by_name`] accepts.
    pub const NAMES: &'static [&'static str] = &[
        "recorded-heatup",
        "normal-production",
        "cold-start",
        "nozzle-only",
        "step-up",
        "single-front",
        "single-middle",
        "single-back",
        "single-nozzle",
    ];
}
