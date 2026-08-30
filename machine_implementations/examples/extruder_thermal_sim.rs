//! Headless runner for the extruder heating simulation.
//!
//! ```text
//! cargo run --release -p machine_implementations --example extruder_thermal_sim -- --help
//! ```

use std::time::Duration;

use machine_implementations::extruder1::simulation::{
    ExtruderThermalParams, Scenario, SimConfig, ThermalSim, Zone, ZoneTuning,
    fit::{self, RecordedRun},
};

const HELP: &str = "\
extruder_thermal_sim — offline simulation of the extruder's 4 heating zones

USAGE:
    --scenario <name>     run a scenario (default: recorded-heatup)
    --out <path>          write the trace as CSV
    --compare             run `recorded-heatup` and score it against the real recording
    --fit [path]          calibrate the model against a recording (default: the built-in one)
    --evals <n>           optimiser budget for --fit (default 300)
    --autotune <zone>     relay-autotune one zone and print the suggested gains
    --kp/--ki/--kd <v>    set a gain on every zone, or `zone=value` for one
                          (repeatable: --kp 0.05 --kp nozzle=0.02)
    --set <key>=<value>   override one model coefficient (repeatable)
    --dt-ctrl <s>         controller period (default 0.001, the real busy loop)
    --sensor-period <s>   EL3204 refresh period (default 0.25)
    --list                list scenario names
    --help

SCENARIOS:
    recorded-heatup cold-start nozzle-only step-up
    single-front single-middle single-back single-nozzle
";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let flag = |name: &str| args.iter().any(|a| a == name);
    let value = |name: &str| {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };

    if flag("--help") || flag("-h") {
        print!("{HELP}");
        return;
    }
    if flag("--list") {
        for n in Scenario::NAMES {
            println!("{n}");
        }
        return;
    }

    let mut params = ExtruderThermalParams::calibrated();
    let mut config = SimConfig::default();

    // --set key=value, repeatable, for exploring one coefficient at a time.
    for (i, a) in args.iter().enumerate() {
        if a != "--set" {
            continue;
        }
        let Some(kv) = args.get(i + 1) else {
            panic!("--set needs key=value");
        };
        let (k, v) = kv
            .split_once('=')
            .unwrap_or_else(|| panic!("--set expects key=value, got {kv}"));
        let v: f64 = v
            .parse()
            .unwrap_or_else(|e| panic!("--set {k}: {v} is not a number ({e})"));
        assert!(params.set_by_name(k, v), "unknown parameter {k}");
    }

    // --kp/--ki/--kd take either a bare value (every zone) or `zone=value`, and
    // are repeatable, so later flags refine earlier ones:
    //   --kp 0.05 --kp nozzle=0.02
    // The four zones are genuinely different plants — the nozzle is ~6 kg of
    // uninsulated steel on a 200 W band, the barrel zones are ~4.4 kg insulated
    // on 700 W — so tuning them apart is usually the point.
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
            let t = &mut config.tuning[p];
            match gain {
                0 => t.kp = v,
                1 => t.ki = v,
                _ => t.kd = v,
            }
        }
    }

    if let Some(v) = value("--dt-ctrl").and_then(|v| v.parse::<f64>().ok()) {
        config.dt_ctrl_s = v;
    }
    if let Some(v) = value("--sensor-period").and_then(|v| v.parse::<f64>().ok()) {
        config.sensor_period_s = v;
    }

    if flag("--fit") {
        let run = match value("--fit").filter(|p| !p.starts_with("--")) {
            Some(path) => {
                let text = std::fs::read_to_string(&path)
                    .unwrap_or_else(|e| panic!("cannot read {path}: {e}"));
                RecordedRun::parse(&text).unwrap_or_else(|e| panic!("cannot parse {path}: {e}"))
            }
            None => RecordedRun::reference(),
        };
        let evals = value("--evals")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(300);
        println!(
            "fitting against {:.0} s of recording, budget {evals} evaluations...",
            run.duration_s()
        );
        let outcome = fit::fit(&run, &params, evals);
        print_residual("before", &outcome.starting_residual);
        print_residual("after ", &outcome.residual);
        println!("\n{} evaluations", outcome.evaluations);
        print_params(&outcome.params);
        let pinned = outcome.params.pinned_parameters(0.01);
        let unexpected: Vec<&str> = pinned
            .iter()
            .copied()
            .filter(|n| !ExtruderThermalParams::EXPECTED_PINNED.contains(n))
            .collect();
        if !pinned.is_empty() {
            println!("\npinned to a bound: {}", pinned.join(", "));
        }
        if unexpected.is_empty() {
            println!(
                "  (all expected: these are unidentifiable from a single all-zones run,\n\
                 \x20  see ExtruderThermalParams::EXPECTED_PINNED)"
            );
        } else {
            println!(
                "\nWARNING: {} unexpected: {}.\n\
                 Either the model is missing a mechanism, or this recording cannot\n\
                 identify them. Widening the bound would only hide it \u{2014} record the\n\
                 single-zone scenarios instead.",
                unexpected.len(),
                unexpected.join(", ")
            );
        }
        return;
    }

    if let Some(zone_name) = value("--autotune") {
        let zone = Zone::ALL
            .iter()
            .find(|z| z.name() == zone_name)
            .copied()
            .unwrap_or_else(|| {
                panic!("unknown zone {zone_name}; expected front/middle/back/nozzle")
            });
        autotune(params, config, zone);
        return;
    }

    if flag("--compare") {
        compare(params, config);
        return;
    }

    let name = value("--scenario").unwrap_or_else(|| "recorded-heatup".to_owned());
    let scenario =
        Scenario::by_name(&name).unwrap_or_else(|| panic!("unknown scenario {name}; try --list"));

    params.ambient_c = params.ambient_c.max(0.0);
    let tuning = config.tuning;
    let started = std::time::Instant::now();
    let mut sim = ThermalSim::new(params, config);
    let trace = sim.run(&scenario);
    let wall = started.elapsed();

    println!(
        "scenario '{}': {:.0} s simulated in {:.2} s wall ({:.0}x realtime)\n",
        scenario.name,
        scenario.duration_s,
        wall.as_secs_f64(),
        scenario.duration_s / wall.as_secs_f64().max(1e-9)
    );
    print_summary(&trace, &tuning);

    if let Some(path) = value("--out") {
        std::fs::write(&path, trace.to_csv())
            .unwrap_or_else(|e| panic!("cannot write {path}: {e}"));
        println!("\nwrote {} samples to {path}", trace.samples.len());
    }
}

fn print_summary(
    trace: &machine_implementations::extruder1::simulation::Trace,
    tuning: &[ZoneTuning; 4],
) {
    println!(
        "{:<8} {:>7} {:>8} {:>8} {:>10} {:>9} {:>9} {:>8} {:>7}",
        "zone", "kp/ki/kd", "setpt", "peak", "overshoot", "t90", "final", "kWh", "relay"
    );
    for zone in Zone::ALL {
        let t90 = trace
            .rise_time_s(zone, 0.9)
            .map_or("never".to_owned(), |v| format!("{v:.0} s"));
        let t = tuning[zone.port()];
        println!(
            "{:<8} {:>7} {:>8.1} {:>8.1} {:>+10.1} {:>9} {:>9.1} {:>8.3} {:>7}",
            zone.name(),
            format!("{}/{}/{}", t.kp, t.ki, t.kd),
            trace.setpoints_c[zone.port()],
            trace.peak_c(zone),
            trace.overshoot_k(zone),
            t90,
            trace.final_c(zone),
            trace.energy_kwh(zone),
            trace.relay_switches(zone),
        );
    }
}

/// Run the recorded scenario closed loop and score it against the real machine.
fn compare(params: ExtruderThermalParams, config: SimConfig) {
    let run = RecordedRun::reference();
    let mut sim = ThermalSim::new(params, config);
    let trace = sim.run(&Scenario::recorded_heatup());

    println!("closed-loop simulation vs. the 2026-02-24 recording\n");
    println!(
        "{:<8} {:>18} {:>18} {:>16}",
        "zone", "peak  sim / real", "final sim / real", "t90 sim / real"
    );

    let measured_peak = |p: usize| {
        run.temperature_c
            .iter()
            .map(|r| r[p])
            .fold(f64::NEG_INFINITY, f64::max)
    };
    let measured_t90 = |p: usize, setpoint: f64| {
        let start = run.temperature_c[0][p];
        let target = 0.9f64.mul_add(setpoint - start, start);
        run.t_s
            .iter()
            .zip(&run.temperature_c)
            .find(|(_, r)| r[p] >= target)
            .map(|(t, _)| *t)
    };

    for zone in Zone::ALL {
        let p = zone.port();
        let real_peak = measured_peak(p);
        let real_final = run.temperature_c[run.temperature_c.len() - 1][p];
        let t90_sim = trace
            .rise_time_s(zone, 0.9)
            .map_or("never".to_owned(), |v| format!("{v:.0}"));
        let t90_real =
            measured_t90(p, trace.setpoints_c[p]).map_or("never".to_owned(), |v| format!("{v:.0}"));
        println!(
            "{:<8} {:>8.1} / {:<7.1} {:>8.1} / {:<7.1} {:>7} / {:<7}",
            zone.name(),
            trace.peak_c(zone),
            real_peak,
            trace.final_c(zone),
            real_final,
            t90_sim,
            t90_real,
        );
    }

    let residual = fit::residual(sim.model().params(), &run);
    println!();
    print_residual("open-loop replay RMS", &residual);
}

fn print_residual(label: &str, r: &fit::FitResidual) {
    print!("{label}: overall {:.2} K  |", r.overall_k);
    for zone in Zone::ALL {
        print!(" {} {:.2}", zone.name(), r.per_zone_k[zone.port()]);
    }
    println!();
}

fn print_params(p: &ExtruderThermalParams) {
    println!("\nfitted parameters:");
    println!("    band_contact_h:        {:.1}", p.band_contact_h);
    println!(
        "    band_heat_cap J/m2K:   {:.0}",
        p.band_heat_capacity_j_per_m2_k
    );
    println!("    k_insulation:          {:.4}", p.k_insulation);
    println!("    bare_convection_coeff: {:.3}", p.bare_convection_coeff);
    println!("    bare_emissivity:       {:.3}", p.bare_emissivity);
    println!("    flange_contact_h:      {:.1}", p.flange_contact_h);
    println!("    gearbox_sink_g:        {:.3}", p.gearbox_sink_g);
    println!("    bore_gap_h:            {:.1}", p.bore_gap_h);
    println!("    sensor_tau_s:          {:.2}", p.sensor_tau_s);
}

fn autotune(params: ExtruderThermalParams, config: SimConfig, zone: Zone) {
    use control_core::controllers::pid_autotuner::{AutoTuneConfig, PidAutoTuner};

    let mut sim = ThermalSim::new(params, config);
    let tuner_config = AutoTuneConfig {
        tune_delta: 5.0,
        max_power: 1.0,
        max_duration: Duration::from_secs(6 * 3600),
    };
    let mut tuner = PidAutoTuner::new(tuner_config);

    let result = sim.run_autotune(zone, 175.0, 22.0, 6.0 * 3600.0, &mut tuner);
    match result {
        Some(r) => {
            println!("autotune on {} converged:", zone.name());
            println!(
                "    kp = {:.5}\n    ki = {:.5}\n    kd = {:.5}",
                r.kp, r.ki, r.kd
            );
            println!("    (ku = {:.4}, tu = {:.1} s)", r.ku, r.tu);
            println!(
                "\ncompare with the shipping gains: kp = {:.3}, ki = {:.3}, kd = {:.3}",
                ZoneTuning::PRODUCTION.kp,
                ZoneTuning::PRODUCTION.ki,
                ZoneTuning::PRODUCTION.kd
            );
        }
        None => println!(
            "autotune on {} did not converge within the simulated budget",
            zone.name()
        ),
    }
}
