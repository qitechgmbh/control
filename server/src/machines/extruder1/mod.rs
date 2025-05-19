use api::{ExtruderV2Events, ExtruderV2Namespace};
use control_core::{
    actors::{
        analog_input_getter::AnalogInputGetter,
        digital_output_setter::DigitalOutputSetter,
        mitsubishi_inverter_rs485::{
            MitsubishiControlRequests, MitsubishiInverterRS485Actor, MitsubishiModbusRequest,
        },
        temperature_input_getter::TemperatureInputGetter,
    },
    machines::Machine,
    modbus::ModbusRequest,
    socketio::namespace::NamespaceCacheingLogic,
};

use pressure_controller::PressureController;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use temperature_controller::TemperatureController;
pub mod act;
pub mod api;
pub mod new;
pub mod pressure_controller;
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
}

pub enum HeatingType {
    Nozzle,
    Front,
    Back,
    Middle,
}

#[derive(Debug)]
pub struct ExtruderV2 {
    inverter: MitsubishiInverterRS485Actor,
    namespace: ExtruderV2Namespace,
    mode: ExtruderV2Mode,
    last_measurement_emit: Instant,
    pressure_sensor: AnalogInputGetter, // EL3024
    uses_rpm: bool,
    rpm: f32,
    bar: f32,
    target_rpm: f32,
    target_bar: f32,

    can_extrude: bool,

    // Temperature TODO: CLEAN UP
    // Heating contains the current temp,target and relais state
    heating_front: Heating,
    heating_middle: Heating,
    heating_back: Heating,
    heating_nozzle: Heating,

    temp_sensor_1: TemperatureInputGetter,
    temp_sensor_2: TemperatureInputGetter,
    temp_sensor_3: TemperatureInputGetter,
    temp_sensor_4: TemperatureInputGetter,

    // heating_relay_1: DigitalOutputSetter,
    // heating_relay_2: DigitalOutputSetter,
    // heating_relay_3: DigitalOutputSetter,
    // heating_relay_4: DigitalOutputSetter,
    temperature_controller_front: TemperatureController,
    temperature_controller_middle: TemperatureController,
    temperature_controller_back: TemperatureController,
    temperature_controller_nozzle: TemperatureController,

    pressure_motor_controller: PressureController,
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
        // self.heating_relay_1.set(false);
        // self.heating_relay_2.set(false);
        // self.heating_relay_3.set(false);

        self.heating_back.heating = false;
        self.heating_front.heating = false;
        self.heating_middle.heating = false;
    }

    // Send Motor Turn Off Request to the Inverter
    fn turn_motor_off(&mut self) {
        self.inverter
            .add_request(MitsubishiControlRequests::StopMotor.into());
    }

    fn turn_motor_on(&mut self) {
        if self.inverter.forward_rotation {
            self.inverter
                .add_request(MitsubishiControlRequests::StartForwardRotation.into());
        } else {
            self.inverter
                .add_request(MitsubishiControlRequests::StartReverseRotation.into());
        }
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
                self.turn_motor_off();
            }
        };
        self.mode = ExtruderV2Mode::Standby;
    }

    // turn off motor if on and keep heating on
    fn switch_to_heat(&mut self) {
        // From what mode are we transitioning ?
        match self.mode {
            ExtruderV2Mode::Standby => (),
            ExtruderV2Mode::Heat => (),
            ExtruderV2Mode::Extrude => self.turn_motor_off(),
        }
        self.mode = ExtruderV2Mode::Heat;
    }

    /// Checks if the extruder is allowed to extrude and then sets can_extrude true or false
    fn set_can_switch_extrude(&mut self) {
        const NINETY_PERCENT: f32 = 0.9;
        let heat_back_is_valid =
            (self.heating_back.temperature / self.heating_back.target_temperature > NINETY_PERCENT)
                && (self.heating_back.temperature > 80.0);

        let heat_middle_is_valid = (self.heating_middle.temperature
            / self.heating_middle.target_temperature
            > NINETY_PERCENT)
            && (self.heating_middle.temperature > 80.0);

        let heat_front_is_valid = (self.heating_front.temperature
            / self.heating_front.target_temperature
            > NINETY_PERCENT)
            && (self.heating_front.temperature > 80.0);

        let heat_nozzle_is_valid = (self.heating_nozzle.temperature
            / self.heating_nozzle.target_temperature
            > NINETY_PERCENT)
            && (self.heating_nozzle.temperature > 80.0);
        // println!(
        //     "{} {} {} {}",
        //     heat_back_is_valid, heat_front_is_valid, heat_middle_is_valid, heat_nozzle_is_valid,
        // );
        self.can_extrude = heat_back_is_valid
            && heat_front_is_valid
            && heat_middle_is_valid
            && heat_nozzle_is_valid;
    }

    // keep heating on, and turn motor on
    fn switch_to_extrude(&mut self) {
        if self.can_extrude == false {
            return;
        }

        match self.mode {
            ExtruderV2Mode::Standby => self.turn_motor_on(),
            ExtruderV2Mode::Heat => self.turn_motor_on(),
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
        self.inverter.forward_rotation = forward;
        if self.mode == ExtruderV2Mode::Extrude {
            let req: MitsubishiModbusRequest = match forward {
                // Our gearbox is inverted!!!
                true => MitsubishiControlRequests::StartReverseRotation.into(),
                false => MitsubishiControlRequests::StartForwardRotation.into(),
            };
            self.inverter.add_request(req);
        }

        self.emit_rotation_state();
    }

    fn emit_rotation_state(&mut self) {
        let event = api::RotationStateEvent {
            forward: self.inverter.forward_rotation.clone(),
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
        self.uses_rpm = uses_rpm.clone();
        self.emit_regulation();
    }

    fn emit_regulation(&mut self) {
        let event = api::RegulationStateEvent {
            uses_rpm: self.uses_rpm,
        }
        .build();
        self.namespace
            .emit_cached(ExtruderV2Events::RegulationStateEvent(event));
    }

    fn set_target_pressure(&mut self, bar: f32) {
        self.target_bar = bar;
    }

    fn set_bar(&mut self) {
        let normalized = self.pressure_sensor.get_normalized();
        let normalized = match normalized {
            Some(normalized) => normalized,
            None => todo!(),
        };
        // assuming full scale pressure of 10 bar
        let bar = normalized * 10.0;
        self.bar = bar;
    }

    fn set_target_rpm(&mut self, rpm: f32) {
        println!("set_target_rpm uses_rpm: {}", self.uses_rpm);
        if self.uses_rpm {
            self.target_rpm = rpm;
            self.inverter.set_running_rpm_target(rpm);
        } else {
            return;
        }
    }
}

// Heating
impl ExtruderV2 {
    // Heating
    fn set_heating_front(&mut self, heating: Heating) {
        self.heating_front = heating.clone();
        self.emit_heating(heating, HeatingType::Front);
    }

    fn set_heating_back(&mut self, heating: Heating) {
        self.heating_back = heating.clone();
        self.emit_heating(heating, HeatingType::Back);
    }

    fn set_heating_middle(&mut self, heating: Heating) {
        self.heating_middle = heating.clone();
        self.emit_heating(heating, HeatingType::Middle);
    }

    fn set_heating_nozzle(&mut self, heating: Heating) {
        self.heating_nozzle = heating.clone();
        self.emit_heating(heating, HeatingType::Nozzle);
    }

    fn emit_heating(&mut self, heating: Heating, heating_type: HeatingType) {
        let event = api::HeatingStateEvent {
            temperature: heating.temperature,
            heating: heating.heating,
            target_temperature: heating.target_temperature,
        }
        .build(heating_type);

        self.namespace
            .emit_cached(ExtruderV2Events::HeatingStateEvent(event));
    }

    fn set_target_temperature(&mut self, target_temperature: f32, heating_type: HeatingType) {
        match heating_type {
            HeatingType::Nozzle => self.heating_nozzle.target_temperature = target_temperature,
            HeatingType::Front => self.heating_front.target_temperature = target_temperature,
            HeatingType::Back => self.heating_back.target_temperature = target_temperature,
            HeatingType::Middle => self.heating_middle.target_temperature = target_temperature,
        }

        match heating_type {
            HeatingType::Nozzle => self.emit_heating(self.heating_nozzle.clone(), heating_type),
            HeatingType::Front => self.emit_heating(self.heating_front.clone(), heating_type),
            HeatingType::Back => self.emit_heating(self.heating_back.clone(), heating_type),
            HeatingType::Middle => self.emit_heating(self.heating_middle.clone(), heating_type),
        }
    }
}

impl ExtruderV2 {
    fn emit_rpm(&mut self) {
        let event = api::RpmStateEvent {
            rpm: self.inverter.current_rpm,
            target_rpm: self.target_rpm,
        }
        .build();
        self.namespace
            .emit_cached(ExtruderV2Events::RpmStateEvent(event));
    }

    fn emit_bar(&mut self) {
        let event = api::PressureStateEvent {
            bar: self.bar,
            target_bar: self.target_bar,
        }
        .build();
        self.namespace
            .emit_cached(ExtruderV2Events::PressureStateEvent(event));
    }
}
