//! Static decoupling of the extruder's four heating zones.
//!
//! The zones are physically coupled through the barrel, so four independent PIDs fight each
//! other: raising one zone's setpoint disturbs its neighbours, and their PIDs then disturb it
//! back. A static decoupler breaks that up by mixing the four PID demands before they reach the
//! heaters:
//!
//! ```text
//! u_actual = D * u_pid
//! ```
//!
//! Each zone keeps its own independent PID (unchanged setpoint, error and gains). Only the
//! mapping from PID demand to heater duty changes, so the four PIDs collectively see a
//! near-diagonal plant.
//!
//! `D` is obtained from a closed-loop step test: with all PIDs running normally, step one zone's
//! setpoint, wait for the whole system to fully re-settle, and record the steady-state change in
//! *every* zone's heater output. Repeated once per zone and normalised so the diagonal is 1, that
//! matrix is the inverse of the plant's DC gain matrix — which is exactly the decoupler.

/// Number of heating zones on an extruder.
pub const ZONE_COUNT: usize = 4;

/// Canonical zone order for [`HeatingDecoupler::apply`] and the matrix constants below.
///
/// This is the order the step tests were recorded in, kept so the matrix literals stay
/// diff-able against the raw measurement. It deliberately does *not* match the relay/thermocouple
/// port numbering (front=0, middle=1, back=2, nozzle=3) — mapping from these indices to the
/// individual `TemperatureController`s happens once, at the call site in `act.rs`.
pub const IDX_NOZZLE: usize = 0;
/// See [`IDX_NOZZLE`].
pub const IDX_FRONT: usize = 1;
/// See [`IDX_NOZZLE`].
pub const IDX_MIDDLE: usize = 2;
/// See [`IDX_NOZZLE`].
pub const IDX_BACK: usize = 3;

/// Identity matrix — passes every PID demand straight through to its own heater.
pub const IDENTITY: [[f64; ZONE_COUNT]; ZONE_COUNT] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// Measured decoupling matrix for the V3 extruder (machine `0x0016`, frontend slug `extruder3`).
///
/// Row `i` is "what zone `i`'s heater gets"; column `k` is "from zone `k`'s PID". The diagonal is
/// 1.0 by construction; the off-diagonals are the measured cross-gains, negated so that a
/// neighbour demanding more power makes this zone back off.
///
/// Off-diagonal magnitudes here are large (up to 0.78), so the row sums are negative. When all
/// four PIDs demand similar power the decoupled outputs go negative and clip to zero duty. If
/// zones are observed parked at 0 % well below setpoint, the step tests need re-running with a
/// larger step and confirmed full re-settling — the numbers, not the code, would be at fault.
pub const EXTRUDER_V3_HEATER_DECOUPLING: [[f64; ZONE_COUNT]; ZONE_COUNT] = [
    //  from nozzle, front,   middle,  back
    [1.0000, -0.7162, -0.4242, -0.5970], // -> nozzle heater
    [-0.4211, 1.0000, -0.1515, -0.6716], // -> front heater
    [-0.6667, -0.6622, 1.0000, -0.4925], // -> middle heater
    [-0.6140, -0.7838, -0.1212, 1.0000], // -> back heater
];

/// Mixes the four zone PID demands into four heater demands.
#[derive(Debug, Clone, PartialEq)]
pub struct HeatingDecoupler {
    matrix: [[f64; ZONE_COUNT]; ZONE_COUNT],
    enabled: bool,
    available: bool,
}

impl HeatingDecoupler {
    /// A decoupler with a measured matrix, enabled by default.
    pub const fn new(matrix: [[f64; ZONE_COUNT]; ZONE_COUNT]) -> Self {
        Self {
            matrix,
            enabled: true,
            available: true,
        }
    }

    /// No decoupling: identity matrix, and flagged unavailable so the frontend hides the toggle.
    /// Used for machine variants whose cross-coupling has not been measured yet.
    pub const fn none() -> Self {
        Self {
            matrix: IDENTITY,
            enabled: false,
            available: false,
        }
    }

    pub const fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Whether this machine has a measured matrix at all.
    pub const fn is_available(&self) -> bool {
        self.available
    }

    /// Applies `u_actual = D * u_pid`, in the zone order documented by [`IDX_NOZZLE`].
    ///
    /// `None` means the zone is disabled or over-temperature: it contributes no demand to the
    /// other rows, and stays `None` on output so it receives no heat of its own.
    ///
    /// The returned values are *unclipped* — each zone applies its own duty-cycle limit, which
    /// differs per zone (the nozzle is capped below 100 %).
    pub fn apply(&self, u_pid: [Option<f64>; ZONE_COUNT]) -> [Option<f64>; ZONE_COUNT] {
        if !self.enabled {
            return u_pid;
        }

        let mut u_actual = [None; ZONE_COUNT];
        for (i, out) in u_actual.iter_mut().enumerate() {
            if u_pid[i].is_none() {
                continue;
            }
            let mut sum = 0.0;
            for k in 0..ZONE_COUNT {
                sum += self.matrix[i][k] * u_pid[k].unwrap_or(0.0);
            }
            *out = Some(sum);
        }
        u_actual
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-9, "expected {b}, got {a}");
    }

    #[test]
    fn identity_matrix_passes_demands_through() {
        let decoupler = HeatingDecoupler::new(IDENTITY);
        let out = decoupler.apply([Some(0.1), Some(0.2), Some(0.3), Some(0.4)]);
        approx(out[IDX_NOZZLE].unwrap(), 0.1);
        approx(out[IDX_FRONT].unwrap(), 0.2);
        approx(out[IDX_MIDDLE].unwrap(), 0.3);
        approx(out[IDX_BACK].unwrap(), 0.4);
    }

    #[test]
    fn disabled_decoupler_passes_demands_through_untouched() {
        let mut decoupler = HeatingDecoupler::new(EXTRUDER_V3_HEATER_DECOUPLING);
        decoupler.set_enabled(false);
        let input = [Some(0.5), Some(0.25), None, Some(1.0)];
        assert_eq!(decoupler.apply(input), input);
    }

    #[test]
    fn none_variant_is_unavailable_and_disabled() {
        let decoupler = HeatingDecoupler::none();
        assert!(!decoupler.is_available());
        assert!(!decoupler.is_enabled());
        let input = [Some(0.7), Some(0.7), Some(0.7), Some(0.7)];
        assert_eq!(decoupler.apply(input), input);
    }

    #[test]
    fn single_active_zone_bleeds_negative_demand_into_neighbours() {
        // Only the nozzle PID is asking for heat: every other row picks up its column-0 term.
        let decoupler = HeatingDecoupler::new(EXTRUDER_V3_HEATER_DECOUPLING);
        let out = decoupler.apply([Some(1.0), Some(0.0), Some(0.0), Some(0.0)]);
        approx(out[IDX_NOZZLE].unwrap(), 1.0);
        approx(out[IDX_FRONT].unwrap(), -0.4211);
        approx(out[IDX_MIDDLE].unwrap(), -0.6667);
        approx(out[IDX_BACK].unwrap(), -0.6140);
    }

    #[test]
    fn mixed_demands_match_hand_computed_product() {
        let decoupler = HeatingDecoupler::new(EXTRUDER_V3_HEATER_DECOUPLING);
        let u = [0.40, 0.30, 0.20, 0.10];
        let out = decoupler.apply([Some(u[0]), Some(u[1]), Some(u[2]), Some(u[3])]);

        for (i, row) in EXTRUDER_V3_HEATER_DECOUPLING.iter().enumerate() {
            let expected: f64 = row
                .iter()
                .zip(u.iter())
                .map(|(coefficient, demand)| coefficient * demand)
                .sum();
            approx(out[i].unwrap(), expected);
        }
    }

    #[test]
    fn inactive_zone_neither_receives_nor_contributes() {
        let decoupler = HeatingDecoupler::new(EXTRUDER_V3_HEATER_DECOUPLING);
        // Middle is over-temperature / disabled; it must not pull its neighbours down.
        let out = decoupler.apply([Some(0.5), Some(0.5), None, Some(0.5)]);

        assert!(out[IDX_MIDDLE].is_none());

        let with_zero = decoupler.apply([Some(0.5), Some(0.5), Some(0.0), Some(0.5)]);
        approx(out[IDX_NOZZLE].unwrap(), with_zero[IDX_NOZZLE].unwrap());
        approx(out[IDX_FRONT].unwrap(), with_zero[IDX_FRONT].unwrap());
        approx(out[IDX_BACK].unwrap(), with_zero[IDX_BACK].unwrap());
    }
}
