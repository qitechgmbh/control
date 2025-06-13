use api::{
    ExtruderSettingsStateEvent, ExtruderV2Events, ExtruderV2Namespace, PidSettings, PidType,
};
use control_core::{machines::Machine, socketio::namespace::NamespaceCacheingLogic};
use screw_speed_controller::ScrewSpeedController;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use temperature_controller::TemperatureController;
use uom::si::{
    angular_velocity::{AngularVelocity, revolution_per_minute},
    f64::{Pressure, ThermodynamicTemperature},
    pressure::bar,
    thermodynamic_temperature::degree_celsius,
};
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
    // Extruder Settings Api Impl
    fn set_nozzle_pressure_limit_is_enabled(&mut self, enabled: bool) {
        self.screw_speed_controller
            .set_nozzle_pressure_limit_is_enabled(enabled);
    }

    /// pressure is represented as bar
    fn set_nozzle_pressure_limit(&mut self, pressure: f64) {
        let nozzle_pressure_limit = Pressure::new::<bar>(pressure);
        self.screw_speed_controller
            .set_nozzle_pressure_limit(nozzle_pressure_limit);
    }

    fn get_nozzle_pressure_limit(&mut self) -> f64 {
        let nozzle_pressure: Pressure = self.screw_speed_controller.get_nozzle_pressure_limit();
        return nozzle_pressure.get::<bar>();
    }

    fn get_nozzle_pressure_limit_enabled(&mut self) -> bool {
        return self
            .screw_speed_controller
            .get_nozzle_pressure_limit_enabled();
    }

    fn emit_extruder_settings(&mut self) {
        let pressure: f64 = self.get_nozzle_pressure_limit();
        let enabled = self.get_nozzle_pressure_limit_enabled();

        let event = ExtruderSettingsStateEvent {
            pressure_limit: pressure,
            pressure_limit_enabled: enabled,
        }
        .build();

        self.namespace
            .emit_cached(ExtruderV2Events::ExtruderSettingsStateEvent(event));
    }
}

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

    // turn off motor if on and keep heating on
    fn switch_to_heat(&mut self) {
        // From what mode are we transitioning ?
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

    // keep heating on, and turn motor on
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
        if (self.screw_speed_controller.get_uses_rpm() == false && uses_rpm == true) {
            self.screw_speed_controller
                .set_target_screw_rpm(self.screw_speed_controller.target_rpm);
        }

        self.screw_speed_controller.reset_pid();
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

    fn set_target_pressure(&mut self, pressure: f64) {
        let pressure = Pressure::new::<bar>(pressure);
        self.screw_speed_controller.set_target_pressure(pressure);
    }

    fn set_target_rpm(&mut self, rpm: f64) {
        let revolution_per_minutes = AngularVelocity::new::<revolution_per_minute>(rpm);
        self.screw_speed_controller
            .set_target_screw_rpm(revolution_per_minutes);
    }
}

// Heating
impl ExtruderV2 {
    fn emit_heating(&mut self, heating: Heating, heating_type: HeatingType) {
        let event = api::HeatingStateEvent {
            temperature: heating.temperature.get::<degree_celsius>(),
            target_temperature: heating.target_temperature.get::<degree_celsius>(),
            wiring_error: heating.wiring_error,
        }
        .build(heating_type);

        self.namespace
            .emit_cached(ExtruderV2Events::HeatingStateEvent(event));
    }

    fn emit_heating_element_power(&mut self, heating_type: HeatingType) {
        let wattage = match heating_type {
            HeatingType::Nozzle => self
                .temperature_controller_nozzle
                .get_heating_element_wattage(),
            HeatingType::Front => self
                .temperature_controller_front
                .get_heating_element_wattage(),
            HeatingType::Back => self
                .temperature_controller_back
                .get_heating_element_wattage(),
            HeatingType::Middle => self
                .temperature_controller_middle
                .get_heating_element_wattage(),
        };

        let event = api::HeatingPowerEvent { wattage }.build(heating_type);
        self.namespace
            .emit_cached(ExtruderV2Events::HeatingPowerEvent(event));
    }

    fn set_target_temperature(&mut self, target_temperature: f64, heating_type: HeatingType) {
        let target_temp = ThermodynamicTemperature::new::<degree_celsius>(target_temperature);

        match heating_type {
            HeatingType::Nozzle => self
                .temperature_controller_nozzle
                .set_target_temperature(target_temp),

            HeatingType::Front => self
                .temperature_controller_front
                .set_target_temperature(target_temp),

            HeatingType::Back => self
                .temperature_controller_back
                .set_target_temperature(target_temp),

            HeatingType::Middle => self
                .temperature_controller_middle
                .set_target_temperature(target_temp),
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
        let rpm = self.screw_speed_controller.get_screw_rpm();
        let target_rpm = self.screw_speed_controller.get_target_rpm();

        let event = api::ScrewStateEvent {
            // use uom here
            rpm: rpm.get::<revolution_per_minute>(),
            target_rpm: target_rpm.get::<revolution_per_minute>(),
        }
        .build();
        self.namespace
            .emit_cached(ExtruderV2Events::ScrewStateEvent(event));
    }

    fn emit_bar(&mut self) {
        let pressure = self.screw_speed_controller.get_pressure();
        let target_pressure = self.screw_speed_controller.get_target_pressure();
        let event = api::PressureStateEvent {
            bar: pressure.get::<bar>(),
            target_bar: target_pressure.get::<bar>(),
        }
        .build();
        self.namespace
            .emit_cached(ExtruderV2Events::PressureStateEvent(event));
    }
}

impl ExtruderV2 {
    fn emit_pressure_pid_settings(&mut self) {
        let kd = self.screw_speed_controller.pid.get_kd();
        let kp = self.screw_speed_controller.pid.get_kp();
        let ki = self.screw_speed_controller.pid.get_ki();
        let event = api::PidSettingsEvent { ki, kp, kd }.build(PidType::Pressure);
        self.namespace
            .emit_cached(ExtruderV2Events::PidSettingsEvent(event));
    }

    fn emit_temperature_pid_settings(&mut self) {
        let kd = self.temperature_controller_front.pid.get_kd();
        let kp = self.temperature_controller_front.pid.get_kp();
        let ki = self.temperature_controller_front.pid.get_ki();
        let event = api::PidSettingsEvent { ki, kp, kd }.build(PidType::Temperature);
        self.namespace
            .emit_cached(ExtruderV2Events::PidSettingsEvent(event));
    }

    fn configure_pressure_pid(&mut self, settings: PidSettings) {
        self.screw_speed_controller
            .pid
            .configure(settings.ki, settings.kp, settings.kd);
    }

    fn configure_temperature_pid(&mut self, settings: PidSettings) {
        self.temperature_controller_back
            .pid
            .configure(settings.ki, settings.kp, settings.kd);
        self.temperature_controller_middle
            .pid
            .configure(settings.ki, settings.kp, settings.kd);
        self.temperature_controller_front
            .pid
            .configure(settings.ki, settings.kp, settings.kd);
    }
}
