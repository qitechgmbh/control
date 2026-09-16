//! Per-zone controller parameters for the extruder's four heating zones.
//!
//! The four zones are genuinely different plants — the nozzle is ~6 kg of
//! uninsulated steel on a 200 W band, a barrel zone is ~5 kg under 700 W with
//! hot neighbours — so every parameter here is per zone, indexed by
//! [`Zone::port`].
//!
//! [`PLANT`] is measured rather than guessed; regenerate it with
//! `cargo run --release -p machine_implementations --features simulation \
//! --example bench_heating -- --identify`, which prints the block to paste back.
//! The gains on top of it come from the search in the same example, scored
//! across a family of plants rather than the nominal fit alone. See
//! `src/extruder1/simulation/README.md`.

use control_core::controllers::heating::ObserverPiParams;

use super::zone::Zone;

/// Ambient the feedforward is referenced to, in °C.
///
/// Matches the thermal model's calibration; `simulation::shipping` asserts they
/// stay in step.
pub const AMBIENT_C: f64 = 22.0;

/// Production duty clamp per zone: 1.0 for the barrel zones, 0.95 for the
/// nozzle.
pub const DEFAULT_MAX_CLAMP: [f64; 4] = [1.0, 1.0, 1.0, 0.95];

/// Effective thermal coefficients of one zone, as the controller sees it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlantCoefficients {
    /// Heat capacity of the steel the zone drives, in J/K. From the initial
    /// full-power ramp rate: `C = P / (dT/dt)`.
    pub metal_capacity_j_per_k: f64,
    /// Everything the steel loses to, in W/K — ambient, neighbouring cold steel,
    /// the gearbox. From a settled open-loop hold: `G = P / dT`.
    pub metal_loss_w_per_k: f64,
    /// Steady-state duty per K above ambient. The feedforward slope.
    pub ff_duty_per_k: f64,
}

/// Measured coefficients per zone, indexed by [`Zone::port`].
///
/// The zone-to-zone differences are physics, arrived at independently of the
/// model: **middle** needs by far the least power, being flanked by heated zones
/// with no cold sink to bleed into; **back** needs the most of the three barrel
/// zones because it feeds the gearbox; and the **nozzle** needs the highest duty
/// per kelvin of all, being uninsulated steel on a 200 W band.
pub const PLANT: [PlantCoefficients; 4] = [
    // front
    PlantCoefficients {
        metal_capacity_j_per_k: 2573.0,
        metal_loss_w_per_k: 0.319,
        ff_duty_per_k: 0.00046,
    },
    // middle
    PlantCoefficients {
        metal_capacity_j_per_k: 2341.0,
        metal_loss_w_per_k: 0.110,
        ff_duty_per_k: 0.00016,
    },
    // back
    PlantCoefficients {
        metal_capacity_j_per_k: 2557.0,
        metal_loss_w_per_k: 0.429,
        ff_duty_per_k: 0.00061,
    },
    // nozzle
    PlantCoefficients {
        metal_capacity_j_per_k: 1858.0,
        metal_loss_w_per_k: 0.488,
        ff_duty_per_k: 0.00244,
    },
];

/// `(kp, ki, tau_filter_s, tau_sensor_s)` per zone, from `--search observer-pi`
/// scored across the whole plant family, then swept by hand on `tau_sensor_s`.
///
/// Two are worth reading rather than trusting. **Middle assumes the longest
/// probe lag** (128 s against 67–110 s elsewhere): it has the lowest loss
/// coefficient of the four, so it both ramps fastest and sheds an excursion
/// slowest, and it is the only zone whose overshoot budget is zero. **Middle
/// also runs with no integral at all** — its feedforward holds setpoint on its
/// own, and an integral is precisely the mechanism that carries a loop past the
/// target on arrival. `middle_reaches_setpoint_without_an_integral` pins that.
///
/// Retuned 2026-09-15 for a quantisation-driven limit cycle found on the real
/// machine: front/back/nozzle each sustain a real, bounded oscillation with a
/// period matching their own `tau_filter_s`, driven by the `SensorLagObserver`
/// lead term amplifying 0.1 °C sensor quantisation into spurious duty swings
/// (see `simulation/README.md`). `bench_heating`'s search previously had no
/// way to see this — a settle-time/overshoot check is blind to a sustained
/// oscillation that never leaves tolerance. The fix wired in
/// `SimConfig::sensor_noise_c` and `Trace::tail_steel_std_dev_k` so the search
/// could score it, at full timing resolution — the coarser `fast` step used
/// for the settle-time sweep badly distorts this specific mechanism and is
/// not trustworthy for it (see `oscillation_cost`'s doc comment). That pass
/// mainly raised nozzle's `ki` (0.00118 → 0.00164).
///
/// Retuned again 2026-09-16, with [`DUTY_SMOOTHING_S`] in the loop: smoothing
/// alone on the previous gains made back and nozzle oscillate *more* (the
/// filter's lag), so `--search observer-pi` was re-run around it (one restart,
/// 560 evaluations). Front, back and nozzle are its output unmodified.
/// **Middle keeps its 2026-09-15 gains**: middle's steel is bimodal under
/// sensor noise (either ~0.01 K or a ~0.08 K cycle, depending on the noise
/// draw), and the searched middle gains fell into the large cycle on far more
/// draws — the search scores a single draw, so it could not see that.
///
/// Steel oscillation against the pre-2026-09-15 gains, averaged over twenty
/// sensor-noise seeds (`retuned_gains_reduce_the_oscillation` in the
/// extruder-simulation repo):
///
/// | | front | middle | back | nozzle |
/// |---|---|---|---|---|
/// | 2026-09-15 gains, no smoothing | −3 % | −37 % | −4 % | −16 % |
/// | **these gains + smoothing** | **−49 %** | **−27 %** | **−29 %** | **−25 %** |
///
/// Middle's difference between the two is within its seed-to-seed noise. The
/// seed average matters: a single seed moves these by tens of percent, and the
/// "~41 % on back" first reported for the 2026-09-15 retune was one lucky draw.
/// **This reduces the oscillation, it does not eliminate it.**
///
/// Kept at the optimiser's full precision rather than rounded. Regenerate with
/// `--search observer-pi` rather than hand-editing these, and compare
/// candidates across many noise seeds, never one.
const OBSERVER_PI_GAINS: [(f64, f64, f64, f64); 4] = [
    (
        0.11230378004259622,
        0.0005476467634249886,
        16.69230314271417,
        90.36188376123808,
    ),
    // middle: 2026-09-15 gains, see above.
    (
        0.07350360679626465,
        0.0,
        18.300373625755313,
        128.3832447528839,
    ),
    (
        0.109752766614537,
        0.0003700771021772699,
        19.541063339537914,
        109.6211364365559,
    ),
    (
        0.3254276690291233,
        0.0021767834934591423,
        20.599532757015353,
        91.07072602061137,
    ),
];

/// Duty-output smoothing time constant per zone, in seconds, indexed by
/// [`Zone::port`].
///
/// The duty-side complement to [`OBSERVER_PI_GAINS`] — the two were tuned
/// together, so change one and re-check the other.
///
/// The observer's lead term turns each 0.1 °C quantisation step into a duty
/// kick of roughly `kp * 0.1 * tau_sensor_s / tau_filter_s`, visible as a
/// stuttering output. A few seconds of low-pass removes it, but it is not free:
/// the filter's lag sits inside a loop that is already close to its
/// quantisation limit cycle, so too long a time constant makes the *steel*
/// oscillate more even as the duty trace looks calmer.
///
/// - **Middle is 0 (off).** Its duty idles near zero, so there is little
///   stutter to remove, and any smoothing tipped it into its large ~0.08 K
///   oscillation mode on most noise draws (16 of 20 at 5 s).
/// - **Nozzle is 5 s**, not the 10 s first tried live in the extruder-simulation
///   hot-reload rig: with these gains, 10 s left nozzle's steel oscillation
///   worse than no smoothing at all (−12 % against pre-fix vs −25 % at 5 s).
///
/// The search behind [`OBSERVER_PI_GAINS`] ran with `[5, 5, 5, 10]`; these
/// values were then chosen against the twenty-seed oscillation check.
pub const DUTY_SMOOTHING_S: [f64; 4] = [5.0, 0.0, 5.0, 5.0];

/// `ObserverPi` parameters per zone, indexed by [`Zone::port`].
///
/// **This is the shipping control law for `MACHINE_EXTRUDER_V2`.**
pub fn observer_pi_params() -> [ObserverPiParams; 4] {
    Zone::ALL.map(|zone| {
        let p = PLANT[zone.port()];
        let (kp, ki, tau_filter_s, tau_sensor_s) = OBSERVER_PI_GAINS[zone.port()];
        ObserverPiParams {
            kp,
            ki,
            tau_sensor_s,
            tau_filter_s,
            // Enough to cover the ~34 K the probe trails by on a cold-start ramp,
            // with margin, and no more: past that a correction this large is far
            // more likely to be a fault than a real gradient.
            lead_max_k: 45.0,
            ff_duty_per_k: p.ff_duty_per_k,
            ambient_c: AMBIENT_C,
            max_clamp: DEFAULT_MAX_CLAMP[zone.port()],
            tau_duty_s: DUTY_SMOOTHING_S[zone.port()],
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn params_are_indexed_by_port() {
        let params = observer_pi_params();
        for zone in Zone::ALL {
            let p = params[zone.port()];
            assert_eq!(p.ff_duty_per_k, PLANT[zone.port()].ff_duty_per_k);
            assert_eq!(p.max_clamp, DEFAULT_MAX_CLAMP[zone.port()]);
            assert_eq!(p.ambient_c, AMBIENT_C);
        }
    }

    /// Every gain has to be physically sane; a sign slip or a zero `tau_filter_s`
    /// would divide by zero inside the observer.
    #[test]
    fn gains_are_within_sane_bounds() {
        for zone in Zone::ALL {
            let p = observer_pi_params()[zone.port()];
            let name = zone.name();
            assert!(p.kp > 0.0, "{name}: kp must be positive");
            assert!(p.ki >= 0.0, "{name}: ki must not be negative");
            assert!(p.tau_filter_s > 0.0, "{name}: tau_filter_s divides a slope");
            assert!(
                p.tau_sensor_s > 3.0 * p.tau_filter_s,
                "{name}: the filter must sit well below the probe's own lag; \
                 got tau_filter={:.1} against tau_sensor={:.1}",
                p.tau_filter_s,
                p.tau_sensor_s
            );
            assert!(
                p.ff_duty_per_k > 0.0,
                "{name}: feedforward must be positive"
            );
            assert!(
                (0.0..=1.0).contains(&p.max_clamp),
                "{name}: max_clamp is a duty"
            );
            assert!(
                p.tau_duty_s >= 0.0 && p.tau_duty_s * 5.0 < p.tau_sensor_s,
                "{name}: duty smoothing must sit well below the probe's own lag; \
                 got tau_duty={:.1} against tau_sensor={:.1}",
                p.tau_duty_s,
                p.tau_sensor_s
            );
        }
    }

    /// The feedforward has to hold the zone on its own at a typical setpoint,
    /// because middle runs with no integral to make up a shortfall.
    #[test]
    fn feedforward_duty_is_plausible_at_setpoint() {
        for zone in Zone::ALL {
            let p = observer_pi_params()[zone.port()];
            let duty = p.ff_duty_per_k * (180.0 - AMBIENT_C);
            assert!(
                (0.01..=0.6).contains(&duty),
                "{}: holding 180 C wants {duty:.3} duty, which is not plausible",
                zone.name()
            );
        }
    }
}
