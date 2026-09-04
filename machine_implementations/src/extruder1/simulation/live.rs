//! A live-drivable extruder heating simulation.
//!
//! Wraps the same [`ThermalSim`] the offline harness uses, but ticked
//! incrementally from a background loop instead of run to completion, so a UI
//! can watch it heat up and change setpoints, algorithm and playback speed
//! while it runs. Everything physical and every control law is exactly what
//! [`super::harness`] and [`super::model`] already are — this only adds the
//! bookkeeping to drive them tick by tick and to swap plant/strategy in place.

use std::time::Duration;

use crate::extruder1::heating_params::observer_pi_params;
use crate::extruder1::zone::Zone;

use super::harness::{RunState, Sample, SimConfig, StrategyConfig, ThermalSim};
use super::params::ExtruderThermalParams;

/// Lower bound on the speed multiplier — near-frozen but still visibly moving.
pub const MIN_SPEED: f64 = 0.1;
/// Upper bound on the speed multiplier. Higher is possible (an hour of
/// heat-up simulates in a few seconds per the harness benchmark), but a UI
/// wants the chart to still be legible, not to jump straight to steady state.
pub const MAX_SPEED: f64 = 200.0;

/// Cap on how many plant steps one [`LiveExtruderSim::tick`] call will run, so
/// a long gap between ticks (the caller stalled, or a high speed multiplier)
/// can't block the task it runs on for an unbounded stretch of wall time.
const MAX_STEPS_PER_TICK: u32 = 20_000;

/// The extruder heating simulation, driven live.
pub struct LiveExtruderSim {
    sim: ThermalSim,
    state: RunState,
    setpoints_c: [f64; 4],
    screw_rpm: f64,
    strategy: StrategyConfig,
    running: bool,
    speed: f64,
    last_sample: Sample,
}

impl LiveExtruderSim {
    /// Cold start at ambient, the shipping observer-PI controller, screw
    /// stopped, running at 1x.
    pub fn new() -> Self {
        let params = ExtruderThermalParams::calibrated();
        let ambient = params.ambient_c;
        Self::build(
            ambient,
            [ambient; 4],
            0.0,
            StrategyConfig::ObserverPi(observer_pi_params()),
            params,
        )
    }

    /// Rebuild the plant and controllers from scratch: a strategy swap, an
    /// extrusion toggle (the melt chain has to be baked into the model at
    /// construction, so turning it on cannot be done in place), and reset all
    /// go through this. Everything about the running simulation except the
    /// physics — `running`/`speed` — is restored by the caller afterwards.
    fn build(
        initial_c: f64,
        setpoints_c: [f64; 4],
        screw_rpm: f64,
        strategy: StrategyConfig,
        mut params: ExtruderThermalParams,
    ) -> Self {
        params.melt.enabled = screw_rpm > 0.0;
        let config = SimConfig {
            strategy: strategy.clone(),
            ..SimConfig::default()
        };
        let mut sim = ThermalSim::new(params, config);
        sim.reset_to_uniform(initial_c);
        sim.apply_setpoints(setpoints_c);
        sim.model_mut().set_screw_rpm(screw_rpm);

        let mut state = RunState::new();
        let last_sample = sim.step_once(&mut state, 0);

        Self {
            sim,
            state,
            setpoints_c,
            screw_rpm,
            strategy,
            running: true,
            speed: 1.0,
            last_sample,
        }
    }

    /// Rebuild in place at `initial_c`, preserving `running`/`speed` and every
    /// other current setting.
    fn rebuild(&mut self, initial_c: f64) {
        let params = self.sim.model().params().clone();
        let setpoints_c = self.setpoints_c;
        let screw_rpm = self.screw_rpm;
        let strategy = self.strategy.clone();
        let running = self.running;
        let speed = self.speed;

        *self = Self::build(initial_c, setpoints_c, screw_rpm, strategy, params);
        self.running = running;
        self.speed = speed;
    }

    /// Mean of what the four zone sensors currently read — used as the
    /// restart temperature for a rebuild that should not visibly jump the
    /// chart (a strategy swap or turning extrusion on mid-run).
    fn mean_sensor_c(&self) -> f64 {
        let sum: f64 = Zone::ALL
            .iter()
            .map(|&z| self.sim.model().sensor_c(z))
            .sum();
        sum / Zone::ALL.len() as f64
    }

    /// Advance the simulation by `speed * wall_elapsed` of simulated time, or
    /// do nothing while paused. Returns the sample as of the end of the call.
    pub fn tick(&mut self, wall_elapsed: Duration) -> &Sample {
        if self.running {
            let sim_seconds = self.speed * wall_elapsed.as_secs_f64();
            let dt_plant = self.sim.dt_plant_s();
            let steps = ((sim_seconds / dt_plant).round() as u32).min(MAX_STEPS_PER_TICK);
            for _ in 0..steps {
                self.last_sample = self.sim.step_once(&mut self.state, 0);
            }
        }
        &self.last_sample
    }

    pub fn set_setpoint(&mut self, zone: Zone, celsius: f64) {
        self.setpoints_c[zone.port()] = celsius;
        self.sim.apply_setpoints(self.setpoints_c);
    }

    pub fn set_all_setpoints(&mut self, celsius: [f64; 4]) {
        self.setpoints_c = celsius;
        self.sim.apply_setpoints(celsius);
    }

    /// Set the screw speed. The first time this turns extrusion on, the plant
    /// is rebuilt with the melt chain modelled — see [`Self::build`] — picking
    /// up from the current mean zone temperature so the chart doesn't jump.
    pub fn set_screw_rpm(&mut self, rpm: f64) {
        self.screw_rpm = rpm.max(0.0);
        if self.screw_rpm > 0.0 && !self.sim.model().melt_is_modelled() {
            self.rebuild(self.mean_sensor_c());
        }
        self.sim.model_mut().set_screw_rpm(self.screw_rpm);
    }

    /// Swap the control law. Rebuilds the plant from the current mean zone
    /// temperature so the swap doesn't jump the chart.
    pub fn set_strategy(&mut self, strategy: StrategyConfig) {
        self.strategy = strategy;
        self.rebuild(self.mean_sensor_c());
    }

    pub fn set_speed(&mut self, speed: f64) {
        self.speed = speed.clamp(MIN_SPEED, MAX_SPEED);
    }

    pub fn play(&mut self) {
        self.running = true;
    }

    pub fn pause(&mut self) {
        self.running = false;
    }

    /// Cold (or warm) restart at `initial_c`, keeping the current setpoints,
    /// screw speed, strategy, `running` state and speed multiplier.
    pub fn reset(&mut self, initial_c: f64) {
        self.rebuild(initial_c);
    }

    pub const fn setpoints_c(&self) -> [f64; 4] {
        self.setpoints_c
    }

    pub const fn screw_rpm(&self) -> f64 {
        self.screw_rpm
    }

    pub const fn strategy(&self) -> &StrategyConfig {
        &self.strategy
    }

    pub const fn running(&self) -> bool {
        self.running
    }

    pub const fn speed(&self) -> f64 {
        self.speed
    }

    pub const fn last_sample(&self) -> &Sample {
        &self.last_sample
    }

    /// Simulated time elapsed since the last [`Self::reset`] (or since this
    /// simulation was created), in seconds.
    pub fn sim_time_s(&self) -> f64 {
        self.state.now_s()
    }
}

impl Default for LiveExtruderSim {
    fn default() -> Self {
        Self::new()
    }
}
