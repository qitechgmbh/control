/// Index of a node inside a [`super::ThermalNetwork`].
///
/// Returned by [`super::ThermalNetwork::add_node`] and used to wire conductances,
/// losses and power injections without juggling raw `usize`s.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(pub usize);

/// A lumped thermal mass.
///
/// One node is one isothermal chunk of matter: a slice of barrel steel, a heater
/// band, the sensing tip of an RTD. It stores energy (`capacity_j_per_k`) and
/// exchanges it with other nodes through [`Conductance`]s and with the environment
/// through [`super::AmbientLoss`].
#[derive(Debug, Clone)]
pub struct Node {
    /// Human readable name, e.g. `"barrel[12]"`. Only used for debugging output.
    pub label: String,
    /// Heat capacity `m * cp` in J/K.
    pub capacity_j_per_k: f64,
    /// Current temperature in °C.
    pub temperature_c: f64,
    /// Externally injected power in W (electrical heating, shear heating, ...).
    ///
    /// This is *held* between steps: set it once when it changes, it is not
    /// cleared by [`super::ThermalNetwork::step`].
    pub power_w: f64,
}

impl Node {
    pub fn new(label: impl Into<String>, capacity_j_per_k: f64, temperature_c: f64) -> Self {
        debug_assert!(
            capacity_j_per_k > 0.0,
            "node heat capacity must be positive"
        );
        Self {
            label: label.into(),
            capacity_j_per_k,
            temperature_c,
            power_w: 0.0,
        }
    }
}

/// A conduction path between two nodes.
///
/// `g_w_per_k` is the total conductance of the path in W/K. For solid conduction
/// over a length `l` with cross section `a` and conductivity `k` that is
/// `k * a / l`; for a contact interface with contact coefficient `h` it is
/// `h * a`.
#[derive(Debug, Clone, Copy)]
pub struct Conductance {
    pub a: NodeId,
    pub b: NodeId,
    pub g_w_per_k: f64,
}

/// Index of a [`Flow`] inside a [`super::ThermalNetwork`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FlowId(pub usize);

/// One end of a [`Flow`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlowTerminal {
    /// A node inside the network.
    Node(NodeId),
    /// Outside the modelled domain. As a source it is the inlet, held at this
    /// temperature; as a target it is the outlet, and the temperature is unused
    /// because the enthalpy simply leaves.
    Boundary(f64),
}

/// Enthalpy carried by a moving stream from one terminal to another.
///
/// Unlike a [`Conductance`], this is **not symmetric**: the stream carries the
/// *upstream* terminal's enthalpy, so only the source temperature appears in the
/// flux even though both ends are charged for it. That asymmetry is what makes
/// it advection rather than conduction, and is why it cannot be a conductance
/// with a clever coefficient.
///
/// # Why the datum
///
/// The edge transports `w_per_k * (T_source - datum_c)`, debited from the source
/// and credited to the target. Along a chain those terms telescope: an interior
/// node nets out to `w * (T_upstream - T_own)`, which is the balance you want,
/// while the datum cancels. Its two jobs are at the ends of the chain and in the
/// choice of `w_per_k`:
///
/// - A machine sitting uniformly at `datum_c` transports exactly zero, so an
///   idle network cannot drift.
/// - When the stream's specific heat varies with temperature — a polymer melting,
///   say — set `w_per_k` to `m_dot * (h(T_source) - h(datum)) / (T_source - datum)`,
///   the *secant* specific heat. The edge then carries exactly
///   `m_dot * (h(T_source) - h(datum))`, so a chain moves exactly
///   `m_dot * (h_upstream - h_own)` per node: true enthalpy transport, with no
///   leak across a phase change. Using the tangent `dh/dT` here instead would
///   advect `cp*T` rather than `h`, which is only the same thing when `cp` is
///   constant.
///
/// Note the asymmetry with node heat capacity, which wants the *tangent*
/// `dh/dT` — that is what makes `dT/dt = q / (m * dh/dT)` right.
///
/// # Energy
///
/// A chain of flows does **not** conserve energy: enthalpy enters at the head
/// and leaves at the tail. That is the intended physics of an open system such
/// as material passing through an extruder; see
/// [`super::ThermalNetwork::stored_energy_j`] and
/// [`super::ThermalNetwork::net_flow_out_w`].
#[derive(Debug, Clone, Copy)]
pub struct Flow {
    /// Where the material comes from.
    pub source: FlowTerminal,
    /// Where it goes.
    pub target: FlowTerminal,
    /// Capacity rate `m_dot * cp` in W/K — see the note on the datum above for
    /// which `cp` when it is not constant.
    pub w_per_k: f64,
    /// Enthalpy datum in °C.
    pub datum_c: f64,
}
