//! Closed-loop harness: the real controller code driving the simulated plant.
//!
//! Everything on the control side is real. The harness constructs the production
//! [`TemperatureController`]s, a real Beckhoff [`EL3204`] and [`EL2004`], and
//! calls [`TemperatureController::update`] exactly as [`crate::extruder1::act`]
//! does. Readings are pushed in as **raw PDO bytes**, so `RtdInput::read`'s
//! `i16 / 10.0` decode — the source of the 0.1 °C quantisation — runs for real.
//!
//! Three clocks run at genuinely different rates, and that difference is the
//! point: [`DT_CTRL_S`] for the controller (thousands of ticks per PWM window),
//! [`SENSOR_PERIOD_S`] for the EL3204 conversion (the reading is *held* between
//! conversions, then jumps a whole quantisation step), and [`DT_PLANT_S`] for
//! the thermal integration. A model ticking the PID once per plant step would
//! miss the derivative behaviour entirely.

use std::time::{Duration, Instant};

use bitvec::prelude::*;
use control_core::controllers::heating::{
    HeatingStrategy, ObserverPi, ObserverPiParams, PidBaseline,
};
use control_core::controllers::pid_autotuner::{AutoTuneResult, PidAutoTuner};
use qitech_lib::ethercat_hal::devices::beckhoff_modules::{el2004::EL2004, el3204::EL3204};
use qitech_lib::ethercat_hal::devices::{EthercatDevice, NewEthercatDevice};
use qitech_lib::ethercat_hal::io::temperature_input::TemperatureInputDevice;
use qitech_lib::units::{ThermodynamicTemperature, thermodynamic_temperature::degree_celsius};

use super::model::ExtruderThermalModel;
use super::params::ExtruderThermalParams;
use super::scenario::Scenario;
use crate::extruder1::Heating;
use crate::extruder1::heating_params::DEFAULT_MAX_CLAMP;
use crate::extruder1::temperature_controller::TemperatureController;
use crate::extruder1::zone::Zone;

/// Thermal integration step in seconds.
pub const DT_PLANT_S: f64 = 0.01;
/// How often the controller runs, in seconds.
///
/// `qitech_control`'s machine loop sleeps 100 µs per iteration and does real
/// work on top, so 1 ms is a representative — slightly conservative — value.
pub const DT_CTRL_S: f64 = 0.001;
/// How often the EL3204 refreshes its reading, in seconds.
///
/// Approximate: with the default 50 Hz filter the terminal converts all four
/// channels in roughly this time. Measure it and adjust when calibrating — the
/// derivative term is very sensitive to this number.
pub const SENSOR_PERIOD_S: f64 = 0.25;

/// PID gains for one zone.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ZoneTuning {
    pub kp: f64,
    pub ki: f64,
    pub kd: f64,
}

impl ZoneTuning {
    /// The gains shipping for `MACHINE_EXTRUDER_V2` (the newest hardware
    /// generation), indexed by [`Zone::port`], from
    /// [`crate::extruder1::new`].
    pub const PRODUCTION: [Self; 4] = [
        Self {
            kp: 0.066,
            ki: 0.0,
            kd: 0.0,
        }, // front
        Self {
            kp: 0.020,
            ki: 0.000003,
            kd: 0.0,
        }, // middle
        Self {
            kp: 0.020,
            ki: 0.000017,
            kd: 0.0,
        }, // back
        Self {
            kp: 0.433,
            ki: 0.002,
            kd: 0.0,
        }, // nozzle
    ];
}

/// Which control law the zones run.
///
/// Every variant carries one entry per zone, indexed by [`Zone::port`], because
/// the four zones are genuinely different plants — the nozzle's kilograms per
/// watt are several times a barrel zone's.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum StrategyConfig {
    /// A PID on the raw reading: what has always shipped.
    Pid([ZoneTuning; 4]),
    /// PI on an observed metal temperature, over a steady-state feedforward.
    ObserverPi([ObserverPiParams; 4]),
}

impl StrategyConfig {
    /// Short name for a results table.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Pid(_) => "pid",
            Self::ObserverPi(_) => "observer-pi",
        }
    }

    fn build(&self, zone: Zone) -> Box<dyn HeatingStrategy> {
        let p = zone.port();
        match self {
            Self::Pid(t) => Box::new(PidBaseline::new(
                t[p].kp,
                t[p].ki,
                t[p].kd,
                DEFAULT_MAX_CLAMP[p],
            )),
            Self::ObserverPi(t) => Box::new(ObserverPi::new(t[p])),
        }
    }
}

/// Plants that all reproduce the recorded heat-up, spread along the axis the
/// calibration cannot resolve.
///
/// `sensor_tau_s` and `band_heat_capacity_j_per_m2_k` trade off almost exactly —
/// both delay the reading relative to the steel — and one recording cannot say
/// which is responsible. The optimiser stopped at `tau = 150 s` with a nearly
/// weightless band, but `tau = 20 s` with a heavy band fits the same data and is
/// a *physically very different machine*, where the overshoot is stored energy
/// rather than measurement lag. Each variant moves lag between the two while
/// keeping `tau_sensor + band_heat_capacity / band_contact_h  ~= 160 s`.
///
/// A control law that is only good at one end of this range is tuned to an
/// artefact of the calibration, not to the machine.
/// `family_members_all_match_the_recording` keeps the set honest.
pub fn plant_family() -> Vec<ExtruderThermalParams> {
    const TOTAL_LAG_S: f64 = 160.0;
    [20.0, 60.0, 100.0, 150.0]
        .into_iter()
        .map(|tau_sensor_s| {
            let base = ExtruderThermalParams::calibrated();
            let band_tau_s = (TOTAL_LAG_S - tau_sensor_s).max(10.0);
            ExtruderThermalParams {
                sensor_tau_s: tau_sensor_s,
                band_heat_capacity_j_per_m2_k: (band_tau_s * base.band_contact_h)
                    .clamp(1_000.0, 30_000.0),
                ..base
            }
        })
        .collect()
}

/// Harness configuration.
#[derive(Debug, Clone)]
pub struct SimConfig {
    pub dt_plant_s: f64,
    pub dt_ctrl_s: f64,
    pub sensor_period_s: f64,
    /// How often a [`Sample`] is appended to the [`Trace`].
    pub record_period_s: f64,
    /// The control law under test.
    pub strategy: StrategyConfig,
    /// Slow-PWM window length.
    pub pwm_period: Duration,
    /// Over-temperature cutout in °C.
    pub max_temperature_c: f64,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            dt_plant_s: DT_PLANT_S,
            dt_ctrl_s: DT_CTRL_S,
            sensor_period_s: SENSOR_PERIOD_S,
            record_period_s: 1.0,
            strategy: StrategyConfig::Pid(ZoneTuning::PRODUCTION),
            pwm_period: Duration::from_millis(500),
            max_temperature_c: 300.0,
        }
    }
}

/// One recorded instant. All arrays are indexed by [`Zone::port`].
#[derive(Debug, Clone)]
pub struct Sample {
    pub t_s: f64,
    /// The setpoint in force at this instant, in °C. Per sample because a
    /// scenario may change setpoints part-way through. `NaN` for open loop.
    pub setpoint_c: [f64; 4],
    /// What the controller saw, after EL3204 quantisation.
    pub sensor_c: [f64; 4],
    /// True barrel steel temperature under the sensor.
    pub steel_c: [f64; 4],
    /// Band heater temperature.
    pub band_c: [f64; 4],
    /// PID duty demand, 0..1 — what the machine reports as "power".
    pub duty: [f64; 4],
    /// Electrical power actually delivered over the last plant step, in W.
    pub power_w: [f64; 4],
    /// Melt temperature under each zone's band, in °C. `NaN` when not modelled.
    pub melt_c: [f64; 4],
    /// Heat the melt is drawing from each zone's steel, in W.
    pub meltload_w: [f64; 4],
    /// Screw speed in rpm.
    pub screw_rpm: f64,
    /// Throughput in kg/h.
    pub throughput_kg_h: f64,
    /// Enthalpy leaving at the die, in W.
    pub melt_out_w: f64,
    /// Viscous shear power going into the melt, in W.
    pub shear_w: f64,
}

/// A simulation run.
#[derive(Debug, Clone, Default)]
pub struct Trace {
    pub samples: Vec<Sample>,
    /// Setpoints used, indexed by [`Zone::port`].
    pub setpoints_c: [f64; 4],
    /// Relay transitions at controller-tick resolution, indexed by [`Zone::port`].
    ///
    /// Counted during the run, not derived from [`Self::samples`]: at 1 Hz
    /// against a 500 ms window a sampled count aliases to almost nothing.
    pub relay_switches: [usize; 4],
    /// Electrical energy delivered per zone in J, indexed by [`Zone::port`].
    ///
    /// Accumulated every plant step, for the same aliasing reason as
    /// [`Self::relay_switches`].
    pub energy_j: [f64; 4],
    /// Enthalpy carried out of the die over the run, in J.
    pub melt_energy_j: f64,
    /// Viscous shear work put into the melt over the run, in J.
    pub shear_energy_j: f64,
}

impl Trace {
    /// `(t_seconds, sensor_temperature)` pairs for one zone, for plotting.
    pub fn sensor_series(&self, zone: Zone) -> Vec<(f32, f32)> {
        self.samples
            .iter()
            .map(|s| (s.t_s as f32, s.sensor_c[zone.port()] as f32))
            .collect()
    }

    /// Highest sensor reading a zone reached.
    pub fn peak_c(&self, zone: Zone) -> f64 {
        self.samples
            .iter()
            .map(|s| s.sensor_c[zone.port()])
            .fold(f64::NEG_INFINITY, f64::max)
    }

    /// Last sensor reading of a zone.
    pub fn final_c(&self, zone: Zone) -> f64 {
        self.samples
            .last()
            .map_or(f64::NAN, |s| s.sensor_c[zone.port()])
    }

    /// Peak minus setpoint, in K. Negative means the zone never got there.
    pub fn overshoot_k(&self, zone: Zone) -> f64 {
        self.peak_c(zone) - self.setpoints_c[zone.port()]
    }

    /// Time in seconds to first reach `fraction` of the way from the starting
    /// temperature to the setpoint. `None` if it never did.
    pub fn rise_time_s(&self, zone: Zone, fraction: f64) -> Option<f64> {
        let first = self.samples.first()?;
        let start = first.sensor_c[zone.port()];
        let target = fraction.mul_add(self.setpoints_c[zone.port()] - start, start);
        self.samples
            .iter()
            .find(|s| s.sensor_c[zone.port()] >= target)
            .map(|s| s.t_s - first.t_s)
    }

    /// Relay transitions over the run, as a proxy for SSR wear.
    pub fn relay_switches(&self, zone: Zone) -> usize {
        self.relay_switches[zone.port()]
    }

    /// Total electrical energy delivered to a zone, in kWh.
    pub fn energy_kwh(&self, zone: Zone) -> f64 {
        self.energy_j[zone.port()] / 3_600_000.0
    }

    /// Enthalpy the material carried out of the die over the run, in kWh.
    pub fn melt_energy_kwh(&self) -> f64 {
        self.melt_energy_j / 3_600_000.0
    }

    /// Shear work the screw put into the material over the run, in kWh.
    pub fn shear_energy_kwh(&self) -> f64 {
        self.shear_energy_j / 3_600_000.0
    }

    /// CSV with a header row, one line per sample.
    ///
    /// Columns are `t_s`, then `setpoint / sensor / steel / band / duty /
    /// power_w / melt / meltload_w`, each as a group of four in
    /// `front, middle, back, nozzle` order, and finally the four machine-wide
    /// columns `screw_rpm, throughput_kg_h, melt_out_w, shear_w`.
    ///
    /// `sensor` against `steel` is the interesting pair: the gap between them is
    /// the measurement lag the controller has to work through. On an extrusion
    /// run, `meltload_w` against `power_w` is the second one — what the material
    /// takes against what the bands put back.
    ///
    /// The per-zone columns are named `<metric>_<zone>` because that is what
    /// `scripts/plot-heating.html` groups on; the machine-wide four have no zone
    /// suffix and the plotter draws them as single series.
    pub fn to_csv(&self) -> String {
        use std::fmt::Write as _;

        let mut out = String::with_capacity(self.samples.len() * 260);
        out.push_str("t_s");
        for tag in [
            "setpoint",
            "sensor",
            "steel",
            "band",
            "duty",
            "power_w",
            "melt",
            "meltload_w",
        ] {
            for zone in Zone::ALL {
                let _ = write!(out, ",{tag}_{}", zone.name());
            }
        }
        out.push_str(",screw_rpm,throughput_kg_h,melt_out_w,shear_w");
        out.push('\n');
        for s in &self.samples {
            let _ = write!(out, "{:.2}", s.t_s);
            for arr in [&s.setpoint_c, &s.sensor_c, &s.steel_c, &s.band_c] {
                for zone in Zone::ALL {
                    let _ = write!(out, ",{:.3}", arr[zone.port()]);
                }
            }
            for zone in Zone::ALL {
                let _ = write!(out, ",{:.4}", s.duty[zone.port()]);
            }
            for zone in Zone::ALL {
                let _ = write!(out, ",{:.2}", s.power_w[zone.port()]);
            }
            for zone in Zone::ALL {
                let _ = write!(out, ",{:.3}", s.melt_c[zone.port()]);
            }
            for zone in Zone::ALL {
                let _ = write!(out, ",{:.2}", s.meltload_w[zone.port()]);
            }
            let _ = write!(
                out,
                ",{:.2},{:.4},{:.2},{:.2}",
                s.screw_rpm, s.throughput_kg_h, s.melt_out_w, s.shear_w
            );
            out.push('\n');
        }
        out
    }
}

/// Loop state that persists across successive [`ThermalSim::step_once`] calls:
/// the controller-tick/sensor-refresh schedule and relay bookkeeping.
///
/// Not scenario-specific — [`Self::new`] is always a cold start at simulated
/// t = 0, with the virtual clock anchored to the real clock at construction.
pub struct RunState {
    /// Anchor for the virtual clock the controllers see. Captured at
    /// construction, not before: a controller's `window_start` comes from the
    /// real clock, and anchoring behind it would make every `duration_since`
    /// saturate to zero.
    t0: Instant,
    now_ns: u64,
    next_ctrl_ns: u64,
    next_sensor_ns: u64,
    heating_enabled: bool,
    relay_was_on: [bool; 4],
    duty: [f64; 4],
    relay_switches: [usize; 4],
}

impl RunState {
    pub fn new() -> Self {
        Self {
            t0: Instant::now(),
            now_ns: 0,
            next_ctrl_ns: 0,
            next_sensor_ns: 0,
            heating_enabled: false,
            relay_was_on: [false; 4],
            duty: [0.0; 4],
            relay_switches: [0; 4],
        }
    }

    /// Simulated time elapsed since this state was created, in seconds.
    pub fn now_s(&self) -> f64 {
        self.now_ns as f64 * 1e-9
    }
}

impl Default for RunState {
    fn default() -> Self {
        Self::new()
    }
}

/// The production controllers wired to the simulated barrel.
pub struct ThermalSim {
    model: ExtruderThermalModel,
    config: SimConfig,
    controllers: Vec<TemperatureController>,
    el3204: EL3204,
    el2004: EL2004,
    /// Sensor sample-and-hold, in °C, indexed by [`Zone::port`].
    held_c: [f64; 4],
    /// Setpoint currently applied to each zone's controller, indexed by
    /// [`Zone::port`]. Tracked here (rather than only as a caller-local) so
    /// [`Self::step_once`] can report it without the caller threading it
    /// through on every call.
    setpoints_c: [f64; 4],
}

impl ThermalSim {
    pub fn new(params: ExtruderThermalParams, config: SimConfig) -> Self {
        let ambient = params.ambient_c;
        let model = ExtruderThermalModel::new(params);

        let controllers = Zone::ALL
            .iter()
            .map(|zone| {
                let p = zone.port();
                TemperatureController::with_strategy(
                    config.strategy.build(*zone),
                    ThermodynamicTemperature::new::<degree_celsius>(0.0),
                    ThermodynamicTemperature::new::<degree_celsius>(config.max_temperature_c),
                    Heating::default(),
                    config.pwm_period,
                    zone.rated_w(),
                    p,
                    p,
                )
            })
            .collect();

        let mut sim = Self {
            model,
            config,
            controllers,
            el3204: EL3204::new(),
            el2004: EL2004::new(),
            held_c: [ambient; 4],
            setpoints_c: [0.0; 4],
        };
        sim.publish_sensors();
        sim
    }

    pub const fn model(&self) -> &ExtruderThermalModel {
        &self.model
    }

    pub const fn model_mut(&mut self) -> &mut ExtruderThermalModel {
        &mut self.model
    }

    /// The plant integration step, in seconds — how much simulated time one
    /// [`Self::step_once`] call covers.
    pub const fn dt_plant_s(&self) -> f64 {
        self.config.dt_plant_s
    }

    /// Put every node at `temperature_c` and forget any prior sensor history —
    /// a cold (or warm) restart short of building a whole new [`ThermalSim`].
    /// The first three lines of [`Self::run`], pulled out so a live driver can
    /// reuse it without going through a [`super::scenario::Scenario`].
    pub fn reset_to_uniform(&mut self, temperature_c: f64) {
        self.model.set_uniform_temperature(temperature_c);
        self.held_c = [temperature_c; 4];
        self.publish_sensors();
    }

    /// Encode `held_c` into an EL3204 process image and feed it to the driver.
    ///
    /// Layout per channel, 32 bits, from `RtdInput::read`:
    /// bit 0 undervoltage, 1 overvoltage, 2..4 limit1, 4..6 limit2, 7 error,
    /// 14 txpdo_state, 15 txpdo_toggle, 16..32 the temperature as i16 in
    /// 0.1 °C units. `read` bails out unless the toggle bit is set.
    fn publish_sensors(&mut self) {
        let mut buf = [0u8; 16];
        for zone in Zone::ALL {
            let p = zone.port();
            let raw = (self.held_c[p] * 10.0).round().clamp(-32768.0, 32767.0) as i16;
            let base = p * 4;
            // txpdo_state (bit 14) and txpdo_toggle (bit 15) live in byte 1.
            buf[base + 1] = 0b1100_0000;
            buf[base + 2..base + 4].copy_from_slice(&raw.to_le_bytes());
        }
        let bits = buf.view_bits::<Lsb0>();
        self.el3204
            .input(bits)
            .expect("EL3204 process image is the right length");
    }

    fn relay_on(&self, zone: Zone) -> bool {
        let rx = &self.el2004.rxpdo;
        match zone.port() {
            0 => rx.channel1.as_ref(),
            1 => rx.channel2.as_ref(),
            2 => rx.channel3.as_ref(),
            _ => rx.channel4.as_ref(),
        }
        .is_some_and(|c| c.value)
    }

    /// One trace row. `sensor_c` is passed in because the closed loop records
    /// the *held* reading the controller acted on, while the open loop reads the
    /// model directly.
    fn sample(
        &self,
        t_s: f64,
        setpoint_c: [f64; 4],
        sensor_c: [f64; 4],
        duty: [f64; 4],
        power_w: [f64; 4],
    ) -> Sample {
        Sample {
            t_s,
            setpoint_c,
            sensor_c,
            steel_c: Zone::ALL.map(|z| self.model.steel_c(z)),
            band_c: Zone::ALL.map(|z| self.model.band_c(z)),
            duty,
            power_w,
            melt_c: Zone::ALL.map(|z| {
                self.model
                    .melt_c_at(z.band().centre_mm())
                    .unwrap_or(f64::NAN)
            }),
            meltload_w: Zone::ALL.map(|z| self.model.melt_load_w(z)),
            screw_rpm: self.model.screw_rpm(),
            throughput_kg_h: self.model.throughput_kg_per_h(),
            melt_out_w: self.model.melt_extraction_w(),
            shear_w: self.model.shear_power_w(),
        }
    }

    pub fn apply_setpoints(&mut self, setpoints_c: [f64; 4]) {
        for zone in Zone::ALL {
            let p = zone.port();
            self.controllers[p].set_target_temperature(ThermodynamicTemperature::new::<
                degree_celsius,
            >(setpoints_c[p]));
        }
        self.setpoints_c = setpoints_c;
    }

    /// Advance the plant and controllers by exactly one `config.dt_plant_s` of
    /// simulated time, running whatever controller ticks and sensor refreshes
    /// fall inside it, and return the resulting sample.
    ///
    /// Setpoints and screw speed are whatever the caller last set through
    /// [`Self::apply_setpoints`]/[`ExtruderThermalModel::set_screw_rpm`] — this
    /// does not consult a scenario's schedule, so both the batch [`Self::run`]
    /// and a live loop driving the sim tick by tick can share it.
    ///
    /// `enable_ns` is the simulated instant, relative to `state`'s `t0`, that
    /// heating switches on; pass `0` to have it on from the first tick.
    pub fn step_once(&mut self, state: &mut RunState, enable_ns: u64) -> Sample {
        let dt_plant = self.config.dt_plant_s;
        let plant_ns = (dt_plant * 1e9) as u64;
        let ctrl_ns = (self.config.dt_ctrl_s * 1e9) as u64;
        let sensor_ns = (self.config.sensor_period_s * 1e9) as u64;

        // Walk the controller ticks that fall inside this plant step,
        // accumulating how long each relay was actually closed. Driving the
        // loop off the plant step rather than a fixed number of controller
        // ticks keeps it correct when `dt_ctrl` is *longer* than `dt_plant`
        // — which is exactly the case you want when investigating a fixed
        // controller sample time.
        let step_end_ns = state.now_ns + plant_ns;
        let mut on_ns = [0u64; 4];
        let mut cursor_ns = state.now_ns;

        while state.next_ctrl_ns < step_end_ns {
            // The relay held its state from `cursor_ns` to this tick.
            let held = state.next_ctrl_ns - cursor_ns;
            for p in 0..4 {
                if state.relay_was_on[p] {
                    on_ns[p] += held;
                }
            }
            cursor_ns = state.next_ctrl_ns;
            state.next_ctrl_ns += ctrl_ns;

            let now_tick_ns = cursor_ns;
            let now = state.t0 + Duration::from_nanos(now_tick_ns);

            if !state.heating_enabled && now_tick_ns >= enable_ns {
                for c in &mut self.controllers {
                    c.allow_heating();
                }
                state.heating_enabled = true;
            }

            // The EL3204 only refreshes on its own conversion cycle; in
            // between, the controller re-reads a held value.
            if now_tick_ns >= state.next_sensor_ns {
                for zone in Zone::ALL {
                    self.held_c[zone.port()] = self.model.sensor_c(zone);
                }
                self.publish_sensors();
                state.next_sensor_ns += sensor_ns;
            }

            for zone in Zone::ALL {
                let p = zone.port();
                self.controllers[p].update(now, &mut self.el2004, &self.el3204);
                let on = self.relay_on(zone);
                if on != state.relay_was_on[p] {
                    state.relay_switches[p] += 1;
                    state.relay_was_on[p] = on;
                }
                state.duty[p] = self.controllers[p].duty();
            }
        }

        // Whatever is left of the plant step after the last controller tick.
        let held = step_end_ns - cursor_ns;
        for p in 0..4 {
            if state.relay_was_on[p] {
                on_ns[p] += held;
            }
        }
        state.now_ns = step_end_ns;

        // Time-weighted mean relay state over the step. The relay is
        // piecewise constant between controller ticks, so this is exact.
        let mut power_w = [0.0f64; 4];
        for zone in Zone::ALL {
            let p = zone.port();
            power_w[p] = (on_ns[p] as f64 / plant_ns as f64) * zone.rated_w();
            self.model.set_band_power(zone, power_w[p]);
        }

        self.model.step(dt_plant);

        // The controller sees the held reading, not the model's current one;
        // the sample records what it saw.
        let sensor_c = self.held_c;
        self.sample(
            state.now_ns as f64 * 1e-9,
            self.setpoints_c,
            sensor_c,
            state.duty,
            power_w,
        )
    }

    /// Run a scenario and return the recorded trace.
    pub fn run(&mut self, scenario: &Scenario) -> Trace {
        self.model.set_uniform_temperature(scenario.initial_c);
        self.held_c = [scenario.initial_c; 4];
        self.publish_sensors();

        assert!(
            !scenario.extrudes() || self.model.melt_is_modelled(),
            "scenario '{}' turns the screw, but this model was built without the \
             melt — set `params.melt.enabled` or the run would silently do nothing",
            scenario.name
        );

        let mut setpoints = scenario.setpoints_c;
        self.apply_setpoints(setpoints);
        let mut pending: Vec<(f64, [f64; 4])> = scenario.changes.clone();
        pending.sort_by(|a, b| a.0.total_cmp(&b.0));
        pending.reverse(); // pop() yields the earliest

        self.model.set_screw_rpm(scenario.screw_rpm);
        let mut pending_rpm: Vec<(f64, f64)> = scenario.rpm_changes.clone();
        pending_rpm.sort_by(|a, b| a.0.total_cmp(&b.0));
        pending_rpm.reverse();

        let mut state = RunState::new();
        let enable_ns = (scenario.heating_enabled_at_s * 1e9) as u64;

        let mut trace = Trace {
            setpoints_c: setpoints,
            ..Default::default()
        };

        let dt_plant = self.config.dt_plant_s;
        let end_ns = (scenario.duration_s * 1e9) as u64;
        let record_ns = (self.config.record_period_s * 1e9) as u64;
        let mut next_record_ns: u64 = 0;

        while state.now_ns < end_ns {
            // Apply any setpoint change that has come due.
            while pending
                .last()
                .is_some_and(|(at, _)| state.now_ns as f64 * 1e-9 >= *at)
            {
                let (_, sp) = pending.pop().expect("checked by the guard above");
                setpoints = sp;
                self.apply_setpoints(setpoints);
                trace.setpoints_c = setpoints;
            }

            // And any screw-speed change. Starting or stopping extrusion is a
            // load step on every zone at once.
            while pending_rpm
                .last()
                .is_some_and(|(at, _)| state.now_ns as f64 * 1e-9 >= *at)
            {
                let (_, rpm) = pending_rpm.pop().expect("checked by the guard above");
                self.model.set_screw_rpm(rpm);
            }

            let sample = self.step_once(&mut state, enable_ns);

            // Accumulated every plant step, for the same aliasing reason noted
            // on `Trace::energy_j`. Uses the sample just produced (i.e. the
            // post-step reading) rather than a separate pre-step read — the two
            // differ by at most one `dt_plant`, which is well inside the loose
            // bounds anything downstream checks these totals against.
            for zone in Zone::ALL {
                let p = zone.port();
                trace.energy_j[p] += sample.power_w[p] * dt_plant;
            }
            trace.melt_energy_j += sample.melt_out_w * dt_plant;
            trace.shear_energy_j += sample.shear_w * dt_plant;

            if state.now_ns >= next_record_ns {
                trace.samples.push(sample);
                next_record_ns += record_ns;
            }
        }

        trace.relay_switches = state.relay_switches;
        trace
    }

    /// Relay-autotune one zone against the simulated plant.
    ///
    /// Drives `zone` from `tuner` alone — the other zones stay off — and returns
    /// the suggested gains, or `None` if the tuner did not converge within
    /// `max_duration_s` of simulated time. The tuner's output is used directly
    /// as the band's duty: the Åström-Hägglund method is bang-bang by design, so
    /// there is no PWM window to emulate.
    ///
    /// The sensor is read through the same EL3204 path as [`Self::run`], so the
    /// tuner sees the same 0.1 °C quantisation and sample-and-hold the real one
    /// would.
    pub fn run_autotune(
        &mut self,
        zone: Zone,
        target_c: f64,
        initial_c: f64,
        max_duration_s: f64,
        tuner: &mut PidAutoTuner,
    ) -> Option<AutoTuneResult> {
        self.model.set_uniform_temperature(initial_c);
        self.held_c = [initial_c; 4];
        self.publish_sensors();

        let t0 = Instant::now();
        tuner.start(t0, target_c);

        let dt = self.config.dt_plant_s;
        let steps = (max_duration_s / dt).round() as u64;
        let sensor_every = ((self.config.sensor_period_s / dt).round() as u64).max(1);

        for i in 0..steps {
            let now = t0 + Duration::from_nanos((i as f64 * dt * 1e9) as u64);
            if i % sensor_every == 0 {
                self.held_c[zone.port()] = self.model.sensor_c(zone);
                self.publish_sensors();
            }
            let measured = self
                .el3204
                .get_input(zone.port())
                .map_or(self.held_c[zone.port()], |r| f64::from(r.temperature));

            let duty = tuner.update(measured, now);
            self.model
                .set_band_power(zone, duty.clamp(0.0, 1.0) * zone.rated_w());
            self.model.step(dt);

            if tuner.is_completed() || tuner.is_failed() {
                break;
            }
        }
        self.model.set_band_power(zone, 0.0);
        tuner.result().ok().cloned()
    }

    /// Drive the plant open loop with fixed per-zone duty cycles.
    ///
    /// Used by [`super::fit`] to replay a recorded run without the controller in
    /// the way, and handy for measuring the plant's own step response.
    pub fn run_open_loop(
        &mut self,
        initial_c: f64,
        duration_s: f64,
        duty_at: &dyn Fn(f64) -> [f64; 4],
    ) -> Trace {
        self.model.set_uniform_temperature(initial_c);
        // Open loop drives the bands directly; there is no relay to count.
        let mut trace = Trace {
            setpoints_c: [f64::NAN; 4],
            ..Default::default()
        };
        let dt = self.config.dt_plant_s;
        let steps = (duration_s / dt).round() as usize;
        let record_every = ((self.config.record_period_s / dt).round() as usize).max(1);

        for i in 0..steps {
            let t = i as f64 * dt;
            let duty = duty_at(t);
            let mut power_w = [0.0; 4];
            for zone in Zone::ALL {
                let p = zone.port();
                power_w[p] = duty[p].clamp(0.0, 1.0) * zone.rated_w();
                trace.energy_j[p] += power_w[p] * dt;
                self.model.set_band_power(zone, power_w[p]);
            }
            self.model.step(dt);

            if i % record_every == 0 {
                let sensor_c = Zone::ALL.map(|z| self.model.sensor_c(z));
                // Open loop: nothing is being asked for.
                trace
                    .samples
                    .push(self.sample(t, [f64::NAN; 4], sensor_c, duty, power_w));
            }
        }
        trace
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The raw PDO encoding must survive the driver's own decode, including its
    /// 0.1 °C quantisation. If this breaks, every sensor reading in the sim is
    /// wrong.
    #[test]
    fn el3204_round_trip_quantises_to_a_tenth_of_a_degree() {
        let mut sim = ThermalSim::new(ExtruderThermalParams::default(), SimConfig::default());
        sim.held_c = [22.34, 150.06, -5.0, 287.99];
        sim.publish_sensors();

        let expect = [22.3_f32, 150.1, -5.0, 288.0];
        for zone in Zone::ALL {
            let got = sim
                .el3204
                .get_input(zone.port())
                .expect("port exists")
                .temperature;
            assert!(
                (got - expect[zone.port()]).abs() < 1e-4,
                "{} read back {got}, expected {}",
                zone.name(),
                expect[zone.port()]
            );
        }
    }

    /// What the machine actually did on 2026-02-24, from
    /// `data/heatup_2026-02-24.csv`: `(peak, t90)` per zone, indexed by
    /// [`Zone::port`]. Setpoints were front 180, middle 180, back 170,
    /// nozzle 175.
    const MEASURED: [(f64, f64); 4] = [
        (194.4, 703.0), // front
        (211.5, 674.0), // middle
        (186.6, 661.0), // back
        (171.2, 1849.0),
    ];

    /// Uniform gains in effect on the machine when `data/heatup_2026-02-24.csv`
    /// was recorded — pre-dates the per-zone retune in
    /// [`ZoneTuning::PRODUCTION`], so the recorded-run comparisons below pin
    /// this rather than following `SimConfig::default()`.
    const RECORDED_RUN_TUNING: ZoneTuning = ZoneTuning {
        kp: 0.16,
        ki: 0.0,
        kd: 0.008,
    };

    fn recorded_run_trace() -> Trace {
        let config = SimConfig {
            strategy: StrategyConfig::Pid([RECORDED_RUN_TUNING; 4]),
            ..SimConfig::default()
        };
        let mut sim = ThermalSim::new(ExtruderThermalParams::calibrated(), config);
        sim.run(&Scenario::recorded_heatup())
    }

    /// The headline claim: the calibrated model reproduces the real heat-up.
    ///
    /// Bounds are loose enough to survive incidental changes but tight enough
    /// that losing the physics would trip them.
    #[test]
    fn calibrated_model_matches_the_recorded_heat_up() {
        let trace = recorded_run_trace();
        for zone in Zone::ALL {
            let (peak, t90) = MEASURED[zone.port()];
            let sim_peak = trace.peak_c(zone);
            assert!(
                (sim_peak - peak).abs() < 4.0,
                "{} peak {sim_peak:.1} C vs measured {peak:.1} C",
                zone.name()
            );
            let sim_t90 = trace
                .rise_time_s(zone, 0.9)
                .unwrap_or_else(|| panic!("{} never reached 90 % of setpoint", zone.name()));
            assert!(
                (sim_t90 - t90).abs() / t90 < 0.15,
                "{} t90 {sim_t90:.0} s vs measured {t90:.0} s",
                zone.name()
            );
        }
    }

    /// Symptom 1: middle overshoots roughly twice as hard as its neighbours,
    /// because it is the only barrel zone with no cold sink to bleed into.
    #[test]
    fn middle_overshoots_hardest() {
        let trace = recorded_run_trace();
        let (front, middle, back) = (
            trace.overshoot_k(Zone::Front),
            trace.overshoot_k(Zone::Middle),
            trace.overshoot_k(Zone::Back),
        );
        assert!(
            middle > front && middle > back,
            "middle {middle:+.1} K should exceed front {front:+.1} K and back {back:+.1} K"
        );
        assert!(
            middle > 20.0,
            "middle overshoot only {middle:+.1} K; the machine does about +31 K"
        );
    }

    /// Symptom 2: the nozzle never arrives. With `ki = 0` the loop is pure
    /// proportional, so it parks at a droop of `duty / kp` below setpoint —
    /// about 4 K on the real machine, permanently.
    #[test]
    fn nozzle_never_reaches_setpoint_and_is_far_slower() {
        let trace = recorded_run_trace();
        let droop = trace.overshoot_k(Zone::Nozzle);
        assert!(
            droop < 0.0,
            "nozzle should sit below setpoint with ki = 0, got {droop:+.1} K"
        );
        let nozzle = trace
            .rise_time_s(Zone::Nozzle, 0.9)
            .expect("nozzle reaches 90 % of setpoint eventually");
        let front = trace
            .rise_time_s(Zone::Front, 0.9)
            .expect("front reaches 90 % of setpoint");
        assert!(
            nozzle > 2.0 * front,
            "nozzle t90 {nozzle:.0} s should be far slower than front's {front:.0} s"
        );
    }

    /// Symptom 3: the overshoot is stored energy, not a control-loop artefact —
    /// the band keeps discharging into the steel long after the relay opens.
    #[test]
    fn zones_keep_climbing_after_their_relays_open() {
        let trace = recorded_run_trace();
        let p = Zone::Middle.port();
        let shutoff = trace
            .samples
            .iter()
            .position(|s| s.t_s > 300.0 && s.power_w[p] == 0.0)
            .expect("middle should shut off during the run");
        let peak = trace
            .samples
            .iter()
            .enumerate()
            .skip(shutoff)
            .max_by(|a, b| a.1.sensor_c[p].total_cmp(&b.1.sensor_c[p]))
            .expect("there are samples after shutoff");
        let coast_k = peak.1.sensor_c[p] - trace.samples[shutoff].sensor_c[p];
        let coast_s = peak.1.t_s - trace.samples[shutoff].t_s;
        assert!(
            coast_k > 5.0 && coast_s > 100.0,
            "middle coasted only {coast_k:.1} K over {coast_s:.0} s after shutoff; \
             the machine does about +26 K over 360 s"
        );
    }

    /// Uncalibrated first-principles values get the masses and the steady state
    /// right but produce essentially no overshoot. Recorded so nobody mistakes
    /// them for the calibrated set.
    #[test]
    fn uncalibrated_parameters_miss_the_overshoot() {
        let mut sim = ThermalSim::new(
            ExtruderThermalParams::first_principles(),
            SimConfig::default(),
        );
        let trace = sim.run(&Scenario::recorded_heatup());
        assert!(
            trace.overshoot_k(Zone::Middle) < 5.0,
            "first-principles parameters unexpectedly overshoot by {:+.1} K",
            trace.overshoot_k(Zone::Middle)
        );
    }

    /// A full hour of machine time has to run fast enough to be worth using.
    #[test]
    fn an_hour_of_heat_up_simulates_in_a_few_seconds() {
        let started = std::time::Instant::now();
        let trace = recorded_run_trace();
        let elapsed = started.elapsed().as_secs_f64();
        assert!(!trace.samples.is_empty());
        assert!(
            elapsed < 60.0,
            "3270 s of machine time took {elapsed:.1} s; that is too slow to iterate with"
        );
    }

    /// Not an assertion — a look at the step response.
    ///
    /// `cargo test -p machine_implementations plot_heat_up -- --nocapture`
    ///
    /// Follows the same `textplots` idiom as
    /// `control_core::helpers::interpolation`'s graph tests.
    #[test]
    fn plot_heat_up() {
        use textplots::{Chart, Plot, Shape};

        let trace = recorded_run_trace();
        let run = super::super::fit::RecordedRun::reference();

        for zone in Zone::ALL {
            let p = zone.port();
            let (peak, t90) = MEASURED[p];
            println!(
                "\n=== {} — setpoint {:.0} C ===\n  simulated: peak {:.1}  t90 {:>4}\n  \
                 measured:  peak {peak:.1}  t90 {t90:.0} s",
                zone.name(),
                trace.setpoints_c[p],
                trace.peak_c(zone),
                trace
                    .rise_time_s(zone, 0.9)
                    .map_or("never".to_owned(), |v| format!("{v:.0} s")),
            );
            let sim = trace.sensor_series(zone);
            let real: Vec<(f32, f32)> = run
                .t_s
                .iter()
                .zip(&run.temperature_c)
                .map(|(t, r)| (*t as f32, r[p] as f32))
                .collect();
            println!("  simulated:");
            Chart::new(120, 50, 0.0, 3270.0)
                .lineplot(&Shape::Lines(&sim))
                .display();
            println!("  measured:");
            Chart::new(120, 50, 0.0, 3270.0)
                .lineplot(&Shape::Lines(&real))
                .display();
        }
    }

    /// Energy must be accumulated during the run, not integrated from the 1 Hz
    /// trace. 1 s is exactly two 500 ms PWM windows, so sampling the relay at
    /// 1 Hz always lands at the same phase — that once made a zone drawing real
    /// power report 0.000 kWh.
    #[test]
    fn delivered_energy_is_not_aliased_away() {
        let trace = recorded_run_trace();
        for zone in Zone::ALL {
            let kwh = trace.energy_kwh(zone);
            assert!(
                kwh > 0.02 && kwh < 1.0,
                "{} delivered {kwh:.4} kWh over the run, which is not plausible",
                zone.name()
            );
        }
        // Sanity against physics: back feeds the gearbox sink and so must use
        // more than middle, which its neighbours keep warm for free.
        assert!(trace.energy_kwh(Zone::Back) > trace.energy_kwh(Zone::Middle));
    }

    /// The run loop must stay correct when the controller period is *longer*
    /// than the plant step — the regime you want when investigating a fixed PID
    /// sample time. It once let simulated time race ahead of the plant, so
    /// nothing heated at all.
    #[test]
    fn a_controller_slower_than_the_plant_step_still_heats() {
        let config = SimConfig {
            dt_ctrl_s: 0.25,
            ..SimConfig::default()
        };
        assert!(config.dt_ctrl_s > config.dt_plant_s);
        let mut sim = ThermalSim::new(ExtruderThermalParams::calibrated(), config);
        let trace = sim.run(&Scenario::cold_start());
        for zone in [Zone::Front, Zone::Middle, Zone::Back] {
            assert!(
                trace.peak_c(zone) > 150.0,
                "{} only reached {:.1} C with a 250 ms controller period",
                zone.name(),
                trace.peak_c(zone)
            );
            assert!(trace.energy_kwh(zone) > 0.02);
        }
    }

    /// The plant family only means anything if every member really is an
    /// equally good reading of the recording. A variant that has drifted away
    /// from it is not an alternative hypothesis, it is just a wrong model, and
    /// scoring controllers against it would be inventing a robustness
    /// requirement rather than discovering one.
    ///
    /// Tolerances are looser than
    /// [`calibrated_model_matches_the_recorded_heat_up`] — these are not refits,
    /// only the nominal fit with lag moved between the two mechanisms — but
    /// tight enough that a variant which stops reproducing the machine is
    /// caught.
    #[test]
    fn family_members_all_match_the_recording() {
        for params in plant_family() {
            let tau = params.sensor_tau_s;
            let config = SimConfig {
                strategy: StrategyConfig::Pid([RECORDED_RUN_TUNING; 4]),
                ..SimConfig::default()
            };
            let mut sim = ThermalSim::new(params, config);
            let trace = sim.run(&Scenario::recorded_heatup());

            for zone in Zone::ALL {
                let (peak, t90) = MEASURED[zone.port()];
                let sim_peak = trace.peak_c(zone);
                assert!(
                    (sim_peak - peak).abs() < 12.0,
                    "tau={tau:.0}: {} peaks at {sim_peak:.1} C vs the recorded {peak:.1} C, \
                     so this is not a plant the recording supports",
                    zone.name()
                );
                let sim_t90 = trace
                    .rise_time_s(zone, 0.9)
                    .unwrap_or_else(|| panic!("tau={tau:.0}: {} never reached 90 %", zone.name()));
                assert!(
                    (sim_t90 - t90).abs() / t90 < 0.25,
                    "tau={tau:.0}: {} t90 {sim_t90:.0} s vs the recorded {t90:.0} s",
                    zone.name()
                );
            }
        }
    }

    /// The family has to actually span the two competing explanations, or it is
    /// only testing one hypothesis dressed up as four.
    #[test]
    fn the_family_spans_both_overshoot_mechanisms() {
        let family = plant_family();
        assert!(
            family.len() >= 3,
            "a family of {} is not a spread",
            family.len()
        );

        let lag_dominated = family
            .iter()
            .find(|p| p.sensor_tau_s > 120.0)
            .expect("family should contain a measurement-lag-dominated plant");
        let storage_dominated = family
            .iter()
            .find(|p| p.sensor_tau_s < 40.0)
            .expect("family should contain a band-storage-dominated plant");

        assert!(
            storage_dominated.band_heat_capacity_j_per_m2_k
                > 5.0 * lag_dominated.band_heat_capacity_j_per_m2_k,
            "the two ends should disagree about band capacity by a lot; got {} vs {}",
            storage_dominated.band_heat_capacity_j_per_m2_k,
            lag_dominated.band_heat_capacity_j_per_m2_k
        );
    }

    /// Mean sensor reading over a time window, for judging settled behaviour.
    fn mean_between(trace: &Trace, t0: f64, t1: f64, zone: Zone) -> f64 {
        let vals: Vec<f64> = trace
            .samples
            .iter()
            .filter(|s| s.t_s >= t0 && s.t_s < t1)
            .map(|s| s.sensor_c[zone.port()])
            .collect();
        vals.iter().sum::<f64>() / vals.len() as f64
    }

    /// Build a sim with the melt modelled and the shipping controller.
    fn extruding_sim() -> ThermalSim {
        let mut params = ExtruderThermalParams::calibrated();
        params.melt.enabled = true;
        let config = SimConfig {
            strategy: StrategyConfig::ObserverPi(
                crate::extruder1::heating_params::observer_pi_params(),
            ),
            ..SimConfig::default()
        };
        ThermalSim::new(params, config)
    }

    /// **The test this whole model was built for.** The shipping controller was
    /// tuned on a heat-up with the screw stopped; here it has to hold setpoint
    /// while cold polymer starts moving through the barrel.
    ///
    /// Shear is switched off so that this is the *cooling* load alone — the
    /// direction the controller can actually do something about, by opening the
    /// heaters further. See
    /// [`the_machine_has_no_answer_to_shear_heat`] for the other direction.
    ///
    /// The bounds are deliberately loose: the melt coefficients have never been
    /// fitted to this machine, so this asserts the shape of the response — there
    /// is a disturbance, and it is absorbed — rather than numbers that would only
    /// be as trustworthy as `MeltParams`.
    #[test]
    fn starting_extrusion_is_a_load_step_the_controller_rejects() {
        let mut params = ExtruderThermalParams::calibrated();
        params.melt.enabled = true;
        params.melt.specific_mechanical_energy_kwh_per_kg = 0.0;
        let config = SimConfig {
            strategy: StrategyConfig::ObserverPi(
                crate::extruder1::heating_params::observer_pi_params(),
            ),
            ..SimConfig::default()
        };
        let trace = ThermalSim::new(params, config).run(&Scenario::extrude_start());

        for zone in Zone::ALL {
            let sp = trace.setpoints_c[zone.port()];
            let before = mean_between(&trace, 3300.0, 3600.0, zone);
            let after = mean_between(&trace, 8100.0, f64::MAX, zone);

            assert!(
                (before - sp).abs() < 3.0,
                "{} had not settled before the load step: {before:.1} vs {sp:.1}",
                zone.name()
            );
            assert!(
                (after - sp).abs() < 8.0,
                "{} never recovered after extrusion started: {after:.1} vs {sp:.1}",
                zone.name()
            );
        }
    }

    /// **A finding, not a controller defect.** With the screw's mechanical work
    /// included, extruding at 60 rpm drives the front of the barrel well above
    /// setpoint and nothing in the machine can bring it back: the loops can only
    /// stop heating, and there is no barrel cooling to call on. Real extruders
    /// carry cooling fans for exactly this.
    ///
    /// Asserted so it cannot quietly disappear — but note the size of it rests
    /// on `specific_mechanical_energy_kwh_per_kg`, which has never been measured
    /// here and is uncertain by a factor of two. Treat this as "the model says
    /// this is worth measuring", not "the machine does this".
    #[test]
    fn the_machine_has_no_answer_to_shear_heat() {
        let trace = extruding_sim().run(&Scenario::extrude_start());

        let sp = trace.setpoints_c[Zone::Front.port()];
        let after = mean_between(&trace, 8100.0, f64::MAX, Zone::Front);
        assert!(
            after > sp + 10.0,
            "front settled at {after:.1} against a {sp:.1} setpoint; if shear no \
             longer overheats it, the melt parameters have changed"
        );

        // And the controller is doing the only thing it can: nothing.
        let duty = trace
            .samples
            .iter()
            .filter(|s| s.t_s > 8100.0)
            .map(|s| s.duty[Zone::Front.port()])
            .fold(0.0_f64, f64::max);
        assert!(
            duty < 0.05,
            "front duty {duty:.3} — the loop should be shut off, not fighting"
        );
    }

    /// Material really does move, and the trace really does record it.
    #[test]
    fn an_extrusion_run_records_what_the_material_did() {
        let trace = extruding_sim().run(&Scenario::extrude_start());

        let last = trace.samples.last().expect("samples");
        assert!((last.screw_rpm - 60.0).abs() < 1e-9);
        assert!((last.throughput_kg_h - 6.0).abs() < 1e-9);
        assert!(
            last.melt_out_w > 0.0,
            "material should be carrying heat out"
        );
        assert!(last.shear_w > 0.0, "the screw should be doing work");
        assert!(
            trace.melt_energy_kwh() > 0.0 && trace.shear_energy_kwh() > 0.0,
            "run energies should accumulate"
        );

        // Nothing before the step, everything after it.
        let before = trace
            .samples
            .iter()
            .find(|s| s.t_s > 1800.0)
            .expect("a sample before the step");
        assert!(before.screw_rpm.abs() < 1e-12);
        assert!(before.melt_out_w.abs() < 1e-9);
    }

    /// The CSV has to carry the new columns, and `scripts/plot-heating.html`
    /// groups per-zone series by a `<metric>_<zone>` suffix — so the names
    /// matter, not just the values.
    #[test]
    fn the_csv_carries_the_melt_columns() {
        let trace = extruding_sim().run(&Scenario::extrude_start());
        let csv = trace.to_csv();
        let header = csv.lines().next().expect("a header");

        for zone in Zone::ALL {
            assert!(header.contains(&format!("melt_{}", zone.name())));
            assert!(header.contains(&format!("meltload_w_{}", zone.name())));
        }
        for tag in ["screw_rpm", "throughput_kg_h", "melt_out_w", "shear_w"] {
            assert!(header.contains(tag), "missing column {tag}");
        }
        // Every row must have exactly as many fields as the header promises.
        let want = header.split(',').count();
        for line in csv.lines().skip(1) {
            assert_eq!(line.split(',').count(), want);
        }
    }

    /// A scenario that turns the screw against a model without the melt would
    /// silently do nothing, which is worse than failing.
    #[test]
    #[should_panic(expected = "turns the screw")]
    fn extruding_without_a_melt_model_is_refused() {
        let mut sim = ThermalSim::new(ExtruderThermalParams::calibrated(), SimConfig::default());
        let _ = sim.run(&Scenario::extrude_start());
    }

    #[test]
    fn relays_start_open_and_the_plant_stays_cold_without_heating() {
        let mut sim = ThermalSim::new(ExtruderThermalParams::default(), SimConfig::default());
        let scenario = Scenario {
            name: "no-heat".into(),
            initial_c: 22.0,
            duration_s: 60.0,
            setpoints_c: [180.0, 180.0, 170.0, 175.0],
            heating_enabled_at_s: f64::INFINITY,
            ..Scenario::default()
        };
        let trace = sim.run(&scenario);
        for zone in Zone::ALL {
            assert!(
                (trace.final_c(zone) - 22.0).abs() < 0.2,
                "{} drifted to {:.2} with heating disabled",
                zone.name(),
                trace.final_c(zone)
            );
        }
    }
}
