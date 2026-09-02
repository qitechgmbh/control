//! A generic lumped-capacitance ("thermal RC network") solver.
//!
//! The network is a set of [`Node`]s (thermal masses) joined by [`Conductance`]s
//! (conduction paths), with optional [`AmbientLoss`] paths to the surrounding
//! air. [`ThermalNetwork::step`] advances it with explicit Euler.
//!
//! Explicit Euler is chosen deliberately: the networks this is built for are
//! small (tens of nodes) and their stiffest time constant is seconds, so a
//! 10 ms step is stable with a wide margin while staying fast enough to
//! simulate hours of machine behaviour in under a second. Check the margin with
//! [`ThermalNetwork::max_stable_dt`] rather than assuming it.
//!
//! Nothing here is machine specific — see
//! `machine_implementations::extruder1::simulation` for the extruder model built
//! on top of it.

pub mod loss;
pub mod node;

pub use loss::AmbientLoss;
pub use node::{Conductance, Node, NodeId};

/// A network of thermal masses.
#[derive(Debug, Clone)]
pub struct ThermalNetwork {
    nodes: Vec<Node>,
    conductances: Vec<Conductance>,
    losses: Vec<(NodeId, AmbientLoss)>,
    /// Air temperature in °C.
    pub ambient_c: f64,
    /// Scratch buffer for per-step net power, kept to avoid allocating in the
    /// hot loop.
    flux: Vec<f64>,
}

impl ThermalNetwork {
    pub const fn new(ambient_c: f64) -> Self {
        Self {
            nodes: Vec::new(),
            conductances: Vec::new(),
            losses: Vec::new(),
            ambient_c,
            flux: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) -> NodeId {
        self.nodes.push(node);
        self.flux.push(0.0);
        NodeId(self.nodes.len() - 1)
    }

    /// Join two nodes with a conduction path of `g_w_per_k` W/K.
    ///
    /// A non-positive conductance is silently ignored, which lets callers write
    /// `connect(a, b, h * area)` without special-casing a disabled path.
    pub fn connect(&mut self, a: NodeId, b: NodeId, g_w_per_k: f64) {
        if g_w_per_k <= 0.0 {
            return;
        }
        debug_assert!(a != b, "cannot connect a node to itself");
        self.conductances.push(Conductance { a, b, g_w_per_k });
    }

    /// Attach a heat loss path from `node` to ambient.
    pub fn add_loss(&mut self, node: NodeId, loss: AmbientLoss) {
        self.losses.push((node, loss));
    }

    pub fn node(&self, id: NodeId) -> &Node {
        &self.nodes[id.0]
    }

    pub fn node_mut(&mut self, id: NodeId) -> &mut Node {
        &mut self.nodes[id.0]
    }

    pub fn temperature(&self, id: NodeId) -> f64 {
        self.nodes[id.0].temperature_c
    }

    pub fn set_temperature(&mut self, id: NodeId, temperature_c: f64) {
        self.nodes[id.0].temperature_c = temperature_c;
    }

    /// Set the externally injected power of a node in W.
    pub fn set_power(&mut self, id: NodeId, power_w: f64) {
        self.nodes[id.0].power_w = power_w;
    }

    pub const fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    /// Total thermal energy stored relative to `ambient_c`, in J.
    ///
    /// Useful as a conservation check in tests.
    pub fn stored_energy_j(&self) -> f64 {
        self.nodes
            .iter()
            .map(|n| n.capacity_j_per_k * (n.temperature_c - self.ambient_c))
            .sum()
    }

    /// Largest step size that keeps explicit Euler stable, in seconds.
    ///
    /// For each node this is `2 * C / sum(G)` over every conductance and loss
    /// path touching it; the network limit is the minimum. Losses are nonlinear,
    /// so they are evaluated at the network's current temperatures — call this
    /// at the hottest state you expect, not while everything is still cold.
    pub fn max_stable_dt(&self) -> f64 {
        let mut sum_g = vec![0.0_f64; self.nodes.len()];
        for c in &self.conductances {
            sum_g[c.a.0] += c.g_w_per_k;
            sum_g[c.b.0] += c.g_w_per_k;
        }
        for (id, loss) in &self.losses {
            sum_g[id.0] += loss.conductance_w_per_k(self.nodes[id.0].temperature_c, self.ambient_c);
        }
        sum_g
            .iter()
            .zip(&self.nodes)
            .map(|(g, n)| {
                if *g <= 0.0 {
                    f64::INFINITY
                } else {
                    2.0 * n.capacity_j_per_k / g
                }
            })
            .fold(f64::INFINITY, f64::min)
    }

    /// Advance the network by `dt` seconds using explicit Euler.
    ///
    /// Node powers are *not* cleared: set them when they change and they persist.
    pub fn step(&mut self, dt: f64) {
        debug_assert!(dt > 0.0, "step size must be positive");

        for (f, n) in self.flux.iter_mut().zip(&self.nodes) {
            *f = n.power_w;
        }

        for c in &self.conductances {
            let q =
                c.g_w_per_k * (self.nodes[c.a.0].temperature_c - self.nodes[c.b.0].temperature_c);
            self.flux[c.a.0] -= q;
            self.flux[c.b.0] += q;
        }

        for (id, loss) in &self.losses {
            self.flux[id.0] -= loss.heat_flow_w(self.nodes[id.0].temperature_c, self.ambient_c);
        }

        for (n, f) in self.nodes.iter_mut().zip(&self.flux) {
            n.temperature_c += f * dt / n.capacity_j_per_k;
        }
    }

    /// Advance by `dt` seconds, internally sub-stepping so that no sub-step
    /// exceeds `max_sub_dt`.
    ///
    /// Use this when the caller's natural cadence (say a 500 ms PWM window) is
    /// coarser than the network's stability limit.
    pub fn step_substepped(&mut self, dt: f64, max_sub_dt: f64) {
        debug_assert!(max_sub_dt > 0.0);
        let steps = (dt / max_sub_dt).ceil().max(1.0);
        let sub = dt / steps;
        for _ in 0..(steps as usize) {
            self.step(sub);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    /// A single node with no losses and no heating must not change temperature.
    #[test]
    fn isolated_node_holds_temperature() {
        let mut net = ThermalNetwork::new(20.0);
        let n = net.add_node(Node::new("a", 1000.0, 150.0));
        for _ in 0..1000 {
            net.step(0.01);
        }
        assert_relative_eq!(net.temperature(n), 150.0, epsilon = 1e-9);
    }

    /// Two connected nodes with no external paths must conserve energy exactly
    /// and equalise at the capacity-weighted mean.
    #[test]
    fn two_node_exchange_conserves_energy() {
        let mut net = ThermalNetwork::new(0.0);
        let a = net.add_node(Node::new("a", 1000.0, 200.0));
        let b = net.add_node(Node::new("b", 3000.0, 0.0));
        net.connect(a, b, 5.0);

        // tau = 1/(1/1000 + 1/3000) / 5 = 150 s; run ~30 tau.
        let e0 = net.stored_energy_j();
        for _ in 0..9_000 {
            net.step(0.5);
        }
        let e1 = net.stored_energy_j();
        assert_relative_eq!(e0, e1, max_relative = 1e-9);

        // Weighted mean: (1000*200 + 3000*0) / 4000 = 50
        assert_relative_eq!(net.temperature(a), 50.0, epsilon = 1e-6);
        assert_relative_eq!(net.temperature(b), 50.0, epsilon = 1e-6);
    }

    /// A single node behind a linear conductance to ambient is a first-order
    /// lag: after one time constant it must have covered 1 - 1/e of the step.
    #[test]
    fn single_pole_step_response_matches_analytic() {
        // C = 500 J/K, G = 5 W/K -> tau = 100 s. 500 W in -> steady state
        // 20 + 500/5 = 120 C.
        let mut net = ThermalNetwork::new(20.0);
        let sink = net.add_node(Node::new("ambient_anchor", 1e12, 20.0));
        let a = net.add_node(Node::new("a", 500.0, 20.0));
        net.connect(a, sink, 5.0);
        net.set_power(a, 500.0);

        let tau = 100.0;
        let dt = 0.01;
        for _ in 0..((tau / dt) as usize) {
            net.step(dt);
        }
        let expected = 100.0f64.mul_add(1.0 - (-1.0_f64).exp(), 20.0);
        assert_relative_eq!(net.temperature(a), expected, max_relative = 1e-3);

        for _ in 0..((10.0 * tau / dt) as usize) {
            net.step(dt);
        }
        assert_relative_eq!(net.temperature(a), 120.0, max_relative = 1e-4);
    }

    /// Steady state with a nonlinear ambient loss must balance power in against
    /// heat out.
    #[test]
    fn steady_state_balances_power_against_loss() {
        let mut net = ThermalNetwork::new(20.0);
        let loss = AmbientLoss::Bare {
            area_m2: 0.05,
            convection_coeff: 2.6,
            emissivity: 0.3,
        };
        let a = net.add_node(Node::new("a", 2000.0, 20.0));
        net.add_loss(a, loss);
        net.set_power(a, 40.0);

        // tau = C/G ~ 2000/0.5 = 4000 s; run ~25 tau. dt = 1 s is far inside the
        // stability limit (2C/G = 8000 s).
        for _ in 0..100_000 {
            net.step(1.0);
        }
        let t = net.temperature(a);
        assert_relative_eq!(loss.heat_flow_w(t, 20.0), 40.0, max_relative = 1e-4);
    }

    #[test]
    fn max_stable_dt_reflects_the_tightest_node() {
        let mut net = ThermalNetwork::new(20.0);
        let big = net.add_node(Node::new("big", 10_000.0, 20.0));
        let small = net.add_node(Node::new("small", 100.0, 20.0));
        net.connect(big, small, 50.0);
        // small: 2 * 100 / 50 = 4 s; big: 2 * 10000 / 50 = 400 s
        assert_relative_eq!(net.max_stable_dt(), 4.0, max_relative = 1e-9);
    }

    #[test]
    fn substepping_matches_direct_stepping() {
        let build = || {
            let mut net = ThermalNetwork::new(20.0);
            let a = net.add_node(Node::new("a", 500.0, 20.0));
            let b = net.add_node(Node::new("b", 500.0, 200.0));
            net.connect(a, b, 5.0);
            (net, a)
        };
        let (mut direct, da) = build();
        for _ in 0..100 {
            direct.step(0.01);
        }
        let (mut subbed, sa) = build();
        subbed.step_substepped(1.0, 0.01);
        assert_relative_eq!(
            direct.temperature(da),
            subbed.temperature(sa),
            max_relative = 1e-12
        );
    }
}
