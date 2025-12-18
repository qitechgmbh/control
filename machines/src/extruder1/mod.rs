#[cfg(not(feature = "mock-machine"))]
use std::time::Instant;

#[cfg(not(feature = "mock-machine"))]
use smol::channel::Receiver;
#[cfg(not(feature = "mock-machine"))]
use smol::channel::Sender;

use serde::{Deserialize, Serialize};

#[cfg(not(feature = "mock-machine"))]
use units::electric_current::ampere;

#[cfg(not(feature = "mock-machine"))]
use units::electric_potential::volt;

#[cfg(not(feature = "mock-machine"))]
use crate::{AsyncThreadMessage, Machine};
use units::f64::*;
use units::thermodynamic_temperature::degree_celsius;

#[cfg(not(feature = "mock-machine"))]
use crate::{
    MACHINE_EXTRUDER_V1, MachineMessage, VENDOR_QITECH,
    extruder1::{
        api::ExtruderV2Namespace, screw_speed_controller::ScrewSpeedController,
        temperature_controller::TemperatureController,
    },
    machine_identification::{MachineIdentification, MachineIdentificationUnique},
};

pub mod act;
pub mod api;
pub mod emit;
pub mod mitsubishi_cs80;
pub mod mock;
pub mod new;
pub mod screw_speed_controller;
pub mod temperature_controller;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum ExtruderV2Mode {
    Standby,
    Heat,
    Extrude,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Heating {
    pub temperature: ThermodynamicTemperature,
    pub heating: bool,
    pub target_temperature: ThermodynamicTemperature,
    pub wiring_error: bool,
}

impl Default for Heating {
    fn default() -> Self {
        Self {
            temperature: ThermodynamicTemperature::new::<degree_celsius>(0.0),
            heating: false,
            target_temperature: ThermodynamicTemperature::new::<degree_celsius>(0.0),
            wiring_error: false,
        }
    }
}

pub enum HeatingType {
    Nozzle,
    Front,
    Back,
    Middle,
}

#[cfg(not(feature = "mock-machine"))]
#[derive(Debug)]
pub struct ExtruderV2 {
    api_receiver: Receiver<MachineMessage>,
    api_sender: Sender<MachineMessage>,
    main_sender: Option<Sender<AsyncThreadMessage>>,

    machine_identification_unique: MachineIdentificationUnique,
    namespace: ExtruderV2Namespace,

    last_measurement_emit: Instant,
    last_status_hash: Option<u64>,
    mode: ExtruderV2Mode,

    screw_speed_controller: ScrewSpeedController,
    temperature_controller_front: TemperatureController,
    temperature_controller_middle: TemperatureController,
    temperature_controller_back: TemperatureController,
    temperature_controller_nozzle: TemperatureController,

    /// Energy tracking for total consumption calculation
    total_energy_kwh: f64,
    last_energy_calculation_time: Option<Instant>,

    /// will be initalized as false and set to true by `emit_state`
    /// This way we can signal to the client that the first state emission is a default state
    emitted_default_state: bool,
}

#[cfg(not(feature = "mock-machine"))]
impl Machine for ExtruderV2 {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}

#[cfg(not(feature = "mock-machine"))]
impl std::fmt::Display for ExtruderV2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ExtruderV2")
    }
}

#[cfg(not(feature = "mock-machine"))]
impl ExtruderV2 {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: MACHINE_EXTRUDER_V1,
    };
}

#[cfg(not(feature = "mock-machine"))]
impl ExtruderV2 {
    /// Calculate combined power consumption in watts
    fn calculate_combined_power(&mut self) -> f64 {
        let motor_power = {
            let motor_status = &self.screw_speed_controller.inverter.motor_status;
            let voltage = motor_status.voltage.get::<volt>();
            let current = motor_status.current.get::<ampere>();
            voltage * current
        };
        let nozzle_power = self
            .temperature_controller_nozzle
            .get_heating_element_wattage();
        let front_power = self
            .temperature_controller_front
            .get_heating_element_wattage();
        let back_power = self
            .temperature_controller_back
            .get_heating_element_wattage();
        let middle_power = self
            .temperature_controller_middle
            .get_heating_element_wattage();

        motor_power + nozzle_power + front_power + back_power + middle_power
    }

    /// Update total energy consumption in kWh
    fn update_total_energy(&mut self, current_power_watts: f64, now: Instant) {
        if let Some(last_time) = self.last_energy_calculation_time {
            let time_delta_hours = now.duration_since(last_time).as_secs_f64() / 3600.0;
            let energy_delta_kwh = (current_power_watts / 1000.0) * time_delta_hours;
            self.total_energy_kwh += energy_delta_kwh;
        }
        self.last_energy_calculation_time = Some(now);
    }

    // Funktionen ohne emit_state bleiben hier

    // Set all relais to ZERO
    // We dont need a function to enable again though, as the act Loop will detect the mode
    fn turn_heating_off(&mut self) {
        self.temperature_controller_back.disable();
        self.temperature_controller_front.disable();
        self.temperature_controller_middle.disable();
        self.temperature_controller_nozzle.disable();
    }

    fn switch_to_standby(&mut self) {
        match self.mode {
            ExtruderV2Mode::Standby => (),
            ExtruderV2Mode::Heat => {
                self.turn_heating_off();
                self.screw_speed_controller.reset_pid();
            }
            ExtruderV2Mode::Extrude => {
                self.turn_heating_off();
                self.screw_speed_controller.turn_motor_off();
                self.screw_speed_controller.reset_pid();
            }
        };
        self.mode = ExtruderV2Mode::Standby;
    }

    fn switch_to_heat(&mut self) {
        match self.mode {
            ExtruderV2Mode::Standby => self.enable_heating(),
            ExtruderV2Mode::Heat => (),
            ExtruderV2Mode::Extrude => {
                self.screw_speed_controller.turn_motor_off();
                self.screw_speed_controller.reset_pid();
            }
        }
        self.mode = ExtruderV2Mode::Heat;
    }

    fn switch_to_extrude(&mut self) {
        match self.mode {
            ExtruderV2Mode::Standby => {
                self.screw_speed_controller.turn_motor_on();
                self.enable_heating();
                self.screw_speed_controller.reset_pid();
            }
            ExtruderV2Mode::Heat => {
                self.screw_speed_controller.turn_motor_on();
                self.enable_heating();
                self.screw_speed_controller.reset_pid();
            }
            ExtruderV2Mode::Extrude => (),
        }
        self.mode = ExtruderV2Mode::Extrude;
    }

    fn switch_mode(&mut self, mode: ExtruderV2Mode) {
        if self.mode == mode {
            return;
        }

        match mode {
            ExtruderV2Mode::Standby => self.switch_to_standby(),
            ExtruderV2Mode::Heat => self.switch_to_heat(),
            ExtruderV2Mode::Extrude => self.switch_to_extrude(),
        }
    }

    fn reset_inverter(&mut self) {
        self.screw_speed_controller.inverter.reset_inverter();
    }
}
