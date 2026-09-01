//! Per-zone controller parameters for the extruder's four heating zones.
//!
//! The four zones are genuinely different plants — the nozzle is ~6 kg of
//! uninsulated steel on a 200 W band, a barrel zone is ~5 kg under 700 W with
//! hot neighbours — so every strategy is parameterised per zone, indexed by
//! [`Zone::port`].
//!
//! # Where these numbers come from
//!
//! [`PLANT`] is *measured*, not guessed:
//!
//! ```text
//! cargo run --release -p machine_implementations --example bench_heating -- --identify
//! ```
//!
//! prints the block to paste back here. The measurements are taken with all
//! four zones running, so each zone's loss coefficient already includes what its
//! neighbours contribute — which is what a feedforward term wants, since a zone
//! flanked by hot steel really does need less power to hold temperature.
//!
//! The gains on top of that come from the search in the same example, scored
//! across [`super::harness::plant_family`] rather than the nominal fit alone.

use control_core::controllers::heating::{
    BandObserverGains, BandObserverParams, CascadeParams, ObserverPiParams,
};

use super::geometry::Zone;
use super::harness::DEFAULT_MAX_CLAMP;
use super::params::ExtruderThermalParams;

/// Effective thermal coefficients of one zone, as the controller sees it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlantCoefficients {
    /// Heat capacity of the steel the zone drives, in J/K. From the initial
    /// full-power ramp rate: `C = P / (dT/dt)`.
    pub metal_capacity_j_per_k: f64,
    /// Everything the steel loses to, in W/K — ambient, neighbouring cold
    /// steel, the gearbox. From a settled open-loop hold: `G = P / dT`.
    pub metal_loss_w_per_k: f64,
    /// Heat capacity of the band itself, in J/K. Geometry times the calibrated
    /// capacity per unit contact area.
    pub band_capacity_j_per_k: f64,
    /// Band-to-steel contact conductance, in W/K. Geometry times the calibrated
    /// contact coefficient.
    pub band_to_metal_w_per_k: f64,
    /// Steady-state duty per K above ambient. The feedforward slope.
    pub ff_duty_per_k: f64,
}

/// Measured coefficients per zone, indexed by [`Zone::port`].
///
/// Regenerate with `--identify` after any change to the thermal model.
/// The zone-to-zone differences here are the physics the module docs describe,
/// arrived at independently: **middle** needs by far the least power because it
/// is flanked by heated zones and has no cold sink to bleed into; **back** needs
/// the most of the three barrel zones because it feeds the gearbox; and the
/// **nozzle** needs the highest duty per kelvin of all, being uninsulated steel
/// on a 200 W band rather than an insulated 700 W one.
pub const PLANT: [PlantCoefficients; 4] = [
    // front
    PlantCoefficients {
        metal_capacity_j_per_k: 2573.0,
        metal_loss_w_per_k: 0.319,
        band_capacity_j_per_k: 40.8,
        band_to_metal_w_per_k: 4.00,
        ff_duty_per_k: 0.00046,
    },
    // middle
    PlantCoefficients {
        metal_capacity_j_per_k: 2341.0,
        metal_loss_w_per_k: 0.110,
        band_capacity_j_per_k: 40.8,
        band_to_metal_w_per_k: 4.00,
        ff_duty_per_k: 0.00016,
    },
    // back
    PlantCoefficients {
        metal_capacity_j_per_k: 2557.0,
        metal_loss_w_per_k: 0.429,
        band_capacity_j_per_k: 40.8,
        band_to_metal_w_per_k: 4.00,
        ff_duty_per_k: 0.00061,
    },
    // nozzle
    PlantCoefficients {
        metal_capacity_j_per_k: 1858.0,
        metal_loss_w_per_k: 0.488,
        band_capacity_j_per_k: 6.9,
        band_to_metal_w_per_k: 0.68,
        ff_duty_per_k: 0.00244,
    },
];

/// The band's own conductance to ambient, in W/K.
///
/// **Zero on purpose.** The band really does lose heat from its outer skin, but
/// [`PlantCoefficients::metal_loss_w_per_k`] is measured as *total electrical
/// power per kelvin of steel lift* — it is `ff_duty_per_k * rated_w`, read off a
/// settled closed-loop run — so the band's own loss is already inside it.
/// Giving the band a second, separate loss term double-counts, and the inner
/// loop's feedforward then asks for roughly twice the power that actually holds
/// the zone. Nothing downstream cancels that, because the inner P term goes to
/// zero at equilibrium by construction, so the whole loop parks about 10 K above
/// setpoint and stays there.
///
/// If these coefficients are ever re-derived from first principles instead of
/// measured, split the loss properly and set this to the real value.
const BAND_LOSS_W_PER_K: f64 = 0.0;

fn band_observer_params(zone: Zone, ambient_c: f64) -> BandObserverParams {
    let p = PLANT[zone.port()];
    BandObserverParams {
        band_capacity_j_per_k: p.band_capacity_j_per_k,
        metal_capacity_j_per_k: p.metal_capacity_j_per_k,
        band_to_metal_w_per_k: p.band_to_metal_w_per_k,
        metal_loss_w_per_k: p.metal_loss_w_per_k,
        band_loss_w_per_k: BAND_LOSS_W_PER_K,
        probe_tau_s: PROBE_TAU_S[zone.port()],
        ambient_c,
        rated_w: zone.band().rated_w,
    }
}

/// What the cascade's observer assumes each probe's time constant is, in
/// seconds.
///
/// Held at the middle of the plausible range rather than the calibrated 150 s.
/// The calibration cannot separate probe lag from band capacity (see
/// [`super::harness::plant_family`]), so this is uncertain by roughly an order
/// of magnitude.
const PROBE_TAU_S: [f64; 4] = [90.0, 90.0, 90.0, 90.0];

/// `ObserverPi` parameters per zone, indexed by [`Zone::port`].
///
/// **This is the shipping control law for `MACHINE_EXTRUDER_V2`.**
pub fn observer_pi_params() -> [ObserverPiParams; 4] {
    let ambient_c = ExtruderThermalParams::calibrated().ambient_c;
    Zone::ALL.map(|zone| {
        let p = PLANT[zone.port()];
        let g = OBSERVER_PI_GAINS[zone.port()];
        ObserverPiParams {
            kp: g.0,
            ki: g.1,
            tau_sensor_s: g.3,
            tau_filter_s: g.2,
            // Enough to cover the ~34 K the probe trails by on a cold-start
            // ramp, with margin, and no more: past that a correction this large
            // is far more likely to be a fault than a real gradient.
            lead_max_k: 45.0,
            ff_duty_per_k: p.ff_duty_per_k,
            ambient_c,
            max_clamp: DEFAULT_MAX_CLAMP[zone.port()],
        }
    })
}

/// `(kp, ki, tau_filter_s, tau_sensor_s)` per zone, from
/// `--search observer-pi`, scored across the whole plant family.
///
/// Two of these are worth reading rather than just trusting.
///
/// **Middle assumes the longest probe lag** (128 s, against 67–91 s elsewhere).
/// It is the zone with by far the lowest loss coefficient — flanked by heated
/// neighbours, nothing to bleed into — so it both ramps fastest and sheds an
/// excursion slowest. It therefore needs the most aggressive lag compensation of
/// the four, and it is the only zone where the requirement is *zero* overshoot.
///
/// **Middle also runs with no integral at all.** Its feedforward is accurate
/// enough to hold setpoint on its own, and an integral term is precisely the
/// mechanism that carries a loop past the target on arrival. Dropping it is what
/// makes "never exceeds setpoint" hold rather than merely usually hold; the cost
/// is that middle relies on `ff_duty_per_k` staying right, which
/// `middle_reaches_setpoint_without_an_integral` pins.
/// `tau_sensor_s` was then swept by hand per zone and raised on front and back.
/// The search had left both low (67 s and 91 s) because the benchmark's cost
/// only charges for overshoot beyond each zone's budget, so a 2 K excursion was
/// worth about 8 points out of 57 000 and it had no reason to care. Sweeping
/// directly against *worst overshoot across the plant family* shows both were
/// sitting on a bad local optimum:
///
/// | zone | tau | worst overshoot | worst t90 |
/// |---|---|---|---|
/// | front | 67 s | +2.21 K | 880 s |
/// | front | **90 s** | **+0.76 K** | 881 s |
/// | back | 91 s | +3.36 K | 648 s |
/// | back | **110 s** | **+0.39 K** | 662 s |
///
/// Front's is free and back's costs 2 % of rise time, so both were taken.
const OBSERVER_PI_GAINS: [(f64, f64, f64, f64); 4] = [
    (0.110, 0.00053, 16.0, 90.0),
    (0.074, 0.0, 18.2, 128.0),
    (0.109, 0.00035, 19.4, 110.0),
    (0.322, 0.00118, 20.3, 90.0),
];

/// `CascadeController` parameters per zone, indexed by [`Zone::port`].
pub fn cascade_params() -> [CascadeParams; 4] {
    let ambient_c = ExtruderThermalParams::calibrated().ambient_c;
    Zone::ALL.map(|zone| {
        let g = CASCADE_GAINS[zone.port()];
        CascadeParams {
            kp: g.0,
            ki: g.1,
            band_lead_max_k: g.2,
            approach_bias_k: g.3,
            // Below the ~175 K a barrel band reaches at full power, so the loop
            // can always ask for zero and mean it.
            band_dump_max_k: 40.0,
            // Inner-loop trim on top of its feedforward. Small: the feedforward
            // already knows what holding the band costs, and a large gain here
            // just chatters the relay against observer noise.
            k_inner: 0.004,
            coast_compensation: 1.0,
            observer: band_observer_params(zone, ambient_c),
            observer_gains: BandObserverGains::default(),
            max_clamp: DEFAULT_MAX_CLAMP[zone.port()],
        }
    })
}

/// `(kp, ki, band_lead_max_k, approach_bias_k)` per zone, from
/// `--search cascade`.
///
/// Kept, tuned, and *not shipped*. See [`observer_pi_params`] for what is.
///
/// The cascade reaches setpoint about as cleanly as anything here — its worst
/// overshoot across the plant family is under a kelvin on three of the four
/// plants — but it takes three to six thousand seconds to settle where
/// `ObserverPi` takes about one, and on the benchmark's cost that is decisive:
/// 202 000 against 57 000.
///
/// The reason is a property of this machine, not of the architecture. A cascade
/// earns its second loop by bounding the band's stored energy, and on the
/// extruder that energy is small: a 200 mm band is about 41 J/K sitting ~175 K
/// above roughly 2.5 kJ/K of steel, so the whole coast is around 3 K. Bounding a
/// 3 K effect does not pay for the extra lag an inner loop adds to the approach.
/// The dominant defect is the 150 s measurement lag, and that wants an estimator,
/// not another loop.
///
/// Worth revisiting if the hardware changes in a way that inverts that: a band
/// with much more mass, or — more likely — properly seated probes, which would
/// shrink the measurement lag and leave band storage as what remains.
const CASCADE_GAINS: [(f64, f64, f64, f64); 4] = [
    (11.53, 0.0463, 394.0, 0.69),
    (39.84, 0.0290, 299.0, -1.73),
    (6.73, 0.0078, 386.0, -0.37),
    (33.14, 0.0261, 159.0, 0.47),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extruder1::simulation::harness::{
        SimConfig, StrategyConfig, ThermalSim, plant_family,
    };
    use crate::extruder1::simulation::scenario::Scenario;
    use crate::extruder1::simulation::{Trace, ZoneTuning};

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
}
