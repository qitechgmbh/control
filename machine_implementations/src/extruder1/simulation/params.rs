//! Tunable coefficients of the extruder thermal model.
//!
//! Everything that geometry *fixes* (masses, areas, band positions) lives in
//! [`super::geometry`] and is not adjustable. Everything in this file is a
//! material property or an interface coefficient that cannot be read off a CAD
//! model — these are what [`super::fit`] calibrates against a recorded run.

/// Coefficients of the extruder thermal model.
///
/// [`Default`] is the calibrated set: values fitted to
/// `data/heatup_2026-02-24.csv`, the same as
/// [`ExtruderThermalParams::calibrated`]. Use
/// [`ExtruderThermalParams::first_principles`] for the uncalibrated handbook
/// values if you want to see what the model predicts before it has seen the
/// machine.
#[derive(Debug, Clone, PartialEq)]
pub struct ExtruderThermalParams {
    /// Ambient air temperature in °C.
    pub ambient_c: f64,

    // ---- steel ----
    /// Thermal conductivity of the barrel/Düse steel in W/(m·K).
    ///
    /// Nitriding steel (31CrMoV9 and friends) sits around 35–42 over the working
    /// range.
    pub k_steel: f64,
    /// Specific heat capacity of the steel in J/(kg·K).
    pub cp_steel: f64,

    // ---- band heaters ----
    /// Heat capacity of a band heater per unit of contact area, in J/(m²·K).
    ///
    /// Deliberately *not* derived from the CAD. `Heizelement Barrel` is modelled
    /// there as a bare Ø65→Ø71 shell, which is a placeholder for a real band with
    /// its sheath and clamp straps. Expressing the capacity per unit contact area
    /// makes the one number scale correctly between the 200 mm barrel bands and
    /// the 34 mm nozzle band.
    ///
    /// The calibrated 8000 J/(m²·K) is ~330 J/K for a 200 mm band, i.e. roughly
    /// 0.7 kg of steel-equivalent — the right order for a band of that size.
    pub band_heat_capacity_j_per_m2_k: f64,
    /// Contact coefficient between a clamped band and the barrel, in W/(m²·K).
    ///
    /// This and [`Self::band_heat_capacity_j_per_m2_k`] are the overshoot
    /// parameters. Their ratio `C / (h·A)` is the band's time constant, and
    /// `P · C / (h·A)` is the energy it has stored when the relay opens — the
    /// energy that then keeps pushing the barrel past setpoint. The calibrated
    /// ~70 W/(m²·K) is low, which is what a clamped band on machined steel with
    /// no heat-transfer compound actually achieves: it puts the band around
    /// 170 K above the barrel while driven.
    pub band_contact_h: f64,

    // ---- insulation sleeve ----
    /// Thermal conductivity of the sleeve in W/(m·K). Ceramic fibre is ~0.05–0.10
    /// at these temperatures.
    pub k_insulation: f64,
    /// Emissivity of the sleeve's outer skin.
    pub insulation_emissivity: f64,

    // ---- bare surfaces ----
    /// Free-convection coefficient of bare steel, in the form `h = c · ΔT^0.25`.
    /// For a horizontal Ø65 cylinder, `1.32 · (1/0.065)^0.25 ≈ 2.6`.
    pub bare_convection_coeff: f64,
    /// Emissivity of bare steel: ~0.25 machined, ~0.8 oxidised.
    pub bare_emissivity: f64,

    // ---- interfaces ----
    /// Contact coefficient across the bolted Düse/barrel flange in W/(m²·K).
    ///
    /// Governs how much the nozzle zone and the front zone pull on each other.
    pub flange_contact_h: f64,
    /// Lumped conductance from the rearmost barrel cell into the gearbox and
    /// bearing housing, in W/K.
    ///
    /// The gearbox end is a large unmodelled heat sink; the back zone's high
    /// steady-state power is mostly this.
    pub gearbox_sink_g: f64,

    // ---- screw ----
    /// Whether to model the screw. It is ~3.5 kg of steel inside the bore and
    /// noticeably slows the heat-up, so leave it on unless you are isolating an
    /// effect.
    pub include_screw: bool,
    /// Coefficient across the bore gap between barrel and screw in W/(m²·K).
    /// Conduction through a fraction of a millimetre of air plus radiation.
    pub bore_gap_h: f64,

    // ---- sensors ----
    /// First-order lag of the RTD in its pocket, in seconds.
    ///
    /// # This is larger than it looks like it should be
    ///
    /// A well-seated RTD in a steel pocket is 5–20 s. Calibration against the
    /// real machine wants ~60 s, and that is not the optimiser misbehaving — it
    /// is visible directly in the recording. While a zone ramps at ~0.21 K/s the
    /// sensor trails the steel by `τ · rate`, so the controller keeps driving
    /// after the steel has already passed setpoint; when the relay finally opens,
    /// the reading climbs to meet the steel. That accounts for roughly a third of
    /// the observed overshoot, with band storage supplying the rest.
    ///
    /// A lag this long means the probes are not tightly coupled to the bore — an
    /// air gap, no heat-transfer compound, or a loose fit. **Worth checking on
    /// the machine**: seating the probes properly would remove a large part of
    /// the overshoot with no change to the control code at all.
    pub sensor_tau_s: f64,
    /// Heat capacity attributed to the sensing tip in J/K. Only its ratio to
    /// [`Self::sensor_tau_s`] matters; it is kept small so the sensor does not
    /// load the barrel.
    pub sensor_heat_capacity: f64,

    // ---- discretisation ----
    /// Nominal axial cell size in mm. 20 mm gives ~54 cells over the full length
    /// and resolves the axial gradients that couple the zones.
    pub cell_size_mm: f64,
}

impl Default for ExtruderThermalParams {
    fn default() -> Self {
        Self::calibrated()
    }
}

impl ExtruderThermalParams {
    /// Handbook values, before the model has seen the machine.
    ///
    /// Kept so the effect of calibration stays visible and reproducible. These
    /// get the thermal masses and the steady-state balance about right — rise
    /// times land within a few percent — but produce **no overshoot at all**,
    /// because they assume a well-coupled band and a fast sensor.
    pub fn first_principles() -> Self {
        Self {
            ambient_c: 22.0,

            k_steel: 42.0,
            cp_steel: 490.0,

            band_heat_capacity_j_per_m2_k: 9_400.0,
            band_contact_h: 800.0,

            k_insulation: 0.07,
            insulation_emissivity: 0.8,

            bare_convection_coeff: 2.6,
            bare_emissivity: 0.35,

            flange_contact_h: 1500.0,
            gearbox_sink_g: 1.5,

            include_screw: true,
            bore_gap_h: 150.0,

            sensor_tau_s: 8.0,
            sensor_heat_capacity: 5.0,

            cell_size_mm: 20.0,
        }
    }

    /// Parameters calibrated against `data/heatup_2026-02-24.csv`.
    ///
    /// Regenerate with:
    ///
    /// ```text
    /// cargo run --release -p machine_implementations \
    ///     --example extruder_thermal_sim -- --fit --evals 4000
    /// ```
    ///
    /// How well this set reproduces that recording — and where it still misses —
    /// is asserted in the tests in [`super::harness`] and summarised in the
    /// [`super`] module docs.
    ///
    /// Closed loop against the reference run these give peaks within 1.5 K and
    /// rise times within 6 % on all four zones, at an open-loop replay RMS of
    /// 5.7 K over the hour.
    ///
    /// `sensor_tau_s` is the optimiser's value but was also confirmed by hand as
    /// an interior optimum: sweeping it, the closed-loop peaks are closest at
    /// 150 s and get worse in both directions.
    pub fn calibrated() -> Self {
        Self {
            band_contact_h: 98.0,
            band_heat_capacity_j_per_m2_k: 1_000.0,
            k_insulation: 0.030,
            bare_convection_coeff: 1.88,
            bare_emissivity: 0.225,
            flange_contact_h: 9_660.0,
            gearbox_sink_g: 40.0,
            bore_gap_h: 10.2,
            sensor_tau_s: 150.0,
            ..Self::first_principles()
        }
    }

    /// Coefficients that the reference calibration is expected to leave sitting
    /// on a bound, with the reason.
    ///
    /// These three are *structurally* unidentifiable from the reference run, not
    /// signs of a broken model:
    ///
    /// - `gearbox_sink_g` — the ~231 mm of bare barrel between the back band and
    ///   the gearbox has an axial conductance of only ~0.5 W/K, so it, not the
    ///   sink, limits the heat flow. Any sink above ~10 W/K pins the end at
    ///   ambient and gives an identical answer.
    /// - `band_heat_capacity_j_per_m2_k` — trades directly against
    ///   `sensor_tau_s`; both delay the reading relative to the steel.
    /// - `k_insulation` — trades against `bare_convection_coeff` and
    ///   `bare_emissivity`; only the total loss is observable.
    ///
    /// Separating them needs the `single-*` scenarios run on the real machine.
    pub const EXPECTED_PINNED: [&'static str; 3] = [
        "band_heat_capacity_j_per_m2_k",
        "k_insulation",
        "gearbox_sink_g",
    ];

    /// The free coefficients, in the order [`Self::apply_vector`] expects.
    ///
    /// Only the parameters that calibration is allowed to move are included —
    /// masses and geometry stay fixed at their CAD values, and `cp_steel` /
    /// `k_steel` are handbook values that should not absorb modelling error.
    pub fn to_vector(&self) -> Vec<f64> {
        vec![
            self.band_contact_h,
            self.band_heat_capacity_j_per_m2_k,
            self.k_insulation,
            self.bare_convection_coeff,
            self.bare_emissivity,
            self.flange_contact_h,
            self.gearbox_sink_g,
            self.bore_gap_h,
            self.sensor_tau_s,
        ]
    }

    /// Physical bounds for each entry of [`Self::to_vector`], as
    /// `(name, min, max)`.
    ///
    /// These are what the hardware and the literature allow, not what makes the
    /// optimiser's life easy. [`Self::pinned_parameters`] reports anything
    /// sitting on a bound rather than letting it pass silently.
    ///
    /// # Reading a pinned fit
    ///
    /// A pinned coefficient means one of two things, and they need different
    /// responses:
    ///
    /// - The **model** is missing a mechanism, and the optimiser is pushing a
    ///   coefficient somewhere unphysical to compensate. Widening the bound hides
    ///   the problem; fix the model.
    /// - The coefficient is **not identifiable** from the recording you fitted
    ///   against. This is the case today: one run with all four zones heating
    ///   together cannot separate nine coefficients, because several of them
    ///   trade off almost exactly — every distributed loss term against
    ///   `gearbox_sink_g`, and `band_heat_capacity_j_per_m2_k` against
    ///   `sensor_tau_s`. The optimiser lands on an arbitrary point along a flat
    ///   valley. The model still *predicts* well; the individual numbers are just
    ///   not separately trustworthy.
    ///
    /// The cure for the second case is more experiments, not more optimiser
    /// budget: heat one zone at a time from cold and let it decay, which
    /// separates that zone's losses from its neighbours' coupling. See the
    /// `single-*` scenarios in [`super::scenario`].
    pub const BOUNDS: [(&'static str, f64, f64); 9] = [
        // A clamped band on machined steel with no heat-transfer compound is
        // genuinely poor; the upper end is a well-seated one.
        ("band_contact_h", 40.0, 1200.0),
        // ~0.1 kg to ~2.5 kg of steel-equivalent per 200 mm band.
        ("band_heat_capacity_j_per_m2_k", 1_000.0, 30_000.0),
        // Low-density ceramic fibre blanket over its working range.
        ("k_insulation", 0.03, 0.15),
        // Free convection off a horizontal Ø65-Ø111 cylinder gives ~2.6 in open
        // still air. The barrel is enclosed in the Oberbau housing, where the air
        // is stagnant and pre-warmed, so the effective value runs lower.
        ("bare_convection_coeff", 1.0, 6.0),
        // Machined steel is ~0.15; oxidised is ~0.8.
        ("bare_emissivity", 0.10, 0.9),
        // Bolted steel-to-steel flange, from a poor contact to near-solid.
        ("flange_contact_h", 200.0, 30_000.0),
        // The gearbox and bearing housing are a large unmodelled sink.
        ("gearbox_sink_g", 0.0, 40.0),
        // From a loose clearance fit to a close one, plus radiation.
        ("bore_gap_h", 5.0, 600.0),
        // From a well-seated probe to one sitting in an air gap. See
        // [`Self::sensor_tau_s`] for why the upper end is this high.
        ("sensor_tau_s", 2.0, 200.0),
    ];

    /// Inverse of [`Self::to_vector`], clamped to [`Self::BOUNDS`].
    pub fn apply_vector(&mut self, v: &[f64]) {
        debug_assert_eq!(v.len(), Self::BOUNDS.len());
        let c = |i: usize| v[i].clamp(Self::BOUNDS[i].1, Self::BOUNDS[i].2);
        self.band_contact_h = c(0);
        self.band_heat_capacity_j_per_m2_k = c(1);
        self.k_insulation = c(2);
        self.bare_convection_coeff = c(3);
        self.bare_emissivity = c(4);
        self.flange_contact_h = c(5);
        self.gearbox_sink_g = c(6);
        self.bore_gap_h = c(7);
        self.sensor_tau_s = c(8);
    }

    /// Which fitted coefficients are sitting on a bound, to within a factor of
    /// `1 + tol`.
    ///
    /// The test is a *ratio*, not a distance, because these coefficients span
    /// orders of magnitude and [`super::fit`] optimises them in log space — being
    /// 5 units from a bound means something very different for `k_insulation`
    /// (range 0.03–0.15) than for `flange_contact_h` (200–30000).
    ///
    /// Used to flag a degenerate fit rather than reporting it as a success.
    pub fn pinned_parameters(&self, tol: f64) -> Vec<&'static str> {
        self.to_vector()
            .iter()
            .zip(Self::BOUNDS)
            .filter(|(v, (_, lo, hi))| {
                let (v, lo, hi) = (**v, *lo, *hi);
                let at_low = if lo <= 0.0 {
                    // A zero lower bound has no meaningful ratio; only "exactly
                    // off" counts as pinned there.
                    v <= 0.0
                } else {
                    v / lo <= 1.0 + tol
                };
                at_low || (v > 0.0 && hi / v <= 1.0 + tol)
            })
            .map(|(_, (name, _, _))| name)
            .collect()
    }

    /// Set one coefficient by the name used in [`Self::BOUNDS`].
    ///
    /// Returns `false` for an unknown name.
    pub fn set_by_name(&mut self, name: &str, value: f64) -> bool {
        match name {
            "band_contact_h" => self.band_contact_h = value,
            "band_heat_capacity_j_per_m2_k" => self.band_heat_capacity_j_per_m2_k = value,
            "k_insulation" => self.k_insulation = value,
            "insulation_emissivity" => self.insulation_emissivity = value,
            "bare_convection_coeff" => self.bare_convection_coeff = value,
            "bare_emissivity" => self.bare_emissivity = value,
            "flange_contact_h" => self.flange_contact_h = value,
            "gearbox_sink_g" => self.gearbox_sink_g = value,
            "bore_gap_h" => self.bore_gap_h = value,
            "sensor_tau_s" => self.sensor_tau_s = value,
            "k_steel" => self.k_steel = value,
            "cp_steel" => self.cp_steel = value,
            "ambient_c" => self.ambient_c = value,
            "cell_size_mm" => self.cell_size_mm = value,
            _ => return false,
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The reference calibration is allowed to pin exactly the three
    /// coefficients documented as unidentifiable. If a refit pins anything
    /// *else*, that is a new modelling problem and should be looked at rather
    /// than absorbed by widening a bound.
    #[test]
    fn only_the_known_unidentifiable_coefficients_are_pinned() {
        let pinned = ExtruderThermalParams::calibrated().pinned_parameters(0.01);
        let unexpected: Vec<_> = pinned
            .iter()
            .filter(|n| !ExtruderThermalParams::EXPECTED_PINNED.contains(n))
            .collect();
        assert!(
            unexpected.is_empty(),
            "unexpected pinned coefficients {unexpected:?}; see \
             ExtruderThermalParams::EXPECTED_PINNED"
        );
    }

    #[test]
    fn vector_round_trips() {
        let a = ExtruderThermalParams::calibrated();
        let mut b = ExtruderThermalParams::first_principles();
        b.apply_vector(&a.to_vector());
        assert_eq!(a.to_vector(), b.to_vector());
    }

    #[test]
    fn set_by_name_rejects_unknown_keys() {
        let mut p = ExtruderThermalParams::calibrated();
        assert!(p.set_by_name("band_contact_h", 123.0));
        assert!((p.band_contact_h - 123.0).abs() < f64::EPSILON);
        assert!(!p.set_by_name("no_such_parameter", 1.0));
    }
}
