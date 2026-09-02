//! Compare heating control strategies for the extruder's four zones, measure the
//! plant coefficients, and search for gains.
//!
//! Strategies are run against the calibrated thermal model over several
//! operating profiles and — this is the part that matters — over a *family* of
//! plants rather than one. The model is calibrated against a single recorded
//! heat-up, and that recording cannot separate `sensor_tau_s` from
//! `band_heat_capacity_j_per_m2_k`: both delay the reading relative to the steel,
//! so the optimiser lands somewhere along a flat valley. `ObserverPi` leans on
//! exactly that split, so scoring it against the nominal fit alone would be
//! marking our own homework.
//!
//! ```text
//! cargo run --release -p machine_implementations --features simulation \
//!     --example bench_heating                      # compare pid vs observer-pi
//! ... -- --identify                                # re-measure the PLANT table
//! ... -- --search observer-pi                      # search gains (or `pid`)
//! ```

use machine_implementations::extruder1::heating_params::{PLANT, observer_pi_params};
use machine_implementations::extruder1::simulation::{
    ExtruderThermalParams, Scenario, SimConfig, StrategyConfig, ThermalSim, Trace, Zone,
    ZoneTuning,
    harness::plant_family,
    optimize::{self, Rng},
};

/// Profiles as entered on the UI in `nozzle, front, middle, back` order, stored
/// here in the simulator's `[front, middle, back, nozzle]` order. Same three the
/// PID search uses, so the numbers are comparable.
const PROFILES: &[(&str, [f64; 4])] = &[
    (
        "A  nozzle200 front200 middle170 back150",
        [200.0, 170.0, 150.0, 200.0],
    ),
    (
        "B  nozzle170 front180 middle160 back140",
        [180.0, 160.0, 140.0, 170.0],
    ),
    (
        "C  nozzle150 front150 middle130 back100",
        [150.0, 130.0, 100.0, 150.0],
    ),
];

const DURATION_S: f64 = 6000.0;
const SETTLE_TOL_K: f64 = 2.5;

/// Overshoot each zone is allowed before it starts costing, in K.
///
/// Middle gets none. It is flanked by heated zones with no cold sink to bleed
/// into, so an excursion there takes far longer to shed than the same excursion
/// on front or back — the recording shows it still 5 K high 20 minutes after the
/// relay opened. Everywhere else a couple of kelvin is not worth slowing the
/// machine for.
const FREE_OVERSHOOT_K: [f64; 4] = [2.0, 0.0, 2.0, 2.0];

const OVERSHOOT_WEIGHT: f64 = 40.0; // seconds of cost per K of excess overshoot
const FINAL_ERR_WEIGHT: f64 = 250.0; // per K of final error when never settled
const NEVER_SETTLED_BASE: f64 = 2.5; // x DURATION_S when a zone never settles

fn profile(sp: [f64; 4]) -> Scenario {
    Scenario {
        name: String::new(),
        initial_c: 22.0,
        duration_s: DURATION_S,
        setpoints_c: sp,
        ..Scenario::default()
    }
}

/// Heat to the profile, settle, then step every zone up by 20 K.
///
/// The second symptom in its own right: a step from an already-hot machine has
/// none of the cold-start saturation to hide behind, and a loop that only looks
/// right on a cold start can still overshoot badly here.
fn step_up_profile(sp: [f64; 4]) -> Scenario {
    let mut s = profile(sp);
    s.duration_s = 9000.0;
    s.changes.push((5000.0, sp.map(|v| v + 20.0)));
    s
}

fn settle_time(trace: &Trace, zone: Zone, tol: f64) -> Option<f64> {
    let p = zone.port();
    let sp = trace.setpoints_c[p];
    let mut last_bad: Option<usize> = None;
    for (i, s) in trace.samples.iter().enumerate() {
        if (s.sensor_c[p] - sp).abs() > tol {
            last_bad = Some(i);
        }
    }
    match last_bad {
        Some(i) if i + 1 < trace.samples.len() => Some(trace.samples[i + 1].t_s),
        Some(_) => None,
        None => trace.samples.first().map(|s| s.t_s),
    }
}

/// Peak reached *after* the last setpoint change, so a step-up run is scored on
/// the step and not on the cold-start ramp that preceded it.
fn overshoot_after(trace: &Trace, zone: Zone, after_s: f64) -> f64 {
    let p = zone.port();
    trace
        .samples
        .iter()
        .filter(|s| s.t_s >= after_s)
        .map(|s| s.sensor_c[p])
        .fold(f64::NEG_INFINITY, f64::max)
        - trace.setpoints_c[p]
}

fn zone_cost(trace: &Trace, zone: Zone) -> f64 {
    let p = zone.port();
    let sp = trace.setpoints_c[p];
    let overshoot = trace.overshoot_k(zone).max(0.0);
    let free = FREE_OVERSHOOT_K[p];
    match settle_time(trace, zone, SETTLE_TOL_K) {
        Some(t) => {
            let mut c = t;
            if overshoot > free {
                c += OVERSHOOT_WEIGHT * (overshoot - free);
            }
            c
        }
        None => {
            NEVER_SETTLED_BASE * DURATION_S + FINAL_ERR_WEIGHT * (trace.final_c(zone) - sp).abs()
        }
    }
}

fn config_for(strategy: &StrategyConfig, dt_plant_s: f64, dt_ctrl_s: f64) -> SimConfig {
    SimConfig {
        strategy: strategy.clone(),
        dt_plant_s,
        dt_ctrl_s,
        ..SimConfig::default()
    }
}

/// Total cost across every profile and every plant in the family.
fn total_cost(strategy: &StrategyConfig, fast: bool) -> f64 {
    let (dt_plant, dt_ctrl) = if fast { (0.05, 0.01) } else { (0.01, 0.001) };
    let mut total = 0.0;
    for params in plant_family() {
        for (_, sp) in PROFILES {
            let mut sim = ThermalSim::new(params.clone(), config_for(strategy, dt_plant, dt_ctrl));
            let trace = sim.run(&profile(*sp));
            for zone in Zone::ALL {
                total += zone_cost(&trace, zone);
            }
        }
    }
    total
}

// ---------------------------------------------------------------- reporting

fn report(strategy: &StrategyConfig) {
    println!("\n================ {} ================", strategy.name());

    for (name, sp) in PROFILES {
        let mut sim = ThermalSim::new(
            ExtruderThermalParams::calibrated(),
            config_for(strategy, 0.01, 0.001),
        );
        let trace = sim.run(&profile(*sp));
        println!("\n{name}");
        println!(
            "  {:<8} {:>7} {:>8} {:>10} {:>8} {:>8} {:>8} {:>8}",
            "zone", "setpt", "peak", "overshoot", "t90", "settle", "final", "relays"
        );
        for zone in Zone::ALL {
            let p = zone.port();
            let t90 = trace
                .rise_time_s(zone, 0.9)
                .map_or("never".to_owned(), |v| format!("{v:.0}"));
            let settle = settle_time(&trace, zone, SETTLE_TOL_K)
                .map_or("never".to_owned(), |v| format!("{v:.0}"));
            println!(
                "  {:<8} {:>7.0} {:>8.1} {:>+10.1} {:>8} {:>8} {:>8.1} {:>8}",
                zone.name(),
                sp[p],
                trace.peak_c(zone),
                trace.overshoot_k(zone),
                t90,
                settle,
                trace.final_c(zone),
                trace.relay_switches(zone),
            );
        }
    }

    // Step up from hot: the second failure mode, on its own.
    let mut sim = ThermalSim::new(
        ExtruderThermalParams::calibrated(),
        config_for(strategy, 0.01, 0.001),
    );
    let trace = sim.run(&step_up_profile(PROFILES[1].1));
    println!("\nstep up +20 K from settled (overshoot measured after the step)");
    println!(
        "  {:<8} {:>7} {:>10} {:>8}",
        "zone", "setpt", "overshoot", "final"
    );
    for zone in Zone::ALL {
        println!(
            "  {:<8} {:>7.0} {:>+10.1} {:>8.1}",
            zone.name(),
            trace.setpoints_c[zone.port()],
            overshoot_after(&trace, zone, 5000.0),
            trace.final_c(zone),
        );
    }

    // Robustness: worst overshoot per zone across the whole plant family.
    println!("\nworst overshoot across the plant family (profile B)");
    println!(
        "  {:<10} {:>8} {:>8} {:>8} {:>8}",
        "plant", "front", "middle", "back", "nozzle"
    );
    for params in plant_family() {
        let mut sim = ThermalSim::new(params.clone(), config_for(strategy, 0.01, 0.001));
        let trace = sim.run(&profile(PROFILES[1].1));
        print!("  tau={:<6.0}", params.sensor_tau_s);
        for zone in Zone::ALL {
            print!(" {:>+8.1}", trace.overshoot_k(zone));
        }
        println!();
    }

    println!(
        "\ntotal cost across profiles x plant family: {:.0}",
        total_cost(strategy, false)
    );
}

// ---------------------------------------------------------------- identify

/// Measure the per-zone plant coefficients the strategies need, from the
/// simulator itself.
///
/// These are effective, *in-situ* numbers: measured with all four zones running
/// the way they do in production, so zone-to-zone coupling is folded into each
/// zone's loss term rather than pretended away. That is what the feedforward
/// wants — a zone flanked by hot neighbours genuinely does need less power.
fn identify() {
    let params = ExtruderThermalParams::calibrated();
    let ambient = params.ambient_c;

    println!("identifying per-zone plant coefficients from the calibrated model\n");

    // ---- effective heat capacity, from the initial open-loop ramp ----
    // Full power from cold: dT/dt = P / C over the first few minutes, before
    // losses or the neighbours matter.
    let mut sim = ThermalSim::new(params.clone(), SimConfig::default());
    let trace = sim.run_open_loop(ambient, 400.0, &|_| [1.0; 4]);
    let at = |t: f64| {
        trace
            .samples
            .iter()
            .min_by(|a, b| (a.t_s - t).abs().total_cmp(&(b.t_s - t).abs()))
            .expect("open-loop trace has samples")
    };
    let (a, b) = (at(120.0), at(360.0));
    let mut capacity = [0.0f64; 4];
    for zone in Zone::ALL {
        let p = zone.port();
        let rate = (b.steel_c[p] - a.steel_c[p]) / (b.t_s - a.t_s);
        capacity[p] = zone.rated_w() / rate;
    }

    // ---- steady-state duty, from settled closed-loop runs ----
    // Measured closed loop rather than by holding a fixed duty, for two reasons:
    // the operating point is then the real one instead of wherever a guessed
    // duty happens to cook the zone to, and the coupling between zones is the
    // production coupling — all four heating, at the temperatures an operator
    // actually asks for.
    //
    // Averaged over the profiles, since a zone's effective loss depends on how
    // hot its neighbours are, and the feedforward has to be one slope for all of
    // them.
    let baseline =
        StrategyConfig::Pid(machine_implementations::extruder1::simulation::ZoneTuning::PRODUCTION);
    let mut ff_samples = [(0.0f64, 0usize); 4];
    for (_, sp) in PROFILES {
        let mut sim = ThermalSim::new(params.clone(), config_for(&baseline, 0.01, 0.001));
        let trace = sim.run(&profile(*sp));
        let last = trace.samples.last().expect("closed-loop trace has samples");
        for zone in Zone::ALL {
            let p = zone.port();
            // Only trust a zone that genuinely settled; a drifting one has no
            // steady-state duty to read.
            if (last.sensor_c[p] - sp[p]).abs() < 3.0 && last.duty[p] > 1e-4 {
                ff_samples[p].0 += last.duty[p] / (last.steel_c[p] - ambient);
                ff_samples[p].1 += 1;
            }
        }
    }

    println!(
        "  {:<8} {:>10} {:>12} {:>14} {:>8}",
        "zone", "C_metal", "loss_W_per_K", "ff_duty_per_K", "samples"
    );
    let mut lines = Vec::new();
    for zone in Zone::ALL {
        let p = zone.port();
        assert!(
            ff_samples[p].1 > 0,
            "{} never settled in any profile, so its steady-state duty could not \
             be measured",
            zone.name()
        );
        let ff = ff_samples[p].0 / ff_samples[p].1 as f64;
        let loss = ff * zone.rated_w();

        println!(
            "  {:<8} {:>10.0} {:>12.3} {:>14.5} {:>8}",
            zone.name(),
            capacity[p],
            loss,
            ff,
            ff_samples[p].1,
        );
        lines.push(format!(
            "    // {}\n    \
             PlantCoefficients {{ metal_capacity_j_per_k: {:.0}, metal_loss_w_per_k: {loss:.3}, \
             ff_duty_per_k: {ff:.5} }},",
            zone.name(),
            capacity[p],
        ));
    }

    println!("\npaste into extruder1::heating_params::PLANT:\n");
    for l in lines {
        println!("{l}");
    }
    println!(
        "\n(currently compiled in: {})",
        PLANT
            .iter()
            .map(|c| format!("{:.4}", c.ff_duty_per_k))
            .collect::<Vec<_>>()
            .join(" ")
    );
}

// ---------------------------------------------------------------- search
//
// Nelder-Mead over a strategy's free parameters, scored across the plant family.
// The simplex itself lives in `simulation::optimize`, shared with the model
// calibration in `simulation::fit`.

/// A strategy's tunable parameters, flattened so the optimiser can work on them
/// without knowing what they mean.
trait Tunable: Copy {
    /// `(name, lo, hi)` per free parameter.
    fn bounds() -> Vec<(&'static str, f64, f64)>;
    fn to_vec(&self) -> Vec<f64>;
    fn from_vec(v: &[f64]) -> Self;
    fn strategy(&self) -> StrategyConfig;
}

/// `[(a, b, c, d); 4]` — four per-zone quantities, laid out zone-major so each
/// group of four in the flat vector is one quantity across all zones.
type PerZone4 = [(f64, f64, f64, f64); 4];

fn flatten(t: &PerZone4) -> Vec<f64> {
    let mut v = Vec::with_capacity(16);
    v.extend(t.iter().map(|x| x.0));
    v.extend(t.iter().map(|x| x.1));
    v.extend(t.iter().map(|x| x.2));
    v.extend(t.iter().map(|x| x.3));
    v
}

fn unflatten(v: &[f64]) -> PerZone4 {
    std::array::from_fn(|p| (v[p], v[4 + p], v[8 + p], v[12 + p]))
}

/// Four per-zone entries of one quantity, all sharing the same bounds.
fn zone_bounds(lo: f64, hi: f64) -> impl Iterator<Item = (&'static str, f64, f64)> {
    Zone::ALL.into_iter().map(move |z| (z.name(), lo, hi))
}

/// Per-zone `(kp, ki, tau_filter_s, tau_sensor_s)` for `ObserverPi`.
///
/// `tau_sensor_s` is in the search because it is the parameter the whole
/// strategy rests on and the one the calibration is least able to pin down —
/// leaving it fixed would tune everything else around a number that might be
/// wrong by an order of magnitude.
#[derive(Debug, Clone, Copy)]
struct ObserverPiTuning(PerZone4);

impl Tunable for ObserverPiTuning {
    fn bounds() -> Vec<(&'static str, f64, f64)> {
        zone_bounds(0.005, 0.6) // kp
            .chain(zone_bounds(0.0, 0.004)) // ki
            .chain(zone_bounds(5.0, 60.0)) // tau_filter_s
            .chain(zone_bounds(10.0, 220.0)) // tau_sensor_s
            .collect()
    }
    fn to_vec(&self) -> Vec<f64> {
        flatten(&self.0)
    }
    fn from_vec(v: &[f64]) -> Self {
        Self(unflatten(v))
    }
    fn strategy(&self) -> StrategyConfig {
        let mut params = observer_pi_params();
        for (p, t) in self.0.iter().enumerate() {
            params[p].kp = t.0;
            params[p].ki = t.1;
            params[p].tau_filter_s = t.2;
            params[p].tau_sensor_s = t.3;
        }
        StrategyConfig::ObserverPi(params)
    }
}

/// Per-zone `(kp, ki, kd, _)` for the plain PID baseline.
///
/// The fourth slot is unused padding so the flat layout is shared with
/// [`ObserverPiTuning`]; it is pinned to a single value so the optimiser cannot
/// waste evaluations on it.
#[derive(Debug, Clone, Copy)]
struct PidTuning(PerZone4);

impl PidTuning {
    fn from_production() -> Self {
        Self(ZoneTuning::PRODUCTION.map(|t| (t.kp, t.ki, t.kd, 0.0)))
    }
}

impl Tunable for PidTuning {
    fn bounds() -> Vec<(&'static str, f64, f64)> {
        zone_bounds(0.005, 1.5) // kp
            .chain(zone_bounds(0.0, 0.01)) // ki
            .chain(zone_bounds(0.0, 0.2)) // kd
            .chain(zone_bounds(0.0, 0.0)) // unused
            .collect()
    }
    fn to_vec(&self) -> Vec<f64> {
        flatten(&self.0)
    }
    fn from_vec(v: &[f64]) -> Self {
        Self(unflatten(v))
    }
    fn strategy(&self) -> StrategyConfig {
        StrategyConfig::Pid(self.0.map(|t| ZoneTuning {
            kp: t.0,
            ki: t.1,
            kd: t.2,
        }))
    }
}

fn clamped<T: Tunable>(v: &[f64]) -> Vec<f64> {
    T::bounds()
        .iter()
        .zip(v)
        .map(|((_, lo, hi), x)| x.clamp(*lo, *hi))
        .collect()
}

/// Multi-start Nelder-Mead. Restarts from random points inside the bounds,
/// because a single descent on this objective reliably parks in a corner.
fn search<T: Tunable>(start: T, restarts: usize, evaluations: usize, seed: u64) -> T {
    let mut rng = Rng::new(seed);
    let bounds = T::bounds();
    let mut best = start.to_vec();
    let mut best_cost = total_cost(&start.strategy(), true);
    println!("start cost {best_cost:.0}");

    for r in 0..restarts {
        let init = if r == 0 {
            best.clone()
        } else {
            bounds
                .iter()
                .map(|(_, lo, hi)| rng.range(*lo, *hi))
                .collect()
        };

        // Step proportional to each parameter's own range, so all dimensions
        // move comparably however differently they are scaled. The optimiser
        // takes one scalar, so search in normalised units and widen inside the
        // cost closure.
        let span: Vec<f64> = bounds.iter().map(|(_, lo, hi)| hi - lo).collect();
        let to_real = |x: &[f64]| -> Vec<f64> {
            clamped::<T>(&x.iter().zip(&span).map(|(v, s)| v * s).collect::<Vec<_>>())
        };
        let normalised: Vec<f64> = init
            .iter()
            .zip(&span)
            .map(|(v, s)| if *s > 0.0 { v / s } else { 0.0 })
            .collect();

        let outcome = optimize::nelder_mead(
            &normalised,
            |x| total_cost(&T::from_vec(&to_real(x)).strategy(), true),
            optimize::Options {
                max_evaluations: evaluations,
                initial_step: 0.15,
                min_step: 0.005,
                ..optimize::Options::default()
            },
        );

        let v = to_real(&outcome.point);
        let c = total_cost(&T::from_vec(&v).strategy(), true);
        if c < best_cost {
            best_cost = c;
            best = v;
            println!("  restart {r}: new best {best_cost:.0}");
        } else {
            println!("  restart {r}: {c:.0}");
        }
    }
    T::from_vec(&best)
}

// ---------------------------------------------------------------- main

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let flag = |name: &str| args.iter().any(|a| a == name);
    let value = |name: &str| {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };

    if flag("--identify") {
        identify();
        return;
    }

    if let Some(which) = value("--search") {
        let restarts = value("--restarts")
            .and_then(|v| v.parse().ok())
            .unwrap_or(6usize);
        let evaluations = value("--evaluations")
            .and_then(|v| v.parse().ok())
            .unwrap_or(400usize);
        let seed = value("--seed")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0x9E37_79B9_7F4A_7C15u64);

        match which.as_str() {
            "observer-pi" => {
                let p = observer_pi_params();
                let start = ObserverPiTuning(
                    [0, 1, 2, 3].map(|i| (p[i].kp, p[i].ki, p[i].tau_filter_s, p[i].tau_sensor_s)),
                );
                let best = search(start, restarts, evaluations, seed);
                println!("\n{:#?}", best.0);
                report(&best.strategy());
            }
            "pid" => {
                let best = search(PidTuning::from_production(), restarts, evaluations, seed);
                println!("\n{:#?}", best.0);
                report(&best.strategy());
            }
            other => panic!("unknown strategy {other}; try observer-pi or pid"),
        }
        return;
    }

    let strategies = [
        StrategyConfig::Pid(ZoneTuning::PRODUCTION),
        StrategyConfig::ObserverPi(observer_pi_params()),
    ];

    let started = std::time::Instant::now();
    let mut summary = Vec::new();
    for s in &strategies {
        report(s);
        summary.push((s.name(), total_cost(s, false)));
    }

    println!("\n================ summary ================");
    println!("  {:<14} {:>12}", "strategy", "total cost");
    for (name, cost) in &summary {
        println!("  {name:<14} {cost:>12.0}");
    }
    println!(
        "\n(cost = seconds to settle, plus {OVERSHOOT_WEIGHT:.0} s per K of overshoot beyond \
         each zone's budget {FREE_OVERSHOOT_K:?}; summed over {} profiles x {} plants)",
        PROFILES.len(),
        plant_family().len()
    );
    println!("benchmark took {:.1} s", started.elapsed().as_secs_f64());
}
