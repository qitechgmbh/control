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
//!      |
//!   (film)                                   only with `MeltParams::enabled`
//!      |
//!  [melt cell]<--(flow)--[melt cell]<--...   material runs back -> nozzle,
//!      |                                     i.e. against increasing x
//!   (shear in)
//! ```
//!
//! # The melt
//!
//! With [`super::params::MeltParams::enabled`], a polymer node joins each cell
//! from the feed throat down to the die, chained by
//! [`control_core::thermal::Flow`] edges that carry enthalpy downstream. Cold
//! material enters at the throat, viscous shear is injected along the screw, and
//! the enthalpy the extrudate leaves with drops out of the model at the die.
//!
//! The chain is ordered by **descending x** — back zone, middle, front, nozzle —
//! which is the reverse of [`Zone::port`] order, so it is built from the geometry
//! and never from a port index. That direction is the whole point: the feed end
//! meets material at ambient and does the melting work, while by the front the
//! polymer is already hot and gives heat back instead. See `README.md`.
//!
//! Making room for the polymer means the screw is no longer a solid Ø25 bar but
//! its measured root profile. With the melt off it stays the solid bar the
//! thermal calibration was fitted against, and the network is unchanged.
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

use control_core::thermal::{
    AmbientLoss, Flow, FlowId, FlowTerminal, Node, NodeId, ThermalNetwork,
};

use super::geometry::{
    self, BAND_INNER_D_MM, BAND_OUTER_D_MM, BORE_D_MM, FEED_X_MM, FLANGE_JOINT_X_MM,
    INSULATION_INNER_D_MM, INSULATION_OUTER_D_MM, INSULATION_X0_MM, INSULATION_X1_MM, SCREW_D_MM,
    SCREW_X0_MM, STEEL_DENSITY_KG_M3, X_MAX_MM, X_MIN_MM, Zone,
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

/// The polymer in the bore, as a chain of advected cells.
///
/// Everything here is indexed one of two ways, and mixing them up is the easy
/// mistake: `nodes`, `mass_kg` and `film_g` are **parallel to `cells`**, while
/// `order`, `flows` and `shear_weight` run in **flow order**, feed to die.
struct MeltChain {
    /// Polymer node per steel cell; `None` behind the feed throat.
    nodes: Vec<Option<NodeId>>,
    /// Polymer mass in kg per steel cell.
    mass_kg: Vec<f64>,
    /// Melt-to-bore conductance in W/K per steel cell, for reporting the load.
    film_g: Vec<f64>,
    /// Cell indices from the feed throat down to the die.
    order: Vec<usize>,
    /// Advection edges: the feed inlet, then one per consecutive pair in
    /// `order`, then the die outlet.
    flows: Vec<FlowId>,
    /// Which cell each flow draws from; `None` for the inlet, which draws from
    /// the feed boundary.
    flow_source: Vec<Option<usize>>,
    /// Share of the shear power each cell in `order` receives; sums to 1.
    shear_weight: Vec<f64>,
    /// Enthalpy datum, equal to the feed temperature.
    datum_c: f64,
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
    /// The polymer, when [`super::params::MeltParams::enabled`] is set.
    melt: Option<MeltChain>,
    /// Current screw speed in rpm.
    screw_rpm: f64,
    /// Throughput at that speed, in kg/s.
    mass_flow_kg_s: f64,
    /// Shear power at that speed, in W, after the drive cap.
    shear_power_w: f64,
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
        // Last, so that enabling the melt cannot shift any existing NodeId.
        let melt = params
            .melt
            .enabled
            .then(|| Self::build_melt(&mut net, &params, &cells, &screw));

        let mut model = Self {
            net,
            params,
            cells,
            screw,
            bands,
            sensors,
            sensor_cells,
            melt,
            screw_rpm: 0.0,
            mass_flow_kg_s: 0.0,
            shear_power_w: 0.0,
        };
        model.refresh_melt();
        model
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
        // Without the melt the screw is the solid Ø25 bar the thermal
        // calibration assumed. With it, the polymer needs the channel, so the
        // screw shrinks to its measured root profile and the bore's heat
        // capacity is shared between steel and polymer.
        let solid_area_m2 = |x_mm: f64| {
            if params.melt.enabled {
                geometry::screw_solid_area_mm2(x_mm) * 1e-6
            } else {
                FRAC_PI_4 * (SCREW_D_MM * 1e-3).powi(2)
            }
        };

        let mut screw: Vec<Option<NodeId>> = vec![None; cells.len()];
        if params.include_screw {
            let screw_rho_cp = STEEL_DENSITY_KG_M3 * params.cp_steel;
            for (i, c) in cells.iter().enumerate() {
                let covered = geometry::overlap(c.x0_mm, c.x1_mm, SCREW_X0_MM, X_MAX_MM);
                if covered <= 0.0 {
                    continue;
                }
                let area_m2 = solid_area_m2(c.centre_mm());
                if area_m2 <= 0.0 {
                    continue;
                }
                let capacity = area_m2 * (covered * 1e-3) * screw_rho_cp;
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
                    let area_m2 = (solid_area_m2(cells[i].centre_mm())
                        + solid_area_m2(cells[i + 1].centre_mm()))
                        * 0.5;
                    net.connect(a, b, params.k_steel * area_m2 / dx);
                }
            }
        }

        screw
    }

    /// The polymer: one node per cell from the feed throat down to the die,
    /// coupled to the bore and chained together by advection.
    ///
    /// The chain is ordered by descending `x`, which is the direction material
    /// actually travels — back zone, middle, front, nozzle, out. That is the
    /// reverse of [`Zone::port`] ordering, which is the wiring order, so the
    /// order is taken from the geometry and never from a port index.
    fn build_melt(
        net: &mut ThermalNetwork,
        params: &ExtruderThermalParams,
        cells: &[Cell],
        screw: &[Option<NodeId>],
    ) -> MeltChain {
        let melt = &params.melt;
        let mut nodes: Vec<Option<NodeId>> = vec![None; cells.len()];
        let mut mass_kg = vec![0.0; cells.len()];
        let mut film_g = vec![0.0; cells.len()];

        // Feed to die: descending x.
        let mut order: Vec<usize> = (0..cells.len())
            .filter(|&i| geometry::channel_volume_mm3(cells[i].x0_mm, cells[i].x1_mm) > 0.0)
            .collect();
        order.sort_by(|a, b| cells[*b].centre_mm().total_cmp(&cells[*a].centre_mm()));

        for &i in &order {
            let c = &cells[i];
            let volume_mm3 = geometry::channel_volume_mm3(c.x0_mm, c.x1_mm);
            let mass = volume_mm3 * 1e-9 * melt.density_kg_m3;
            let capacity = mass * melt.apparent_cp_j_per_kg_k(params.ambient_c);
            let id = net.add_node(Node::new(format!("melt[{i}]"), capacity, params.ambient_c));

            // Against the bore, and against the screw root it wraps.
            let wetted = geometry::overlap(c.x0_mm, c.x1_mm, X_MIN_MM, FEED_X_MM) * 1e-3;
            let g_bore = melt.film_h * PI * (BORE_D_MM * 1e-3) * wetted;
            net.connect(c.id, id, g_bore);
            if let Some(screw_id) = screw[i] {
                let root_m = geometry::screw_root_d_mm(c.centre_mm()) * 1e-3;
                net.connect(screw_id, id, melt.film_h * PI * root_m * wetted);
            }

            nodes[i] = Some(id);
            mass_kg[i] = mass;
            film_g[i] = g_bore;
        }

        // Shear goes where the screw is actually working the polymer.
        let mut shear_weight: Vec<f64> = order
            .iter()
            .map(|&i| geometry::overlap(cells[i].x0_mm, cells[i].x1_mm, SCREW_X0_MM, FEED_X_MM))
            .collect();
        let total: f64 = shear_weight.iter().sum();
        if total > 0.0 {
            for w in &mut shear_weight {
                *w /= total;
            }
        }

        // Advection: in from the feed boundary, along the chain, out at the die.
        // Rates start at zero — a stopped screw — and are set by `refresh_melt`.
        let datum_c = melt.feed_c;
        let mut flows = Vec::with_capacity(order.len() + 1);
        let mut flow_source = Vec::with_capacity(order.len() + 1);
        let mut source = FlowTerminal::Boundary(melt.feed_c);
        let mut previous: Option<usize> = None;
        for &i in &order {
            let target = FlowTerminal::Node(nodes[i].expect("just created"));
            flows.push(net.add_flow(Flow {
                source,
                target,
                w_per_k: 0.0,
                datum_c,
            }));
            flow_source.push(previous);
            source = target;
            previous = Some(i);
        }
        flows.push(net.add_flow(Flow {
            source,
            target: FlowTerminal::Boundary(melt.feed_c),
            w_per_k: 0.0,
            datum_c,
        }));
        flow_source.push(previous);

        MeltChain {
            nodes,
            mass_kg,
            film_g,
            order,
            flows,
            flow_source,
            shear_weight,
            datum_c,
        }
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

    /// Set the screw speed in rpm, which is what sets throughput.
    ///
    /// Throughput is linear in speed at
    /// [`super::params::MeltParams::kg_per_h_per_rpm`]: the machine's maximum 100 rpm is
    /// 10 kg/h. Shear power follows from it too. A model built without the melt
    /// ignores this — check with [`Self::is_extruding`] rather than assuming it
    /// took.
    ///
    /// Re-rates the existing advection edges; it never rebuilds the network, so
    /// every `NodeId` stays valid and a speed change costs a handful of writes.
    pub fn set_screw_rpm(&mut self, rpm: f64) {
        self.screw_rpm = rpm.max(0.0);
        self.mass_flow_kg_s = self.params.melt.mass_flow_kg_per_s(self.screw_rpm);
        self.shear_power_w = self.params.melt.shear_power_w(self.screw_rpm);
        self.refresh_melt();
    }

    /// Screw speed in rpm.
    pub const fn screw_rpm(&self) -> f64 {
        self.screw_rpm
    }

    /// Throughput in kg/h.
    pub fn throughput_kg_per_h(&self) -> f64 {
        self.mass_flow_kg_s * 3600.0
    }

    /// Shear power currently going into the melt, in W, after the drive cap.
    pub const fn shear_power_w(&self) -> f64 {
        self.shear_power_w
    }

    /// Whether the drive cap is holding shear power below what the specific
    /// mechanical energy asks for. A run that silently saturates is worth
    /// noticing rather than reading as physics.
    pub fn shear_power_is_capped(&self) -> bool {
        let uncapped =
            self.params.melt.specific_mechanical_energy_kwh_per_kg * 3.6e6 * self.mass_flow_kg_s;
        uncapped > self.params.melt.max_shear_power_w + 1e-9
    }

    /// Whether this model has a melt chain at all.
    pub const fn melt_is_modelled(&self) -> bool {
        self.melt.is_some()
    }

    /// Whether this model has a melt chain and material is moving through it.
    pub fn is_extruding(&self) -> bool {
        self.melt.is_some() && self.mass_flow_kg_s > 0.0
    }

    /// Melt temperature in the cell nearest `x_mm`, in °C.
    pub fn melt_c_at(&self, x_mm: f64) -> Option<f64> {
        let melt = self.melt.as_ref()?;
        let i = *melt.order.iter().min_by(|a, b| {
            (self.cells[**a].centre_mm() - x_mm)
                .abs()
                .total_cmp(&(self.cells[**b].centre_mm() - x_mm).abs())
        })?;
        melt.nodes[i].map(|id| self.net.temperature(id))
    }

    /// Temperature of the melt as it leaves the die, in °C.
    pub fn melt_out_c(&self) -> Option<f64> {
        let melt = self.melt.as_ref()?;
        let last = *melt.order.last()?;
        melt.nodes[last].map(|id| self.net.temperature(id))
    }

    /// Net enthalpy the material carries out of the machine, in W.
    ///
    /// Positive means the polymer is taking heat away — what the bands have to
    /// make up. Shear heat is *not* netted off here; it enters the melt as node
    /// power and shows up separately in [`Self::shear_power_w`].
    pub fn melt_extraction_w(&self) -> f64 {
        self.net.net_flow_out_w()
    }

    /// Heat the melt is drawing from one zone's barrel steel, in W.
    ///
    /// Attributed by which cells lie under the zone's band, so the unbanded
    /// stretches — the bare tail and the gap between bands — belong to no zone
    /// and are left out of all four.
    pub fn melt_load_w(&self, zone: Zone) -> f64 {
        let Some(melt) = self.melt.as_ref() else {
            return 0.0;
        };
        let band = zone.band();
        melt.order
            .iter()
            .filter(|&&i| {
                geometry::overlap(
                    self.cells[i].x0_mm,
                    self.cells[i].x1_mm,
                    band.x0_mm,
                    band.x1_mm,
                ) > 0.0
            })
            .filter_map(|&i| {
                melt.nodes[i].map(|id| {
                    melt.film_g[i]
                        * (self.net.temperature(self.cells[i].id) - self.net.temperature(id))
                })
            })
            .sum()
    }

    /// Total polymer in the machine, in kg.
    pub fn polymer_mass_kg(&self) -> f64 {
        self.melt.as_ref().map_or(0.0, |m| m.mass_kg.iter().sum())
    }

    /// Bring the melt nodes into line with their own temperatures.
    ///
    /// Two things move as the polymer heats, and they want *different* specific
    /// heats — see [`super::params::MeltParams::secant_cp_j_per_kg_k`]:
    ///
    /// - a node's heat capacity uses the tangent `dh/dT`, which is what makes
    ///   `dT/dt = Q/(m·dh/dT)` right;
    /// - an advection edge's rate uses the secant against the datum, which is
    ///   what makes it carry exact enthalpy rather than `cp·T`.
    ///
    /// Cheap: two piecewise evaluations per cell, against a network of a
    /// hundred-odd nodes and several hundred edges.
    fn refresh_melt(&mut self) {
        let Some(melt) = self.melt.as_ref() else {
            return;
        };
        let params = &self.params.melt;

        for &i in &melt.order {
            let Some(id) = melt.nodes[i] else { continue };
            let t = self.net.temperature(id);
            self.net.node_mut(id).capacity_j_per_k =
                melt.mass_kg[i] * params.apparent_cp_j_per_kg_k(t);
        }

        for (slot, &flow) in melt.flows.iter().enumerate() {
            let source_c = match melt.flow_source[slot] {
                Some(i) => melt.nodes[i].map_or(melt.datum_c, |id| self.net.temperature(id)),
                None => params.feed_c,
            };
            let w = self.mass_flow_kg_s * params.secant_cp_j_per_kg_k(source_c, melt.datum_c);
            self.net.set_flow_w(flow, w);
        }

        for (slot, &i) in melt.order.iter().enumerate() {
            if let Some(id) = melt.nodes[i] {
                self.net
                    .set_power(id, self.shear_power_w * melt.shear_weight[slot]);
            }
        }
    }

    /// Advance the plant by `dt` seconds.
    pub fn step(&mut self, dt: f64) {
        self.refresh_melt();
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

    fn melt_model() -> ExtruderThermalModel {
        let mut p = ExtruderThermalParams::default();
        p.melt.enabled = true;
        ExtruderThermalModel::new(p)
    }

    /// The default model must be exactly the one the heat-up recording was
    /// fitted against. If enabling the melt by accident ever became the default,
    /// every calibration test in `harness` would move with it.
    #[test]
    fn the_melt_is_off_unless_it_is_asked_for() {
        let plain = model();
        assert!(plain.melt.is_none());
        assert_eq!(plain.network().flow_count(), 0);
        assert!(plain.polymer_mass_kg().abs() < f64::EPSILON);
        assert!(!plain.is_extruding());

        // And enabling it really does add the chain.
        let with_melt = melt_model();
        assert!(with_melt.network().flow_count() > 40);
        assert!(with_melt.polymer_mass_kg() > 0.2);
    }

    /// Making room for the polymer takes steel out of the bore and puts polymer
    /// in. The two nearly cancel — polymer holds about four times the heat per
    /// kilogram — which is why the existing calibration survives roughly intact.
    #[test]
    fn splitting_the_bore_barely_changes_its_heat_capacity() {
        let capacity = |m: &ExtruderThermalModel, prefix: &str| -> f64 {
            m.network()
                .nodes()
                .iter()
                .filter(|n| n.label.starts_with(prefix))
                .map(|n| n.capacity_j_per_k)
                .sum()
        };
        let solid = capacity(&model(), "screw");
        let m = melt_model();
        let split = capacity(&m, "screw") + capacity(&m, "melt");

        assert!(
            (split - solid).abs() < 0.25 * solid,
            "bore heat capacity went from {solid:.0} to {split:.0} J/K; \
             more than a quarter would put the calibration in doubt"
        );
        // The steel really did get lighter, even so.
        assert!(capacity(&m, "screw") < 0.75 * solid);
    }

    /// Material runs back to middle to front to nozzle, which is the reverse of
    /// `Zone::port` ordering. Assert it in zone terms so a geometry edit cannot
    /// silently turn the machine around.
    #[test]
    fn the_melt_flows_back_to_middle_to_front_to_nozzle() {
        let m = melt_model();
        let melt = m.melt.as_ref().expect("melt enabled");

        // Strictly descending x, feed to die.
        for w in melt.order.windows(2) {
            assert!(m.cells[w[0]].centre_mm() > m.cells[w[1]].centre_mm());
        }

        // The first cell each zone's band sees, in flow order.
        let first_of = |zone: Zone| {
            let b = zone.band();
            melt.order
                .iter()
                .position(|&i| {
                    geometry::overlap(m.cells[i].x0_mm, m.cells[i].x1_mm, b.x0_mm, b.x1_mm) > 0.0
                })
                .expect("every zone has melt under it")
        };
        let (back, middle, front, nozzle) = (
            first_of(Zone::Back),
            first_of(Zone::Middle),
            first_of(Zone::Front),
            first_of(Zone::Nozzle),
        );
        assert!(
            back < middle && middle < front && front < nozzle,
            "flow order came out back {back}, middle {middle}, front {front}, nozzle {nozzle}"
        );
    }

    /// A stopped screw must be inert: no transport, no shear, nothing moving.
    ///
    /// Ambient is put at the soak temperature so that the ordinary heat loss
    /// cannot be mistaken for the melt doing something.
    #[test]
    fn a_stopped_screw_transports_nothing() {
        let mut p = ExtruderThermalParams::default();
        p.melt.enabled = true;
        p.ambient_c = 180.0;
        let mut m = ExtruderThermalModel::new(p);
        m.set_uniform_temperature(180.0);
        m.set_screw_rpm(0.0);
        for _ in 0..10_000 {
            m.step(0.01);
        }
        assert!(!m.is_extruding());
        assert_relative_eq!(m.melt_extraction_w(), 0.0, epsilon = 1e-9);
        assert_relative_eq!(m.melt_out_c().expect("melt enabled"), 180.0, epsilon = 1e-6);
        for zone in Zone::ALL {
            assert_relative_eq!(m.melt_load_w(zone), 0.0, epsilon = 1e-9);
        }
    }

    #[test]
    fn throughput_is_linear_in_screw_speed() {
        let mut m = melt_model();
        m.set_screw_rpm(100.0);
        assert_relative_eq!(m.throughput_kg_per_h(), 10.0, max_relative = 1e-9);
        m.set_screw_rpm(50.0);
        assert_relative_eq!(m.throughput_kg_per_h(), 5.0, max_relative = 1e-9);
        m.set_screw_rpm(10.0);
        assert_relative_eq!(m.throughput_kg_per_h(), 1.0, max_relative = 1e-9);
    }

    /// The advection edges must carry exactly the enthalpy the material gained,
    /// `m_dot * (h(T_out) - h(T_feed))`. This is the test that fails if the
    /// tangent specific heat is ever used where the secant belongs.
    #[test]
    fn the_extraction_is_the_enthalpy_the_material_leaves_with() {
        let mut m = melt_model();
        m.set_uniform_temperature(190.0);
        m.set_screw_rpm(60.0);
        for _ in 0..60_000 {
            m.step(0.01);
        }

        let melt = &m.params.melt;
        let out_c = m.melt_out_c().expect("melt enabled");
        let expected = melt.mass_flow_kg_per_s(60.0)
            * (melt.specific_enthalpy_j_per_kg(out_c)
                - melt.specific_enthalpy_j_per_kg(melt.feed_c));
        assert_relative_eq!(m.melt_extraction_w(), expected, max_relative = 1e-6);
    }

    /// Extrusion is a large load. At the rated 10 kg/h the material carries off
    /// a serious fraction of the 2300 W of installed band heating — which is the
    /// whole reason for modelling it.
    #[test]
    fn the_extraction_load_is_a_large_fraction_of_installed_power() {
        let mut m = melt_model();
        m.set_uniform_temperature(190.0);
        m.set_screw_rpm(100.0);
        // Hold the barrel hot: the point is the steady demand, not the sag.
        for _ in 0..30_000 {
            for zone in Zone::ALL {
                m.set_band_power(zone, zone.rated_w());
            }
            m.step(0.01);
        }
        let installed: f64 = Zone::ALL.iter().map(|z| z.rated_w()).sum();
        let load = m.melt_extraction_w();
        assert!(
            (0.25 * installed..1.1 * installed).contains(&load),
            "extraction {load:.0} W against {installed:.0} W installed is not \
             the order the hand calculation gives (~990 W)"
        );
    }

    /// Cold material entering pulls the barrel down — with the screw's own
    /// mechanical work set aside, so this is the melting load alone. See
    /// [`extrusion_is_nearly_thermally_neutral_at_the_default_shear`] for what
    /// happens once shear is put back.
    #[test]
    fn cold_material_pulls_the_zones_down() {
        let run = |rpm: f64| {
            let mut p = ExtruderThermalParams::default();
            p.melt.enabled = true;
            p.melt.specific_mechanical_energy_kwh_per_kg = 0.0;
            let mut m = ExtruderThermalModel::new(p);
            m.set_uniform_temperature(190.0);
            m.set_screw_rpm(rpm);
            for _ in 0..60_000 {
                for zone in [Zone::Front, Zone::Middle, Zone::Back] {
                    m.set_band_power(zone, 350.0);
                }
                m.step(0.01);
            }
            m
        };
        let idle = run(0.0);
        let extruding = run(60.0);
        for zone in [Zone::Front, Zone::Middle, Zone::Back] {
            assert!(
                extruding.steel_c(zone) < idle.steel_c(zone) - 1.0,
                "{} zone: {:.1} C extruding vs {:.1} C idle",
                zone.name(),
                extruding.steel_c(zone),
                idle.steel_c(zone)
            );
        }
    }

    /// **The finding this model exists to surface.** At the default specific
    /// mechanical energy the screw's own work very nearly pays for melting the
    /// polymer, so starting extrusion is a much smaller net load than the
    /// ~990 W of enthalpy leaving the die suggests — and locally, near the
    /// front, it can even heat.
    ///
    /// This near-cancellation rests entirely on
    /// [`super::params::MeltParams::specific_mechanical_energy_kwh_per_kg`],
    /// which has never been measured on this machine and is uncertain by a
    /// factor of two either way. The test asserts the cancellation is real at
    /// the default, not that the machine behaves this way.
    #[test]
    fn extrusion_is_nearly_thermally_neutral_at_the_default_shear() {
        let mut m = melt_model();
        m.set_uniform_temperature(190.0);
        m.set_screw_rpm(100.0);
        for _ in 0..60_000 {
            for zone in Zone::ALL {
                m.set_band_power(zone, zone.rated_w());
            }
            m.step(0.01);
        }
        let installed: f64 = Zone::ALL.iter().map(|z| z.rated_w()).sum();
        let net = m.melt_extraction_w() - m.shear_power_w();
        assert!(
            net.abs() < 0.35 * installed,
            "net load {net:.0} W (extraction {:.0} minus shear {:.0}) against \
             {installed:.0} W installed — if this has drifted far from neutral, \
             the specific mechanical energy or the throughput constant changed",
            m.melt_extraction_w(),
            m.shear_power_w()
        );
    }

    /// The back zone meets the coldest material, so it does the most of the
    /// melting work; by the nozzle the polymer is nearly up to temperature.
    /// A direct consequence of the flow direction.
    #[test]
    fn the_back_zone_carries_most_of_the_melting_load() {
        let mut m = melt_model();
        m.set_uniform_temperature(190.0);
        m.set_screw_rpm(60.0);
        for _ in 0..60_000 {
            for zone in Zone::ALL {
                m.set_band_power(zone, zone.rated_w());
            }
            m.step(0.01);
        }
        let (back, middle, front) = (
            m.melt_load_w(Zone::Back),
            m.melt_load_w(Zone::Middle),
            m.melt_load_w(Zone::Front),
        );
        assert!(
            back > middle && middle > front,
            "melt load back {back:.0} W, middle {middle:.0} W, front {front:.0} W \
             should fall in flow order"
        );
    }

    /// The material gets hotter the further it travels.
    #[test]
    fn the_melt_arrives_hotter_at_each_zone_downstream() {
        let mut m = melt_model();
        m.set_uniform_temperature(190.0);
        m.set_screw_rpm(60.0);
        for _ in 0..60_000 {
            for zone in Zone::ALL {
                m.set_band_power(zone, zone.rated_w());
            }
            m.step(0.01);
        }
        let at = |zone: Zone| m.melt_c_at(zone.band().centre_mm()).expect("melt enabled");
        let (back, middle, front) = (at(Zone::Back), at(Zone::Middle), at(Zone::Front));
        assert!(
            back < middle && middle < front,
            "melt temperature back {back:.1}, middle {middle:.1}, front {front:.1} \
             should rise along the flow"
        );
    }

    /// Shear heat opposes the melting load, and at the rated throughput the two
    /// are the same order. Whether extruding cools the barrel at all depends on
    /// a coefficient nobody has measured, which is why it is a parameter.
    #[test]
    fn shear_heating_offsets_part_of_the_melting_load() {
        let net_demand = |sme: f64| {
            let mut p = ExtruderThermalParams::default();
            p.melt.enabled = true;
            p.melt.specific_mechanical_energy_kwh_per_kg = sme;
            let mut m = ExtruderThermalModel::new(p);
            m.set_uniform_temperature(190.0);
            m.set_screw_rpm(60.0);
            for _ in 0..60_000 {
                for zone in Zone::ALL {
                    m.set_band_power(zone, zone.rated_w());
                }
                m.step(0.01);
            }
            m.melt_extraction_w() - m.shear_power_w()
        };
        assert!(
            net_demand(0.20) < net_demand(0.0) - 100.0,
            "raising the specific mechanical energy must reduce what the \
             heaters have to supply"
        );
    }

    #[test]
    fn shear_power_is_capped_at_the_drive_rating() {
        let mut p = ExtruderThermalParams::default();
        p.melt.enabled = true;
        p.melt.specific_mechanical_energy_kwh_per_kg = 0.25;
        let mut m = ExtruderThermalModel::new(p);
        m.set_screw_rpm(100.0);
        assert!(m.shear_power_is_capped());
        assert_relative_eq!(
            m.shear_power_w(),
            m.params.melt.max_shear_power_w,
            max_relative = 1e-9
        );
    }

    /// The melt cells are small and advection is Courant limited, so check the
    /// margin at the worst case: maximum throughput and a hot machine.
    #[test]
    fn is_stable_at_the_harness_time_step_while_extruding() {
        let mut m = melt_model();
        m.set_uniform_temperature(300.0);
        m.set_screw_rpm(100.0);
        let limit = m.max_stable_dt();
        assert!(
            limit > 10.0 * super::super::harness::DT_PLANT_S,
            "stability limit {limit:.3} s leaves too little margin over the \
             {:.3} s plant step at full throughput",
            super::super::harness::DT_PLANT_S
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
