#[cfg(not(feature = "mock-machine"))]
// Contains Implementations for All functions that use emit_state
use crate::extruder1::{
    HeatingType,
    api::{
        ExtruderSettingsState, HeatingState, HeatingStates, InverterStatusState, PidSettings,
        PidSettingsStates, PressureState, RegulationState, RotationState, ScrewState,
        TemperaturePid,
    },
};
#[cfg(not(feature = "mock-machine"))]
use control_core::helpers::hasher_serializer::hash_with_serde_model;
#[cfg(not(feature = "mock-machine"))]
use control_core::socketio::event::BuildEvent;
#[cfg(not(feature = "mock-machine"))]
use control_core::socketio::namespace::NamespaceCacheingLogic;
#[cfg(not(feature = "mock-machine"))]
use units::angular_velocity::AngularVelocity;
#[cfg(not(feature = "mock-machine"))]
use units::pressure::{Pressure, bar};
#[cfg(not(feature = "mock-machine"))]
use units::thermodynamic_temperature::ThermodynamicTemperature;
#[cfg(not(feature = "mock-machine"))]
use units::{angular_velocity::revolution_per_minute, thermodynamic_temperature::degree_celsius};

#[cfg(not(feature = "mock-machine"))]
use super::{ExtruderV3, ExtruderV3Mode, api::StateEvent};

#[cfg(not(feature = "mock-machine"))]
impl ExtruderV3 {
    pub fn build_state_event(&mut self) -> StateEvent {
        use crate::{
            extruder1::api::{TemperaturePid, TemperaturePidStates},
            extruder2::api::ModeState,
        };

        StateEvent {
            is_default_state: !std::mem::replace(&mut self.emitted_default_state, true),
            rotation_state: RotationState {
                forward: self.screw_speed_controller.get_rotation_direction(),
            },
            mode_state: ModeState {
                mode: self.mode.clone(),
            },
            regulation_state: RegulationState {
                uses_rpm: self.screw_speed_controller.get_uses_rpm(),
            },
            pressure_state: PressureState {
                target_bar: self
                    .screw_speed_controller
                    .get_target_pressure()
                    .get::<bar>(),
                wiring_error: self.screw_speed_controller.get_wiring_error(),
            },
            screw_state: ScrewState {
                target_rpm: self
                    .screw_speed_controller
                    .get_target_rpm()
                    .get::<revolution_per_minute>(),
            },
            heating_states: HeatingStates {
                nozzle: HeatingState {
                    target_temperature: self
                        .temperature_controller_nozzle
                        .heating
                        .target_temperature
                        .get::<degree_celsius>(),
                    wiring_error: self.temperature_controller_nozzle.heating.wiring_error,
                },
                front: HeatingState {
                    target_temperature: self
                        .temperature_controller_front
                        .heating
                        .target_temperature
                        .get::<degree_celsius>(),
                    wiring_error: self.temperature_controller_front.heating.wiring_error,
                },
                back: HeatingState {
                    target_temperature: self
                        .temperature_controller_back
                        .heating
                        .target_temperature
                        .get::<degree_celsius>(),
                    wiring_error: self.temperature_controller_back.heating.wiring_error,
                },
                middle: HeatingState {
                    target_temperature: self
                        .temperature_controller_middle
                        .heating
                        .target_temperature
                        .get::<degree_celsius>(),
                    wiring_error: self.temperature_controller_middle.heating.wiring_error,
                },
            },
            extruder_settings_state: ExtruderSettingsState {
                pressure_limit: self
                    .screw_speed_controller
                    .get_nozzle_pressure_limit()
                    .get::<bar>(),
                pressure_limit_enabled: self
                    .screw_speed_controller
                    .get_nozzle_pressure_limit_enabled(),
            },
            inverter_status_state: InverterStatusState {
                running: self.screw_speed_controller.inverter.status.running,
                forward_running: self.screw_speed_controller.inverter.status.forward_running,
                reverse_running: self.screw_speed_controller.inverter.status.reverse_running,
                up_to_frequency: self.screw_speed_controller.inverter.status.su,
                overload_warning: self.screw_speed_controller.inverter.status.ol,
                no_function: self.screw_speed_controller.inverter.status.no_function,
                output_frequency_detection: self.screw_speed_controller.inverter.status.fu,
                abc_fault: self.screw_speed_controller.inverter.status.abc_,
                fault_occurence: self.screw_speed_controller.inverter.status.fault_occurence,
            },
            pid_settings: PidSettingsStates {
                temperature: TemperaturePidStates {
                    front: TemperaturePid {
                        ki: self.temperature_controller_front.pid.get_ki(),
                        kp: self.temperature_controller_front.pid.get_kp(),
                        kd: self.temperature_controller_front.pid.get_kd(),
                        zone: String::from("front"),
                    },
                    middle: TemperaturePid {
                        ki: self.temperature_controller_middle.pid.get_ki(),
                        kp: self.temperature_controller_middle.pid.get_kp(),
                        kd: self.temperature_controller_middle.pid.get_kd(),
                        zone: String::from("middle"),
                    },
                    back: TemperaturePid {
                        ki: self.temperature_controller_back.pid.get_ki(),
                        kp: self.temperature_controller_back.pid.get_kp(),
                        kd: self.temperature_controller_back.pid.get_kd(),
                        zone: String::from("back"),
                    },
                    nozzle: TemperaturePid {
                        ki: self.temperature_controller_nozzle.pid.get_ki(),
                        kp: self.temperature_controller_nozzle.pid.get_kp(),
                        kd: self.temperature_controller_nozzle.pid.get_kd(),
                        zone: String::from("nozzle"),
                    },
                },
                pressure: PidSettings {
                    ki: self.screw_speed_controller.pid.get_ki(),
                    kp: self.screw_speed_controller.pid.get_kp(),
                    kd: self.screw_speed_controller.pid.get_kd(),
                },
            },
        }
    }
}

#[cfg(not(feature = "mock-machine"))]
impl ExtruderV3 {
    pub fn emit_state(&mut self) {
        use super::api::ExtruderV3Events;

        let state = self.build_state_event();
        let hash = hash_with_serde_model(self.screw_speed_controller.get_inverter_status());
        self.last_status_hash = Some(hash);
        let event = state.build();
        self.namespace.emit(ExtruderV3Events::State(event));
    }

    pub fn maybe_emit_state_event(&mut self) {
        let old_status_hash = match self.last_status_hash {
            Some(event) => event,
            None => {
                self.emit_state();
                return;
            }
        };
        let status = self.screw_speed_controller.get_inverter_status();
        let new_status_hash = hash_with_serde_model(status);
        if new_status_hash != old_status_hash {
            self.emit_state();
        }
    }

    pub fn emit_live_values(&mut self) {
        use std::time::Instant;

        use crate::extruder2::api::{ExtruderV3Events, LiveValuesEvent};
        let now = Instant::now();
        let combined_power = self.calculate_combined_power();
        self.update_total_energy(combined_power, now);

        let live_values = LiveValuesEvent {
            motor_status: self.screw_speed_controller.get_motor_status().into(),
            pressure: self.screw_speed_controller.get_pressure().get::<bar>(),
            nozzle_temperature: self
                .temperature_controller_nozzle
                .heating
                .temperature
                .get::<degree_celsius>(),
            front_temperature: self
                .temperature_controller_front
                .heating
                .temperature
                .get::<degree_celsius>(),
            back_temperature: self
                .temperature_controller_back
                .heating
                .temperature
                .get::<degree_celsius>(),
            middle_temperature: self
                .temperature_controller_middle
                .heating
                .temperature
                .get::<degree_celsius>(),
            nozzle_power: self
                .temperature_controller_nozzle
                .get_heating_element_wattage(),
            front_power: self
                .temperature_controller_front
                .get_heating_element_wattage(),
            back_power: self
                .temperature_controller_back
                .get_heating_element_wattage(),
            middle_power: self
                .temperature_controller_middle
                .get_heating_element_wattage(),
            combined_power,
            total_energy_kwh: self.total_energy_kwh,
        };

        let event = live_values.build();
        self.namespace.emit(ExtruderV3Events::LiveValues(event));
    }

    // === Steuerungsfunktionen mit emit_state ===

    pub fn set_nozzle_pressure_limit_is_enabled(&mut self, enabled: bool) {
        self.screw_speed_controller
            .set_nozzle_pressure_limit_is_enabled(enabled);
        self.emit_state();
    }

    pub fn set_nozzle_pressure_limit(&mut self, pressure: f64) {
        self.screw_speed_controller
            .set_nozzle_pressure_limit(Pressure::new::<bar>(pressure));
        self.emit_state();
    }

    pub fn enable_heating(&mut self) {
        self.temperature_controller_back.allow_heating();
        self.temperature_controller_front.allow_heating();
        self.temperature_controller_middle.allow_heating();
        self.temperature_controller_nozzle.allow_heating();
        self.emit_state();
    }

    pub fn set_rotation_state(&mut self, forward: bool) {
        self.screw_speed_controller.set_rotation_direction(forward);
        self.emit_state();
    }

    pub fn set_mode_state(&mut self, mode: ExtruderV3Mode) {
        self.switch_mode(mode);
        self.emit_state();
    }

    pub fn set_regulation(&mut self, uses_rpm: bool) {
        if !self.screw_speed_controller.get_uses_rpm() && uses_rpm {
            self.screw_speed_controller.set_target_screw_rpm(
                self.screw_speed_controller.target_rpm,
                AngularVelocity::new::<revolution_per_minute>(3000.0),
                2,
            );
            self.screw_speed_controller.set_uses_rpm(uses_rpm);
        }

        if self.screw_speed_controller.get_uses_rpm() && !uses_rpm {
            self.screw_speed_controller.set_uses_rpm(uses_rpm);
            self.screw_speed_controller.start_pressure_regulation();
        }
        self.emit_state();
    }

    pub fn set_target_pressure(&mut self, pressure: f64) {
        self.screw_speed_controller
            .set_target_pressure(Pressure::new::<bar>(pressure));
        self.emit_state();
    }

    pub fn set_target_rpm(&mut self, rpm: f64) {
        self.screw_speed_controller.set_target_screw_rpm(
            AngularVelocity::new::<revolution_per_minute>(rpm),
            AngularVelocity::new::<revolution_per_minute>(3000.0),
            2,
        );
        self.emit_state();
    }

    pub fn set_target_temperature(&mut self, target_temperature: f64, heating_type: HeatingType) {
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
        self.emit_state();
    }

    pub fn configure_pressure_pid(&mut self, settings: PidSettings) {
        self.screw_speed_controller
            .pid
            .configure(settings.ki, settings.kp, settings.kd);
        self.emit_state();
    }

    pub fn configure_temperature_pid(&mut self, settings: TemperaturePid) {
        match settings.zone.as_str() {
            "front" => {
                self.temperature_controller_front.pid.configure(
                    settings.ki,
                    settings.kp,
                    settings.kd,
                );
            }
            "middle" => {
                self.temperature_controller_middle.pid.configure(
                    settings.ki,
                    settings.kp,
                    settings.kd,
                );
            }
            "back" => {
                self.temperature_controller_back.pid.configure(
                    settings.ki,
                    settings.kp,
                    settings.kd,
                );
            }
            "nozzle" => {
                self.temperature_controller_nozzle.pid.configure(
                    settings.ki,
                    settings.kp,
                    settings.kd,
                );
            }
            _ => tracing::warn!("Unknown zone: {}", settings.zone),
        }
        self.emit_state();
    }
}
