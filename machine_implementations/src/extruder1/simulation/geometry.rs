//! Extruder barrel geometry, measured from `Oberbau_UBG.step` in the repository
//! root.
//!
//! # Coordinate system
//!
//! `x` is the extruder axis in **millimetres**, in the STEP file's own global
//! frame: the nozzle tip is at `x = -249`, the gearbox end of the barrel at
//! `x = +839`. Increasing `x` runs from the die towards the feed throat, i.e.
//! *against* the melt flow.
//!
//! ```text
//!  -249      -77  -43 -20  0    4        204 206      406 408      608        839
//!    |---- Düse ----|###|--|====|---------------- barrel ----------------------|
//!                nozzle    flange  [  FRONT  ]  [  MIDDLE  ]  [   BACK   ]  bare tail
//!                 band     joint   |<--------- insulation sleeve --------->|
//! ```
//!
//! # Provenance
//!
//! Every constant here was read out of the STEP file's B-rep, not estimated:
//! the band positions and widths are the axial extents of the `CYLINDRICAL_SURFACE`
//! faces at R = 32.5 / 35.5 mm on the three `Heizelement Barrel` instances and the
//! `Heizelement Düse`; the sleeve is `Isoliermanschette Heizband-komplettes Rohr`
//! at R = 35.5 → 55.5 mm; the barrel profile comes from `Schneckenzylinder`.
//! The steel density is the file's own `density measure` property, 7850 kg/m³.

use std::f64::consts::{FRAC_PI_4, PI};

/// Steel density in kg/m³, from the STEP file's material property.
pub const STEEL_DENSITY_KG_M3: f64 = 7850.0;

/// Outer diameter of the barrel and of the Düse's heated section, in mm.
pub const BARREL_OUTER_D_MM: f64 = 65.0;
/// Screw bore diameter in mm.
pub const BORE_D_MM: f64 = 25.0;
/// Outer diameter of the bolted flanges in mm.
pub const FLANGE_OUTER_D_MM: f64 = 110.0;

/// Axial position of the bolted Düse/barrel joint in mm.
///
/// The Düse's flange occupies −40…−20 and the barrel's −20…0, so the mating
/// plane is exactly `x = -20`. This is the "not directly connected" interface:
/// the two halves only touch over a bolted annular face, so it conducts far
/// worse than solid steel.
pub const FLANGE_JOINT_X_MM: f64 = -20.0;

/// Frontmost point of the model (nozzle tip).
pub const X_MIN_MM: f64 = -249.0;
/// Rearmost point of the model (gearbox flange).
pub const X_MAX_MM: f64 = 839.0;

/// Band heater inner diameter in mm (clamps onto the Ø65 barrel).
pub const BAND_INNER_D_MM: f64 = 65.0;
/// Band heater outer diameter in mm.
pub const BAND_OUTER_D_MM: f64 = 71.0;

/// Insulation sleeve extent and diameters, in mm.
pub const INSULATION_X0_MM: f64 = 4.0;
pub const INSULATION_X1_MM: f64 = 608.0;
pub const INSULATION_INNER_D_MM: f64 = 71.0;
pub const INSULATION_OUTER_D_MM: f64 = 111.0;

/// Axial extent of the screw, clipped to the modelled domain, in mm.
///
/// The `Schnecke` part runs −73…892; everything past `X_MAX_MM` is inside the
/// gearbox and is not modelled.
pub const SCREW_X0_MM: f64 = -73.0;
pub const SCREW_X1_MM: f64 = X_MAX_MM;
/// Screw outer diameter in mm.
pub const SCREW_D_MM: f64 = 25.0;

pub use crate::extruder1::zone::Zone;

impl Zone {
    /// Where this zone's heater band sits on the barrel. Its rated power is on
    /// [`Zone::rated_w`], which production code needs without the CAD model.
    pub const fn band(self) -> Band {
        match self {
            // Global X 4…204, 206…406, 408…608; all three identical 200 mm bands.
            Self::Front => Band {
                x0_mm: 4.0,
                x1_mm: 204.0,
            },
            Self::Middle => Band {
                x0_mm: 206.0,
                x1_mm: 406.0,
            },
            Self::Back => Band {
                x0_mm: 408.0,
                x1_mm: 608.0,
            },
            // Global X −77…−43: only 34 mm wide, and it sits on the Düse, on the
            // far side of the flange joint from the other three.
            Self::Nozzle => Band {
                x0_mm: -77.0,
                x1_mm: -43.0,
            },
        }
    }
}

/// A band heater clamped around the barrel. Its rated power lives on
/// [`Zone::rated_w`].
#[derive(Debug, Clone, Copy)]
pub struct Band {
    pub x0_mm: f64,
    pub x1_mm: f64,
}

impl Band {
    pub const fn width_mm(&self) -> f64 {
        self.x1_mm - self.x0_mm
    }

    /// Axial centre, where the zone's sensor pocket is drilled.
    pub const fn centre_mm(&self) -> f64 {
        (self.x0_mm + self.x1_mm) * 0.5
    }

    /// Contact area against the barrel in m².
    pub fn contact_area_m2(&self) -> f64 {
        PI * (BAND_INNER_D_MM / 1000.0) * (self.width_mm() / 1000.0)
    }

    /// Volume of the band's own shell in m³ (the Ø65 → Ø71 annulus).
    pub fn shell_volume_m3(&self) -> f64 {
        let r_in = BAND_INNER_D_MM / 2000.0;
        let r_out = BAND_OUTER_D_MM / 2000.0;
        PI * r_out.mul_add(r_out, -(r_in * r_in)) * (self.width_mm() / 1000.0)
    }
}

/// One constant-cross-section run of the barrel/Düse profile.
#[derive(Debug, Clone, Copy)]
pub struct Segment {
    pub x0_mm: f64,
    pub x1_mm: f64,
    pub outer_d_mm: f64,
    pub bore_d_mm: f64,
}

/// The steel profile along the axis, front to back.
///
/// # Approximation
///
/// The nozzle group is modelled as one Ø65 tube with a Ø25 bore. In the CAD it is
/// really the `Düse` plus a nested `Düsenplatte`, `Düsenplatte mit Sieb`,
/// `Breaker Plate` and `Breaker Ring`, with local counterbores (Ø55 at −81.5…−76.5,
/// Ø40 at −27…−20) and bolt holes. Since the whole group is bolted solid and heated
/// by a single band it behaves as one lump, so only the total mass matters — and
/// the counterbores and bolt holes roughly cancel the nested parts. This is the
/// largest geometric approximation in the model and the first thing to revisit if
/// calibration cannot match the nozzle zone's time constant.
pub const PROFILE: &[Segment] = &[
    // --- Nozzle group (Düse), heated by the 34 mm nozzle band ---
    Segment {
        x0_mm: X_MIN_MM,
        x1_mm: -43.0,
        outer_d_mm: BARREL_OUTER_D_MM,
        bore_d_mm: BORE_D_MM,
    },
    // Düse flange. CAD shows Ø110 from −40; carried back to −43 for continuity.
    Segment {
        x0_mm: -43.0,
        x1_mm: FLANGE_JOINT_X_MM,
        outer_d_mm: FLANGE_OUTER_D_MM,
        bore_d_mm: BORE_D_MM,
    },
    // --- Barrel (Schneckenzylinder) ---
    // Front flange, Ø110 over −20…0.
    Segment {
        x0_mm: FLANGE_JOINT_X_MM,
        x1_mm: 0.0,
        outer_d_mm: FLANGE_OUTER_D_MM,
        bore_d_mm: BORE_D_MM,
    },
    // Main body, Ø65 over 0…797 — this is what the three bands sit on.
    Segment {
        x0_mm: 0.0,
        x1_mm: 797.0,
        outer_d_mm: BARREL_OUTER_D_MM,
        bore_d_mm: BORE_D_MM,
    },
    // Rear flange, Ø110 over 797…820.
    Segment {
        x0_mm: 797.0,
        x1_mm: 820.0,
        outer_d_mm: FLANGE_OUTER_D_MM,
        bore_d_mm: BORE_D_MM,
    },
    // Gearbox spigot, Ø76 over 820…839.
    Segment {
        x0_mm: 820.0,
        x1_mm: X_MAX_MM,
        outer_d_mm: 76.0,
        bore_d_mm: BORE_D_MM,
    },
];

/// Steel cross-sectional area at `x`, in mm². Zero outside the modelled extent.
pub fn steel_area_mm2(x_mm: f64) -> f64 {
    for s in PROFILE {
        if x_mm >= s.x0_mm && x_mm < s.x1_mm {
            return FRAC_PI_4
                * s.outer_d_mm
                    .mul_add(s.outer_d_mm, -(s.bore_d_mm * s.bore_d_mm));
        }
    }
    0.0
}

/// Outer diameter at `x`, in mm. Falls back to the barrel diameter off the ends.
pub fn outer_d_mm(x_mm: f64) -> f64 {
    for s in PROFILE {
        if x_mm >= s.x0_mm && x_mm < s.x1_mm {
            return s.outer_d_mm;
        }
    }
    BARREL_OUTER_D_MM
}

/// Steel volume between `x0` and `x1` in mm³, integrating [`steel_area_mm2`].
///
/// Uses a fine midpoint rule so that a segment boundary falling inside the
/// interval is captured accurately.
pub fn steel_volume_mm3(x0_mm: f64, x1_mm: f64) -> f64 {
    const SUBDIVISIONS: usize = 200;
    let dx = (x1_mm - x0_mm) / SUBDIVISIONS as f64;
    (0..SUBDIVISIONS)
        .map(|i| steel_area_mm2((i as f64 + 0.5).mul_add(dx, x0_mm)) * dx)
        .sum()
}

/// Steel mass between `x0` and `x1` in kg.
pub fn steel_mass_kg(x0_mm: f64, x1_mm: f64) -> f64 {
    steel_volume_mm3(x0_mm, x1_mm) * 1e-9 * STEEL_DENSITY_KG_M3
}

/// Length of overlap between `[a0, a1]` and `[b0, b1]`, never negative.
pub fn overlap(a0: f64, a1: f64, b0: f64, b1: f64) -> f64 {
    (a1.min(b1) - a0.max(b0)).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn barrel_annulus_matches_hand_calculation() {
        // pi/4 * (65^2 - 25^2) = pi/4 * 3600 = 2827.43 mm^2
        assert_relative_eq!(steel_area_mm2(300.0), 2827.433, max_relative = 1e-5);
        // -> 0.02219 kg per mm of barrel
        assert_relative_eq!(steel_mass_kg(300.0, 301.0), 0.0221953, max_relative = 1e-4);
    }

    #[test]
    fn each_barrel_zone_carries_about_four_and_a_half_kilos() {
        for zone in [Zone::Front, Zone::Middle, Zone::Back] {
            let b = zone.band();
            let m = steel_mass_kg(b.x0_mm, b.x1_mm);
            assert!(
                (4.3..4.6).contains(&m),
                "{} zone steel mass {m:.3} kg outside expected range",
                zone.name()
            );
        }
    }

    #[test]
    fn nozzle_group_is_heavier_than_a_barrel_zone_on_a_third_the_power() {
        let nozzle_mass = steel_mass_kg(X_MIN_MM, FLANGE_JOINT_X_MM);
        let front = Zone::Front.band();
        let front_mass = steel_mass_kg(front.x0_mm, front.x1_mm);

        assert!(
            nozzle_mass > front_mass,
            "nozzle group {nozzle_mass:.2} kg should outweigh a barrel zone {front_mass:.2} kg"
        );
        // ~6 kg on 200 W, versus ~4.4 kg on 700 W: the ratio of (mass / power) is
        // what makes the nozzle zone so much slower.
        let nozzle_ratio = nozzle_mass / Zone::Nozzle.rated_w();
        let front_ratio = front_mass / Zone::Front.rated_w();
        assert!(
            nozzle_ratio > 4.0 * front_ratio,
            "nozzle kg/W {nozzle_ratio:.5} should be several times front's {front_ratio:.5}"
        );
    }

    #[test]
    fn whole_barrel_mass_is_plausible() {
        let m = steel_mass_kg(FLANGE_JOINT_X_MM, X_MAX_MM);
        assert!(
            (19.0..24.0).contains(&m),
            "barrel mass {m:.2} kg outside plausible range"
        );
    }

    #[test]
    fn nozzle_band_is_much_smaller_than_the_barrel_bands() {
        let n = Zone::Nozzle.band();
        let f = Zone::Front.band();
        assert_relative_eq!(n.width_mm(), 34.0, epsilon = 1e-9);
        assert_relative_eq!(f.width_mm(), 200.0, epsilon = 1e-9);
        assert!(n.contact_area_m2() < f.contact_area_m2() / 5.0);
    }

    #[test]
    fn bands_do_not_overlap_and_sit_inside_the_sleeve() {
        let mut bands: Vec<Band> = [Zone::Front, Zone::Middle, Zone::Back]
            .iter()
            .map(|z| z.band())
            .collect();
        bands.sort_by(|a, b| a.x0_mm.total_cmp(&b.x0_mm));
        for w in bands.windows(2) {
            assert!(w[0].x1_mm <= w[1].x0_mm, "barrel bands must not overlap");
        }
        assert!(bands[0].x0_mm >= INSULATION_X0_MM);
        assert!(bands[2].x1_mm <= INSULATION_X1_MM);
        // The nozzle band is deliberately outside the sleeve.
        let n = Zone::Nozzle.band();
        assert!(n.x1_mm < INSULATION_X0_MM);
    }

    #[test]
    fn ports_match_the_production_wiring() {
        assert_eq!(Zone::Front.port(), 0);
        assert_eq!(Zone::Middle.port(), 1);
        assert_eq!(Zone::Back.port(), 2);
        assert_eq!(Zone::Nozzle.port(), 3);
    }
}
