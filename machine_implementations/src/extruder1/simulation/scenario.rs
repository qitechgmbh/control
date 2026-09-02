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
    /// Screw speed in rpm from the start of the run. 0 is not extruding; 100 is
    /// the machine maximum, which is 10 kg/h.
    ///
    /// Only does anything on a model built with
    /// [`super::params::MeltParams::enabled`].
    pub screw_rpm: f64,
    /// Later speed changes as `(at_seconds, rpm)`, applied in order.
    pub rpm_changes: Vec<(f64, f64)>,
}

impl Default for Scenario {
    fn default() -> Self {
        Self {
            name: String::new(),
            initial_c: 22.0,
            duration_s: 3600.0,
            setpoints_c: [0.0; 4],
            heating_enabled_at_s: 0.0,
            changes: Vec::new(),
            screw_rpm: 0.0,
            rpm_changes: Vec::new(),
        }
    }
}

impl Scenario {
    fn base(name: &str, setpoints_c: [f64; 4], duration_s: f64) -> Self {
        Self {
            name: name.to_owned(),
            duration_s,
            setpoints_c,
            ..Self::default()
        }
    }

    /// Whether this run ever turns the screw, and so needs a model with the
    /// melt enabled.
    pub fn extrudes(&self) -> bool {
        self.screw_rpm > 0.0 || self.rpm_changes.iter().any(|(_, rpm)| *rpm > 0.0)
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

    /// Heat to a normal profile, settle, then start extruding at 60 rpm
    /// (6 kg/h). **The load-disturbance test this whole model exists for**: the
    /// shipping controller was tuned on a heat-up with the screw stopped and has
    /// never been asked to hold setpoint against material moving through.
    pub fn extrude_start() -> Self {
        let mut s = Self::base("extrude-start", [180.0, 180.0, 170.0, 175.0], 8400.0);
        // Not before 3600 s: the nozzle is ~6 kg of steel on a 200 W band and
        // takes over half an hour to arrive (see `README.md`). Starting the
        // screw while it is still climbing would measure the heat-up, not the
        // load step.
        s.rpm_changes.push((3600.0, 60.0));
        s
    }

    /// The same, walked up to the machine's maximum in three steps, to find
    /// where a zone runs out of duty.
    pub fn extrude_ramp() -> Self {
        let mut s = Self::base("extrude-ramp", [180.0, 180.0, 170.0, 175.0], 12_000.0);
        s.rpm_changes
            .extend([(3600.0, 30.0), (6000.0, 60.0), (8400.0, 100.0)]);
        s
    }

    /// Already hot and already extruding, then the screw stops. The *unloading*
    /// transient: a loop that has wound up to cover a several-hundred-watt sink
    /// suddenly does not have to.
    pub fn extrude_stop() -> Self {
        let mut s = Self::base("extrude-stop", [180.0, 180.0, 170.0, 175.0], 3600.0);
        s.initial_c = 180.0;
        s.screw_rpm = 100.0;
        s.rpm_changes.push((1200.0, 0.0));
        s
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
            "extrude-start" => Self::extrude_start(),
            "extrude-ramp" => Self::extrude_ramp(),
            "extrude-stop" => Self::extrude_stop(),
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
        "extrude-start",
        "extrude-ramp",
        "extrude-stop",
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `NAMES` is what `--list` prints and `by_name` is what `--scenario`
    /// accepts; a scenario in one and not the other is a broken flag.
    #[test]
    fn every_listed_scenario_resolves() {
        for name in Scenario::NAMES {
            let s = Scenario::by_name(name)
                .unwrap_or_else(|| panic!("{name} is listed but by_name does not know it"));
            assert_eq!(&s.name, name);
        }
    }

    #[test]
    fn only_the_extrusion_scenarios_turn_the_screw() {
        for name in Scenario::NAMES {
            let s = Scenario::by_name(name).expect("listed");
            assert_eq!(
                s.extrudes(),
                name.starts_with("extrude-"),
                "{name} disagrees with its own name about whether it extrudes"
            );
        }
    }
}
