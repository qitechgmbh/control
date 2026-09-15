//! Does the shipping heating configuration actually meet its requirements?
//!
//! The parameters themselves live in [`crate::extruder1::heating_params`] —
//! production code, no simulation needed. These tests point the thermal model at
//! them and assert the behaviour that justified shipping them: middle never
//! overshoots, nothing else overshoots much, every zone arrives, and none of it
//! is bought with a slower heat-up or more relay wear.

#![cfg(test)]

use control_core::controllers::heating::ObserverPiParams;

use super::harness::{
    REALISTIC_SENSOR_NOISE_C, SimConfig, StrategyConfig, ThermalSim, Trace, ZoneTuning,
    plant_family,
};
use super::params::ExtruderThermalParams;
use super::scenario::Scenario;
use crate::extruder1::heating_params::{AMBIENT_C, DEFAULT_MAX_CLAMP, PLANT, observer_pi_params};
use crate::extruder1::zone::Zone;

/// `[front, middle, back, nozzle]`, matching the simulator's port order.
const PROFILE_A: [f64; 4] = [200.0, 170.0, 150.0, 200.0];
const PROFILE_B: [f64; 4] = [180.0, 160.0, 140.0, 170.0];
const PROFILE_C: [f64; 4] = [150.0, 130.0, 100.0, 150.0];

fn scenario(setpoints_c: [f64; 4]) -> Scenario {
    Scenario {
        name: String::new(),
        initial_c: 22.0,
        duration_s: 6000.0,
        setpoints_c,
        heating_enabled_at_s: 0.0,
        changes: Vec::new(),
    }
}

fn run(strategy: StrategyConfig, params: ExtruderThermalParams, sp: [f64; 4]) -> Trace {
    let config = SimConfig {
        strategy,
        ..SimConfig::default()
    };
    ThermalSim::new(params, config).run(&scenario(sp))
}

fn shipping() -> StrategyConfig {
    StrategyConfig::ObserverPi(observer_pi_params())
}

/// The headline requirement, and the one the machine is judged on: middle
/// must not exceed its setpoint.
///
/// Checked on every plant in the family, not just the nominal fit — and on
/// the profiles where holding setpoint is physically possible at all. See
/// [`middle_cannot_be_held_below_its_neighbours`] for the case where it is
/// not, and why that is thermodynamics rather than tuning.
#[test]
fn middle_never_overshoots() {
    for params in plant_family() {
        let tau = params.sensor_tau_s;
        for sp in [PROFILE_B, PROFILE_C] {
            let trace = run(shipping(), params.clone(), sp);
            let overshoot = trace.overshoot_k(Zone::Middle);
            assert!(
                overshoot <= 0.5,
                "tau={tau:.0}, setpoints {sp:?}: middle overshot by {overshoot:+.2} K; \
                 it is the zone with no cold sink to shed an excursion into, so its \
                 budget is zero"
            );
        }
    }
}

/// Middle cannot be held below the steel around it.
///
/// On profile A the operator asks for front 200 and middle 170. With its own
/// heater never energised, middle still arrives at about 171 °C — conduction
/// from a neighbour 30 K hotter, across solid barrel steel. There is no
/// control law that beats this; the only actuator is a heater, and the
/// machine has no way to remove heat.
///
/// The test exists so that the +2 K seen on profile A is not mistaken for a
/// controller defect and "fixed" by detuning something that is already right.
#[test]
fn middle_cannot_be_held_below_its_neighbours() {
    let mut heater_off = PROFILE_A;
    heater_off[Zone::Middle.port()] = 0.0;
    let floor =
        run(shipping(), ExtruderThermalParams::calibrated(), heater_off).final_c(Zone::Middle);

    assert!(
        floor > PROFILE_A[Zone::Middle.port()],
        "this test is only meaningful while the floor ({floor:.1} C) is above \
         middle's {} C setpoint",
        PROFILE_A[Zone::Middle.port()]
    );

    // And the controller has to get within a hair of that floor rather than
    // sitting well above it.
    let trace = run(shipping(), ExtruderThermalParams::calibrated(), PROFILE_A);
    let settled = trace.final_c(Zone::Middle);
    assert!(
        settled < floor + 1.0,
        "middle settles at {settled:.1} C against an unheated floor of \
         {floor:.1} C, so {:.1} K of that really is the controller",
        settled - floor
    );
}

/// Everything else gets a small budget rather than none, because the zones
/// that *can* shed an excursion are not worth slowing the machine down for.
#[test]
fn the_other_zones_stay_within_budget() {
    for params in plant_family() {
        let tau = params.sensor_tau_s;
        for sp in [PROFILE_A, PROFILE_B, PROFILE_C] {
            let trace = run(shipping(), params.clone(), sp);
            for zone in [Zone::Front, Zone::Back, Zone::Nozzle] {
                let overshoot = trace.overshoot_k(zone);
                assert!(
                    overshoot <= 2.0,
                    "tau={tau:.0}, setpoints {sp:?}: {} overshot by {overshoot:+.2} K",
                    zone.name()
                );
            }
        }
    }
}

/// The other half of the request: no overshoot when the setpoint is raised
/// from an already-hot machine, which is a different regime from a cold
/// start because there is no long saturated ramp to hide behind.
#[test]
fn stepping_up_from_hot_does_not_overshoot() {
    let mut s = scenario(PROFILE_B);
    s.duration_s = 9000.0;
    s.changes.push((5000.0, PROFILE_B.map(|v| v + 20.0)));

    let config = SimConfig {
        strategy: shipping(),
        ..SimConfig::default()
    };
    let trace = ThermalSim::new(ExtruderThermalParams::calibrated(), config).run(&s);

    for zone in Zone::ALL {
        let p = zone.port();
        let peak_after = trace
            .samples
            .iter()
            .filter(|x| x.t_s >= 5000.0)
            .map(|x| x.sensor_c[p])
            .fold(f64::NEG_INFINITY, f64::max);
        let overshoot = peak_after - trace.setpoints_c[p];
        let budget = if zone == Zone::Middle { 2.0 } else { 2.5 };
        assert!(
            overshoot <= budget,
            "{} overshot the +20 K step by {overshoot:+.2} K",
            zone.name()
        );
    }
}

/// Avoiding overshoot by crawling would not be a fix. The new loop has to
/// arrive at least as fast as the PID it replaces.
#[test]
fn it_heats_no_slower_than_the_pid_it_replaces() {
    let baseline = StrategyConfig::Pid(ZoneTuning::PRODUCTION);
    for sp in [PROFILE_A, PROFILE_B, PROFILE_C] {
        let params = ExtruderThermalParams::calibrated();
        let old = run(baseline.clone(), params.clone(), sp);
        let new = run(shipping(), params, sp);

        for zone in Zone::ALL {
            let (Some(t_old), Some(t_new)) =
                (old.rise_time_s(zone, 0.9), new.rise_time_s(zone, 0.9))
            else {
                panic!("{} failed to reach 90 % of setpoint", zone.name());
            };
            // Middle is allowed more, because it is the zone that was given
            // a zero-overshoot budget and the two are the same trade. The
            // PID's faster t90 there is not really speed: it reaches 90 % of
            // setpoint quickly *on its way past it by 15 K*, which is the
            // behaviour being removed.
            let allowed = if zone == Zone::Middle { 1.20 } else { 1.10 };
            assert!(
                t_new <= t_old * allowed,
                "setpoints {sp:?}: {} t90 went from {t_old:.0} s to {t_new:.0} s, \
                 more than the {:.0} % this zone trades for its overshoot budget",
                zone.name(),
                (allowed - 1.0) * 100.0,
            );
        }
    }
}

/// The old loop's other failure: with `ki = 0` the nozzle parked several
/// kelvin below setpoint forever. Feedforward, not integral, is what fixes
/// that — so it has to actually arrive.
#[test]
fn every_zone_reaches_its_setpoint() {
    for sp in [PROFILE_A, PROFILE_B, PROFILE_C] {
        let trace = run(shipping(), ExtruderThermalParams::calibrated(), sp);
        for zone in Zone::ALL {
            // Middle on profile A is held above setpoint by its neighbours;
            // that is the floor, not droop.
            if zone == Zone::Middle && sp == PROFILE_A {
                continue;
            }
            let error = trace.final_c(zone) - sp[zone.port()];
            assert!(
                error.abs() < 1.0,
                "setpoints {sp:?}: {} settled {error:+.2} K from setpoint",
                zone.name()
            );
        }
    }
}

/// Temperature must not be bought with SSR wear.
///
/// The bar is deliberately coarse. A 500 ms PWM window caps switching at two
/// transitions per window, and every zone that is modulating at all runs
/// close to that ceiling, so small differences here are phase, not wear.
/// What this is guarding against is a control law that chatters — an order
/// of magnitude more switching, not ten percent.
#[test]
fn relay_wear_is_no_worse_than_the_pid() {
    let baseline = StrategyConfig::Pid(ZoneTuning::PRODUCTION);
    for sp in [PROFILE_A, PROFILE_B, PROFILE_C] {
        let params = ExtruderThermalParams::calibrated();
        let old = run(baseline.clone(), params.clone(), sp);
        let new = run(shipping(), params, sp);
        for zone in Zone::ALL {
            let (a, b) = (old.relay_switches(zone), new.relay_switches(zone));
            assert!(
                b <= a + a / 4 + 100,
                "setpoints {sp:?}: {} switched {b} times against the PID's {a}",
                zone.name()
            );
        }
    }
}

/// Middle runs without an integral term, which is what guarantees it never
/// climbs past setpoint — so its feedforward has to be accurate enough to
/// arrive on its own. If `PLANT` is ever re-identified and this drifts,
/// middle will quietly start sitting low.
#[test]
fn middle_reaches_setpoint_without_an_integral() {
    assert!(
        observer_pi_params()[Zone::Middle.port()].ki == 0.0,
        "this test is about the no-integral configuration"
    );
    for sp in [PROFILE_B, PROFILE_C] {
        let trace = run(shipping(), ExtruderThermalParams::calibrated(), sp);
        let error = trace.final_c(Zone::Middle) - sp[Zone::Middle.port()];
        assert!(
            error.abs() < 1.0,
            "setpoints {sp:?}: middle settled {error:+.2} K out on feedforward alone",
        );
    }
}

/// The feedforward is referenced to a fixed ambient, so it has to agree with the
/// one the model was calibrated at. Production must not depend on the simulation
/// to find that number, but it does have to stay in step with it.
#[test]
fn shipping_ambient_matches_the_calibration() {
    let calibrated = ExtruderThermalParams::calibrated().ambient_c;
    assert!(
        (AMBIENT_C - calibrated).abs() < 1e-9,
        "heating_params::AMBIENT_C is {AMBIENT_C} but the model calibrates at \
         {calibrated}; the feedforward slopes in PLANT were measured at the latter"
    );
}

/// `observer_pi_params` is the only thing production reads out of all this, so
/// pin its shape against the strategy the simulation actually runs.
#[test]
fn the_simulated_strategy_is_the_shipping_one() {
    let shipped: [ObserverPiParams; 4] = observer_pi_params();
    let StrategyConfig::ObserverPi(simulated) = shipping() else {
        panic!("the shipping strategy must be ObserverPi");
    };
    assert_eq!(shipped, simulated);
}

// ---------------------------------------------------------------- quantisation limit cycle
//
// The gains that shipped before the `bench_heating` cost function accounted
// for tail oscillation. A settle-time/overshoot check cannot see this: front,
// back and nozzle each sustain a real, bounded temperature oscillation once
// the sensor's own measurement noise is modelled, with a period landing
// within a few seconds of that zone's own `tau_filter_s` — see
// `harness::REALISTIC_SENSOR_NOISE_C` and `README.md`.

/// `(kp, ki, tau_filter_s, tau_sensor_s)` per zone, `[front, middle, back,
/// nozzle]` — `heating_params::OBSERVER_PI_GAINS` before the oscillation fix.
/// A literal snapshot, not a reference to `observer_pi_params()` (which now
/// carries the retuned gains), so this stays meaningful regardless of future
/// retunes.
const PRE_FIX_OBSERVER_PI_GAINS: [(f64, f64, f64, f64); 4] = [
    (0.110, 0.00053, 16.0, 90.0),
    (0.074, 0.0, 18.2, 128.0),
    (0.109, 0.00035, 19.4, 110.0),
    (0.322, 0.00118, 20.3, 90.0),
];

fn observer_pi_params_from(gains: [(f64, f64, f64, f64); 4]) -> [ObserverPiParams; 4] {
    Zone::ALL.map(|zone| {
        let p = PLANT[zone.port()];
        let (kp, ki, tau_filter_s, tau_sensor_s) = gains[zone.port()];
        ObserverPiParams {
            kp,
            ki,
            tau_sensor_s,
            tau_filter_s,
            lead_max_k: 45.0,
            ff_duty_per_k: p.ff_duty_per_k,
            ambient_c: AMBIENT_C,
            max_clamp: DEFAULT_MAX_CLAMP[zone.port()],
        }
    })
}

/// Long, steady hold with realistic sensor noise — the direct instrument for
/// the tail oscillation a settle-time/overshoot check cannot see.
fn run_with_noise(strategy: StrategyConfig, sp: [f64; 4]) -> Trace {
    let config = SimConfig {
        strategy,
        sensor_noise_c: REALISTIC_SENSOR_NOISE_C,
        ..SimConfig::default()
    };
    ThermalSim::new(ExtruderThermalParams::calibrated(), config).run(&scenario(sp))
}

/// Threshold above the bare noise floor a zone's *steel* (not sensor) tail
/// std-dev has to clear to count as "really oscillating" rather than just
/// carrying the irreducible sensor noise through — see
/// `Trace::tail_steel_std_dev_k`. Set from what the pre-fix gains actually
/// measure on `PROFILE_B` (front 0.017, back 0.017, nozzle 0.041 K), with
/// margin below the smallest of the three.
const REAL_OSCILLATION_K: f64 = 0.012;

/// Proves the simulation now reproduces the real-machine symptom that
/// motivated this fix: held at a normal setpoint, with the gains that shipped
/// before the retune, front/back/nozzle sustain a real steel-temperature
/// oscillation — not just sensor noise passing through — while middle (heater
/// usually at 0 W, nothing for a spurious kick to swing) does not.
#[test]
fn pre_fix_gains_oscillate_under_realistic_sensor_noise() {
    let strategy = StrategyConfig::ObserverPi(observer_pi_params_from(PRE_FIX_OBSERVER_PI_GAINS));
    let trace = run_with_noise(strategy, PROFILE_B);
    for zone in [Zone::Front, Zone::Back, Zone::Nozzle] {
        let osc = trace.tail_steel_std_dev_k(zone, 0.2);
        assert!(
            osc > REAL_OSCILLATION_K,
            "{}: expected a real oscillation with the pre-fix gains, steel tail std-dev only \
             {osc:.4} K",
            zone.name()
        );
    }
}

/// The fix, as far as the 2026-09-15 retune actually got: a budget-limited
/// `bench_heating --search observer-pi` run measurably reduced Back and
/// Nozzle's oscillation (~41 % and ~27 %) without regressing Front or Middle
/// — it did not eliminate it. Pin that honestly rather than a threshold this
/// gain set cannot clear; a longer search or duty-side smoothing is the next
/// step if the reduction is not enough on the real machine.
#[test]
fn retuned_gains_reduce_the_oscillation() {
    let pre_fix = StrategyConfig::ObserverPi(observer_pi_params_from(PRE_FIX_OBSERVER_PI_GAINS));
    let before = run_with_noise(pre_fix, PROFILE_B);
    let after = run_with_noise(shipping(), PROFILE_B);

    for zone in [Zone::Back, Zone::Nozzle] {
        let (b, a) = (
            before.tail_steel_std_dev_k(zone, 0.2),
            after.tail_steel_std_dev_k(zone, 0.2),
        );
        assert!(
            a < 0.8 * b,
            "{}: expected at least a 20% reduction, {b:.4} K -> {a:.4} K",
            zone.name()
        );
    }
    // Front and Middle were not the target of this retune; guard against a
    // regression on them, not a specific improvement.
    for zone in [Zone::Front, Zone::Middle] {
        let (b, a) = (
            before.tail_steel_std_dev_k(zone, 0.2),
            after.tail_steel_std_dev_k(zone, 0.2),
        );
        assert!(
            a < b + 0.01,
            "{}: the retune should not make this zone meaningfully worse, {b:.4} K -> {a:.4} K",
            zone.name()
        );
    }
}
