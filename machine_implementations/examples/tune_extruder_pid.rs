//! Offline per-zone PID search for the extruder's four heating zones.
//!
//! Each heater gets its own gains (`kp`, `ki` and `kd`, all searched per zone)
//! and is scored against the three "stepped-down" target profiles, starting
//! from a 22 °C cold start each time.
//!
//! ## ki, kd and anti-windup
//!
//! All three gains are free parameters and are searched per zone.
//!
//! - `ki` is only usable because `TemperatureController` now calls
//!   `PidController::update_with_antiwindup`, which freezes the integral while
//!   the output is saturated. Without that, any `ki > 0` winds up on the
//!   cold-start ramp and the zone overshoots by 100+ K.
//! - `kd` multiplies EL3204 quantisation noise (`ed = 0.1 / 0.001 = 100 K/s`
//!   on every LSB step). The search over `kd` confirmed it adds only relay
//!   chatter and never better settling, so it is pinned to 0.
//!
//! ## Usage
//!
//! ```text
//! cargo run --release -p machine_implementations --example tune_extruder_pid -- \
//!     --search 300 --threads 8
//!
//! cargo run --release -p machine_implementations --example tune_extruder_pid -- \
//!     --kp 0.3 --kp nozzle=0.5   # evaluate one candidate (report only)
//! ```
//!
//! Setpoints are stored in scenario order `[front, middle, back, nozzle]`; the
//! profile names spell out the `nozzle/front/middle/back` order the operator
//! enters on the UI.

use std::time::Instant;

use machine_implementations::extruder1::simulation::{
    geometry::Zone,
    harness::{SimConfig, ThermalSim, Trace, ZoneTuning},
    params::ExtruderThermalParams,
    scenario::Scenario,
};

type Tuning4 = [ZoneTuning; 4];

/// Time-step settings. The search runs coarse and fast; the winner is then
/// validated at the real timesteps (plant 0.01 s, control 0.001 s).
#[derive(Debug, Clone, Copy)]
struct SimSettings {
    dt_plant_s: f64,
    dt_ctrl_s: f64,
}

impl SimSettings {
    const REAL: Self = Self {
        dt_plant_s: 0.01,
        dt_ctrl_s: 0.001,
    };
    const FAST: Self = Self {
        dt_plant_s: 0.05,
        dt_ctrl_s: 0.01,
    };
}

/// Profiles as entered on the UI in `nozzle, front, middle, back` order, stored
/// here in the simulator's `[front, middle, back, nozzle]` order.
const PROFILES: &[(&str, [f64; 4])] = &[
    // name, [front, middle, back, nozzle]
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
const FREE_OVERSHOOT_K: f64 = 3.0;
const OVERSHOOT_WEIGHT: f64 = 40.0; // seconds of cost per K of excess overshoot
const FINAL_ERR_WEIGHT: f64 = 250.0; // per K of final error when never settled
const NEVER_SETTLED_BASE: f64 = 2.5; // × DURATION_S when a zone never settles

fn profile(sp: [f64; 4]) -> Scenario {
    Scenario {
        name: String::new(),
        initial_c: 22.0,
        duration_s: DURATION_S,
        setpoints_c: sp,
        heating_enabled_at_s: 0.0,
        changes: Vec::new(),
    }
}

/// First instant after which the sensor stays within `tol` of the setpoint for
/// the rest of the run. `None` if it never stabilised within the horizon.
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

/// Per-zone cost: the time to settle within `SETTLE_TOL_K` of the setpoint,
/// plus an overshoot penalty. A zone that never settles (droop, or a setpoint
/// below its neighbours' conduction floor) is punished by a large finite cost
/// proportional to how far it ends up from the setpoint, so the search still
/// prefers gains that get it *to* temperature rather than just avoiding
/// overshoot.
fn zone_cost(trace: &Trace, zone: Zone) -> f64 {
    let p = zone.port();
    let sp = trace.setpoints_c[p];
    let overshoot = trace.overshoot_k(zone).max(0.0);
    match settle_time(trace, zone, SETTLE_TOL_K) {
        Some(t) => {
            let mut c = t;
            if overshoot > FREE_OVERSHOOT_K {
                c += OVERSHOOT_WEIGHT * (overshoot - FREE_OVERSHOOT_K);
            }
            c
        }
        None => {
            NEVER_SETTLED_BASE * DURATION_S + FINAL_ERR_WEIGHT * (trace.final_c(zone) - sp).abs()
        }
    }
}

fn total_cost(tuning: &Tuning4, s: SimSettings) -> f64 {
    let params = ExtruderThermalParams::calibrated();
    let mut total = 0.0;
    for (_, sp) in PROFILES {
        let config = SimConfig {
            tuning: *tuning,
            dt_plant_s: s.dt_plant_s,
            dt_ctrl_s: s.dt_ctrl_s,
            ..SimConfig::default()
        };
        let mut sim = ThermalSim::new(params.clone(), config);
        let trace = sim.run(&profile(*sp));
        for zone in Zone::ALL {
            total += zone_cost(&trace, zone);
        }
    }
    total
}

fn report(tuning: &Tuning4, s: SimSettings) {
    let params = ExtruderThermalParams::calibrated();
    println!(
        "gains: front kp={:.3} ki={:.6} kd={:.4} | middle kp={:.3} ki={:.6} kd={:.4} | back kp={:.3} ki={:.6} kd={:.4} | nozzle kp={:.3} ki={:.6} kd={:.4}",
        tuning[0].kp,
        tuning[0].ki,
        tuning[0].kd,
        tuning[1].kp,
        tuning[1].ki,
        tuning[1].kd,
        tuning[2].kp,
        tuning[2].ki,
        tuning[2].kd,
        tuning[3].kp,
        tuning[3].ki,
        tuning[3].kd,
    );
    for (name, sp) in PROFILES {
        let config = SimConfig {
            tuning: *tuning,
            dt_plant_s: s.dt_plant_s,
            dt_ctrl_s: s.dt_ctrl_s,
            ..SimConfig::default()
        };
        let mut sim = ThermalSim::new(params.clone(), config);
        let trace = sim.run(&profile(*sp));
        println!("\n{name}");
        println!(
            "  {:<8} {:>7} {:>8} {:>10} {:>8} {:>8} {:>8}",
            "zone", "setpt", "peak", "overshoot", "t90", "settle", "final"
        );
        for zone in Zone::ALL {
            let p = zone.port();
            let t90 = trace
                .rise_time_s(zone, 0.9)
                .map_or("never".to_owned(), |v| format!("{v:.0}"));
            let settle = settle_time(&trace, zone, SETTLE_TOL_K)
                .map_or("never".to_owned(), |v| format!("{v:.0}"));
            println!(
                "  {:<8} {:>7.0} {:>8.1} {:>+10.1} {:>8} {:>8} {:>8.1}",
                zone.name(),
                sp[p],
                trace.peak_c(zone),
                trace.overshoot_k(zone),
                t90,
                settle,
                trace.final_c(zone),
            );
        }
    }
}

// ---- tiny xorshift RNG so the search is reproducible without deps ----

struct Rng(u64);

impl Rng {
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
    fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.f64()
    }
    fn choose<'a, T>(&mut self, xs: &'a [T]) -> &'a T {
        &xs[(self.next_u64() % xs.len() as u64) as usize]
    }
}

/// Gain bounds for the search. `kp` is the real knob; `ki` and `kd` are kept
/// in their useful range (see the module doc).
const KP_LO: f64 = 0.02;
const KP_HI: f64 = 0.8;
const KI_HI: f64 = 0.01;
const KI_OPTIONS: [f64; 6] = [0.0, 0.0005, 0.001, 0.002, 0.004, 0.008];
// kd multiplies EL3204 quantisation noise and is set to 0: the search over
// kd confirmed it only adds relay chatter, never better settling.
const KD_OPTIONS: [f64; 1] = [0.0];

fn random_tuning(rng: &mut Rng) -> Tuning4 {
    let mut t = ZoneTuning::PRODUCTION;
    for p in 0..4 {
        t[p] = ZoneTuning {
            kp: rng.range(KP_LO, KP_HI),
            ki: *rng.choose(&KI_OPTIONS),
            kd: *rng.choose(&KD_OPTIONS),
        };
    }
    t
}

/// Perturb `base`: kp multiplicatively, ki/kd mostly kept with an occasional
/// jump to a different option so refinement hones in instead of re-rolling.
fn jitter(rng: &mut Rng, base: &Tuning4) -> Tuning4 {
    let mut t = *base;
    for p in 0..4 {
        t[p].kp = (base[p].kp * rng.range(0.7, 1.3)).clamp(KP_LO, KP_HI);
        t[p].ki = if rng.f64() < 0.6 {
            base[p].ki
        } else {
            *rng.choose(&KI_OPTIONS)
        };
        t[p].kd = if rng.f64() < 0.6 {
            base[p].kd
        } else {
            *rng.choose(&KD_OPTIONS)
        };
    }
    t
}

// ---- Nelder-Mead optimisation over the 8 free gains (kp/ki per zone) ----

/// Flat parameter vector: `[kp_front, kp_middle, kp_back, kp_nozzle,
/// ki_front, ki_middle, ki_back, ki_nozzle]`.
fn to_vec(t: &Tuning4) -> Vec<f64> {
    let mut v = Vec::with_capacity(8);
    for p in 0..4 {
        v.push(t[p].kp);
    }
    for p in 0..4 {
        v.push(t[p].ki);
    }
    v
}

fn from_vec(v: &[f64]) -> Tuning4 {
    let mut t = [ZoneTuning {
        kp: 0.0,
        ki: 0.0,
        kd: 0.0,
    }; 4];
    for p in 0..4 {
        t[p] = ZoneTuning {
            kp: v[p],
            ki: v[4 + p],
            kd: 0.0,
        };
    }
    t
}

fn clamp_params(v: &mut [f64]) {
    for p in 0..4 {
        v[p] = v[p].clamp(KP_LO, KP_HI);
    }
    for p in 4..8 {
        v[p] = v[p].clamp(0.0, KI_HI);
    }
}

/// Nelder–Mead simplex optimisation of a deterministic black-box cost.
/// Converges on a local minimum without needing gradients.
fn nelder_mead<F: Fn(&[f64]) -> f64>(
    f: &F,
    x0: &[f64],
    step: &[f64],
    iters: usize,
) -> (Vec<f64>, f64) {
    let n = x0.len();
    let mut simplex: Vec<Vec<f64>> = vec![x0.to_vec()];
    for i in 0..n {
        let mut p = x0.to_vec();
        p[i] += step[i];
        clamp_params(&mut p);
        simplex.push(p);
    }
    let mut fs: Vec<f64> = simplex.iter().map(|p| f(p)).collect();

    for _ in 0..iters {
        // Rank the simplex by cost, ascending.
        let mut order: Vec<usize> = (0..simplex.len()).collect();
        order.sort_by(|&a, &b| fs[a].partial_cmp(&fs[b]).unwrap());
        let sorted: Vec<Vec<f64>> = order.iter().map(|&i| simplex[i].clone()).collect();
        let sfs: Vec<f64> = order.iter().map(|&i| fs[i]).collect();
        simplex = sorted;
        fs = sfs;

        let best = simplex[0].clone();
        let worst = simplex[n].clone();

        // Centroid of all points except the worst.
        let mut c = vec![0.0; n];
        for p in simplex.iter().take(n) {
            for i in 0..n {
                c[i] += p[i];
            }
        }
        for ci in c.iter_mut() {
            *ci /= n as f64;
        }

        // Reflect.
        let mut r = vec![0.0; n];
        for i in 0..n {
            r[i] = c[i] + (c[i] - worst[i]);
        }
        clamp_params(&mut r);
        let fr = f(&r);

        if fr < fs[0] {
            // Expand.
            let mut e = vec![0.0; n];
            for i in 0..n {
                e[i] = c[i] + 2.0 * (r[i] - c[i]);
            }
            clamp_params(&mut e);
            let fe = f(&e);
            if fe < fr {
                simplex[n] = e;
                fs[n] = fe;
            } else {
                simplex[n] = r;
                fs[n] = fr;
            }
        } else if fr < fs[n - 1] {
            simplex[n] = r;
            fs[n] = fr;
        } else {
            let mut improved = false;
            if fr < fs[n] {
                // Outside contraction.
                let mut oc = vec![0.0; n];
                for i in 0..n {
                    oc[i] = c[i] + 0.5 * (r[i] - c[i]);
                }
                clamp_params(&mut oc);
                let foc = f(&oc);
                if foc <= fr {
                    simplex[n] = oc;
                    fs[n] = foc;
                    improved = true;
                }
            } else {
                // Inside contraction.
                let mut ic = vec![0.0; n];
                for i in 0..n {
                    ic[i] = c[i] + 0.5 * (worst[i] - c[i]);
                }
                clamp_params(&mut ic);
                let fic = f(&ic);
                if fic < fs[n] {
                    simplex[n] = ic;
                    fs[n] = fic;
                    improved = true;
                }
            }
            if !improved {
                // Shrink everything towards the best point.
                for j in 1..=n {
                    for i in 0..n {
                        simplex[j][i] = best[i] + 0.5 * (simplex[j][i] - best[i]);
                    }
                    clamp_params(&mut simplex[j]);
                    fs[j] = f(&simplex[j]);
                }
            }
        }
    }

    let mut best_i = 0;
    for i in 1..=n {
        if fs[i] < fs[best_i] {
            best_i = i;
        }
    }
    (simplex[best_i].clone(), fs[best_i])
}

fn evaluate_parallel(cands: &[Tuning4], threads: usize, s: SimSettings) -> Vec<f64> {
    let n = cands.len();
    if threads <= 1 || n < threads * 2 {
        return cands.iter().map(|t| total_cost(t, s)).collect();
    }
    let chunk = n.div_ceil(threads);
    let mut handles = Vec::new();
    for ch in cands.chunks(chunk) {
        let ch = ch.to_vec();
        handles.push(std::thread::spawn(move || {
            ch.iter().map(|t| total_cost(t, s)).collect::<Vec<_>>()
        }));
    }
    let mut out = Vec::with_capacity(n);
    for h in handles {
        out.extend(h.join().expect("search thread panicked"));
    }
    out
}

fn print_best(t: &Tuning4) {
    println!(
        "best: front kp={:.3} ki={:.6} kd={:.4} | middle kp={:.3} ki={:.6} kd={:.4} | back kp={:.3} ki={:.6} kd={:.4} | nozzle kp={:.3} ki={:.6} kd={:.4}",
        t[0].kp,
        t[0].ki,
        t[0].kd,
        t[1].kp,
        t[1].ki,
        t[1].kd,
        t[2].kp,
        t[2].ki,
        t[2].kd,
        t[3].kp,
        t[3].ki,
        t[3].kd,
    );
}

/// Jitter `best` and keep the cheapest candidate, evaluated at `s_real`.
fn real_refine(
    best: Tuning4,
    evals: usize,
    threads: usize,
    seed: u64,
    s_real: SimSettings,
) -> (Tuning4, f64) {
    let mut rng = Rng(seed);
    let mut cands = vec![best];
    for _ in 0..evals {
        cands.push(jitter(&mut rng, &best));
    }
    let costs = evaluate_parallel(&cands, threads, s_real);
    let mut bb = best;
    let mut bc = costs[0];
    for (t, &c) in cands.iter().zip(&costs).skip(1) {
        if c < bc {
            bb = *t;
            bc = c;
        }
    }
    (bb, bc)
}

fn search(
    evals: usize,
    threads: usize,
    seed: u64,
    s_fast: SimSettings,
    s_real: SimSettings,
) -> Tuning4 {
    let mut rng = Rng(seed);
    let mut candidates: Vec<Tuning4> = vec![ZoneTuning::PRODUCTION];
    for _ in 0..evals {
        candidates.push(random_tuning(&mut rng));
    }

    let t_start = Instant::now();
    let costs = evaluate_parallel(&candidates, threads, s_fast);

    let mut best_i = 0;
    for i in 0..costs.len() {
        if costs[i] < costs[best_i] {
            best_i = i;
        }
    }
    let mut best = candidates[best_i];
    let mut best_cost = costs[best_i];
    println!(
        "broad search (fast ts): {} candidates in {:.0}s; best cost {:.0} (production {:.0})",
        evals + 1,
        t_start.elapsed().as_secs_f64(),
        best_cost,
        costs[0]
    );

    // Refinement rounds: jitter all 12 gains around the best, keep global best.
    for round in 1..=2 {
        let round_evals = evals / 2;
        let mut refine_cands = Vec::with_capacity(round_evals);
        for _ in 0..round_evals {
            refine_cands.push(jitter(&mut rng, &best));
        }
        let rcosts = evaluate_parallel(&refine_cands, threads, s_fast);
        for (t, &c) in refine_cands.iter().zip(&rcosts) {
            if c < best_cost {
                best = *t;
                best_cost = c;
            }
        }
        println!("refine round {round} (fast ts): best cost {best_cost:.0}");
    }

    // Final refinement at the REAL timesteps: the derivative term scales with
    // 1/dt_ctrl, so kd has to be judged against the real 1 kHz control loop.
    let real_evals = (evals / 8).clamp(60, 200);
    let (real_best, real_cost) = real_refine(best, real_evals, threads, seed, s_real);
    println!("final refine (real ts): {real_evals} candidates, best cost {real_cost:.0}");
    print_best(&real_best);
    real_best
}

/// Multistart Nelder–Mead: run `restarts` independent simplex searches in
/// parallel (one per thread), each converging on a local minimum of the fast-
/// timestep cost, then polish the best at the real timesteps.
fn optimize(
    restarts: usize,
    iters: usize,
    threads: usize,
    seed: u64,
    s_fast: SimSettings,
    s_real: SimSettings,
) -> Tuning4 {
    let step = [0.2, 0.2, 0.2, 0.2, 0.002, 0.002, 0.002, 0.002];
    let mut rng = Rng(seed);
    let mut starts: Vec<Vec<f64>> = vec![to_vec(&ZoneTuning::PRODUCTION)];
    for _ in 1..restarts {
        starts.push(to_vec(&random_tuning(&mut rng)));
    }

    let t_start = Instant::now();
    let mut handles = Vec::new();
    for (i, start) in starts.into_iter().enumerate() {
        handles.push(std::thread::spawn(move || {
            let f = |v: &[f64]| total_cost(&from_vec(v), s_fast);
            let (v, cost) = nelder_mead(&f, &start, &step, iters);
            (i, v, cost)
        }));
    }
    let mut best: Option<(Tuning4, f64)> = None;
    for h in handles {
        let (i, v, cost) = h.join().expect("optimizer thread panicked");
        println!("restart {i}: Nelder-Mead -> cost {cost:.0}");
        match &best {
            Some((_, c)) if *c <= cost => {}
            _ => best = Some((from_vec(&v), cost)),
        }
    }
    let (best_t, best_c) = best.expect("at least one restart");
    println!(
        "Nelder-Mead best (fast ts): cost {best_c:.0} in {:.0}s",
        t_start.elapsed().as_secs_f64()
    );

    let (real_best, real_cost) = real_refine(best_t, 80, threads, seed, s_real);
    println!("real-timestep refine: best cost {real_cost:.0}");
    print_best(&real_best);
    real_best
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let flag = |name: &str| args.iter().any(|a| a == name);
    let value = |name: &str| {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };

    let mut tuning = ZoneTuning::PRODUCTION;

    for (i, a) in args.iter().enumerate() {
        let gain = match a.as_str() {
            "--kp" => 0,
            "--ki" => 1,
            "--kd" => 2,
            _ => continue,
        };
        let spec = args
            .get(i + 1)
            .unwrap_or_else(|| panic!("{a} needs a value, or zone=value"));
        let (ports, raw) = match spec.split_once('=') {
            Some((name, v)) => {
                let zone = Zone::ALL
                    .iter()
                    .find(|z| z.name() == name)
                    .copied()
                    .unwrap_or_else(|| {
                        panic!("{a}: unknown zone {name}; expected front/middle/back/nozzle")
                    });
                (vec![zone.port()], v)
            }
            None => (
                Zone::ALL
                    .iter()
                    .copied()
                    .map(Zone::port)
                    .collect::<Vec<_>>(),
                spec.as_str(),
            ),
        };
        let v: f64 = raw
            .parse()
            .unwrap_or_else(|e| panic!("{a}: {raw} is not a number ({e})"));
        for p in ports {
            match gain {
                0 => tuning[p].kp = v,
                1 => tuning[p].ki = v,
                _ => tuning[p].kd = v,
            }
        }
    }

    if flag("--optimize") {
        let restarts = value("--optimize")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(8);
        let iters = value("--nm-iters")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(100);
        let threads = value("--threads")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(1);
        let seed = value("--seed")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0x9E37_79B9_7F4A_7C15);

        let mut s = SimSettings::FAST;
        if let Some(v) = value("--dt-plant").and_then(|v| v.parse::<f64>().ok()) {
            s.dt_plant_s = v;
        }
        if let Some(v) = value("--dt-ctrl").and_then(|v| v.parse::<f64>().ok()) {
            s.dt_ctrl_s = v;
        }
        println!(
            "optimize: {restarts} Nelder-Mead restarts x {iters} iterations (dt_plant={}, dt_ctrl={})",
            s.dt_plant_s, s.dt_ctrl_s
        );

        tuning = optimize(restarts, iters, threads, seed, s, SimSettings::REAL);

        println!("\n== validating winner at real timesteps (plant 0.01 s, ctrl 0.001 s) ==");
        report(&tuning, SimSettings::REAL);
        println!(
            "\ntotal cost at real timesteps: {:.0}",
            total_cost(&tuning, SimSettings::REAL)
        );
        return;
    }

    if flag("--search") {
        let evals = value("--search")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(300);
        let threads = value("--threads")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(1);
        let seed = value("--seed")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0x9E37_79B9_7F4A_7C15);

        // Coarse/fast timesteps for the search, overridable. The winner is
        // validated at the real timesteps afterwards.
        let mut s = SimSettings::FAST;
        if let Some(v) = value("--dt-plant").and_then(|v| v.parse::<f64>().ok()) {
            s.dt_plant_s = v;
        }
        if let Some(v) = value("--dt-ctrl").and_then(|v| v.parse::<f64>().ok()) {
            s.dt_ctrl_s = v;
        }
        println!(
            "search timesteps: dt_plant={} s, dt_ctrl={} s",
            s.dt_plant_s, s.dt_ctrl_s
        );

        tuning = search(evals, threads, seed, s, SimSettings::REAL);

        println!("\n== validating winner at real timesteps (plant 0.01 s, ctrl 0.001 s) ==");
        report(&tuning, SimSettings::REAL);
        println!(
            "\ntotal cost at real timesteps: {:.0}",
            total_cost(&tuning, SimSettings::REAL)
        );
        return;
    }

    let mut s = SimSettings::REAL;
    if let Some(v) = value("--dt-plant").and_then(|v| v.parse::<f64>().ok()) {
        s.dt_plant_s = v;
    }
    if let Some(v) = value("--dt-ctrl").and_then(|v| v.parse::<f64>().ok()) {
        s.dt_ctrl_s = v;
    }

    report(&tuning, s);
    println!(
        "\ntotal cost (lower is better): {:.0}",
        total_cost(&tuning, s)
    );
}
