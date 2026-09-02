//! A generic lumped-capacitance ("thermal RC network") solver.
//!
//! The network is a set of [`Node`]s (thermal masses) joined by [`Conductance`]s
//! (conduction paths), with optional [`AmbientLoss`] paths to the surrounding
//! air. [`ThermalNetwork::step`] advances it with explicit Euler.
//!
//! [`Flow`] adds the one asymmetric edge: enthalpy carried by material moving
//! through the network, which makes it an *open* system — energy enters at the
//! head of a flow chain and leaves at its tail. Everything else conserves.
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
pub use node::{Conductance, Flow, FlowId, FlowTerminal, Node, NodeId};

/// A network of thermal masses.
#[derive(Debug, Clone)]
pub struct ThermalNetwork {
    nodes: Vec<Node>,
    conductances: Vec<Conductance>,
    flows: Vec<Flow>,
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
            flows: Vec::new(),
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

    /// Add a mass-transport path. See [`Flow`] for the transported quantity and
    /// what the datum is for.
    ///
    /// Unlike [`Self::connect`], a zero-rate flow is still recorded rather than
    /// dropped: a stopped machine is a normal state, and the edge has to survive
    /// to be re-rated by [`Self::set_flow_w`] when it starts.
    pub fn add_flow(&mut self, flow: Flow) -> FlowId {
        debug_assert!(flow.w_per_k >= 0.0, "flow rate must not be negative");
        self.flows.push(flow);
        FlowId(self.flows.len() - 1)
    }

    /// Change the capacity rate of an existing flow, in W/K.
    pub fn set_flow_w(&mut self, id: FlowId, w_per_k: f64) {
        debug_assert!(w_per_k >= 0.0, "flow rate must not be negative");
        self.flows[id.0].w_per_k = w_per_k;
    }

    /// Heat currently carried by one flow, in W.
    pub fn flow_heat_w(&self, id: FlowId) -> f64 {
        let f = &self.flows[id.0];
        f.w_per_k * (self.terminal_c(f.source) - f.datum_c)
    }

    /// Net enthalpy leaving the network through flow boundaries, in W.
    ///
    /// The term missing from a conservation check: over a step,
    /// `d(stored_energy_j) = (sum of node powers - losses - net_flow_out_w) * dt`.
    pub fn net_flow_out_w(&self) -> f64 {
        self.flows
            .iter()
            .map(|f| {
                let q = f.w_per_k * (self.terminal_c(f.source) - f.datum_c);
                match (f.source, f.target) {
                    (FlowTerminal::Boundary(_), FlowTerminal::Node(_)) => -q,
                    (FlowTerminal::Node(_), FlowTerminal::Boundary(_)) => q,
                    _ => 0.0,
                }
            })
            .sum()
    }

    pub const fn flow_count(&self) -> usize {
        self.flows.len()
    }

    fn terminal_c(&self, t: FlowTerminal) -> f64 {
        match t {
            FlowTerminal::Node(id) => self.nodes[id.0].temperature_c,
            FlowTerminal::Boundary(c) => c,
        }
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
    /// Useful as a conservation check in tests — but only for a network with no
    /// [`Flow`]s. Flows are an open boundary: enthalpy enters at the head of a
    /// chain and leaves at its tail, so this quantity is not expected to be
    /// conserved once any flow is carrying material.
    pub fn stored_energy_j(&self) -> f64 {
        self.nodes
            .iter()
            .map(|n| n.capacity_j_per_k * (n.temperature_c - self.ambient_c))
            .sum()
    }

    /// Largest step size that keeps explicit Euler stable, in seconds.
    ///
    /// For each node this is `2 * C / sum(G)` over every conductance, flow and
    /// loss path touching it; the network limit is the minimum. Losses are
    /// nonlinear, so they are evaluated at the network's current temperatures —
    /// call this at the hottest state you expect, not while everything is still
    /// cold. Flows are rate dependent, so call it at the highest rate you expect
    /// too.
    pub fn max_stable_dt(&self) -> f64 {
        let mut sum_g = vec![0.0_f64; self.nodes.len()];
        for c in &self.conductances {
            sum_g[c.a.0] += c.g_w_per_k;
            sum_g[c.b.0] += c.g_w_per_k;
        }
        // A flow's flux depends on its *source's* temperature with slope `-w`
        // (the source is debited what it sends), and not at all on its target's,
        // so it loads only the sending node.
        //
        // The factor of two reports the Courant limit `dt <= C / (m_dot * cp)`
        // rather than diffusion's `2C/G`, because advection needs the tighter
        // one to be *usable*, not merely convergent. Writing the update as
        // `T_i += v * (T_upstream - T_i)` with `v = w*dt/C`:
        //
        // - `v <= 1` is monotone: a node moves towards its upstream temperature
        //   and never past it.
        // - `1 < v < 2` still decays (a chain with a fixed inlet is lower
        //   triangular with `1 - v` on the diagonal, so it converges) but
        //   overshoots, ringing a step change through the chain as alternating
        //   over- and under-shoot. That is nonsense as physics — material
        //   cannot arrive hotter than it left — even though it is not blow-up.
        // - `v >= 2` genuinely diverges.
        //
        // Contributing `2w` here makes the shared `2C/sum_g` below come out as
        // the monotonicity limit, which is the one worth honouring.
        for f in &self.flows {
            if let FlowTerminal::Node(id) = f.source {
                sum_g[id.0] = 2.0f64.mul_add(f.w_per_k.max(0.0), sum_g[id.0]);
            }
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

        // Upwind advection: the stream carries the *source's* enthalpy above the
        // datum, debited there and credited to the target. Along a chain these
        // telescope into `w * (T_upstream - T_own)` per node.
        for f in &self.flows {
            let q = f.w_per_k * (self.terminal_c(f.source) - f.datum_c);
            if let FlowTerminal::Node(id) = f.source {
                self.flux[id.0] -= q;
            }
            if let FlowTerminal::Node(id) = f.target {
                self.flux[id.0] += q;
            }
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

    /// Wire a stream of `cells` cells, each coupled to a wall held at `wall_c`,
    /// fed from a boundary at 0 °C and discharging to one.
    fn flow_chain(cells: usize, w: f64, g: f64, wall_c: f64) -> (ThermalNetwork, Vec<NodeId>) {
        let mut net = ThermalNetwork::new(0.0);
        let wall = net.add_node(Node::new("wall", 1e12, wall_c));

        let mut chain = Vec::new();
        let mut source = FlowTerminal::Boundary(0.0);
        for i in 0..cells {
            let id = net.add_node(Node::new(format!("cell[{i}]"), 50.0, 0.0));
            net.add_flow(Flow {
                source,
                target: FlowTerminal::Node(id),
                w_per_k: w,
                datum_c: 0.0,
            });
            net.connect(id, wall, g);
            chain.push(id);
            source = FlowTerminal::Node(id);
        }
        // The outlet: what the last cell sends leaves the network.
        net.add_flow(Flow {
            source,
            target: FlowTerminal::Boundary(0.0),
            w_per_k: w,
            datum_c: 0.0,
        });
        (net, chain)
    }

    /// A flow is one-way: a hot downstream node must never warm a cold upstream
    /// one, however long it runs.
    #[test]
    fn advection_only_moves_heat_downstream() {
        let mut net = ThermalNetwork::new(0.0);
        let up = net.add_node(Node::new("up", 100.0, 0.0));
        let down = net.add_node(Node::new("down", 100.0, 100.0));
        net.add_flow(Flow {
            source: FlowTerminal::Node(up),
            target: FlowTerminal::Node(down),
            w_per_k: 2.0,
            datum_c: 0.0,
        });

        for _ in 0..1_000 {
            net.step(0.01);
        }
        assert_relative_eq!(net.temperature(up), 0.0, epsilon = 1e-12);
    }

    /// A stream through a chain of cells, each losing heat to a fixed wall,
    /// approaches the wall exponentially in *cell index*: each cell divides the
    /// remaining difference by `1 + g/w`.
    #[test]
    fn a_flow_chain_reaches_the_analytic_steady_state() {
        const CELLS: usize = 6;
        const W: f64 = 4.0; // m_dot * cp
        const G: f64 = 1.0; // wall coupling per cell

        let (mut net, chain) = flow_chain(CELLS, W, G, 100.0);
        for _ in 0..200_000 {
            net.step(0.01);
        }

        // Steady state of cell i: w*(T_{i-1} - T_i) + g*(wall - T_i) = 0
        //   =>  100 - T_i = (100 - T_{i-1}) * w / (w + g)
        let mut expected = 0.0_f64;
        for id in chain {
            expected = 100.0 - (100.0 - expected) * W / (W + G);
            // The chain settles from the head down, so the tail arrives last;
            // 1e-4 is far tighter than any modelling error.
            assert_relative_eq!(net.temperature(id), expected, max_relative = 1e-4);
        }
    }

    /// The books have to balance: at steady state what the wall gives up must
    /// equal what the stream carries out of the network.
    #[test]
    fn flow_carries_the_walls_heat_out_of_the_network() {
        let (mut net, chain) = flow_chain(6, 4.0, 1.0, 100.0);
        for _ in 0..200_000 {
            net.step(0.01);
        }

        let from_wall: f64 = chain.iter().map(|id| 100.0 - net.temperature(*id)).sum();
        assert_relative_eq!(net.net_flow_out_w(), from_wall, max_relative = 1e-4);
    }

    /// A network sitting uniformly at the datum transports nothing, at any rate.
    /// This is what keeps an idle machine from drifting.
    #[test]
    fn a_uniform_stream_at_the_datum_transports_nothing() {
        let (mut net, chain) = flow_chain(6, 4.0, 0.0, 0.0);
        for _ in 0..10_000 {
            net.step(0.01);
        }
        for id in chain {
            assert_relative_eq!(net.temperature(id), 0.0, epsilon = 1e-12);
        }
        assert_relative_eq!(net.net_flow_out_w(), 0.0, epsilon = 1e-12);
    }

    /// The reported limit for a flow is the Courant one, `dt <= C / w` — a
    /// factor of two tighter than diffusion's `2C/G`. What it buys is
    /// monotonicity: inside it a node moves towards the temperature feeding it
    /// and never past it; outside it the node overshoots its own supply, which
    /// is not physics.
    #[test]
    fn max_stable_dt_is_the_monotonicity_limit_for_flow() {
        // A complete one-cell stream: material in from a 100 C supply, and out
        // again. Both edges are needed — an inlet on its own is a constant
        // power injection with no equilibrium, because the cell's own outflow is
        // what makes it relax towards what feeds it.
        let build = || {
            let mut net = ThermalNetwork::new(0.0);
            let cold = net.add_node(Node::new("cold", 100.0, 0.0));
            net.add_flow(Flow {
                source: FlowTerminal::Boundary(100.0),
                target: FlowTerminal::Node(cold),
                w_per_k: 50.0,
                datum_c: 0.0,
            });
            net.add_flow(Flow {
                source: FlowTerminal::Node(cold),
                target: FlowTerminal::Boundary(0.0),
                w_per_k: 50.0,
                datum_c: 0.0,
            });
            (net, cold)
        };

        // C / w = 100 / 50 = 2 s, not the 4 s a diffusive path would allow.
        let (net, _) = build();
        let limit = net.max_stable_dt();
        assert_relative_eq!(limit, 2.0, max_relative = 1e-9);

        let (mut inside, a) = build();
        inside.step(limit);
        assert!(
            inside.temperature(a) <= 100.0 + 1e-9,
            "at the limit the node should just reach its supply, not pass it"
        );

        let (mut outside, b) = build();
        outside.step(limit * 1.5);
        assert!(
            outside.temperature(b) > 100.0,
            "past the limit the node overshoots the 100 C feeding it — \
             the reported limit is what keeps that from happening"
        );
    }

    /// The rate is meant to be raised after the chain is wired — that is how a
    /// stopped machine starts.
    #[test]
    fn a_zero_rate_flow_is_inert_until_it_is_rated() {
        let mut net = ThermalNetwork::new(0.0);
        let down = net.add_node(Node::new("down", 100.0, 0.0));
        let f = net.add_flow(Flow {
            source: FlowTerminal::Boundary(100.0),
            target: FlowTerminal::Node(down),
            w_per_k: 0.0,
            datum_c: 0.0,
        });

        for _ in 0..1_000 {
            net.step(0.01);
        }
        assert_relative_eq!(net.temperature(down), 0.0, epsilon = 1e-12);
        assert!(net.max_stable_dt().is_infinite());

        net.set_flow_w(f, 5.0);
        for _ in 0..1_000 {
            net.step(0.01);
        }
        assert!(net.temperature(down) > 1.0);
    }

    /// The reason the edge carries `w * (T - datum)` from *both* ends rather
    /// than `w * (T_up - T_own)` into one: with a temperature dependent `cp`,
    /// setting `w` to the secant `m_dot * (h(T) - h(datum)) / (T - datum)` makes
    /// the chain transport exact enthalpy differences. Model a two-cell stream
    /// with a `cp` that doubles above 50 °C and check the energy books.
    #[test]
    fn a_secant_rate_transports_exact_enthalpy() {
        const MDOT: f64 = 0.5; // kg/s
        const CP_LO: f64 = 1000.0;
        const CP_HI: f64 = 2000.0;
        const BREAK_C: f64 = 50.0;

        // h(T) with a kink at BREAK_C.
        let h = |t: f64| {
            if t <= BREAK_C {
                CP_LO * t
            } else {
                CP_LO.mul_add(BREAK_C, CP_HI * (t - BREAK_C))
            }
        };
        // The secant rate the caller is expected to supply.
        let rate = |t: f64| {
            if (t - 0.0).abs() < 1e-9 {
                MDOT * CP_LO
            } else {
                MDOT * (h(t) - h(0.0)) / (t - 0.0)
            }
        };

        let mut net = ThermalNetwork::new(0.0);
        let a = net.add_node(Node::new("a", 1e12, 80.0)); // pinned upstream, above the kink
        let b = net.add_node(Node::new("b", 5_000.0, 20.0)); // downstream, below it
        let inlet = net.add_flow(Flow {
            source: FlowTerminal::Node(a),
            target: FlowTerminal::Node(b),
            w_per_k: rate(80.0),
            datum_c: 0.0,
        });
        let outlet = net.add_flow(Flow {
            source: FlowTerminal::Node(b),
            target: FlowTerminal::Boundary(0.0),
            w_per_k: rate(20.0),
            datum_c: 0.0,
        });

        // Advance, re-rating the outlet from b's own temperature each step, and
        // integrate the enthalpy that crossed each boundary.
        let dt = 0.001;
        let mut carried_in = 0.0;
        let mut carried_out = 0.0;
        for _ in 0..200_000 {
            net.set_flow_w(outlet, rate(net.temperature(b)));
            carried_in = net.flow_heat_w(inlet).mul_add(dt, carried_in);
            carried_out = net.flow_heat_w(outlet).mul_add(dt, carried_out);
            net.step(dt);
        }

        // b ends up at the upstream temperature, having absorbed exactly the
        // enthalpy difference on the way. Node capacity here is a plain C, so
        // compare against what actually accumulated.
        assert_relative_eq!(net.temperature(b), 80.0, max_relative = 1e-6);
        let stored = 5_000.0 * (80.0 - 20.0);
        assert_relative_eq!(carried_in - carried_out, stored, max_relative = 1e-3);
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
