use crate::machines::{MACHINE_EXTRUDER_V1, VENDOR_QITECH};
use api::{
    ExtruderSettingsState, ExtruderV2Events, ExtruderV2Namespace, HeatingState, HeatingStates,
    InverterStatusState, LiveValuesEvent, ModeState, PidSettings, PidSettingsStates, PressureState,
    RegulationState, RotationState, ScrewState, StateEvent,
};
use control_core::helpers::hasher_serializer::hash_with_serde_model;
use control_core::socketio::event::BuildEvent;
use control_core::{
    machines::identification::MachineIdentification, socketio::namespace::NamespaceCacheingLogic,
};
use control_core_derive::Machine;
use screw_speed_controller::ScrewSpeedController;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use temperature_controller::TemperatureController;
use uom::si::{
    angular_velocity::{AngularVelocity, revolution_per_minute},
    electric_current::ampere,
    electric_potential::volt,
    f64::{Pressure, ThermodynamicTemperature},
    pressure::bar,
    thermodynamic_temperature::degree_celsius,
};
pub mod act;
pub mod api;
pub mod mitsubishi_cs80;
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

#[derive(Debug, Machine)]
pub struct ExtruderV2 {
    namespace: ExtruderV2Namespace,
    last_measurement_emit: Instant,
    last_state_event_hash: Option<u64>,
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

impl ExtruderV2 {
    pub fn build_state_event(&mut self) -> StateEvent {
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
                temperature: PidSettings {
                    ki: 0.0, // TODO: Add temperature PID settings when available
                    kp: 0.0,
                    kd: 0.0,
                },
                pressure: PidSettings {
                    ki: self.screw_speed_controller.pid.get_ki(),
                    kp: self.screw_speed_controller.pid.get_kp(),
                    kd: self.screw_speed_controller.pid.get_kd(),
                },
            },
        }
    }

    pub fn maybe_emit_state_event(&mut self) {
        let old_state_hash = match self.last_state_event_hash.clone() {
            Some(event) => event,
            None => {
                self.emit_state();
                return;
            }
        };
        let mut new_state = self.build_state_event();
        let new_state_hash = hash_with_serde_model(&mut new_state);
        let should_emit = new_state_hash != old_state_hash;
        if should_emit {
            self.namespace
                .emit(ExtruderV2Events::State(new_state.build()));
            self.last_state_event_hash = Some(new_state_hash);
        }
    }
}

impl std::fmt::Display for ExtruderV2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ExtruderV2")
    }
}

impl ExtruderV2 {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: MACHINE_EXTRUDER_V1,
    };
}

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

    pub fn emit_live_values(&mut self) {
        let now = Instant::now();
        let combined_power = self.calculate_combined_power();

        // Update energy consumption
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
        self.namespace.emit(ExtruderV2Events::LiveValues(event));
    }

    pub fn emit_state(&mut self) {
        let state = self.build_state_event();
        let hash = hash_with_serde_model(state.clone());
        self.last_state_event_hash = Some(hash);
        let event = state.build();
        self.namespace.emit(ExtruderV2Events::State(event));
    }

    // Extruder Settings Api Impl
    const fn set_nozzle_pressure_limit_is_enabled(&mut self, enabled: bool) {
        self.screw_speed_controller
            .set_nozzle_pressure_limit_is_enabled(enabled);
    }

    /// pressure is represented as bar
    fn set_nozzle_pressure_limit(&mut self, pressure: f64) {
        let nozzle_pressure_limit = Pressure::new::<bar>(pressure);
        self.screw_speed_controller
            .set_nozzle_pressure_limit(nozzle_pressure_limit);
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

    const fn enable_heating(&mut self) {
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
    }

    fn reset_inverter(&mut self) {
        self.screw_speed_controller.inverter.reset_inverter();
    }

    fn set_mode_state(&mut self, mode: ExtruderV2Mode) {
        self.switch_mode(mode);
    }
}

// Motor
impl ExtruderV2 {
    fn set_regulation(&mut self, uses_rpm: bool) {
        if !self.screw_speed_controller.get_uses_rpm() && uses_rpm {
            self.screw_speed_controller
                .set_target_screw_rpm(self.screw_speed_controller.target_rpm);
            self.screw_speed_controller.set_uses_rpm(uses_rpm);
        }

        if self.screw_speed_controller.get_uses_rpm() && !uses_rpm {
            self.screw_speed_controller.set_uses_rpm(uses_rpm);
            self.screw_speed_controller.start_pressure_regulation();
        }
        self.emit_state();
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
    }
}

impl ExtruderV2 {
    const fn configure_pressure_pid(&mut self, settings: PidSettings) {
        self.screw_speed_controller
            .pid
            .configure(settings.ki, settings.kp, settings.kd);
    }
}
