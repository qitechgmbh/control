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

/// The screw's root (core) diameter along the axis, measured from
/// `Oberbau_UBG.step`.
///
/// [`SCREW_D_MM`] is the flight *tip* diameter, which is why the model filled
/// the whole Ø25 bore with steel until the melt was added. The polymer really
/// travels in the annulus between the root and the bore, and the root is not
/// constant: this is a textbook three-zone screw, deep where cold pellets enter
/// and shallow where melt is pumped into the die.
///
/// # Provenance
///
/// Read out of the STEP file the same way the barrel [`PROFILE`] was. The
/// `Schnecke` part is placed in the assembly by a pure +879 mm translation along
/// x (`ITEM_DEFINED_TRANSFORMATION` #164), so its local coordinates map to the
/// global frame by adding 879 — which is what puts the flighted section's local
/// −952…−52 at the global −73…827 that [`SCREW_X0_MM`] already recorded. The
/// root diameters are the radii of the `CYLINDRICAL_SURFACE` and
/// `CONICAL_SURFACE` faces between the flights: Ø21.2 metering, a 0.007 rad
/// cone tapering to Ø13.6, then Ø13.6 through the feed section, and a Ø19
/// journal behind it.
///
/// Only used when [`super::params::MeltParams::enabled`] is set; with the melt
/// off the screw stays the solid Ø25 cylinder the thermal calibration assumed.
pub const SCREW_ROOT_PROFILE: &[RootSegment] = &[
    // Nose and metering section: the shallowest channel, next to the die.
    RootSegment {
        x0_mm: -73.0,
        x1_mm: 97.0,
        d0_mm: 21.2,
        d1_mm: 21.2,
    },
    // Compression: the cone, Ø21.2 back to Ø13.6 over 530 mm.
    RootSegment {
        x0_mm: 97.0,
        x1_mm: 627.0,
        d0_mm: 21.2,
        d1_mm: 13.6,
    },
    // Feed: the deepest channel, where the cold pellets drop in.
    RootSegment {
        x0_mm: 627.0,
        x1_mm: 747.0,
        d0_mm: 13.6,
        d1_mm: 13.6,
    },
    // The journal behind the feed throat. Flighted, but nothing is in it.
    RootSegment {
        x0_mm: 747.0,
        x1_mm: 827.0,
        d0_mm: 19.0,
        d1_mm: 19.0,
    },
];

/// One run of the screw root, linear from `d0_mm` to `d1_mm`.
#[derive(Debug, Clone, Copy)]
pub struct RootSegment {
    pub x0_mm: f64,
    pub x1_mm: f64,
    pub d0_mm: f64,
    pub d1_mm: f64,
}

/// Axial position of the feed throat in mm, where material enters.
///
/// Taken in the middle of the measured deep-channel feed section (627…747 mm),
/// which is the only place on the screw shaped to accept pellets. The throat
/// opening itself is in the feed housing, which is not part of the modelled
/// geometry, so this is the screw's evidence rather than the hopper's.
///
/// Its exact value matters little: the 231 mm of bare barrel behind the back
/// band sits near ambient anyway, so material enters cold wherever in it the
/// throat is.
pub const FEED_X_MM: f64 = 700.0;

/// Screw root diameter at `x`, in mm. Zero where the screw does not reach.
pub fn screw_root_d_mm(x_mm: f64) -> f64 {
    for s in SCREW_ROOT_PROFILE {
        if x_mm >= s.x0_mm && x_mm < s.x1_mm {
            let t = (x_mm - s.x0_mm) / (s.x1_mm - s.x0_mm);
            return (s.d1_mm - s.d0_mm).mul_add(t, s.d0_mm);
        }
    }
    0.0
}

/// Cross-section of the polymer channel at `x`, in mm².
///
/// Inside the screw's extent this is the annulus between the root and the bore;
/// ahead of the screw, in the Düse, the full bore is melt. Zero outside the
/// modelled machine and behind the feed throat, where there is no material yet.
///
/// # Approximations
///
/// The flight land is ignored — it occupies roughly a tenth of the annulus on a
/// square-pitched screw, and resolving the helix would buy a correction far
/// smaller than the uncertainty in [`super::params::MeltParams::film_h`], which
/// absorbs it.
///
/// The channel is also treated as completely full. The metering section is; the
/// feed section is not, since pellets enter at a lower bulk density and only
/// compact as they melt. That makes the modelled residence time too long at low
/// throughput without affecting the steady-state thermal load.
pub fn channel_area_mm2(x_mm: f64) -> f64 {
    if x_mm < X_MIN_MM || x_mm > FEED_X_MM {
        return 0.0;
    }
    let root = screw_root_d_mm(x_mm);
    FRAC_PI_4 * root.mul_add(-root, BORE_D_MM * BORE_D_MM)
}

/// Solid cross-section of the screw at `x`, in mm² — whatever of the bore the
/// polymer is not using. Zero where the screw does not reach.
///
/// Together with [`channel_area_mm2`] this exactly fills the bore, which is the
/// invariant that keeps the two from drifting apart.
pub fn screw_solid_area_mm2(x_mm: f64) -> f64 {
    let root = screw_root_d_mm(x_mm);
    FRAC_PI_4 * root * root
}

/// Polymer channel volume between `x0` and `x1` in mm³, integrating
/// [`channel_area_mm2`] with the same midpoint rule as [`steel_volume_mm3`].
pub fn channel_volume_mm3(x0_mm: f64, x1_mm: f64) -> f64 {
    const SUBDIVISIONS: usize = 200;
    let dx = (x1_mm - x0_mm) / SUBDIVISIONS as f64;
    (0..SUBDIVISIONS)
        .map(|i| channel_area_mm2((i as f64 + 0.5).mul_add(dx, x0_mm)) * dx)
        .sum()
}

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

    /// The screw and the polymer must exactly fill the bore between them — if
    /// they ever stop doing that, one of the two areas has drifted.
    #[test]
    fn the_screw_and_the_channel_fill_the_bore() {
        let bore = FRAC_PI_4 * BORE_D_MM * BORE_D_MM;
        for x in [-70.0, 0.0, 97.0, 300.0, 627.0, 700.0] {
            assert_relative_eq!(
                channel_area_mm2(x) + screw_solid_area_mm2(x),
                bore,
                max_relative = 1e-9
            );
        }
    }

    /// Ahead of the screw the Düse is a plain tube, so all of it is melt.
    #[test]
    fn the_nozzle_bore_is_all_melt() {
        let bore = FRAC_PI_4 * BORE_D_MM * BORE_D_MM;
        assert_relative_eq!(channel_area_mm2(-150.0), bore, max_relative = 1e-9);
        assert!(screw_solid_area_mm2(-150.0).abs() < f64::EPSILON);
    }

    /// The measured three-zone screw: deep where cold pellets enter, shallow
    /// where melt is pumped into the die. Getting this backwards would make the
    /// feed section the shallow one, which is the opposite of how a screw works.
    #[test]
    fn the_channel_is_deepest_at_the_feed_end() {
        let metering = channel_area_mm2(0.0);
        let compression = channel_area_mm2(350.0);
        let feed = channel_area_mm2(680.0);
        assert!(
            feed > compression && compression > metering,
            "channel areas should grow towards the feed: \
             metering {metering:.0}, compression {compression:.0}, feed {feed:.0} mm²"
        );
        // Ø13.6 root against a Ø25 bore is a 5.7 mm channel; Ø21.2 is 1.9 mm.
        assert_relative_eq!(screw_root_d_mm(680.0), 13.6, max_relative = 1e-9);
        assert_relative_eq!(screw_root_d_mm(0.0), 21.2, max_relative = 1e-9);
    }

    /// Roughly a third of a kilo of polymer sits in the machine. At the rated
    /// 10 kg/h that is a residence time of a couple of minutes, which is the
    /// right order for a single-screw extruder.
    #[test]
    fn the_polymer_inventory_is_about_a_third_of_a_kilo() {
        let volume_mm3 = channel_volume_mm3(X_MIN_MM, FEED_X_MM);
        let mass_kg = volume_mm3 * 1e-9 * 1200.0;
        assert!(
            (0.25..0.45).contains(&mass_kg),
            "polymer inventory {mass_kg:.3} kg outside the expected range"
        );

        let residence_s = mass_kg / (10.0 / 3600.0);
        assert!(
            (60.0..240.0).contains(&residence_s),
            "residence time {residence_s:.0} s at 10 kg/h is not plausible"
        );
    }

    /// Making room for the polymer takes real steel out of the bore, and that
    /// is the one way enabling the melt perturbs the existing calibration.
    #[test]
    fn making_room_for_the_melt_lightens_the_screw() {
        let solid = |x0: f64, x1: f64| {
            const N: usize = 2000;
            let dx = (x1 - x0) / N as f64;
            (0..N)
                .map(|i| screw_solid_area_mm2((i as f64 + 0.5).mul_add(dx, x0)) * dx)
                .sum::<f64>()
                * 1e-9
                * STEEL_DENSITY_KG_M3
        };
        let rooted = solid(SCREW_X0_MM, 827.0);
        let as_a_solid_bar = FRAC_PI_4
            * SCREW_D_MM
            * SCREW_D_MM
            * (827.0 - SCREW_X0_MM)
            * 1e-9
            * STEEL_DENSITY_KG_M3;
        assert!(
            rooted < as_a_solid_bar * 0.6,
            "the real screw {rooted:.2} kg should be far lighter than the \
             {as_a_solid_bar:.2} kg solid bar the model used to assume"
        );
    }

    #[test]
    fn ports_match_the_production_wiring() {
        assert_eq!(Zone::Front.port(), 0);
        assert_eq!(Zone::Middle.port(), 1);
        assert_eq!(Zone::Back.port(), 2);
        assert_eq!(Zone::Nozzle.port(), 3);
    }
}
