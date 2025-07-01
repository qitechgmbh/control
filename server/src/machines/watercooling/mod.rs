use control_core::{machines::Machine, socketio::namespace::NamespaceCacheingLogic};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use temperature_controller::TemperatureController;
use uom::si::{f64::ThermodynamicTemperature, thermodynamic_temperature::degree_celsius};

use crate::machines::watercooling::api::{WaterCoolingEvents, WaterCoolingNamespace};

pub mod act;
pub mod api;
pub mod new;
pub mod temperature_controller;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum WaterCoolingMode {
    Standby,
    Cool,
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
pub struct WaterCooling {
    namespace: WaterCoolingNamespace,
    mode: WaterCoolingMode,
    last_measurement_emit: Instant,
    // temperature_controller: TemperatureController,
}

impl std::fmt::Display for WaterCooling {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WaterCooling")
    }
}
impl Machine for WaterCooling {}

impl WaterCooling {
    fn turn_cooling_off(&mut self) {
        //self.temperature_controller.disable();
    }

    fn enable_cooling(&mut self) {
        //self.temperature_controller.allow_cooling();
    }

    // Turn cooling OFF and do nothing
    fn switch_to_standby(&mut self) {
        match self.mode {
            WaterCoolingMode::Standby => (),
            WaterCoolingMode::Cool => {
                self.turn_cooling_off();
            }
        };
        self.mode = WaterCoolingMode::Standby;
    }

    // turn on motor and cool
    fn switch_to_cool(&mut self) {
        // From what mode are we transitioning ?
        match self.mode {
            WaterCoolingMode::Standby => self.enable_cooling(),
            WaterCoolingMode::Cool => (),
        }
        self.mode = WaterCoolingMode::Cool;
    }

    fn switch_mode(&mut self, mode: WaterCoolingMode) {
        if self.mode == mode {
            return;
        }

        match mode {
            WaterCoolingMode::Standby => self.switch_to_standby(),
            WaterCoolingMode::Cool => self.switch_to_cool(),
        }
    }
}

impl WaterCooling {
    fn set_mode_state(&mut self, mode: WaterCoolingMode) {
        self.switch_mode(mode);

        self.emit_mode_state();
    }

    fn emit_mode_state(&mut self) {
        let event = api::ModeEvent {
            mode: self.mode.clone(),
        }
        .build();
        self.namespace.emit(WaterCoolingEvents::ModeEvent(event));
    }
}

// Cooling
impl WaterCooling {
    fn emit_cooling(&mut self, cooling: Cooling) {
        let event = api::CoolingStateEvent {
            temperature: cooling.temperature.get::<degree_celsius>(),
            target_temperature: cooling.target_temperature.get::<degree_celsius>(),
        }
        .build();

        self.namespace
            .emit(WaterCoolingEvents::CoolingStateEvent(event));
    }

    fn emit_cooling_element_power(&mut self) {
        let wattage = 64.20; //self.temperature_controller.get_cooling_element_wattage();

        let event = api::CoolingPowerEvent { wattage }.build();
        self.namespace
            .emit(WaterCoolingEvents::CoolingPowerEvent(event));
    }

    fn set_target_temperature(&mut self, target_temperature: f64) {
        let target_temp = ThermodynamicTemperature::new::<degree_celsius>(target_temperature);

        // self.temperature_controller
        //     .set_target_temperature(target_temp);

        // self.emit_cooling(self.temperature_controller.cooling.clone());
    }
}
