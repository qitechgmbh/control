use control_core::socketio::namespace::NamespaceCacheingLogic;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use units::f64::*;
use units::{thermodynamic_temperature::degree_celsius, volume_rate::liter_per_minute};

use crate::{AsyncThreadMessage, Machine, MachineMessage};
use crate::{
    MACHINE_AQUAPATH_V1, VENDOR_QITECH,
    aquapath1::{
        api::{
            AquaPathV1Events, AquaPathV1Namespace, FlowState, FlowStates, LiveValuesEvent,
            ModeState, StateEvent, TempState, TempStates,
        },
        controller::Controller,
    },
    machine_identification::MachineIdentification,
};

use super::machine_identification::MachineIdentificationUnique;
use smol::channel::{Receiver, Sender};

pub mod act;
pub mod api;
pub mod controller;
pub mod new;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum AquaPathV1Mode {
    Standby,
    Auto,
}

pub enum AquaPathSideType {
    Front,
    Back,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Temperature {
    pub temperature: ThermodynamicTemperature,
    pub cooling: bool,
    pub heating: bool,
    pub target_temperature: ThermodynamicTemperature,
}

impl Machine for AquaPathV1 {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}

impl Default for Temperature {
    fn default() -> Self {
        Self {
            temperature: ThermodynamicTemperature::new::<degree_celsius>(0.0),
            cooling: false,
            heating: false,
            target_temperature: ThermodynamicTemperature::new::<degree_celsius>(0.0),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Flow {
    pub flow: VolumeRate,
    pub pump: bool,
    pub should_pump: bool,
}

impl Default for Flow {
    fn default() -> Self {
        Self {
            flow: VolumeRate::new::<liter_per_minute>(0.0),
            pump: false,
            should_pump: false,
        }
    }
}

#[derive(Debug)]
pub struct AquaPathV1 {
    api_receiver: Receiver<MachineMessage>,
    api_sender: Sender<MachineMessage>,
    machine_identification_unique: MachineIdentificationUnique,
    namespace: AquaPathV1Namespace,
    mode: AquaPathV1Mode,
    last_measurement_emit: Instant,
    front_controller: Controller,
    back_controller: Controller,
    main_sender: Option<Sender<AsyncThreadMessage>>,
}

impl AquaPathV1 {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: MACHINE_AQUAPATH_V1,
    };
}

impl std::fmt::Display for AquaPathV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Aquapath")
    }
}

impl AquaPathV1 {
    pub fn emit_live_values(&mut self) {
        let live_values = LiveValuesEvent {
            front_temperature: self
                .front_controller
                .current_temperature
                .get::<degree_celsius>(),
            back_temperature: self
                .back_controller
                .current_temperature
                .get::<degree_celsius>(),
            front_flow: self.front_controller.current_flow.get::<liter_per_minute>(),
            back_flow: self.back_controller.current_flow.get::<liter_per_minute>(),
            front_temp_reservoir: self.front_controller.temp_reservoir.get::<degree_celsius>(),
            back_temp_reservoir: self.back_controller.temp_reservoir.get::<degree_celsius>(),
        };
        let event = live_values.build();
        self.namespace.emit(AquaPathV1Events::LiveValues(event));
    }

    pub fn emit_state(&mut self) {
        let state = StateEvent {
            is_default_state: false,
            mode_state: ModeState {
                mode: self.mode.clone(),
            },
            temperature_states: TempStates {
                front: TempState {
                    temperature: self
                        .front_controller
                        .current_temperature
                        .get::<degree_celsius>(),

                    target_temperature: self
                        .front_controller
                        .target_temperature
                        .get::<degree_celsius>(),
                },
                back: TempState {
                    temperature: self
                        .back_controller
                        .current_temperature
                        .get::<degree_celsius>(),
                    target_temperature: self
                        .back_controller
                        .target_temperature
                        .get::<degree_celsius>(),
                },
            },
            flow_states: FlowStates {
                front: FlowState {
                    flow: self.front_controller.current_flow.get::<liter_per_minute>(),
                    should_flow: self.front_controller.should_pump,
                },
                back: FlowState {
                    flow: self.back_controller.current_flow.get::<liter_per_minute>(),
                    should_flow: self.back_controller.should_pump,
                },
            },
        };

        let event = state.build();
        self.namespace.emit(AquaPathV1Events::State(event));
    }
}
impl AquaPathV1 {
    fn turn_cooling_off(&mut self) {
        self.front_controller.disable_cooling();
        self.back_controller.disable_cooling();
    }

    fn turn_cooling_on(&mut self) {
        self.front_controller.allow_cooling();
        self.back_controller.allow_cooling();
    }

    fn turn_heating_off(&mut self) {
        self.front_controller.disallow_heating();
        self.back_controller.disallow_heating();
    }

    fn turn_heating_on(&mut self) {
        self.front_controller.allow_heating();
        self.back_controller.allow_heating();
    }

    fn turn_pump_on(&mut self) {
        self.front_controller.allow_pump();
        self.back_controller.allow_pump();
    }

    fn turn_pump_off(&mut self) {
        self.front_controller.disallow_pump();
        self.back_controller.disallow_pump();
    }

    fn turn_off_all(&mut self) {
        self.turn_cooling_off();
        self.turn_heating_off();
        self.turn_pump_off();
    }

    fn turn_on_all(&mut self) {
        self.turn_cooling_on();
        self.turn_heating_on();
        self.turn_pump_on();
    }
    // Turn all OFF and do nothing
    fn switch_to_standby(&mut self) {
        match self.mode {
            AquaPathV1Mode::Standby => (),
            AquaPathV1Mode::Auto => self.turn_off_all(),
        };
        self.mode = AquaPathV1Mode::Standby;
    }

    fn switch_to_auto(&mut self) {
        match self.mode {
            AquaPathV1Mode::Auto => (),
            AquaPathV1Mode::Standby => self.turn_on_all(),
        }
        self.mode = AquaPathV1Mode::Auto;
    }

    fn switch_mode(&mut self, mode: AquaPathV1Mode) {
        if self.mode == mode {
            return;
        }
        match mode {
            AquaPathV1Mode::Standby => self.switch_to_standby(),
            AquaPathV1Mode::Auto => self.switch_to_auto(),
        }
    }
}

impl AquaPathV1 {
    fn set_mode_state(&mut self, mode: AquaPathV1Mode) {
        self.switch_mode(mode.clone());
        self.emit_state();
    }
}

impl AquaPathV1 {
    fn set_target_temperature(&mut self, temperature: f64, cooling_type: AquaPathSideType) {
        let target_temp = ThermodynamicTemperature::new::<degree_celsius>(temperature);

        match cooling_type {
            AquaPathSideType::Back => self.back_controller.set_target_temperature(target_temp),
            AquaPathSideType::Front => self.front_controller.set_target_temperature(target_temp),
        }
        self.emit_state();
    }

    fn set_should_pump(&mut self, should_pump: bool, cooling_type: AquaPathSideType) {
        match cooling_type {
            AquaPathSideType::Back => self.back_controller.set_should_pump(should_pump),
            AquaPathSideType::Front => self.front_controller.set_should_pump(should_pump),
        }
        self.emit_state();
    }
}
