pub mod act;
pub mod build;
pub mod mitsubishi_cs80;
pub mod screw_speed_controller;
pub mod temperature_controller;

use qitech_framework::EnumProperty;
use qitech_framework::MachineIdentification;
use qitech_framework::machine::ActResult;
use qitech_framework::machine::BuildContext;
use qitech_framework::machine::BuildResult;
use qitech_framework::machine::ConfigProperty;
use qitech_framework::machine::MachineDescriptor;
use qitech_framework::machine::Measurement;
use qitech_framework::machine::OperationCapability;
use qitech_framework::machine::StateProperty;
use qitech_framework::vendors;
use qitech_lib::ethercat_hal::io::analog_input::AnalogInputDevice;
use qitech_lib::ethercat_hal::io::digital_output::DigitalOutputDevice;
use qitech_lib::ethercat_hal::io::serial_interface::SerialInterfaceDevice;
use qitech_lib::ethercat_hal::io::temperature_input::TemperatureInputDevice;
use qitech_lib::units::{Energy, Power, Time, time::second};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use screw_speed_controller::ScrewSpeedController;
use temperature_controller::TemperatureController;

pub const VARIANT_V1: usize = 0;
pub const VARIANT_V2: usize = 1;

pub type ExtruderV1 = Extruder<VARIANT_V1>;
pub type ExtruderV2 = Extruder<VARIANT_V2>;

// --- types ---

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, EnumProperty)]
pub enum Mode {
    #[default]
    Standby,
    Heat,
    Extrude,
}

/// Which quantity the screw speed is regulated against.
///
/// Replaces the old `uses_rpm: bool`; `Rpm` is the old `true`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, EnumProperty)]
pub enum Regulation {
    #[default]
    Rpm,
    Pressure,
}

impl Regulation {
    pub const fn uses_rpm(self) -> bool {
        matches!(self, Self::Rpm)
    }
}

/// Mirrors the private `AutoTuneState` of [`crate::controllers::pid_autotuner`] so it can be
/// exported as a state property.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, EnumProperty)]
pub enum AutoTuneState {
    #[default]
    NotStarted,
    Running,
    Completed,
    Failed,
}

impl AutoTuneState {
    /// Maps the `&str` returned by `PidAutoTuner::state()`.
    pub fn from_tuner_state(state: &str) -> Self {
        match state {
            "running" => Self::Running,
            "completed" => Self::Completed,
            "failed" => Self::Failed,
            _ => Self::NotStarted,
        }
    }
}

/// The three gains of one PID loop, exposed as config properties.
///
/// `PidController::configure` resets the loop's accumulated error, so it must only be called when
/// a gain actually changed — hence the `applied` snapshot rather than reconfiguring every tick.
pub struct PidGains {
    kp: ConfigProperty<f64>,
    ki: ConfigProperty<f64>,
    kd: ConfigProperty<f64>,
    applied: (f64, f64, f64),
}

impl PidGains {
    pub fn init(
        ctx: &mut BuildContext,
        paths: PidGainPaths,
        defaults: (f64, f64, f64),
    ) -> BuildResult<Self> {
        let (kp, ki, kd) = defaults;

        Ok(Self {
            kp: ctx.config::<f64>(paths.kp).default(kp).build()?,
            ki: ctx.config::<f64>(paths.ki).default(ki).build()?,
            kd: ctx.config::<f64>(paths.kd).default(kd).build()?,
            applied: (kp, ki, kd),
        })
    }

    /// Returns `Some((ki, kp, kd))` — argument order of `PidController::configure` — when a gain
    /// has changed since the last call, otherwise `None`.
    pub fn take_change(&mut self) -> Option<(f64, f64, f64)> {
        let current = (self.kp.get(), self.ki.get(), self.kd.get());

        if current == self.applied {
            return None;
        }

        self.applied = current;
        Some((current.1, current.0, current.2))
    }

    /// Writes gains back into the config properties without triggering a reconfigure — used when
    /// the auto-tuner has already applied them to the controller itself.
    pub fn adopt(&mut self, kp: f64, ki: f64, kd: f64) {
        let _ = self.kp.set(kp);
        let _ = self.ki.set(ki);
        let _ = self.kd.set(kd);
        self.applied = (kp, ki, kd);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PidGainPaths {
    pub kp: &'static str,
    pub ki: &'static str,
    pub kd: &'static str,
}

/// One heating zone. Each zone owns the same set of resources under a different schema path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Zone {
    Nozzle,
    Front,
    Middle,
    Back,
}

impl Zone {
    pub const fn paths(self) -> ZonePaths {
        match self {
            Self::Nozzle => ZonePaths {
                target_temperature: "heating.nozzle.target_temperature",
                temperature: "heating.nozzle.temperature",
                power: "heating.nozzle.power",
                wiring_error: "heating.nozzle.wiring_error",
                gains: PidGainPaths {
                    kp: "pid.temperature.nozzle.kp",
                    ki: "pid.temperature.nozzle.ki",
                    kd: "pid.temperature.nozzle.kd",
                },
            },
            Self::Front => ZonePaths {
                target_temperature: "heating.front.target_temperature",
                temperature: "heating.front.temperature",
                power: "heating.front.power",
                wiring_error: "heating.front.wiring_error",
                gains: PidGainPaths {
                    kp: "pid.temperature.front.kp",
                    ki: "pid.temperature.front.ki",
                    kd: "pid.temperature.front.kd",
                },
            },
            Self::Middle => ZonePaths {
                target_temperature: "heating.middle.target_temperature",
                temperature: "heating.middle.temperature",
                power: "heating.middle.power",
                wiring_error: "heating.middle.wiring_error",
                gains: PidGainPaths {
                    kp: "pid.temperature.middle.kp",
                    ki: "pid.temperature.middle.ki",
                    kd: "pid.temperature.middle.kd",
                },
            },
            Self::Back => ZonePaths {
                target_temperature: "heating.back.target_temperature",
                temperature: "heating.back.temperature",
                power: "heating.back.power",
                wiring_error: "heating.back.wiring_error",
                gains: PidGainPaths {
                    kp: "pid.temperature.back.kp",
                    ki: "pid.temperature.back.ki",
                    kd: "pid.temperature.back.kd",
                },
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ZonePaths {
    pub target_temperature: &'static str,
    pub temperature: &'static str,
    pub power: &'static str,
    pub wiring_error: &'static str,
    pub gains: PidGainPaths,
}

// --- machine ---

pub struct Extruder<const VARIANT: usize> {
    // --- hardware ---
    relais_output: Rc<RefCell<dyn DigitalOutputDevice>>,
    temperature_input: Rc<RefCell<dyn TemperatureInputDevice>>,
    serial_interface: Rc<RefCell<dyn SerialInterfaceDevice>>,
    pressure_sensor: Rc<RefCell<dyn AnalogInputDevice>>,

    // --- components ---
    pub(super) screw_speed_controller: ScrewSpeedController,
    pub(super) temperature_controller_front: TemperatureController,
    pub(super) temperature_controller_middle: TemperatureController,
    pub(super) temperature_controller_back: TemperatureController,
    pub(super) temperature_controller_nozzle: TemperatureController,

    // --- config ---
    /// UI-only flag: whether the frontend shows a target temperature setter for the nozzle.
    /// The control loop never reads it; it is held so the resource stays owned by the machine.
    #[allow(dead_code)]
    nozzle_temperature_target_enabled: ConfigProperty<bool>,

    // --- state ---
    mode: StateProperty<Mode>,

    // --- measurements ---
    combined_power: Measurement<Power>,
    total_energy: Measurement<Energy>,

    last_energy_calculation_time: Option<Instant>,
}

impl MachineDescriptor for Extruder<VARIANT_V1> {
    const IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor_id: vendors::QITECH.id,
        machine_id: 4,
    };

    const SCHEMA: &'static str = include_str!("../../../schemas/extruder_v1.yaml");
}

impl MachineDescriptor for Extruder<VARIANT_V2> {
    const IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor_id: vendors::QITECH.id,
        machine_id: 22,
    };

    const SCHEMA: &'static str = include_str!("../../../schemas/extruder_v2.yaml");
}

impl<const VARIANT: usize> Extruder<VARIANT> {
    // --- power / energy ---

    /// Combined power draw: motor plus all four heating elements.
    fn calculate_combined_power(&self) -> Power {
        let motor_power = self.screw_speed_controller.get_motor_power();

        motor_power
            + self.temperature_controller_nozzle.heating_element_wattage()
            + self.temperature_controller_front.heating_element_wattage()
            + self.temperature_controller_back.heating_element_wattage()
            + self.temperature_controller_middle.heating_element_wattage()
    }

    /// Integrates combined power into the total energy counter.
    pub(super) fn update_energy(&mut self, now: Instant) {
        let power = self.calculate_combined_power();
        self.combined_power.set(power);

        if let Some(last_time) = self.last_energy_calculation_time {
            let elapsed = Time::new::<second>(now.duration_since(last_time).as_secs_f64());
            let total = self.total_energy.get() + power * elapsed;
            self.total_energy.set(total);
        }

        self.last_energy_calculation_time = Some(now);
    }

    // --- mode ---

    fn turn_heating_off(&mut self, digital_out: &mut dyn DigitalOutputDevice) {
        self.temperature_controller_back.disable(digital_out);
        self.temperature_controller_front.disable(digital_out);
        self.temperature_controller_middle.disable(digital_out);
        self.temperature_controller_nozzle.disable(digital_out);
    }

    pub(super) fn enable_heating(&mut self) {
        self.temperature_controller_back.allow_heating();
        self.temperature_controller_front.allow_heating();
        self.temperature_controller_middle.allow_heating();
        self.temperature_controller_nozzle.allow_heating();
    }

    fn switch_to_standby(&mut self, digital_out: &mut dyn DigitalOutputDevice) {
        match self.mode.get() {
            Mode::Standby => (),
            Mode::Heat => {
                self.turn_heating_off(digital_out);
                self.screw_speed_controller.reset_pid();
            }
            Mode::Extrude => {
                self.turn_heating_off(digital_out);
                self.screw_speed_controller.turn_motor_off();
                self.screw_speed_controller.reset_pid();
            }
        };

        self.mode.set(Mode::Standby);
    }

    fn switch_to_heat(&mut self) {
        match self.mode.get() {
            Mode::Standby => self.enable_heating(),
            Mode::Heat => (),
            Mode::Extrude => {
                self.screw_speed_controller.turn_motor_off();
                self.screw_speed_controller.reset_pid();
            }
        }

        self.mode.set(Mode::Heat);
    }

    fn switch_to_extrude(&mut self) {
        match self.mode.get() {
            Mode::Standby | Mode::Heat => {
                self.screw_speed_controller.turn_motor_on();
                self.enable_heating();
                self.screw_speed_controller.reset_pid();
            }
            Mode::Extrude => (),
        }

        self.mode.set(Mode::Extrude);
    }

    /// Command entry point for a mode transition.
    pub fn set_mode(&mut self, mode: Mode) -> ActResult {
        if self.mode.get() == mode {
            return Ok(());
        }

        match mode {
            Mode::Standby => {
                // Cloned so the relais stay borrowable while `self` is mutated.
                let relais = self.relais_output.clone();
                self.switch_to_standby(&mut *relais.borrow_mut());
            }
            Mode::Heat => self.switch_to_heat(),
            Mode::Extrude => self.switch_to_extrude(),
        }

        Ok(())
    }

    pub(super) fn mode(&self) -> Mode {
        self.mode.get()
    }

    /// Records a mode change that the control loop performed itself, without re-running the
    /// transition side effects.
    pub(super) fn set_mode_internal(&mut self, mode: Mode) {
        self.mode.set(mode);
    }

    // --- commands ---

    pub fn reset_inverter(&mut self) -> ActResult {
        self.screw_speed_controller.reset_inverter();
        Ok(())
    }

    /// The auto-tuner can only drive the actuator while the machine is extruding under pressure
    /// regulation. Previously this was only a doc comment on `start_pressure_pid_autotune`.
    pub fn can_autotune(&self) -> OperationCapability {
        if self.mode.get() != Mode::Extrude {
            return OperationCapability::forbidden("requires extrude mode");
        }

        if self.screw_speed_controller.regulation().uses_rpm() {
            return OperationCapability::forbidden("requires pressure regulation");
        }

        OperationCapability::Allowed
    }

    pub fn start_autotune(&mut self) -> ActResult {
        self.screw_speed_controller
            .start_pressure_autotune(Instant::now());
        Ok(())
    }

    pub fn stop_autotune(&mut self) -> ActResult {
        self.screw_speed_controller.stop_autotune();
        Ok(())
    }
}
