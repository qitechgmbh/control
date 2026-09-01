//! Offline thermal simulation of the extruder's four heating zones.
//!
//! This exists so heating behaviour can be studied and tuned without the
//! machine. A cold-start-to-180 °C run that takes an hour on the real extruder
//! finishes here in a couple of seconds, and it is deterministic, so a change in
//! gains produces a comparable change in the trace.
//!
//! It models the **newer extruder** (`MACHINE_EXTRUDER_V2`, the "V3" role
//! layout) — the machine the CAD model `Oberbau_UBG.step` and the recording in
//! [`data`](self#calibration-data) both come from.
//!
//! # What is simulated and what is real
//!
//! The *plant* is simulated: [`model::ExtruderThermalModel`] is a 1-D axial
//! network of barrel steel, screw, band heaters, insulation and RTD pockets,
//! built from [`geometry`] constants measured off the CAD.
//!
//! The *controller is the shipping code*. [`harness::ThermalSim`] constructs the
//! production [`crate::extruder1::temperature_controller::TemperatureController`]
//! and drives it with a real `EL3204` and `EL2004`, feeding the sensor in as raw
//! PDO bytes. So the control law, the duty clamp, the 500 ms slow-PWM window and
//! the 0.1 °C quantisation are not reimplemented here — a fix to the controller
//! shows up in the simulation immediately, which is the whole point.
//!
//! Which control law runs is [`harness::StrategyConfig`], so alternatives can be
//! compared against the same plant without either of them being a mock-up. The
//! three that exist live in `control_core::controllers::heating`.
//!
//! Simulated time is synthetic `Instant`s, so the run is decoupled from the wall
//! clock. Nothing here touches the `mock-machine` feature.
//!
//! # Quick start
//!
//! ```no_run
//! use machine_implementations::extruder1::simulation::{
//!     harness::{SimConfig, ThermalSim},
//!     params::ExtruderThermalParams,
//!     scenario::Scenario,
//!     geometry::Zone,
//! };
//!
//! let mut sim = ThermalSim::new(ExtruderThermalParams::calibrated(), SimConfig::default());
//! let trace = sim.run(&Scenario::cold_start());
//! println!("middle overshoot: {:.1} K", trace.overshoot_k(Zone::Middle));
//! ```
//!
//! Or from the shell:
//!
//! ```text
//! cargo run --release -p machine_implementations --example extruder_thermal_sim -- \
//!     --scenario cold-start --out /tmp/run.csv
//! ```
//!
//! # Calibration data
//!
//! `data/heatup_2026-02-24.csv` is a real hour-long heat-up exported from the
//! machine: per-zone temperature and PID duty at 1 Hz, from 22 °C with setpoints
//! front 180, middle 180, back 170, nozzle 175 °C.
//!
//! Note on that file: the firmware of the day reported power as
//! `duty * heating_element_wattage` with the **wrong** wattages configured
//! (900 W barrel, 150 W nozzle; the hardware is 700 W and 200 W). The CSV
//! therefore stores recovered *duty*, not watts, and the model applies the real
//! ratings from [`geometry::Zone::band`]. If you re-export from the machine,
//! check that constant first — `scripts/record-extruder.mjs` writes this format.
//!
//! Run `--fit` to refit [`params::ExtruderThermalParams`] against it; see
//! [`fit`].
//!
//! # What the recording shows
//!
//! Measured against those setpoints, the shipping gains
//! (`kp = 0.16, ki = 0, kd = 0.008`, identical on all four zones) give:
//!
//! | zone | peak | overshoot | t90 | settles at |
//! |---|---|---|---|---|
//! | front | 194.4 | +14.4 K | 703 s | 181.0 |
//! | middle | 211.5 | **+31.5 K** | 674 s | 185.6 (still falling) |
//! | back | 186.6 | +16.6 K | 661 s | 168.4 |
//! | nozzle | 171.2 | **−3.8 K** (never arrives) | 1849 s | 171.2 |
//!
//! The three effects the simulation has to reproduce, and why they happen:
//!
//! - **Middle overshoots twice as hard.** It is flanked by heated zones, while
//!   front bleeds into the cold Düse across the flange and back bleeds into the
//!   231 mm unheated tail and the gearbox. Same 700 W, same gains, much less
//!   escape.
//! - **Overshoot at all.** Two mechanisms, roughly comparable in size. The bands
//!   store energy and keep discharging into the steel after the relay opens —
//!   with every barrel zone at 0 W from ~790 s, middle still climbs another
//!   26.5 K over the next 360 s. And the RTDs lag the steel badly (see
//!   [`params::ExtruderThermalParams::sensor_tau_s`]), so the controller is
//!   still driving after the steel has passed setpoint.
//! - **The nozzle never reaches setpoint.** With `ki = 0` the loop is pure
//!   proportional, so it parks at a droop of `duty / kp`; at ~97 W of a 200 W
//!   band that is 3.8 K short, forever. On the way up it is also clamped at
//!   `max_clamp = 0.95` for half an hour, because it is ~6 kg of uninsulated
//!   steel on a 34 mm band.
//!
//! # How accurate is it
//!
//! With [`params::ExtruderThermalParams::calibrated`], closed loop against that
//! recording:
//!
//! | zone | peak sim / real | t90 sim / real |
//! |---|---|---|
//! | front | 195.2 / 194.4 | 695 / 703 s |
//! | middle | 211.7 / 211.5 | 637 / 674 s |
//! | back | 187.7 / 186.6 | 651 / 661 s |
//! | nozzle | 172.3 / 171.2 | 1875 / 1849 s |
//!
//! Open-loop replay RMS over the hour is ~5.7 K. Asserted by the tests in
//! [`harness`].
//!
//! **What this does and does not license.** The model reproduces the machine's
//! *behaviour* well, so it is a sound rig for comparing control strategies —
//! which is what it is for. The individual coefficients in [`params`] are a
//! different matter: one run with all four zones heating together cannot
//! identify nine of them, and three sit on a bound (see
//! [`params::ExtruderThermalParams::EXPECTED_PINNED`]). Do not quote them as
//! measurements. Recording the `single-*` scenarios in [`scenario`] on the real
//! machine — one zone from cold, then decay — would separate them.
//!
//! # Controller problems this rig exposed, and what happened to them
//!
//! All of these were found here rather than on the machine, and all are now
//! fixed. Kept as a record of what the rig is for.
//!
//! 1. **The derivative term was quantisation noise.** `update` runs every ~1 ms
//!    on a value the EL3204 only refreshes every few tens of ms in 0.1 °C steps.
//!    When it did move, `ed = 0.1 / 0.001 = 100 K/s`, and `kd * ed = 0.8` — most
//!    of the duty range, from one LSB. *Fixed* by never differentiating the raw
//!    reading: `SensorLagObserver` low-passes first and takes the slope of the
//!    filtered signal, which needs no division by `dt` at all.
//! 2. **No anti-windup.** *Fixed* previously by
//!    `PidController::update_with_antiwindup`, and the feedforward now means the
//!    integral is only ever carrying a small residual anyway.
//! 3. **One set of gains for four very different plants.** *Fixed*: every
//!    parameter is per zone, in [`tuning`].
//! 4. **PWM window off-by-one.** `elapsed` was not recomputed after
//!    `window_start = now`, so the relay was forced off for one tick per window.
//!    *Fixed* in `TemperatureController::update`.
//! 5. **A failed sensor read as 0 °C**, which is maximum error, which is full
//!    heat demand — the heater running flat out exactly when nothing could see
//!    how hot it was getting. *Fixed*: `wiring_error` now opens the relay.
//!
//! # The overshoot, and what actually fixed it
//!
//! Not the gains. The RTDs have a time constant of order **150 s**, and the
//! barrel ramps at ~0.23 K/s, so the controller spent the whole heat-up reading
//! a value about **34 K stale** and shut off that far late. That single number
//! is essentially the entire +31.5 K middle-zone overshoot, and no PID on the
//! raw signal can remove it, because from inside the loop a stale reading is
//! indistinguishable from being genuinely that far away.
//!
//! What ships now is [`control_core::controllers::heating::ObserverPi`]:
//! reconstruct the steel temperature from the reading and its filtered slope,
//! and regulate *that*, over a feedforward that already knows the duty the
//! setpoint costs. Measured here, across the whole of
//! [`harness::plant_family`]:
//!
//! | zone | PID overshoot | ObserverPi | PID settle | ObserverPi |
//! |---|---|---|---|---|
//! | front | +8.8 K | +0.3 K | 1445 s | 808 s |
//! | middle | **+15.8 K** | **+0.4 K** | 2073 s | 846 s |
//! | back | +6.5 K | +0.1 K | 1040 s | 672 s |
//! | nozzle | +0.1 K (never arrives) | +0.2 K (arrives) | 2131 s | 2105 s |
//!
//! Compare strategies with the `bench_heating` example; the shipping
//! configuration is asserted by the tests in [`tuning`].
//!
//! **Still worth doing on the machine.** A 150 s probe lag is what one sitting
//! in an air gap does, not one that is properly seated. Fixing the seating with
//! heat-transfer compound would remove the *cause* rather than compensating for
//! it, and would make everything here work better. See
//! [`params::ExtruderThermalParams::sensor_tau_s`].
//!
//! # Relay autotuning
//!
//! `control_core`'s [`PidAutoTuner`](control_core::controllers::pid_autotuner::PidAutoTuner)
//! already exists but is only wired to pressure. [`harness::ThermalSim::run_autotune`]
//! points it at a simulated zone, so temperature autotuning can be developed
//! without tying up the machine for an hour per attempt:
//!
//! ```text
//! cargo run --release -p machine_implementations \
//!     --example extruder_thermal_sim -- --autotune middle
//! ```

pub mod fit;
pub mod geometry;
pub mod harness;
pub mod model;
pub mod params;
pub mod scenario;
pub mod tuning;

pub use geometry::Zone;
pub use harness::{SimConfig, StrategyConfig, ThermalSim, Trace, ZoneTuning};
pub use model::ExtruderThermalModel;
pub use params::ExtruderThermalParams;
pub use scenario::Scenario;
