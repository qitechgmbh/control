use control_core::{
    machines::{Machine, identification::MachineIdentification},
    socketio::namespace::NamespaceCacheingLogic,
};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use uom::si::{
    f64::ThermodynamicTemperature, thermodynamic_temperature::degree_celsius,
    volume_rate::liter_per_minute,
};

use crate::machines::{
    MACHINE_AQUAPATH_V1, VENDOR_QITECH,
    aquapath1::api::{
        AquaPathV1Events, AquaPathV1Namespace, CoolingState, CoolingStates, LiveValuesEvent,
        ModeState, StateEvent,
    },
};

pub mod act;
pub mod api;
pub mod cooling_controller;
pub mod flow_sensor;
pub mod new;
//pub mod temperature_controller;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum AquaPathV1Mode {
    Standby,
    Cool,
    Heat,
}

pub enum CoolingType {
    Front,
    Back,
}

impl Machine for AquaPathV1 {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cooling {
    pub temperature: ThermodynamicTemperature,
    pub cooling: bool,
    pub target_temperature: ThermodynamicTemperature,
}

impl Default for Cooling {
    fn default() -> Self {
        Self {
            temperature: ThermodynamicTemperature::new::<degree_celsius>(0.0),
            cooling: false,
            target_temperature: ThermodynamicTemperature::new::<degree_celsius>(0.0),
        }
    }
}

#[derive(Debug)]
pub struct AquaPathV1 {
    namespace: AquaPathV1Namespace,
    mode: AquaPathV1Mode,
    last_measurement_emit: Instant,
    flow_sensor1: flow_sensor::FlowSensor,
    flow_sensor2: flow_sensor::FlowSensor,
    cooling_controller_front: cooling_controller::CoolingController,
    cooling_controller_back: cooling_controller::CoolingController,
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
                .cooling_controller_front
                .current_tempetature
                .get::<degree_celsius>(),
            back_temperature: self
                .cooling_controller_back
                .current_tempetature
                .get::<degree_celsius>(),
            flow_sensor1: self.flow_sensor1.current_flow.get::<liter_per_minute>(),
            flow_sensor2: self.flow_sensor2.current_flow.get::<liter_per_minute>(),
        };
        let event = live_values.build();
        self.namespace.emit(AquaPathV1Events::LiveValues(event));
    }

    pub fn emit_state(&mut self) {
        let state = StateEvent {
            is_default_state: false, // Placeholder for default state;
            mode_state: ModeState {
                mode: self.mode.clone(),
            },
            cooling_states: CoolingStates {
                front: CoolingState {
                    temperature: self
                        .cooling_controller_front
                        .current_tempetature
                        .get::<degree_celsius>(),
                    target_temperature: self
                        .cooling_controller_front
                        .target_tempetature
                        .get::<degree_celsius>(),
                },
                back: CoolingState {
                    temperature: self
                        .cooling_controller_back
                        .current_tempetature
                        .get::<degree_celsius>(),
                    target_temperature: self
                        .cooling_controller_back
                        .target_tempetature
                        .get::<degree_celsius>(),
                },
            },
        };

        let event = state.build();
        self.namespace.emit(AquaPathV1Events::State(event));
    }
}
impl AquaPathV1 {
    fn turn_cooling_off(&mut self) {
        self.cooling_controller_front.disable();
        self.cooling_controller_back.disable();
    }

    fn turn_cooling_on(&mut self) {
        self.cooling_controller_front.allow_cooling();
        self.cooling_controller_back.allow_cooling();
    }

    fn turn_heating_off(&mut self) {
        //turn off heating
    }

    fn turn_heating_on(&mut self) {
        //turn on heating
    }

    // Turn all OFF and do nothing
    fn switch_to_standby(&mut self) {
        match self.mode {
            AquaPathV1Mode::Standby => (),
            AquaPathV1Mode::Cool => {
                self.turn_heating_off();
                self.turn_cooling_off();
            }
            AquaPathV1Mode::Heat => {
                self.turn_heating_off();
                self.turn_cooling_off();
            }
        };
        self.mode = AquaPathV1Mode::Standby;
    }

    // turn on motor and cool
    fn switch_to_cool(&mut self) {
        match self.mode {
            AquaPathV1Mode::Standby => {
                self.turn_cooling_on();
                self.turn_heating_off();
            }
            AquaPathV1Mode::Cool => (),
            AquaPathV1Mode::Heat => {
                self.turn_cooling_on();
                self.turn_heating_off();
            }
        }
        self.mode = AquaPathV1Mode::Cool;
    }

    fn switch_to_heat(&mut self) {
        match self.mode {
            AquaPathV1Mode::Standby => {
                self.turn_cooling_off();
                self.turn_heating_on();
            }
            AquaPathV1Mode::Cool => {
                self.turn_cooling_off();
                self.turn_heating_on();
            }
            AquaPathV1Mode::Heat => (),
        }
        self.mode = AquaPathV1Mode::Heat;
    }

    fn switch_mode(&mut self, mode: AquaPathV1Mode) {
        if self.mode == mode {
            return;
        }
        match mode {
            AquaPathV1Mode::Standby => self.switch_to_standby(),
            AquaPathV1Mode::Cool => self.switch_to_cool(),
            AquaPathV1Mode::Heat => self.switch_to_heat(),
        }
    }
}

impl AquaPathV1 {
    fn set_mode_state(&mut self, mode: AquaPathV1Mode) {
        self.switch_mode(mode);

        self.emit_state();
    }
}

// Cooling
impl AquaPathV1 {
    fn set_temperature(&mut self, temperature: f64, cooling_type: CoolingType) {
        // Placeholder for setting temperature
        let target_temp = ThermodynamicTemperature::new::<degree_celsius>(temperature);

        match cooling_type {
            CoolingType::Back => self
                .cooling_controller_back
                .set_target_temperature(target_temp),

            CoolingType::Front => self
                .cooling_controller_front
                .set_target_temperature(target_temp),
        }

        self.emit_state();
    }
}
