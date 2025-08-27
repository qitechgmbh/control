use control_core::{
    machines::{Machine, identification::MachineIdentification},
    socketio::namespace::NamespaceCacheingLogic,
};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use uom::si::{
    f64::{ThermodynamicTemperature, VolumeRate},
    thermodynamic_temperature::degree_celsius,
    volume_rate::liter_per_minute,
};

use crate::machines::{
    MACHINE_AQUAPATH_V1, VENDOR_QITECH,
    aquapath1::{
        api::{
            AquaPathV1Events, AquaPathV1Namespace, FlowState, FlowStates, LiveValuesEvent,
            ModeState, StateEvent, TempState, TempStates,
        },
        flow_controller::FlowController,
        temperature_controller::TemperatureController,
    },
};

pub mod act;
pub mod api;
pub mod flow_controller;
pub mod new;
pub mod temperature_controller;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum AquaPathV1Mode {
    Standby,
    Auto,
}

pub enum AquaPathSideType {
    Front,
    Back,
}

impl Machine for AquaPathV1 {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Temperature {
    pub temperature: ThermodynamicTemperature,
    pub cooling: bool,
    pub heating: bool,
    pub target_temperature: ThermodynamicTemperature,
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
    pub target_flow: VolumeRate,
}

impl Default for Flow {
    fn default() -> Self {
        Self {
            flow: VolumeRate::new::<liter_per_minute>(0.0),
            pump: false,
            target_flow: VolumeRate::new::<liter_per_minute>(0.0),
        }
    }
}

#[derive(Debug)]
pub struct AquaPathV1 {
    namespace: AquaPathV1Namespace,
    mode: AquaPathV1Mode,
    last_measurement_emit: Instant,
    flow_controller_front: FlowController,
    flow_controller_back: FlowController,
    temp_controller_front: TemperatureController,
    temp_controller_back: TemperatureController,
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
                .temp_controller_front
                .current_temperature
                .get::<degree_celsius>(),
            back_temperature: self
                .temp_controller_back
                .current_temperature
                .get::<degree_celsius>(),
            front_flow: self
                .flow_controller_front
                .current_flow
                .get::<liter_per_minute>(),
            back_flow: self
                .flow_controller_back
                .current_flow
                .get::<liter_per_minute>(),
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
                        .temp_controller_front
                        .current_temperature
                        .get::<degree_celsius>(),
                    target_temperature: self
                        .temp_controller_front
                        .target_temperature
                        .get::<degree_celsius>(),
                },
                back: TempState {
                    temperature: self
                        .temp_controller_back
                        .current_temperature
                        .get::<degree_celsius>(),
                    target_temperature: self
                        .temp_controller_back
                        .target_temperature
                        .get::<degree_celsius>(),
                },
            },
            flow_states: FlowStates {
                front: FlowState {
                    flow: self
                        .flow_controller_front
                        .current_flow
                        .get::<liter_per_minute>(),
                    target_flow: self
                        .flow_controller_front
                        .target_flow
                        .get::<liter_per_minute>(),
                },
                back: FlowState {
                    flow: self
                        .flow_controller_back
                        .current_flow
                        .get::<liter_per_minute>(),
                    target_flow: self
                        .flow_controller_back
                        .target_flow
                        .get::<liter_per_minute>(),
                },
            },
        };

        let event = state.build();
        self.namespace.emit(AquaPathV1Events::State(event));
    }
}
impl AquaPathV1 {
    fn turn_cooling_off(&mut self) {
        self.temp_controller_front.disable_cooling();
        self.temp_controller_back.disable_cooling();
    }

    fn turn_cooling_on(&mut self) {
        self.temp_controller_front.enable_cooling();
        self.temp_controller_back.enable_cooling();
    }

    fn turn_heating_off(&mut self) {
        self.temp_controller_front.disable_heating();
        self.temp_controller_back.disable_heating();
    }

    fn turn_heating_on(&mut self) {
        self.temp_controller_front.enable_heating();
        self.temp_controller_back.enable_heating();
    }

    fn turn_pump_on(&mut self) {
        self.flow_controller_front.enable();
        self.flow_controller_back.enable();
    }

    fn turn_pump_off(&mut self) {
        self.flow_controller_front.disable();
        self.flow_controller_back.disable();
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
        tracing::info!("mode {:?}", self.mode);

        match mode {
            AquaPathV1Mode::Standby => self.switch_to_standby(),
            AquaPathV1Mode::Auto => self.switch_to_auto(),
        }
    }
}

impl AquaPathV1 {
    fn set_mode_state(&mut self, mode: AquaPathV1Mode) {
        tracing::info!("mode {:?}", mode);

        self.switch_mode(mode.clone());
        self.emit_state();
    }
}

impl AquaPathV1 {
    fn set_target_temperature(&mut self, temperature: f64, cooling_type: AquaPathSideType) {
        let target_temp = ThermodynamicTemperature::new::<degree_celsius>(temperature);

        match cooling_type {
            AquaPathSideType::Back => self
                .temp_controller_back
                .set_target_temperature(target_temp),

            AquaPathSideType::Front => self
                .temp_controller_front
                .set_target_temperature(target_temp),
        }

        self.emit_state();
    }
    fn set_target_flow(&mut self, flow: f64, cooling_type: AquaPathSideType) {
        let target_flow = VolumeRate::new::<liter_per_minute>(flow);

        match cooling_type {
            AquaPathSideType::Back => self.flow_controller_back.set_target_flow(target_flow),

            AquaPathSideType::Front => self.flow_controller_front.set_target_flow(target_flow),
        }

        self.emit_state();
    }
}
