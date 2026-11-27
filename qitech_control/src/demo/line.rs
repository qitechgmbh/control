//! The demo production line: which machines it has, the settings they start with and
//! the shape of every measurement.
//!
//! A line is the extruder, the water bath (aquapath), the diameter laser and the winder,
//! the same machine types and schemas as on a real line, so every measurement the real
//! machines export at 32 Hz is exported here too.

use qitech_framework::MachineIdentification;
use qitech_framework::MachineInstanceIdentification;
use qitech_framework::machine::MachineDescriptor;

use crate::demo::signal::Base;
use crate::demo::signal::Signal;
use crate::machines::ExtruderV2;
use crate::machines::LaserV1;
use crate::machines::WinderV1_Regular;
use crate::machines::aquapath::AquapathV1;

/// Serial of every demo machine. High enough to never clash with a real EEPROM serial.
pub const DEMO_SERIAL: u16 = 64001;

/// A value a demo config or state property starts with. Properties without one get a
/// neutral default from their schema type (0, false, first enum variant, null).
#[derive(Debug, Clone, Copy)]
pub enum Initial {
    Float(f64),
    Bool(bool),
    Enum(&'static str),
}

pub struct DemoMachine {
    pub machine: MachineIdentification,
    /// The machine type's schema (YAML).
    #[allow(dead_code)] // read by the tests
    pub schema: &'static str,
    pub config: &'static [(&'static str, Initial)],
    pub state: &'static [(&'static str, Initial)],
    pub measurements: &'static [(&'static str, Signal)],
}

impl DemoMachine {
    pub const fn ident(&self) -> MachineInstanceIdentification {
        MachineInstanceIdentification::from_ident(self.machine, DEMO_SERIAL)
    }

    pub fn config_initial(&self, path: &str) -> Option<Initial> {
        self.config
            .iter()
            .find(|(p, _)| *p == path)
            .map(|(_, v)| *v)
    }

    pub fn state_initial(&self, path: &str) -> Option<Initial> {
        self.state.iter().find(|(p, _)| *p == path).map(|(_, v)| *v)
    }

    /// Default of a config property as a number, for signals that follow a target.
    pub fn config_float(&self, path: &str) -> f64 {
        match self.config_initial(path) {
            Some(Initial::Float(v)) => v,
            Some(Initial::Bool(b)) => b as u8 as f64,
            _ => 0.0,
        }
    }
}

/// The machines of one demo line.
pub fn machines() -> &'static [DemoMachine] {
    &LINE
}

static LINE: [DemoMachine; 4] = [EXTRUDER, AQUAPATH, LASER, WINDER];

// --- shorthands ---
const MIN: f64 = 60.0;
const HOUR: f64 = 3600.0;

const fn wave(base: f64, slow: (f64, f64), fast: (f64, f64), noise: f64) -> Signal {
    Signal::Wave {
        base: Base::Const(base),
        slow,
        fast,
        noise,
    }
}

/// Oscillates around the current value of a config property (e.g. a target temperature).
const fn follow(
    path: &'static str,
    offset: f64,
    slow: (f64, f64),
    fast: (f64, f64),
    noise: f64,
) -> Signal {
    Signal::Wave {
        base: Base::Config { path, offset },
        slow,
        fast,
        noise,
    }
}

// --- extruder (ExtruderV2, UI "extruder3") ---
const EXTRUDER: DemoMachine = DemoMachine {
    machine: ExtruderV2::IDENTIFICATION,
    schema: ExtruderV2::SCHEMA,
    config: &[
        ("screw.direction", Initial::Enum("forward")),
        ("screw.regulation", Initial::Enum("rpm")),
        ("screw.target_rpm", Initial::Float(42.0)),
        ("screw.target_pressure", Initial::Float(85.0)),
        ("extrusion.min_temperature", Initial::Float(160.0)),
        ("pressure.limit", Initial::Float(250.0)),
        ("pressure.limit_enabled", Initial::Bool(true)),
        ("pressure.autotune.tune_delta", Initial::Float(5.0)),
        ("pressure.autotune.frequency_step", Initial::Float(1.0)),
        ("heating.algorithm", Initial::Enum("pid")),
        ("heating.nozzle.target_temperature", Initial::Float(215.0)),
        ("heating.nozzle.target_enabled", Initial::Bool(true)),
        ("heating.front.target_temperature", Initial::Float(205.0)),
        ("heating.middle.target_temperature", Initial::Float(195.0)),
        ("heating.back.target_temperature", Initial::Float(185.0)),
        ("pid.pressure.kp", Initial::Float(0.04)),
        ("pid.pressure.ki", Initial::Float(0.002)),
        ("pid.pressure.kd", Initial::Float(0.0)),
        ("pid.temperature.nozzle.kp", Initial::Float(0.16)),
        ("pid.temperature.nozzle.ki", Initial::Float(0.0)),
        ("pid.temperature.nozzle.kd", Initial::Float(0.008)),
        ("pid.temperature.front.kp", Initial::Float(0.16)),
        ("pid.temperature.front.ki", Initial::Float(0.0)),
        ("pid.temperature.front.kd", Initial::Float(0.008)),
        ("pid.temperature.middle.kp", Initial::Float(0.16)),
        ("pid.temperature.middle.ki", Initial::Float(0.0)),
        ("pid.temperature.middle.kd", Initial::Float(0.008)),
        ("pid.temperature.back.kp", Initial::Float(0.16)),
        ("pid.temperature.back.ki", Initial::Float(0.0)),
        ("pid.temperature.back.kd", Initial::Float(0.008)),
    ],
    state: &[
        ("mode", Initial::Enum("extrude")),
        ("pressure.autotune.state", Initial::Enum("not_started")),
        ("inverter.running", Initial::Bool(true)),
        ("inverter.forward_running", Initial::Bool(true)),
        ("inverter.up_to_frequency", Initial::Bool(true)),
    ],
    measurements: &[
        (
            "motor.rpm",
            follow("screw.target_rpm", 0.0, (0.6, 20.0 * MIN), (0.3, 7.0), 0.15),
        ),
        (
            "motor.frequency",
            wave(28.0, (0.4, 20.0 * MIN), (0.2, 7.0), 0.05),
        ),
        (
            "motor.voltage",
            wave(230.0, (2.0, 3.0 * HOUR), (0.5, 50.0), 0.4),
        ),
        (
            "motor.current",
            wave(6.5, (0.4, 40.0 * MIN), (0.2, 9.0), 0.08),
        ),
        (
            "motor.power",
            wave(1450.0, (90.0, 40.0 * MIN), (35.0, 9.0), 12.0),
        ),
        (
            "pressure.value",
            follow(
                "screw.target_pressure",
                0.0,
                (6.0, 25.0 * MIN),
                (2.5, 11.0),
                0.8,
            ),
        ),
        (
            "pressure.autotune_progress",
            wave(0.0, (0.0, 1.0), (0.0, 1.0), 0.0),
        ),
        (
            "heating.nozzle.temperature",
            follow(
                "heating.nozzle.target_temperature",
                0.0,
                (1.2, 15.0 * MIN),
                (0.4, 50.0),
                0.15,
            ),
        ),
        (
            "heating.nozzle.power",
            wave(380.0, (140.0, 15.0 * MIN), (60.0, 50.0), 25.0),
        ),
        (
            "heating.front.temperature",
            follow(
                "heating.front.target_temperature",
                0.0,
                (1.5, 18.0 * MIN),
                (0.5, 60.0),
                0.15,
            ),
        ),
        (
            "heating.front.power",
            wave(520.0, (160.0, 18.0 * MIN), (70.0, 60.0), 25.0),
        ),
        (
            "heating.middle.temperature",
            follow(
                "heating.middle.target_temperature",
                0.0,
                (1.4, 21.0 * MIN),
                (0.5, 70.0),
                0.15,
            ),
        ),
        (
            "heating.middle.power",
            wave(480.0, (150.0, 21.0 * MIN), (65.0, 70.0), 25.0),
        ),
        (
            "heating.back.temperature",
            follow(
                "heating.back.target_temperature",
                0.0,
                (1.6, 24.0 * MIN),
                (0.6, 80.0),
                0.15,
            ),
        ),
        (
            "heating.back.power",
            wave(450.0, (150.0, 24.0 * MIN), (60.0, 80.0), 25.0),
        ),
        (
            "power.combined",
            wave(3300.0, (260.0, 20.0 * MIN), (90.0, 30.0), 30.0),
        ),
        // 3.3 kW continuously
        (
            "energy.total",
            Signal::Ramp {
                rate: 3.3 / HOUR,
                wrap: 0.0,
            },
        ),
    ],
};

// --- water bath (AquapathV1, UI "aquapath1") ---
const AQUAPATH: DemoMachine = DemoMachine {
    machine: AquapathV1::IDENTIFICATION,
    schema: AquapathV1::SCHEMA,
    config: &[
        ("left_target_temperature", Initial::Float(24.0)),
        ("right_target_temperature", Initial::Float(18.0)),
        ("ambient_temperature_calibration", Initial::Float(0.0)),
        ("left_fan_max_revolutions", Initial::Float(100.0)),
        ("right_fan_max_revolutions", Initial::Float(100.0)),
        ("left_tolerance_config.heating", Initial::Float(0.4)),
        ("left_tolerance_config.cooling", Initial::Float(0.8)),
        ("right_tolerance_config.heating", Initial::Float(0.4)),
        ("right_tolerance_config.cooling", Initial::Float(0.8)),
        ("left_pid_config.kp", Initial::Float(0.16)),
        ("left_pid_config.ki", Initial::Float(0.0)),
        ("left_pid_config.kd", Initial::Float(0.008)),
        ("right_pid_config.kp", Initial::Float(0.16)),
        ("right_pid_config.ki", Initial::Float(0.0)),
        ("right_pid_config.kd", Initial::Float(0.008)),
        ("left_thermal_flow_settle_duration", Initial::Float(30.0)),
        ("right_thermal_flow_settle_duration", Initial::Float(30.0)),
        ("left_pump_cooldown_min_temperature", Initial::Float(40.0)),
        ("right_pump_cooldown_min_temperature", Initial::Float(40.0)),
    ],
    state: &[
        ("mode_state.mode", Initial::Enum("auto")),
        ("left_should_flow", Initial::Bool(true)),
        ("right_should_flow", Initial::Bool(true)),
        ("left_has_flow", Initial::Bool(true)),
        ("right_has_flow", Initial::Bool(true)),
        ("left_cooling_mode", Initial::Enum("ramp")),
        ("right_cooling_mode", Initial::Enum("ramp")),
        (
            "left_thermal_safety_state.thermal_delay",
            Initial::Float(30.0),
        ),
        (
            "left_thermal_safety_state.cooldown_min_temperature",
            Initial::Float(40.0),
        ),
        (
            "right_thermal_safety_state.thermal_delay",
            Initial::Float(30.0),
        ),
        (
            "right_thermal_safety_state.cooldown_min_temperature",
            Initial::Float(40.0),
        ),
    ],
    measurements: &[
        ("left_flow", wave(6.2, (0.3, 35.0 * MIN), (0.1, 13.0), 0.06)),
        (
            "right_flow",
            wave(5.8, (0.3, 32.0 * MIN), (0.1, 11.0), 0.06),
        ),
        (
            "left_temperature",
            follow(
                "left_target_temperature",
                0.0,
                (0.35, 25.0 * MIN),
                (0.1, 90.0),
                0.04,
            ),
        ),
        (
            "right_temperature",
            follow(
                "right_target_temperature",
                0.0,
                (0.35, 27.0 * MIN),
                (0.1, 85.0),
                0.04,
            ),
        ),
        (
            "left_revolutions",
            wave(55.0, (12.0, 25.0 * MIN), (4.0, 90.0), 1.0),
        ),
        (
            "right_revolutions",
            wave(62.0, (12.0, 27.0 * MIN), (4.0, 85.0), 1.0),
        ),
        (
            "left_power",
            wave(180.0, (40.0, 25.0 * MIN), (12.0, 90.0), 4.0),
        ),
        (
            "right_power",
            wave(210.0, (40.0, 27.0 * MIN), (12.0, 85.0), 4.0),
        ),
        (
            "left_total_energy",
            Signal::Ramp {
                rate: 0.18 / HOUR,
                wrap: 0.0,
            },
        ),
        (
            "right_total_energy",
            Signal::Ramp {
                rate: 0.21 / HOUR,
                wrap: 0.0,
            },
        ),
        (
            "left_pump_cooldown_remaining",
            wave(0.0, (0.0, 1.0), (0.0, 1.0), 0.0),
        ),
        (
            "right_pump_cooldown_remaining",
            wave(0.0, (0.0, 1.0), (0.0, 1.0), 0.0),
        ),
        (
            "left_heating_startup_wait_remaining",
            wave(0.0, (0.0, 1.0), (0.0, 1.0), 0.0),
        ),
        (
            "right_heating_startup_wait_remaining",
            wave(0.0, (0.0, 1.0), (0.0, 1.0), 0.0),
        ),
    ],
};

// --- diameter laser (LaserV1, UI "laser1") ---
const LASER: DemoMachine = DemoMachine {
    machine: LaserV1::IDENTIFICATION,
    schema: LaserV1::SCHEMA,
    config: &[
        ("diameter.target", Initial::Float(1.75)),
        ("diameter.tolerance.upper", Initial::Float(0.05)),
        ("diameter.tolerance.lower", Initial::Float(0.05)),
        ("global_warning", Initial::Bool(false)),
    ],
    state: &[("in_tolerance", Initial::Bool(true))],
    measurements: &[
        (
            "diameter",
            follow(
                "diameter.target",
                0.0,
                (0.012, 12.0 * MIN),
                (0.006, 4.0),
                0.004,
            ),
        ),
        (
            "diameter_x",
            follow(
                "diameter.target",
                0.004,
                (0.012, 12.0 * MIN),
                (0.007, 4.0),
                0.005,
            ),
        ),
        (
            "diameter_y",
            follow(
                "diameter.target",
                -0.004,
                (0.012, 12.0 * MIN),
                (0.007, 4.5),
                0.005,
            ),
        ),
        (
            "roundness",
            wave(0.985, (0.004, 12.0 * MIN), (0.002, 4.0), 0.001),
        ),
    ],
};

// --- winder (WinderV1_Regular, UI "winder2") ---
const WINDER: DemoMachine = DemoMachine {
    machine: WinderV1_Regular::IDENTIFICATION,
    schema: WinderV1_Regular::SCHEMA,
    config: &[
        ("spool.direction", Initial::Enum("forward")),
        (
            "spool.speed_controller.algorithm",
            Initial::Enum("adaptive"),
        ),
        ("spool.speed_controller.speed_min", Initial::Float(0.0)),
        ("spool.speed_controller.speed_max", Initial::Float(150.0)),
        (
            "spool.speed_controller.adaptive.tension_target",
            Initial::Float(0.7),
        ),
        (
            "spool.speed_controller.adaptive.radius_learning_rate",
            Initial::Float(0.5),
        ),
        (
            "spool.speed_controller.adaptive.max_speed_multiplier",
            Initial::Float(4.0),
        ),
        (
            "spool.speed_controller.adaptive.acceleration_factor",
            Initial::Float(0.2),
        ),
        (
            "spool.speed_controller.adaptive.deacceleration_urgency_multiplier",
            Initial::Float(15.0),
        ),
        ("puller.direction", Initial::Enum("forward")),
        ("puller.gear_ratio", Initial::Enum("one_to_one")),
        ("puller.speed_controller.algorithm", Initial::Enum("direct")),
        (
            "puller.speed_controller.speed_desired",
            Initial::Float(12.0),
        ),
        (
            "puller.speed_controller.adaptive.speed_delta_max",
            Initial::Float(10.0),
        ),
        (
            "puller.speed_controller.adaptive.increase_per_step",
            Initial::Float(0.5),
        ),
        (
            "puller.speed_controller.adaptive.tolerance_limit",
            Initial::Float(0.05),
        ),
        (
            "puller.speed_controller.adaptive.adjustment_distance",
            Initial::Float(1.0),
        ),
        ("traverse.limit_inner", Initial::Float(22.0)),
        ("traverse.limit_outer", Initial::Float(92.0)),
        ("traverse.step_size", Initial::Float(1.75)),
        ("traverse.padding", Initial::Float(0.88)),
        ("traverse.edge_dwell_revolutions", Initial::Float(0.0)),
        ("traverse.start_dwell_revolutions", Initial::Float(0.0)),
        ("spool_automatic.required_meters", Initial::Float(330.0)),
        ("spool_automatic.action", Initial::Enum("no_action")),
    ],
    state: &[
        ("mode", Initial::Enum("wind")),
        ("spool.mode", Initial::Enum("wind")),
        ("spool.speed_controller.enabled", Initial::Bool(true)),
        ("puller.mode", Initial::Enum("pull")),
        ("traverse.mode", Initial::Enum("traverse")),
        ("traverse.state", Initial::Enum("traversing_out")),
        ("traverse.homed", Initial::Bool(true)),
        ("tension_arm.zero", Initial::Float(0.0)),
    ],
    measurements: &[
        ("spool.rpm", wave(38.0, (9.0, 50.0 * MIN), (2.0, 6.0), 0.5)),
        // a 330 m spool at 12 m/min, then the next one
        (
            "spool.progress",
            Signal::Ramp {
                rate: 12.0 / MIN,
                wrap: 330.0,
            },
        ),
        (
            "spool.speed_controller.speed",
            wave(38.0, (9.0, 50.0 * MIN), (2.0, 6.0), 0.3),
        ),
        (
            "spool.speed_controller.filament_tension",
            follow(
                "spool.speed_controller.adaptive.tension_target",
                0.0,
                (0.03, 9.0 * MIN),
                (0.04, 3.0),
                0.02,
            ),
        ),
        (
            "spool.speed_controller.adaptive.speed_factor",
            wave(1.0, (0.05, 50.0 * MIN), (0.01, 6.0), 0.005),
        ),
        (
            "puller.speed",
            follow(
                "puller.speed_controller.speed_desired",
                0.0,
                (0.05, 10.0 * MIN),
                (0.03, 5.0),
                0.02,
            ),
        ),
        (
            "puller.speed_controller.speed_target",
            follow(
                "puller.speed_controller.speed_desired",
                0.0,
                (0.0, 1.0),
                (0.0, 1.0),
                0.0,
            ),
        ),
        (
            "puller.speed_controller.adaptive.modulation",
            wave(0.0, (0.02, 10.0 * MIN), (0.01, 5.0), 0.005),
        ),
        // the adaptive puller corrects every metre
        (
            "puller.speed_controller.adaptive.distance_since_adjustment",
            Signal::Ramp {
                rate: 12.0 / MIN,
                wrap: 1.0,
            },
        ),
        // one traverse pass across a 70 mm spool at 1.75 mm per spool revolution
        (
            "traverse.position",
            Signal::Triangle {
                lo: 22.0,
                hi: 92.0,
                period_s: 126.0,
                noise: 0.05,
            },
        ),
        (
            "tension_arm.angle",
            wave(42.0, (3.0, 9.0 * MIN), (2.5, 3.0), 0.6),
        ),
    ],
};
