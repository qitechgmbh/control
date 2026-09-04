use control_core::socketio::event::Event;
use machine_implementations::extruder1::simulation::{StrategyConfig, Zone};
use serde::Serialize;

/// One value per heating zone, named rather than positional — the convention
/// every other machine namespace's wire events already use (see
/// `extruder1::api::HeatingStates`) — so the frontend doesn't have to know
/// `Zone::port()` order to read it.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct PerZone<T> {
    pub front: T,
    pub middle: T,
    pub back: T,
    pub nozzle: T,
}

impl<T: Copy> PerZone<T> {
    pub fn from_array(values: [T; 4]) -> Self {
        Self {
            front: values[Zone::Front.port()],
            middle: values[Zone::Middle.port()],
            back: values[Zone::Back.port()],
            nozzle: values[Zone::Nozzle.port()],
        }
    }
}

/// Sensor readings, emitted at the tick rate of the background simulation
/// task (~30 Hz), mirroring a real machine's `LiveValuesEvent`.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct LiveValuesEvent {
    /// Simulated time elapsed since the last reset, in seconds.
    pub sim_time_s: f64,
    /// What the controller sees, after EL3204 quantisation, in °C.
    pub sensor_c: PerZone<f64>,
    /// True barrel steel temperature under the sensor, in °C.
    pub steel_c: PerZone<f64>,
    /// Band heater temperature, in °C.
    pub band_c: PerZone<f64>,
    /// Controller duty demand, 0..1.
    pub duty: PerZone<f64>,
    /// Electrical power delivered over the last plant step, in W.
    pub power_w: PerZone<f64>,
    /// Melt temperature under each zone's band, in °C; `null` while extrusion
    /// is off (the melt isn't modelled).
    pub melt_c: PerZone<Option<f64>>,
    pub screw_rpm: f64,
    pub throughput_kg_h: f64,
}

impl LiveValuesEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("LiveValuesEvent", self.clone())
    }
}

/// Configuration, emitted whenever a mutation changes it — mirrors a real
/// machine's `StateEvent`. `strategy` is the same [`StrategyConfig`] a
/// `SetStrategy` mutation sends in, so the two directions share one shape.
#[derive(Serialize, Debug, Clone)]
pub struct StateEvent {
    pub setpoints_c: PerZone<f64>,
    pub strategy: StrategyConfig,
    pub screw_rpm: f64,
    pub running: bool,
    /// Simulated seconds per wall second.
    pub speed: f64,
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}
