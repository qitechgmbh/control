//! The extruder's thermal plant, as a 1-D axial network.
//!
//! # Structure
//!
//! ```text
//!                     ambient
//!                        ^
//!                    (insulated / bare)
//!                        |
//!   [band]---(contact)---+                 one band node per zone
//!      |
//!  (contact)
//!      |
//!  [steel cell]--(steel)--[steel cell]--...      ~54 cells, 20 mm each
//!      |    \                                     nozzle group and barrel are
//!  (bore gap) `--(lag)--[sensor]                  joined by the flange contact
//!      |
//!  [screw cell]--(steel)--[screw cell]--...
//! ```
//!
//! Splitting the band from the steel is what produces overshoot: the band runs
//! far hotter than the barrel and keeps discharging into it after the relay
//! opens. A model that injects heater power straight into the steel cannot
//! reproduce the +31 K middle-zone overshoot seen on the real machine.
//!
//! The nozzle group and the barrel are separate cell chains joined only by the
//! bolted flange contact, matching the physical split the CAD shows at
//! `x = -20 mm`.

use std::f64::consts::{FRAC_PI_4, PI};

use control_core::thermal::{AmbientLoss, Node, NodeId, ThermalNetwork};

use super::geometry::{
    self, BAND_INNER_D_MM, BAND_OUTER_D_MM, BORE_D_MM, FLANGE_JOINT_X_MM, INSULATION_INNER_D_MM,
    INSULATION_OUTER_D_MM, INSULATION_X0_MM, INSULATION_X1_MM, SCREW_D_MM, SCREW_X0_MM,
    STEEL_DENSITY_KG_M3, X_MAX_MM, X_MIN_MM, Zone,
};
use super::params::ExtruderThermalParams;

/// One axial slice of barrel or Düse steel.
#[derive(Debug, Clone, Copy)]
struct Cell {
    id: NodeId,
    x0_mm: f64,
    x1_mm: f64,
    /// Mean steel cross-section over the cell in mm².
    area_mm2: f64,
}

impl Cell {
    const fn length_mm(&self) -> f64 {
        self.x1_mm - self.x0_mm
    }
    const fn centre_mm(&self) -> f64 {
        (self.x0_mm + self.x1_mm) * 0.5
    }
}

/// The extruder's heating plant.
pub struct ExtruderThermalModel {
    net: ThermalNetwork,
    params: ExtruderThermalParams,
    cells: Vec<Cell>,
    /// Screw cells, parallel to `cells`; `None` where the screw does not reach
    /// or when the screw is not modelled.
    screw: Vec<Option<NodeId>>,
    /// Band node per zone, indexed by [`Zone::port`].
    bands: [NodeId; 4],
    /// Sensor node per zone, indexed by [`Zone::port`].
    sensors: [NodeId; 4],
    /// Index into `cells` of the cell each zone's sensor sits in.
    sensor_cells: [usize; 4],
}

impl ExtruderThermalModel {
    /// Build the network and set every node to `ambient_c`.
    pub fn new(params: ExtruderThermalParams) -> Self {
        let mut net = ThermalNetwork::new(params.ambient_c);

        let cells = Self::build_steel(&mut net, &params);
        let screw = Self::build_screw(&mut net, &params, &cells);
        let bands = Self::build_bands(&mut net, &params, &cells);
        Self::build_ambient_losses(&mut net, &params, &cells);
        let (sensors, sensor_cells) = Self::build_sensors(&mut net, &params, &cells);

        Self {
            net,
            params,
            cells,
            screw,
            bands,
            sensors,
            sensor_cells,
        }
    }

    /// The barrel steel: two chains of axial cells, split at the flange joint.
    fn build_steel(net: &mut ThermalNetwork, params: &ExtruderThermalParams) -> Vec<Cell> {
        let rho_cp = STEEL_DENSITY_KG_M3 * params.cp_steel;

        let mut cells: Vec<Cell> = Vec::new();
        let mut chain_bounds: Vec<(usize, usize)> = Vec::new();

        for (x_start, x_end, label) in [
            (X_MIN_MM, FLANGE_JOINT_X_MM, "nozzle"),
            (FLANGE_JOINT_X_MM, X_MAX_MM, "barrel"),
        ] {
            let n = (((x_end - x_start) / params.cell_size_mm).round() as usize).max(1);
            let dx = (x_end - x_start) / n as f64;
            let first = cells.len();
            for i in 0..n {
                let x0 = (i as f64).mul_add(dx, x_start);
                let x1 = x0 + dx;
                let volume_mm3 = geometry::steel_volume_mm3(x0, x1);
                let capacity = volume_mm3 * 1e-9 * rho_cp;
                let id = net.add_node(Node::new(
                    format!("{label}[{i}]"),
                    capacity,
                    params.ambient_c,
                ));
                cells.push(Cell {
                    id,
                    x0_mm: x0,
                    x1_mm: x1,
                    area_mm2: volume_mm3 / dx,
                });
            }
            chain_bounds.push((first, cells.len()));
        }

        // Axial conduction inside each chain. Series resistance of the two half
        // cells, so a diameter step (the flanges) is handled correctly.
        for &(first, end) in &chain_bounds {
            for i in first..end.saturating_sub(1) {
                let a = &cells[i];
                let b = &cells[i + 1];
                let r_a = (a.length_mm() * 0.5e-3) / (params.k_steel * a.area_mm2 * 1e-6);
                let r_b = (b.length_mm() * 0.5e-3) / (params.k_steel * b.area_mm2 * 1e-6);
                net.connect(a.id, b.id, 1.0 / (r_a + r_b));
            }
        }

        // The bolted Düse/barrel joint: contact conductance over the mating
        // annular face, not solid steel.
        let (nozzle_first, nozzle_end) = chain_bounds[0];
        let (barrel_first, _) = chain_bounds[1];
        debug_assert!(nozzle_end > nozzle_first);
        let joint_area_m2 = geometry::steel_area_mm2(FLANGE_JOINT_X_MM - 1.0) * 1e-6;
        net.connect(
            cells[nozzle_end - 1].id,
            cells[barrel_first].id,
            params.flange_contact_h * joint_area_m2,
        );

        cells
    }

    fn build_screw(
        net: &mut ThermalNetwork,
        params: &ExtruderThermalParams,
        cells: &[Cell],
    ) -> Vec<Option<NodeId>> {
        // ---- screw ----
        let mut screw: Vec<Option<NodeId>> = vec![None; cells.len()];
        if params.include_screw {
            let screw_area_m2 = FRAC_PI_4 * (SCREW_D_MM * 1e-3).powi(2);
            let screw_rho_cp = STEEL_DENSITY_KG_M3 * params.cp_steel;
            for (i, c) in cells.iter().enumerate() {
                let covered = geometry::overlap(c.x0_mm, c.x1_mm, SCREW_X0_MM, X_MAX_MM);
                if covered <= 0.0 {
                    continue;
                }
                let capacity = screw_area_m2 * (covered * 1e-3) * screw_rho_cp;
                let id = net.add_node(Node::new(format!("screw[{i}]"), capacity, params.ambient_c));
                // Across the bore gap into the surrounding steel.
                let gap_area = PI * (BORE_D_MM * 1e-3) * (covered * 1e-3);
                net.connect(c.id, id, params.bore_gap_h * gap_area);
                screw[i] = Some(id);
            }
            // Axial conduction along the screw itself.
            for i in 0..cells.len().saturating_sub(1) {
                if let (Some(a), Some(b)) = (screw[i], screw[i + 1]) {
                    let dx = (cells[i].length_mm() + cells[i + 1].length_mm()) * 0.5e-3;
                    net.connect(a, b, params.k_steel * screw_area_m2 / dx);
                }
            }
        }

        screw
    }

    fn build_bands(
        net: &mut ThermalNetwork,
        params: &ExtruderThermalParams,
        cells: &[Cell],
    ) -> [NodeId; 4] {
        // ---- band heaters ----
        // One node per band, separate from the steel it heats. That separation is
        // what produces overshoot: while driven, the band sits ~170 K above the
        // barrel, and when the relay opens it keeps discharging for minutes.
        let mut bands = [NodeId(0); 4];
        for zone in Zone::ALL {
            let band = zone.band();
            let id = net.add_node(Node::new(
                format!("band_{}", zone.name()),
                band.contact_area_m2() * params.band_heat_capacity_j_per_m2_k,
                params.ambient_c,
            ));
            bands[zone.port()] = id;

            // Contact with every cell the band overlaps, in proportion to the
            // overlap length.
            for c in cells {
                let covered = geometry::overlap(c.x0_mm, c.x1_mm, band.x0_mm, band.x1_mm);
                if covered <= 0.0 {
                    continue;
                }
                let area = PI * (BAND_INNER_D_MM * 1e-3) * (covered * 1e-3);
                net.connect(c.id, id, params.band_contact_h * area);
            }

            // The band, not the steel underneath it, is what faces the room.
            let insulated =
                geometry::overlap(band.x0_mm, band.x1_mm, INSULATION_X0_MM, INSULATION_X1_MM);
            let bare = band.width_mm() - insulated;
            if insulated > 0.0 {
                net.add_loss(
                    id,
                    AmbientLoss::Insulated {
                        k_insulation: params.k_insulation,
                        r_inner_m: INSULATION_INNER_D_MM / 2000.0,
                        r_outer_m: INSULATION_OUTER_D_MM / 2000.0,
                        length_m: insulated * 1e-3,
                        convection_coeff: params.bare_convection_coeff,
                        emissivity: params.insulation_emissivity,
                    },
                );
            }
            if bare > 0.0 {
                net.add_loss(
                    id,
                    AmbientLoss::Bare {
                        area_m2: PI * (BAND_OUTER_D_MM * 1e-3) * (bare * 1e-3),
                        convection_coeff: params.bare_convection_coeff,
                        emissivity: params.bare_emissivity,
                    },
                );
            }
        }

        bands
    }

    fn build_ambient_losses(
        net: &mut ThermalNetwork,
        params: &ExtruderThermalParams,
        cells: &[Cell],
    ) {
        // ---- ambient loss from the steel that no band covers ----
        for c in cells {
            let banded: f64 = Zone::ALL
                .iter()
                .map(|z| {
                    let b = z.band();
                    geometry::overlap(c.x0_mm, c.x1_mm, b.x0_mm, b.x1_mm)
                })
                .sum();
            let exposed = (c.length_mm() - banded).max(0.0);
            if exposed <= 0.0 {
                continue;
            }
            let d = geometry::outer_d_mm(c.centre_mm());
            net.add_loss(
                c.id,
                AmbientLoss::Bare {
                    area_m2: PI * (d * 1e-3) * (exposed * 1e-3),
                    convection_coeff: params.bare_convection_coeff,
                    emissivity: params.bare_emissivity,
                },
            );
        }

        // End faces, and the gearbox acting as a heat sink on the rear cell.
        let end_area = |x: f64| geometry::steel_area_mm2(x) * 1e-6;
        net.add_loss(
            cells[0].id,
            AmbientLoss::Bare {
                area_m2: end_area(X_MIN_MM + 1.0),
                convection_coeff: params.bare_convection_coeff,
                emissivity: params.bare_emissivity,
            },
        );
        let last = cells.len() - 1;
        if params.gearbox_sink_g > 0.0 {
            let sink = net.add_node(Node::new("gearbox_sink", 1e9, params.ambient_c));
            net.connect(cells[last].id, sink, params.gearbox_sink_g);
        }
    }

    fn build_sensors(
        net: &mut ThermalNetwork,
        params: &ExtruderThermalParams,
        cells: &[Cell],
    ) -> ([NodeId; 4], [usize; 4]) {
        // ---- sensors ----
        // One RTD per zone, in a pocket in the barrel wall under the band centre.
        let mut sensors = [NodeId(0); 4];
        let mut sensor_cells = [0usize; 4];
        for zone in Zone::ALL {
            let centre = zone.band().centre_mm();
            let idx = cells
                .iter()
                .position(|c| centre >= c.x0_mm && centre < c.x1_mm)
                .unwrap_or_else(|| {
                    // Fall back to the nearest cell centre.
                    cells
                        .iter()
                        .enumerate()
                        .min_by(|a, b| {
                            (a.1.centre_mm() - centre)
                                .abs()
                                .total_cmp(&(b.1.centre_mm() - centre).abs())
                        })
                        .map(|(i, _)| i)
                        .expect("model always has cells")
                });
            let id = net.add_node(Node::new(
                format!("sensor_{}", zone.name()),
                params.sensor_heat_capacity,
                params.ambient_c,
            ));
            net.connect(
                cells[idx].id,
                id,
                params.sensor_heat_capacity / params.sensor_tau_s,
            );
            sensors[zone.port()] = id;
            sensor_cells[zone.port()] = idx;
        }

        (sensors, sensor_cells)
    }

    pub const fn params(&self) -> &ExtruderThermalParams {
        &self.params
    }

    /// Electrical power currently dissipated in a zone's band, in W.
    pub fn set_band_power(&mut self, zone: Zone, watts: f64) {
        self.net.set_power(self.bands[zone.port()], watts);
    }

    /// What the zone's RTD tip is at, in °C — before EL3204 quantisation.
    pub fn sensor_c(&self, zone: Zone) -> f64 {
        self.net.temperature(self.sensors[zone.port()])
    }

    /// Temperature of the band heater itself, in °C. Runs well above the steel
    /// while driven; this is the reservoir that causes the overshoot.
    pub fn band_c(&self, zone: Zone) -> f64 {
        self.net.temperature(self.bands[zone.port()])
    }

    /// Temperature of the barrel steel the zone's sensor sits in, in °C.
    pub fn steel_c(&self, zone: Zone) -> f64 {
        self.net
            .temperature(self.cells[self.sensor_cells[zone.port()]].id)
    }

    /// Steel temperature at an arbitrary axial position, in °C.
    pub fn steel_c_at(&self, x_mm: f64) -> f64 {
        let c = self
            .cells
            .iter()
            .min_by(|a, b| {
                (a.centre_mm() - x_mm)
                    .abs()
                    .total_cmp(&(b.centre_mm() - x_mm).abs())
            })
            .expect("model always has cells");
        self.net.temperature(c.id)
    }

    /// Put every node at the same temperature, e.g. for a cold start.
    pub fn set_uniform_temperature(&mut self, temperature_c: f64) {
        for i in 0..self.net.node_count() {
            self.net.set_temperature(NodeId(i), temperature_c);
        }
        // The gearbox sink is an infinite reservoir; it stays at ambient.
        if let Some(idx) = self
            .net
            .nodes()
            .iter()
            .position(|n| n.label == "gearbox_sink")
        {
            self.net.set_temperature(NodeId(idx), self.params.ambient_c);
        }
    }

    /// Advance the plant by `dt` seconds.
    pub fn step(&mut self, dt: f64) {
        self.net.step(dt);
    }

    /// Explicit-Euler stability limit at the current temperatures, in seconds.
    pub fn max_stable_dt(&self) -> f64 {
        self.net.max_stable_dt()
    }

    /// Total steel mass represented by the cell chain, in kg. Diagnostics only.
    pub fn steel_mass_kg(&self) -> f64 {
        self.cells
            .iter()
            .map(|c| geometry::steel_mass_kg(c.x0_mm, c.x1_mm))
            .sum()
    }

    /// Number of screw cells actually created.
    pub fn screw_cell_count(&self) -> usize {
        self.screw.iter().filter(|s| s.is_some()).count()
    }

    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    pub const fn network(&self) -> &ThermalNetwork {
        &self.net
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    fn model() -> ExtruderThermalModel {
        ExtruderThermalModel::new(ExtruderThermalParams::default())
    }

    #[test]
    fn discretisation_covers_the_whole_machine() {
        let m = model();
        // 229 mm nozzle group + 859 mm barrel at ~20 mm cells.
        assert_eq!(m.cell_count(), 11 + 43);
        assert_relative_eq!(m.steel_mass_kg(), 27.3, max_relative = 0.05);
        assert!(m.screw_cell_count() > 40);
    }

    #[test]
    fn is_stable_at_the_harness_time_step() {
        let mut m = model();
        // Evaluate hot: the loss terms grow with temperature, so a cold network
        // reports an optimistic limit.
        m.set_uniform_temperature(300.0);
        let limit = m.max_stable_dt();
        assert!(
            limit > 10.0 * super::super::harness::DT_PLANT_S,
            "stability limit {limit:.3} s leaves too little margin over the \
             {:.3} s plant step",
            super::super::harness::DT_PLANT_S
        );
    }

    #[test]
    fn no_power_means_no_change() {
        let mut m = model();
        m.set_uniform_temperature(m.params.ambient_c);
        for _ in 0..10_000 {
            m.step(0.01);
        }
        for zone in Zone::ALL {
            assert_relative_eq!(m.sensor_c(zone), m.params.ambient_c, epsilon = 1e-6);
        }
    }

    /// The band must run hotter than the steel it is heating, otherwise the
    /// overshoot mechanism is missing.
    #[test]
    fn band_runs_hotter_than_the_steel() {
        let mut m = model();
        m.set_uniform_temperature(22.0);
        m.set_band_power(Zone::Front, 700.0);
        for _ in 0..30_000 {
            m.step(0.01);
        }
        let band = m.band_c(Zone::Front);
        let steel = m.steel_c(Zone::Front);
        assert!(
            band > steel + 10.0,
            "band {band:.1} C should lead steel {steel:.1} C while driven"
        );
    }

    /// The measured open-loop ramp: with all three barrel bands at their real
    /// 700 W, the recorded machine rose roughly 36 K per 150 s early in the
    /// heat-up (see `data/heatup_2026-02-24.csv`).
    #[test]
    fn open_loop_ramp_rate_is_in_the_measured_range() {
        let mut m = model();
        m.set_uniform_temperature(22.0);
        for zone in [Zone::Front, Zone::Middle, Zone::Back] {
            m.set_band_power(zone, zone.rated_w());
        }
        // Skip the first 150 s so the band and sensor have caught up, then
        // measure over the next 150 s, matching how the figure was read off the
        // recording.
        let dt = 0.01;
        for _ in 0..15_000 {
            m.step(dt);
        }
        let t0 = m.steel_c(Zone::Front);
        for _ in 0..15_000 {
            m.step(dt);
        }
        let rise = m.steel_c(Zone::Front) - t0;
        assert!(
            (25.0..48.0).contains(&rise),
            "front rose {rise:.1} K in 150 s; the recording gives ~36 K"
        );
    }

    /// Middle is flanked by heated zones, front and back leak into cold steel,
    /// so with identical power the middle must end up hottest.
    #[test]
    fn middle_runs_hottest_under_equal_power() {
        let mut m = model();
        m.set_uniform_temperature(22.0);
        for zone in [Zone::Front, Zone::Middle, Zone::Back] {
            m.set_band_power(zone, 700.0);
        }
        for _ in 0..60_000 {
            m.step(0.01);
        }
        let (f, mi, b) = (
            m.steel_c(Zone::Front),
            m.steel_c(Zone::Middle),
            m.steel_c(Zone::Back),
        );
        assert!(
            mi > f && mi > b,
            "middle {mi:.1} should exceed front {f:.1} and back {b:.1}"
        );
    }

    #[test]
    fn sensor_lags_the_steel_it_sits_in() {
        let mut m = model();
        m.set_uniform_temperature(22.0);
        m.set_band_power(Zone::Front, 700.0);
        for _ in 0..2_000 {
            m.step(0.01);
        }
        assert!(m.sensor_c(Zone::Front) < m.steel_c(Zone::Front));
    }
}
