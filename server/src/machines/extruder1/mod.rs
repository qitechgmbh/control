use api::{ExtruderV2Events, ExtruderV2Namespace};
use control_core::{
    actors::{
        analog_input_getter::AnalogInputGetter,
        mitsubishi_inverter_rs485::{
            MitsubishiControlRequests, MitsubishiInverterRS485Actor, MitsubishiModbusRequest,
        },
    },
    machines::Machine,
    socketio::namespace::NamespaceCacheingLogic,
};

use screw_speed_controller::ScrewSpeedController;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use temperature_controller::TemperatureController;
pub mod act;
pub mod api;
pub mod new;
pub mod screw_speed_controller;
pub mod temperature_controller;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum ExtruderV2Mode {
    Standby,
    Heat,
    Extrude,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct Heating {
    pub temperature: f32,
    pub heating: bool,
    pub target_temperature: f32,
    pub wiring_error: bool,
}

impl Default for Heating {
    fn default() -> Self {
        Self {
            temperature: 0.0,
            heating: false,
            target_temperature: 150.0,
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

#[derive(Debug)]
pub struct ExtruderV2 {
    namespace: ExtruderV2Namespace,
    mode: ExtruderV2Mode,
    last_measurement_emit: Instant,
    screw_speed_controller: ScrewSpeedController,
    temperature_controller_front: TemperatureController,
    temperature_controller_middle: TemperatureController,
    temperature_controller_back: TemperatureController,
    temperature_controller_nozzle: TemperatureController,
}

impl std::fmt::Display for ExtruderV2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ExtruderV2")
    }
}
impl Machine for ExtruderV2 {}

impl ExtruderV2 {
    // Set all relais to ZERO
    // We dont need a function to enable again though, as the act Loop will detect the mode
    fn turn_heating_off(&mut self) {
        self.temperature_controller_back.disable();
        self.temperature_controller_front.disable();
        self.temperature_controller_middle.disable();
        self.temperature_controller_nozzle.disable();
    }

    fn enable_heating(&mut self) {
        self.temperature_controller_back.allow_heating();
        self.temperature_controller_front.allow_heating();
        self.temperature_controller_middle.allow_heating();
        self.temperature_controller_nozzle.allow_heating();
    }

    // Turn heating OFF and do nothing
    fn switch_to_standby(&mut self) {
        match self.mode {
            ExtruderV2Mode::Standby => (),
            ExtruderV2Mode::Heat => {
                self.turn_heating_off();
            }
            ExtruderV2Mode::Extrude => {
                self.turn_heating_off();
                self.screw_speed_controller.turn_motor_off();
            }
        };
        self.mode = ExtruderV2Mode::Standby;
    }

    // turn off motor if on and keep heating on
    fn switch_to_heat(&mut self) {
        // From what mode are we transitioning ?
        match self.mode {
            ExtruderV2Mode::Standby => self.enable_heating(),
            ExtruderV2Mode::Heat => (),
            ExtruderV2Mode::Extrude => self.screw_speed_controller.turn_motor_off(),
        }
        self.mode = ExtruderV2Mode::Heat;
    }

    // keep heating on, and turn motor on
    fn switch_to_extrude(&mut self) {
        match self.mode {
            ExtruderV2Mode::Standby => {
                self.screw_speed_controller.turn_motor_on();
                self.enable_heating();
            }
            ExtruderV2Mode::Heat => {
                self.screw_speed_controller.turn_motor_on();
                self.enable_heating();
            }
            ExtruderV2Mode::Extrude => (), // Do nothing, we are already extruding
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
}

impl ExtruderV2 {
    fn set_rotation_state(&mut self, forward: bool) {
        self.screw_speed_controller.set_rotation_direction(forward);
        self.emit_rotation_state();
    }

    fn emit_rotation_state(&mut self) {
        let event = api::RotationStateEvent {
            forward: self.screw_speed_controller.get_rotation_direction(),
        }
        .build();
        self.namespace
            .emit_cached(ExtruderV2Events::RotationStateEvent(event))
    }

    fn set_mode_state(&mut self, mode: ExtruderV2Mode) {
        self.switch_mode(mode);

        self.emit_mode_state();
    }

    fn emit_mode_state(&mut self) {
        let event = api::ModeEvent {
            mode: self.mode.clone(),
        }
        .build();
        self.namespace
            .emit_cached(ExtruderV2Events::ModeEvent(event));
    }
}

// Motor
impl ExtruderV2 {
    fn set_regulation(&mut self, uses_rpm: bool) {
        self.screw_speed_controller.set_uses_rpm(uses_rpm);
        self.emit_regulation();
    }

    fn emit_regulation(&mut self) {
        let event = api::RegulationStateEvent {
            uses_rpm: self.screw_speed_controller.get_uses_rpm(),
        }
        .build();
        self.namespace
            .emit_cached(ExtruderV2Events::RegulationStateEvent(event));
    }

    fn set_target_pressure(&mut self, bar: f32) {
        self.screw_speed_controller.set_target_pressure(bar);
    }

    fn set_target_rpm(&mut self, rpm: f32) {
        self.screw_speed_controller.set_target_screw_rpm(rpm);
    }
}

// Heating
impl ExtruderV2 {
    fn emit_heating(&mut self, heating: Heating, heating_type: HeatingType) {
        let event = api::HeatingStateEvent {
            temperature: heating.temperature,
            heating: heating.heating,
            target_temperature: heating.target_temperature,
            wiring_error: heating.wiring_error,
        }
        .build(heating_type);

        self.namespace
            .emit_cached(ExtruderV2Events::HeatingStateEvent(event));
    }

    fn set_target_temperature(&mut self, target_temperature: f32, heating_type: HeatingType) {
        match heating_type {
            HeatingType::Nozzle => self
                .temperature_controller_nozzle
                .set_target_temperature(target_temperature),

            HeatingType::Front => self
                .temperature_controller_front
                .set_target_temperature(target_temperature),

            HeatingType::Back => self
                .temperature_controller_back
                .set_target_temperature(target_temperature),

            HeatingType::Middle => self
                .temperature_controller_middle
                .set_target_temperature(target_temperature),
        }

        match heating_type {
            HeatingType::Nozzle => self.emit_heating(
                self.temperature_controller_nozzle.heating.clone(),
                heating_type,
            ),
            HeatingType::Front => self.emit_heating(
                self.temperature_controller_front.heating.clone(),
                heating_type,
            ),
            HeatingType::Back => self.emit_heating(
                self.temperature_controller_back.heating.clone(),
                heating_type,
            ),
            HeatingType::Middle => self.emit_heating(
                self.temperature_controller_middle.heating.clone(),
                heating_type,
            ),
        }
    }
}

impl ExtruderV2 {
    fn emit_rpm(&mut self) {
        let event = api::ScrewStateEvent {
            // use uom here
            rpm: self.screw_speed_controller.get_screw_rpm(),
            target_rpm: self.screw_speed_controller.target_rpm,
        }
        .build();
        self.namespace
            .emit_cached(ExtruderV2Events::ScrewStateEvent(event));
    }

    fn emit_bar(&mut self) {
        let event = api::PressureStateEvent {
            bar: self.screw_speed_controller.get_pressure(),
            target_bar: self.screw_speed_controller.target_pressure,
        }
        .build();
        self.namespace
            .emit_cached(ExtruderV2Events::PressureStateEvent(event));
    }
}
