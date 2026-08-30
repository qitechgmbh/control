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
