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

    // ---- material throughput ----
    /// What the polymer passing through the machine does thermally. Disabled by
    /// default, in which case the model is exactly the idle machine the rest of
    /// these coefficients were calibrated against.
    pub melt: MeltParams,
}

/// The polymer moving through the barrel: what it is, how much of it, and what
/// the screw does to it.
///
/// # Status: not validated
///
/// Every number here is a handbook value or a hand calculation. Unlike the
/// coefficients on [`ExtruderThermalParams`], **none of it has been fitted to a
/// recording of the real machine**, because no recording of an extrusion run
/// exists yet. The model's *structure* is standard single-screw practice and the
/// magnitudes are right; the individual coefficients are not measurements. See
/// `README.md` for what recording would fix that.
///
/// Deliberately kept off [`ExtruderThermalParams::COEFFICIENTS`] so that the
/// existing nine-coefficient calibration and [`super::fit`] are untouched.
#[derive(Debug, Clone, PartialEq)]
pub struct MeltParams {
    /// Whether to model the polymer at all.
    ///
    /// With this off the bore holds the solid Ø25 screw the calibration assumed
    /// and there is no melt chain, so the network is identical to the one the
    /// heat-up recording was fitted against.
    ///
    /// With it on, the screw is cut back to its measured root profile to make
    /// room for the channel: about 1.8 kg of steel comes out and 0.32 kg of
    /// polymer goes in. Polymer holds roughly four times the heat per kilogram,
    /// so the bore's total heat capacity only drops by around a sixth — bounded
    /// by `splitting_the_bore_barely_changes_its_heat_capacity` in
    /// [`super::model`]. That is small, but it is not nothing: with the melt on
    /// this is no longer exactly the plant the nine thermal coefficients were
    /// fitted to. See `README.md`.
    pub enabled: bool,

    /// Throughput per screw rpm, in kg/h. Linear: 100 rpm — the machine maximum
    /// — gives 10 kg/h, 10 rpm gives 1 kg/h.
    pub kg_per_h_per_rpm: f64,

    // ---- material ----
    /// Melt density in kg/m³. ~1200 for PLA, ~1240 for PETG, ~750 for PP.
    pub density_kg_m3: f64,
    /// Specific heat of the solid polymer in J/(kg·K).
    pub cp_solid: f64,
    /// Specific heat of the melt in J/(kg·K).
    pub cp_melt: f64,
    /// Latent heat of fusion in J/kg. PLA is only partly crystalline, so this is
    /// far below a polyolefin's ~200 kJ/kg.
    pub latent_heat_j_per_kg: f64,
    /// Melting point in °C.
    pub melt_temperature_c: f64,
    /// Half-width of the melting range in K, over which the latent heat is
    /// smeared into an apparent heat capacity. See
    /// [`Self::specific_enthalpy_j_per_kg`].
    pub melt_window_k: f64,
    /// Temperature the material arrives at the feed throat at, in °C.
    pub feed_c: f64,

    // ---- interfaces ----
    /// Heat transfer coefficient between the melt and the barrel bore, in
    /// W/(m²·K).
    ///
    /// The most uncertain coefficient here after
    /// [`Self::specific_mechanical_energy_kwh_per_kg`]. A thin sheared melt film
    /// against a metal wall transfers far better than stagnant polymer, and the
    /// value depends on how full the channel is, which varies along the screw.
    pub film_h: f64,

    // ---- the screw's mechanical work ----
    /// Specific mechanical energy in kWh/kg: the shaft work the screw puts into
    /// each kilogram of polymer, essentially all of which becomes heat.
    ///
    /// Small single-screw extruders run 0.1–0.25 kWh/kg. **This term is the same
    /// order as the melting load it opposes** — at 10 kg/h, 0.10 kWh/kg is
    /// ~1000 W of shear heat against ~990 W carried out — so whether extruding
    /// heats or cools the barrel is decided by a number nobody has measured on
    /// this machine. Treat any conclusion that depends on the sign with care.
    pub specific_mechanical_energy_kwh_per_kg: f64,
    /// Ceiling on shear power in W: the drive cannot deliver more than its
    /// nameplate, whatever the specific energy says.
    ///
    /// **Placeholder.** The drive's rating is not in the codebase —
    /// [`crate::extruder1::screw_speed_controller::ScrewSpeedController`] carries
    /// only the 60 Hz frequency ceiling and the pole count. Read it off the motor
    /// and correct this.
    pub max_shear_power_w: f64,
}

impl Default for MeltParams {
    fn default() -> Self {
        Self::pla()
    }
}

impl MeltParams {
    /// PLA, disabled. Enable with [`Self::enabled`].
    pub const fn pla() -> Self {
        Self {
            enabled: false,
            kg_per_h_per_rpm: 0.1,

            density_kg_m3: 1200.0,
            cp_solid: 1800.0,
            cp_melt: 2100.0,
            latent_heat_j_per_kg: 45_000.0,
            melt_temperature_c: 160.0,
            melt_window_k: 10.0,
            feed_c: 22.0,

            film_h: 300.0,

            specific_mechanical_energy_kwh_per_kg: 0.10,
            max_shear_power_w: 1500.0,
        }
    }

    /// Throughput at a given screw speed, in kg/s.
    pub fn mass_flow_kg_per_s(&self, screw_rpm: f64) -> f64 {
        (screw_rpm.max(0.0) * self.kg_per_h_per_rpm) / 3600.0
    }

    /// Total shear power at a given screw speed, in W, capped at the drive's
    /// rating.
    pub fn shear_power_w(&self, screw_rpm: f64) -> f64 {
        let joules_per_kg = self.specific_mechanical_energy_kwh_per_kg * 3.6e6;
        (self.mass_flow_kg_per_s(screw_rpm) * joules_per_kg).min(self.max_shear_power_w)
    }

    /// Bounds of the melting range in °C.
    const fn melting_range(&self) -> (f64, f64) {
        (
            self.melt_temperature_c - self.melt_window_k,
            self.melt_temperature_c + self.melt_window_k,
        )
    }

    /// Specific heat inside the melting range, in J/(kg·K) — the sensible mean
    /// plus the latent heat smeared across the window.
    fn cp_window(&self) -> f64 {
        let sensible = (self.cp_solid + self.cp_melt) * 0.5;
        sensible + self.latent_heat_j_per_kg / (2.0 * self.melt_window_k)
    }

    /// Specific enthalpy at `t_c`, in J/kg, relative to 0 °C.
    ///
    /// Piecewise linear with a steep middle section: solid below the melting
    /// range, melt above it, and in between a slope that carries the latent heat.
    ///
    /// A lumped-capacitance network integrates `C dT/dt = Q`, which cannot
    /// represent a true phase change — the material would have to absorb energy
    /// at constant temperature, and a node with a finite capacity cannot. The
    /// standard workaround is this: smear the latent heat over a narrow range so
    /// that *crossing* it costs the right number of joules. It also raises the
    /// node's capacity there, which helps rather than hurts the step limit.
    pub fn specific_enthalpy_j_per_kg(&self, t_c: f64) -> f64 {
        let (lo, hi) = self.melting_range();
        if t_c <= lo {
            self.cp_solid * t_c
        } else if t_c >= hi {
            // Solid up to `lo`, then the whole melting range, then melt.
            let h_hi = self.cp_window().mul_add(hi - lo, self.cp_solid * lo);
            self.cp_melt.mul_add(t_c - hi, h_hi)
        } else {
            self.cp_window().mul_add(t_c - lo, self.cp_solid * lo)
        }
    }

    /// Tangent specific heat `dh/dT` at `t_c`, in J/(kg·K).
    ///
    /// This is what a node's heat capacity wants: it makes `dT/dt = Q/(m·dh/dT)`
    /// correct. It is *not* what a [`control_core::thermal::Flow`] wants — see
    /// [`Self::secant_cp_j_per_kg_k`].
    pub fn apparent_cp_j_per_kg_k(&self, t_c: f64) -> f64 {
        let (lo, hi) = self.melting_range();
        if t_c <= lo {
            self.cp_solid
        } else if t_c >= hi {
            self.cp_melt
        } else {
            self.cp_window()
        }
    }

    /// Secant specific heat between `datum_c` and `t_c`, in J/(kg·K).
    ///
    /// `(h(T) - h(datum)) / (T - datum)`. This is what an advection edge wants:
    /// with it, `w = m_dot * secant_cp` makes the edge carry exactly
    /// `m_dot * (h(T) - h(datum))`, so a chain transports true enthalpy
    /// differences and nothing leaks across the melting range. Using the tangent
    /// here instead would advect `cp*T` rather than `h`, which is only the same
    /// thing while `cp` is constant — i.e. everywhere except the one place in
    /// this machine that matters.
    pub fn secant_cp_j_per_kg_k(&self, t_c: f64, datum_c: f64) -> f64 {
        let dt = t_c - datum_c;
        if dt.abs() < 1e-9 {
            return self.apparent_cp_j_per_kg_k(datum_c);
        }
        (self.specific_enthalpy_j_per_kg(t_c) - self.specific_enthalpy_j_per_kg(datum_c)) / dt
    }

    /// Set one melt coefficient by its `melt.` CLI name. Returns `false` for an
    /// unknown name.
    pub fn set_by_name(&mut self, name: &str, value: f64) -> bool {
        match name {
            "enabled" => self.enabled = value != 0.0,
            "kg_per_h_per_rpm" => self.kg_per_h_per_rpm = value,
            "density_kg_m3" => self.density_kg_m3 = value,
            "cp_solid" => self.cp_solid = value,
            "cp_melt" => self.cp_melt = value,
            "latent_heat_j_per_kg" => self.latent_heat_j_per_kg = value,
            "melt_temperature_c" => self.melt_temperature_c = value,
            "melt_window_k" => self.melt_window_k = value,
            "feed_c" => self.feed_c = value,
            "film_h" => self.film_h = value,
            "specific_mechanical_energy_kwh_per_kg" => {
                self.specific_mechanical_energy_kwh_per_kg = value;
            }
            "max_shear_power_w" => self.max_shear_power_w = value,
            _ => return false,
        }
        true
    }
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

            melt: MeltParams::pla(),
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
    /// Covers the free coefficients in [`Self::COEFFICIENTS`], the fixed values
    /// a caller may still want to override for an experiment, and the melt
    /// coefficients under a `melt.` prefix (`melt.film_h`, `melt.enabled`, ...).
    pub fn set_by_name(&mut self, name: &str, value: f64) -> bool {
        if let Some(c) = Self::COEFFICIENTS.iter().find(|c| c.name == name) {
            *c.get(self) = value;
            return true;
        }
        if let Some(key) = name.strip_prefix("melt.") {
            return self.melt.set_by_name(key, value);
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

    #[test]
    fn melt_coefficients_are_reachable_under_their_prefix() {
        let mut p = ExtruderThermalParams::calibrated();
        assert!(p.set_by_name("melt.film_h", 450.0));
        assert!((p.melt.film_h - 450.0).abs() < f64::EPSILON);
        assert!(p.set_by_name("melt.enabled", 1.0));
        assert!(p.melt.enabled);
        assert!(!p.set_by_name("melt.no_such_parameter", 1.0));
    }

    /// The melt model must not join the calibration: [`super::fit`] optimises
    /// exactly the nine coefficients the heat-up recording can identify, and an
    /// extrusion term is not one of them.
    #[test]
    fn the_melt_model_is_not_part_of_the_fitted_vector() {
        let a = ExtruderThermalParams::calibrated();
        let mut b = ExtruderThermalParams::calibrated();
        b.melt.enabled = true;
        b.melt.film_h = 1_234.0;
        assert_eq!(a.to_vector(), b.to_vector());
    }

    #[test]
    fn the_machine_maximum_of_a_hundred_rpm_is_ten_kilos_an_hour() {
        let m = MeltParams::pla();
        assert!((m.mass_flow_kg_per_s(100.0) * 3600.0 - 10.0).abs() < 1e-9);
        assert!((m.mass_flow_kg_per_s(10.0) * 3600.0 - 1.0).abs() < 1e-9);
        assert!(m.mass_flow_kg_per_s(0.0).abs() < f64::EPSILON);
        // A stopped screw cannot pump backwards.
        assert!(m.mass_flow_kg_per_s(-5.0).abs() < f64::EPSILON);
    }

    /// Crossing the melting range must cost the latent heat, whatever window it
    /// is smeared over — that is the whole point of the apparent-capacity trick.
    #[test]
    fn the_melting_range_costs_the_latent_heat() {
        for window in [2.0, 10.0, 25.0] {
            let m = MeltParams {
                melt_window_k: window,
                ..MeltParams::pla()
            };
            let lo = m.melt_temperature_c - window;
            let hi = m.melt_temperature_c + window;
            let crossing = m.specific_enthalpy_j_per_kg(hi) - m.specific_enthalpy_j_per_kg(lo);
            let sensible = (m.cp_solid + m.cp_melt) * 0.5 * (hi - lo);
            assert!(
                (crossing - sensible - m.latent_heat_j_per_kg).abs() < 1e-6,
                "window {window} K: latent heat came out as {:.1} J/kg",
                crossing - sensible
            );
        }
    }

    /// Enthalpy must be continuous and strictly increasing, and its slope must
    /// be exactly the tangent `cp` everywhere — otherwise node capacities and
    /// transported enthalpy disagree about what the material is.
    #[test]
    fn enthalpy_is_smooth_and_its_slope_is_the_apparent_cp() {
        let m = MeltParams::pla();
        let mut previous = m.specific_enthalpy_j_per_kg(0.0);
        let mut t = 0.5;
        while t <= 300.0 {
            let h = m.specific_enthalpy_j_per_kg(t);
            assert!(h > previous, "enthalpy must increase with temperature");

            let eps = 1e-4;
            let slope = (m.specific_enthalpy_j_per_kg(t + eps)
                - m.specific_enthalpy_j_per_kg(t - eps))
                / (2.0 * eps);
            // Skip the two kinks, where a centred difference straddles a corner.
            let (lo, hi) = (
                m.melt_temperature_c - m.melt_window_k,
                m.melt_temperature_c + m.melt_window_k,
            );
            if (t - lo).abs() > 1e-3 && (t - hi).abs() > 1e-3 {
                assert!(
                    (slope - m.apparent_cp_j_per_kg_k(t)).abs() < 1.0,
                    "at {t} °C, dh/dT is {slope:.1} but apparent cp is {:.1}",
                    m.apparent_cp_j_per_kg_k(t)
                );
            }
            previous = h;
            t += 0.5;
        }
    }

    /// The secant is what makes an advection edge exact: `w * (T - datum)` with
    /// `w = m_dot * secant_cp` must equal `m_dot * (h(T) - h(datum))`, including
    /// across the melting range where the tangent would be wrong.
    #[test]
    fn the_secant_rate_carries_exact_enthalpy() {
        let m = MeltParams::pla();
        let datum = 22.0;
        for t in [22.0, 100.0, 155.0, 160.0, 165.0, 200.0, 260.0] {
            let carried = m.secant_cp_j_per_kg_k(t, datum) * (t - datum);
            let exact = m.specific_enthalpy_j_per_kg(t) - m.specific_enthalpy_j_per_kg(datum);
            assert!(
                (carried - exact).abs() < 1e-6,
                "at {t} °C the edge would carry {carried:.1} J/kg, not {exact:.1}"
            );
        }

        // Across the melting range itself the tangent is wrong by the whole
        // latent heat, which is what the distinction exists to avoid. Material
        // entering at the start of melting and leaving at the end really carries
        // 84 kJ/kg; a tangent rate taken at the outlet would say 42 kJ/kg and
        // silently lose every joule of the phase change.
        let (lo, hi) = (
            m.melt_temperature_c - m.melt_window_k,
            m.melt_temperature_c + m.melt_window_k,
        );
        let exact = m.specific_enthalpy_j_per_kg(hi) - m.specific_enthalpy_j_per_kg(lo);
        let with_tangent = m.apparent_cp_j_per_kg_k(hi) * (hi - lo);
        assert!(
            (exact - with_tangent - m.latent_heat_j_per_kg).abs() < 0.5 * m.latent_heat_j_per_kg,
            "the tangent should be short by about the latent heat, not {:.0} J/kg",
            exact - with_tangent
        );
    }

    #[test]
    fn shear_power_is_capped_at_the_drive_rating() {
        let m = MeltParams {
            specific_mechanical_energy_kwh_per_kg: 0.25,
            ..MeltParams::pla()
        };
        // 0.25 kWh/kg at 10 kg/h would be 2500 W; the drive cannot.
        assert!((m.shear_power_w(100.0) - m.max_shear_power_w).abs() < 1e-9);
        assert!(m.shear_power_w(0.0).abs() < f64::EPSILON);
    }
}
