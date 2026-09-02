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
    /// Per unit area rather than absolute, so one number scales correctly between
    /// the 200 mm barrel bands and the 34 mm nozzle band. Not from the CAD, which
    /// models the band as a bare shell without its sheath and clamp straps.
    pub band_heat_capacity_j_per_m2_k: f64,
    /// Contact coefficient between a clamped band and the barrel, in W/(m²·K).
    ///
    /// This and [`Self::band_heat_capacity_j_per_m2_k`] are the band-storage
    /// half of the overshoot: their ratio `C / (h·A)` is the band's time constant
    /// and `P · C / (h·A)` the energy still pushing the barrel when the relay
    /// opens. The calibrated value is low, as a clamped band on machined steel
    /// with no heat-transfer compound is — it sits ~170 K above the barrel while
    /// driven.
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
    /// **Much larger than a well-seated probe's 5–20 s**, and not an optimiser
    /// artefact — it is visible directly in the recording, and it is the dominant
    /// half of the overshoot. A lag this long means the probes are not tightly
    /// coupled to the bore: an air gap, a loose fit, or no heat-transfer
    /// compound. Worth fixing on the machine; see `README.md`.
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
    /// Kept so the effect of calibration stays visible. These get the masses and
    /// the steady-state balance about right, but produce **no overshoot at all**:
    /// they assume a well-coupled band and a fast sensor.
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
    /// Closed loop against that run these give peaks within 1.5 K and rise times
    /// within 6 % on all four zones, at an open-loop replay RMS of 5.7 K over the
    /// hour; the tests in [`super::harness`] assert it. `sensor_tau_s` was also
    /// confirmed by hand as an interior optimum. Regenerate with:
    ///
    /// ```text
    /// cargo run --release -p machine_implementations --features simulation \
    ///     --example extruder_thermal_sim -- --fit --evals 4000
    /// ```
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

    /// The free coefficients, in [`Self::COEFFICIENTS`] order.
    pub fn to_vector(&self) -> Vec<f64> {
        // `get` needs `&mut`, and this is cheap; the clone keeps the signature
        // taking `&self`, which every caller wants.
        let mut copy = self.clone();
        Self::COEFFICIENTS
            .iter()
            .map(|c| *c.get(&mut copy))
            .collect()
    }

    /// Inverse of [`Self::to_vector`], clamped to each coefficient's bounds.
    pub fn apply_vector(&mut self, v: &[f64]) {
        debug_assert_eq!(v.len(), Self::COEFFICIENTS.len());
        for (coeff, value) in Self::COEFFICIENTS.iter().zip(v) {
            *coeff.get(self) = value.clamp(coeff.min, coeff.max);
        }
    }

    /// The free coefficients: name, physical bounds, and how to reach the field.
    ///
    /// One table rather than four parallel lists, so [`Self::to_vector`],
    /// [`Self::apply_vector`], [`Self::set_by_name`] and
    /// [`Self::pinned_parameters`] cannot drift out of order — adding a
    /// coefficient is one line.
    ///
    /// Only what calibration is allowed to move is here: masses and geometry stay
    /// at their CAD values, and `cp_steel` / `k_steel` are handbook values that
    /// should not absorb modelling error.
    ///
    /// The bounds are what the hardware and the literature allow, not what makes
    /// the optimiser's life easy. A coefficient sitting on one means either the
    /// model is missing a mechanism — widening the bound hides that, fix the model
    /// — or the coefficient is not identifiable from the recording, which is the
    /// case for the three in [`Self::EXPECTED_PINNED`]. The cure for the second is
    /// more experiments, not more optimiser budget: the `single-*` scenarios in
    /// [`super::scenario`] heat one zone at a time and separate its losses from
    /// its neighbours' coupling.
    pub const COEFFICIENTS: [Coefficient; 9] = [
        // A clamped band on machined steel with no heat-transfer compound is
        // genuinely poor; the upper end is a well-seated one.
        Coefficient::new("band_contact_h", 40.0, 1200.0, |p| &mut p.band_contact_h),
        // ~0.1 kg to ~2.5 kg of steel-equivalent per 200 mm band.
        Coefficient::new("band_heat_capacity_j_per_m2_k", 1_000.0, 30_000.0, |p| {
            &mut p.band_heat_capacity_j_per_m2_k
        }),
        // Low-density ceramic fibre blanket over its working range.
        Coefficient::new("k_insulation", 0.03, 0.15, |p| &mut p.k_insulation),
        // Free convection off a horizontal Ø65-Ø111 cylinder gives ~2.6 in open
        // still air. The barrel is enclosed in the Oberbau housing, where the air
        // is stagnant and pre-warmed, so the effective value runs lower.
        Coefficient::new("bare_convection_coeff", 1.0, 6.0, |p| {
            &mut p.bare_convection_coeff
        }),
        // Machined steel is ~0.15; oxidised is ~0.8.
        Coefficient::new("bare_emissivity", 0.10, 0.9, |p| &mut p.bare_emissivity),
        // Bolted steel-to-steel flange, from a poor contact to near-solid.
        Coefficient::new("flange_contact_h", 200.0, 30_000.0, |p| {
            &mut p.flange_contact_h
        }),
        // The gearbox and bearing housing are a large unmodelled sink.
        Coefficient::new("gearbox_sink_g", 0.0, 40.0, |p| &mut p.gearbox_sink_g),
        // From a loose clearance fit to a close one, plus radiation.
        Coefficient::new("bore_gap_h", 5.0, 600.0, |p| &mut p.bore_gap_h),
        // From a well-seated probe to one sitting in an air gap. See
        // [`Self::sensor_tau_s`] for why the upper end is this high.
        Coefficient::new("sensor_tau_s", 2.0, 200.0, |p| &mut p.sensor_tau_s),
    ];

    /// Which fitted coefficients are sitting on a bound, to within a factor of
    /// `1 + tol`. Used to flag a degenerate fit rather than report it a success.
    ///
    /// A *ratio*, not a distance: these coefficients span orders of magnitude and
    /// [`super::fit`] optimises them in log space.
    pub fn pinned_parameters(&self, tol: f64) -> Vec<&'static str> {
        self.to_vector()
            .iter()
            .zip(Self::COEFFICIENTS)
            .filter(|(v, c)| {
                let v = **v;
                let at_low = if c.min <= 0.0 {
                    // A zero lower bound has no meaningful ratio; only "exactly
                    // off" counts as pinned there.
                    v <= 0.0
                } else {
                    v / c.min <= 1.0 + tol
                };
                at_low || (v > 0.0 && c.max / v <= 1.0 + tol)
            })
            .map(|(_, c)| c.name)
            .collect()
    }

    /// Set one coefficient by name, unclamped. Returns `false` for an unknown
    /// name.
    ///
    /// Covers the free coefficients in [`Self::COEFFICIENTS`] plus the fixed
    /// values a caller may still want to override for an experiment.
    pub fn set_by_name(&mut self, name: &str, value: f64) -> bool {
        if let Some(c) = Self::COEFFICIENTS.iter().find(|c| c.name == name) {
            *c.get(self) = value;
            return true;
        }
        match name {
            "insulation_emissivity" => self.insulation_emissivity = value,
            "k_steel" => self.k_steel = value,
            "cp_steel" => self.cp_steel = value,
            "ambient_c" => self.ambient_c = value,
            "cell_size_mm" => self.cell_size_mm = value,
            _ => return false,
        }
        true
    }
}

/// One calibratable coefficient: what it is called, what values are physical,
/// and how to reach it on [`ExtruderThermalParams`].
#[derive(Clone, Copy)]
pub struct Coefficient {
    pub name: &'static str,
    pub min: f64,
    pub max: f64,
    field: fn(&mut ExtruderThermalParams) -> &mut f64,
}

impl Coefficient {
    const fn new(
        name: &'static str,
        min: f64,
        max: f64,
        field: fn(&mut ExtruderThermalParams) -> &mut f64,
    ) -> Self {
        Self {
            name,
            min,
            max,
            field,
        }
    }

    pub fn get<'a>(&self, params: &'a mut ExtruderThermalParams) -> &'a mut f64 {
        (self.field)(params)
    }
}

impl std::fmt::Debug for Coefficient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} in {}..={}", self.name, self.min, self.max)
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
