use crate::extruder1::{
    ExtruderV2Mode, HeatingType,
    api::{ExtruderV2Events, LiveValuesEvent, ModeState, PidSettings, StateEvent, TemperaturePid},
    mock::ExtruderV2,
};

use control_core::{
    helpers::hasher_serializer::hash_with_serde_model,
    socketio::{event::BuildEvent, namespace::NamespaceCacheingLogic},
};

impl ExtruderV2 {
    pub fn build_state_event(&mut self) -> StateEvent {
        // bad performance wise, but doesnt matter its only a mock machine
        StateEvent {
            is_default_state: !std::mem::replace(&mut self.emitted_default_state, true),
            rotation_state: self.rotation_state.clone(),
            mode_state: self.mode_state.clone(),
            regulation_state: self.regulation_state.clone(),
            pressure_state: self.pressure_state.clone(),
            screw_state: self.screw_state.clone(),
            heating_states: self.heating_states.clone(),
            extruder_settings_state: self.extruder_settings_state.clone(),
            inverter_status_state: self.inverter_status_state.clone(),
            pid_settings: self.pid_settings.clone(),
        }
    }
}

impl ExtruderV2 {
    pub fn emit_state(&mut self) {
        let state = self.build_state_event();
        let hash = hash_with_serde_model(self.inverter_status_state.clone());
        self.last_status_hash = Some(hash);
        let event = state.build();
        self.namespace.emit(ExtruderV2Events::State(event));
    }

    pub fn maybe_emit_state_event(&mut self) {
        let old_status_hash = match self.last_status_hash {
            Some(event) => event,
            None => {
                self.emit_state();
                return;
            }
        };
        let status = self.inverter_status_state.clone();
        let new_status_hash = hash_with_serde_model(status);
        if new_status_hash != old_status_hash {
            self.emit_state();
        }
    }

    pub fn emit_live_values(&mut self) {
        let live_values = LiveValuesEvent {
            motor_status: self.motor_status.clone(),
            pressure: self.pressure,
            nozzle_temperature: self.nozzle_temperature,
            front_temperature: self.front_temperature,
            back_temperature: self.back_temperature,
            middle_temperature: self.middle_temperature,
            nozzle_power: self.nozzle_power,
            front_power: self.front_power,
            back_power: self.back_power,
            middle_power: self.middle_power,
            combined_power: self.combined_power,
            total_energy_kwh: self.total_energy_kwh,
        };

        let event = live_values.build();
        self.namespace.emit(ExtruderV2Events::LiveValues(event));
    }

    pub fn set_nozzle_pressure_limit_is_enabled(&mut self, enabled: bool) {
        self.extruder_settings_state.pressure_limit_enabled = enabled;
        self.emit_state();
    }

    pub fn set_nozzle_pressure_limit(&mut self, pressure: f64) {
        self.extruder_settings_state.pressure_limit = pressure;
        self.emit_state();
    }

    pub fn enable_heating(&mut self) {
        self.back_heating_allowed = true;
        self.front_heating_allowed = true;
        self.nozzle_heating_allowed = true;
        self.back_heating_allowed = true;
        self.emit_state();
    }

    pub fn set_rotation_state(&mut self, forward: bool) {
        self.rotation_state.forward = forward;
        self.emit_state();
    }

    pub fn set_mode_state(&mut self, mode: ExtruderV2Mode) {
        self.mode_state = ModeState { mode };
        self.emit_state();
    }

    pub fn set_regulation(&mut self, uses_rpm: bool) {
        self.regulation_state.uses_rpm = uses_rpm;
        self.emit_state();
    }

    pub fn set_target_pressure(&mut self, pressure: f64) {
        self.target_pressure = pressure;
        self.emit_state();
    }

    pub fn set_target_rpm(&mut self, rpm: f64) {
        self.screw_state.target_rpm = rpm;
        self.emit_state();
    }

    pub fn set_target_temperature(&mut self, target_temperature: f64, heating_type: HeatingType) {
        match heating_type {
            HeatingType::Nozzle => {
                self.heating_states.nozzle.target_temperature = target_temperature
            }
            HeatingType::Front => self.heating_states.front.target_temperature = target_temperature,
            HeatingType::Back => self.heating_states.back.target_temperature = target_temperature,
            HeatingType::Middle => {
                self.heating_states.middle.target_temperature = target_temperature
            }
        }
        self.emit_state();
    }

    pub fn configure_pressure_pid(&mut self, settings: PidSettings) {
        self.pid_settings.pressure.ki = settings.ki;
        self.pid_settings.pressure.kp = settings.kp;
        self.pid_settings.pressure.kd = settings.kd;
        self.emit_state();
    }

    pub fn configure_temperature_pid(&mut self, settings: TemperaturePid) {
        match settings.zone.as_str() {
            "front" => {
                self.pid_settings.temperature.front.ki = settings.ki;
                self.pid_settings.temperature.front.kp = settings.kp;
                self.pid_settings.temperature.front.kd = settings.kd;
            }
            "middle" => {
                self.pid_settings.temperature.middle.ki = settings.ki;
                self.pid_settings.temperature.middle.kp = settings.kp;
                self.pid_settings.temperature.middle.kd = settings.kd;
            }
            "back" => {
                self.pid_settings.temperature.back.ki = settings.ki;
                self.pid_settings.temperature.back.kp = settings.kp;
                self.pid_settings.temperature.back.kd = settings.kd;
            }
            "nozzle" => {
                self.pid_settings.temperature.nozzle.ki = settings.ki;
                self.pid_settings.temperature.nozzle.kp = settings.kp;
                self.pid_settings.temperature.nozzle.kd = settings.kd;
            }
            _ => tracing::warn!("Unknown zone: {}", settings.zone),
        }
        self.emit_state();
    }
}
