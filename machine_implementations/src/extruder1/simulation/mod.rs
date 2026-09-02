//! Offline thermal simulation of the extruder's four heating zones.
//!
//! Heating behaviour can be studied and tuned without the machine: a
//! cold-start-to-180 °C run that takes an hour on the real extruder finishes
//! here in a couple of seconds, deterministically. It models the newer extruder
//! (`MACHINE_EXTRUDER_V2`), the machine the CAD model and the reference
//! recording both come from.
//!
//! The *plant* is simulated — [`model::ExtruderThermalModel`] is a 1-D axial
//! network of barrel steel, screw, band heaters, insulation and RTD pockets,
//! built from [`geometry`] constants measured off the CAD. The *controller is
//! the shipping code*: [`harness::ThermalSim`] constructs the production
//! [`TemperatureController`](crate::extruder1::temperature_controller::TemperatureController)
//! and drives it through a real `EL3204` and `EL2004`, feeding the sensor in as
//! raw PDO bytes, so the control law, the duty clamp, the 500 ms PWM window and
//! the 0.1 °C quantisation are not reimplemented here.
//!
//! **See `README.md` next to this file** for the calibration data, how accurate
//! the model is, what it says about the machine, and the controller defects it
//! turned up.
//!
//! ```no_run
//! use machine_implementations::extruder1::simulation::{
//!     harness::{SimConfig, ThermalSim},
//!     params::ExtruderThermalParams,
//!     scenario::Scenario,
//! };
//! use machine_implementations::extruder1::zone::Zone;
//!
//! let mut sim = ThermalSim::new(ExtruderThermalParams::calibrated(), SimConfig::default());
//! let trace = sim.run(&Scenario::cold_start());
//! println!("middle overshoot: {:.1} K", trace.overshoot_k(Zone::Middle));
//! ```
//!
//! Or from the shell:
//!
//! ```text
//! cargo run --release -p machine_implementations --features simulation \
//!     --example extruder_thermal_sim -- --scenario cold-start --out /tmp/run.csv
//! ```

pub mod fit;
pub mod geometry;
pub mod harness;
pub mod model;
pub mod optimize;
pub mod params;
pub mod scenario;
mod shipping;

pub use crate::extruder1::zone::Zone;
pub use harness::{SimConfig, StrategyConfig, ThermalSim, Trace, ZoneTuning};
pub use model::ExtruderThermalModel;
pub use params::ExtruderThermalParams;
pub use scenario::Scenario;
