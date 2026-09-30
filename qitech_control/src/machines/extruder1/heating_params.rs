use qitech_control_core::controllers::heating::HeatingStrategy;
use qitech_control_core::controllers::heating::ObserverPi;
use qitech_control_core::controllers::heating::ObserverPiParams;
use qitech_control_core::controllers::heating::PidBaseline;

use crate::machines::extruder1::HeatingAlgorithm;
use crate::machines::extruder1::Zone;

/// Ambient the feedforward is referenced to, in °C.
pub const AMBIENT_C: f64 = 22.0;

/// `(kp, ki, kd)` of the plain PID, the same for every zone.
pub const PID_GAINS: (f64, f64, f64) = (0.16, 0.0, 0.008);

/// Production duty clamp per zone: 1.0 for the barrel zones, 0.95 for the
/// nozzle.
pub const DEFAULT_MAX_CLAMP: [f64; 4] = [1.0, 1.0, 1.0, 0.95];

/// Effective thermal coefficients of one zone, as the controller sees it.
///
/// The watt-valued fields below were fitted against a V2 heat-up with `P` taken
/// as 700 W on the barrel and 200 W on the nozzle, which we later established
/// are V1's bands — a V2 runs 900 W and 150 W. Only [`Self::ff_duty_per_k`]
/// reaches the observer, and it is a duty, fitted against the real duty trace,
/// so the gains are unaffected. The two below are a record of the fit and are
/// off by the ratio of the assumed rating to the real one: the barrel
/// capacities read ~1.29x low, the nozzle ~1.33x high. Re-derive them before
/// quoting either as a physical property of the machine.
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
const OBSERVER_PI_GAINS: [(f64, f64, f64, f64); 4] = [
    // ----- front -----
    (
        0.11108586013317108,
        0.0005372999000549317,
        17.06717050075531,
        90.38324475288391,
    ),
    // ----- middle -----
    (
        0.07350360679626465,
        0.0,
        18.300373625755313,
        128.3832447528839,
    ),
    // ----- back -----
    (
        0.11008586013317108,
        0.0003572999000549316,
        19.36341073513031,
        109.72138714790346,
    ),
    // ----- nozzle -----
    (
        0.3230858601331711,
        0.0016372999000549317,
        20.40037362575531,
        90.38324475288391,
    ),
];

/// `ObserverPi` parameters per zone, indexed by [`Zone::port`].
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
        }
    })
}

/// A fresh control law for `zone`, carrying that algorithm's default gains.
pub fn build_strategy(algorithm: HeatingAlgorithm, zone: Zone) -> Box<dyn HeatingStrategy> {
    match algorithm {
        HeatingAlgorithm::ObserverPi => {
            Box::new(ObserverPi::new(observer_pi_params()[zone.port()]))
        }
        HeatingAlgorithm::Pid => {
            let (kp, ki, kd) = PID_GAINS;
            Box::new(PidBaseline::new(kp, ki, kd, DEFAULT_MAX_CLAMP[zone.port()]))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_algorithm_builds_with_its_own_default_gains() {
        for zone in Zone::ALL {
            let observer = build_strategy(HeatingAlgorithm::ObserverPi, zone);
            let p = observer_pi_params()[zone.port()];
            assert_eq!(observer.pid().get_kp(), p.kp);
            assert_eq!(observer.pid().get_ki(), p.ki);
            assert_eq!(observer.pid().get_kd(), 0.0);

            let pid = build_strategy(HeatingAlgorithm::Pid, zone);
            assert_eq!(
                (pid.pid().get_kp(), pid.pid().get_ki(), pid.pid().get_kd()),
                PID_GAINS
            );
        }
    }

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
            assert!(p.kp > 0.0, "{zone:?}: kp must be positive");
            assert!(p.ki >= 0.0, "{zone:?}: ki must not be negative");
            assert!(
                p.tau_filter_s > 0.0,
                "{zone:?}: tau_filter_s divides a slope"
            );
            assert!(
                p.tau_sensor_s > 3.0 * p.tau_filter_s,
                "{zone:?}: the filter must sit well below the probe's own lag; \
                 got tau_filter={:.1} against tau_sensor={:.1}",
                p.tau_filter_s,
                p.tau_sensor_s
            );
            assert!(
                p.ff_duty_per_k > 0.0,
                "{zone:?}: feedforward must be positive"
            );
            assert!(
                (0.0..=1.0).contains(&p.max_clamp),
                "{zone:?}: max_clamp is a duty"
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
                "{zone:?}: holding 180 C wants {duty:.3} duty, which is not plausible"
            );
        }
    }
}
