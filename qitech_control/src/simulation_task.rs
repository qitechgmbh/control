//! Background task driving the live extruder heating simulation and streaming
//! it over the `/simulation` Socket.io namespace.
//!
//! Independent of the EtherCAT machine loop: spawned once alongside the API
//! server (see `setup_api_and_websock`), not tied to any hardware or to
//! `run_machines()`'s tick. There is exactly one simulation for the whole
//! backend, so — unlike a real machine — this owns its
//! [`LiveExtruderSim`](machine_implementations::extruder1::simulation::LiveExtruderSim)
//! directly rather than through the `machines`/`machines_with_channel`
//! registry, and reaches its namespace straight through
//! [`SharedAppState::socketio_setup`] the same way [`SharedAppState`]'s own
//! `send_*` helpers reach `/main`.

use std::sync::Arc;
use std::time::{Duration, Instant};

use machine_implementations::extruder1::simulation::{
    LiveExtruderSim, Sample, StrategyConfig, Zone,
};
use serde::Deserialize;
use serde_json::Value;
use tokio::sync::mpsc;

use crate::SharedAppState;
use crate::apis::socketio::simulation_namespace::{
    LiveValuesEvent, PerZone, SimulationEvents, StateEvent,
};
use crate::app_state::get_async_runtime;
use control_core::socketio::namespace::NamespaceCacheingLogic;

/// How often the simulation is advanced and a fresh `LiveValuesEvent` is
/// emitted — ~30 Hz, matching the throttle a real machine's `act()` loop uses.
const TICK: Duration = Duration::from_millis(33);

/// Mutations accepted on `POST /api/v1/simulation/mutate`, as the tagged JSON
/// shape every machine's `Mutation` enum already uses on the wire:
/// `{"SetScrewRpm": 60.0}`, `{"SetSetpoint": {"zone": "front", "celsius": 180.0}}`.
#[derive(Deserialize)]
pub enum SimulationMutation {
    SetSetpoint { zone: Zone, celsius: f64 },
    SetAllSetpoints([f64; 4]),
    SetStrategy(StrategyConfig),
    SetScrewRpm(f64),
    SetSpeed(f64),
    Play {},
    Pause {},
    Reset { initial_c: f64 },
}

fn apply_mutation(sim: &mut LiveExtruderSim, value: Value) -> bool {
    let mutation: SimulationMutation = match serde_json::from_value(value) {
        Ok(mutation) => mutation,
        Err(err) => {
            tracing::warn!("invalid simulation mutation: {err}");
            return false;
        }
    };
    match mutation {
        SimulationMutation::SetSetpoint { zone, celsius } => sim.set_setpoint(zone, celsius),
        SimulationMutation::SetAllSetpoints(celsius) => sim.set_all_setpoints(celsius),
        SimulationMutation::SetStrategy(strategy) => sim.set_strategy(strategy),
        SimulationMutation::SetScrewRpm(rpm) => sim.set_screw_rpm(rpm),
        SimulationMutation::SetSpeed(speed) => sim.set_speed(speed),
        SimulationMutation::Play {} => sim.play(),
        SimulationMutation::Pause {} => sim.pause(),
        SimulationMutation::Reset { initial_c } => sim.reset(initial_c),
    }
    true
}

/// Read the current state into a [`StateEvent`] and emit it.
///
/// Takes the built event rather than `&LiveExtruderSim`: the simulation holds
/// a `Box<dyn HeatingStrategy>`, which is `Send` but not `Sync`, so a shared
/// reference to it can't be held across the `.await` below inside a spawned
/// task. Building the (plain-data, `Send + Sync`) event synchronously first
/// keeps the borrow of `sim` out of the async part entirely.
async fn emit_state(state: &SharedAppState, event: StateEvent) {
    let mut guard = state.socketio_setup.namespaces.write().await;
    guard
        .simulation_namespace
        .emit(SimulationEvents::State(event.build()));
    drop(guard);
}

async fn emit_live_values(state: &SharedAppState, event: LiveValuesEvent) {
    let mut guard = state.socketio_setup.namespaces.write().await;
    guard
        .simulation_namespace
        .emit(SimulationEvents::LiveValues(event.build()));
    drop(guard);
}

fn state_event(sim: &LiveExtruderSim) -> StateEvent {
    StateEvent {
        setpoints_c: PerZone::from_array(sim.setpoints_c()),
        strategy: sim.strategy().clone(),
        screw_rpm: sim.screw_rpm(),
        running: sim.running(),
        speed: sim.speed(),
    }
}

fn live_values_event(sample: &Sample) -> LiveValuesEvent {
    LiveValuesEvent {
        sim_time_s: sample.t_s,
        sensor_c: PerZone::from_array(sample.sensor_c),
        steel_c: PerZone::from_array(sample.steel_c),
        band_c: PerZone::from_array(sample.band_c),
        duty: PerZone::from_array(sample.duty),
        power_w: PerZone::from_array(sample.power_w),
        melt_c: PerZone::from_array(sample.melt_c.map(|c| (!c.is_nan()).then_some(c))),
        screw_rpm: sample.screw_rpm,
        throughput_kg_h: sample.throughput_kg_h,
    }
}

/// Register the mutate channel and run the simulation loop until the process
/// exits. Call once, alongside `setup_api_and_websock`.
pub fn spawn_simulation_task(state: Arc<SharedAppState>) {
    let (tx, mut rx) = mpsc::channel::<Value>(64);
    let rt = get_async_runtime();

    rt.spawn(async move {
        state.simulation_sender.write().await.replace(tx);

        let mut sim = LiveExtruderSim::new();
        emit_state(&state, state_event(&sim)).await;

        let mut interval = tokio::time::interval(TICK);
        let mut last_tick = Instant::now();
        loop {
            tokio::select! {
                Some(value) = rx.recv() => {
                    if apply_mutation(&mut sim, value) {
                        let event = state_event(&sim);
                        emit_state(&state, event).await;
                    }
                }
                _ = interval.tick() => {
                    let now = Instant::now();
                    let elapsed = now.duration_since(last_tick);
                    last_tick = now;
                    let event = live_values_event(sim.tick(elapsed));
                    emit_live_values(&state, event).await;
                }
            }
        }
    });
}
