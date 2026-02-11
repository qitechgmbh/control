use super::Gluetex;
use crate::MachineCrossConnectionState;
use crate::gluetex::api::{
    GluetexEvents, HeatingPidSettings, HeatingZone, LiveValuesEvent, SpoolAutomaticActionMode,
};
use crate::machine_identification::MachineIdentificationUnique;
use control_core::socketio::event::BuildEvent;
use control_core::socketio::namespace::NamespaceCacheingLogic;

impl Gluetex {
    pub fn emit_state(&mut self) {
        let event = self.build_state_event().build();
        self.namespace.emit(GluetexEvents::State(event));
    }

    pub fn emit_live_values(&mut self) {
        let event = self.build_live_values_event().build();
        self.namespace.emit(GluetexEvents::LiveValues(event));
    }

    pub fn reset_spool_progress(&mut self) {
        self.live_values.spool_progress = 0.0;
    }

    pub fn set_connected_machine(&mut self, ident: MachineIdentificationUnique) {
        self.connected_machine_state = MachineCrossConnectionState {
            machine_identification_unique: Some(ident),
            is_available: true,
        };
    }

    pub fn disconnect_machine(&mut self, ident: MachineIdentificationUnique) {
        if self
            .connected_machine_state
            .machine_identification_unique
            .as_ref()
            == Some(&ident)
        {
            self.connected_machine_state = MachineCrossConnectionState {
                machine_identification_unique: None,
                is_available: false,
            };
        }
    }

    pub fn set_heating_target(&mut self, zone: HeatingZone, target_temperature: f64) {
        let zone_state = match zone {
            HeatingZone::Zone1 => &mut self.heating_states.zone_1,
            HeatingZone::Zone2 => &mut self.heating_states.zone_2,
            HeatingZone::Zone3 => &mut self.heating_states.zone_3,
            HeatingZone::Zone4 => &mut self.heating_states.zone_4,
            HeatingZone::Zone5 => &mut self.heating_states.zone_5,
            HeatingZone::Zone6 => &mut self.heating_states.zone_6,
        };
        zone_state.target_temperature = target_temperature;
    }

    pub fn set_heating_autotune(&mut self, zone: HeatingZone, active: bool) {
        let zone_state = match zone {
            HeatingZone::Zone1 => &mut self.heating_states.zone_1,
            HeatingZone::Zone2 => &mut self.heating_states.zone_2,
            HeatingZone::Zone3 => &mut self.heating_states.zone_3,
            HeatingZone::Zone4 => &mut self.heating_states.zone_4,
            HeatingZone::Zone5 => &mut self.heating_states.zone_5,
            HeatingZone::Zone6 => &mut self.heating_states.zone_6,
        };
        zone_state.autotuning_active = active;
        zone_state.autotuning_progress = 0.0;
    }

    pub fn set_heating_pid(&mut self, settings: HeatingPidSettings) {
        let zone_key = settings.zone.to_lowercase();
        let target = match zone_key.as_str() {
            "zone1" | "zone_1" => Some(&mut self.heating_pid_settings.zone_1),
            "zone2" | "zone_2" => Some(&mut self.heating_pid_settings.zone_2),
            "zone3" | "zone_3" => Some(&mut self.heating_pid_settings.zone_3),
            "zone4" | "zone_4" => Some(&mut self.heating_pid_settings.zone_4),
            "zone5" | "zone_5" => Some(&mut self.heating_pid_settings.zone_5),
            "zone6" | "zone_6" => Some(&mut self.heating_pid_settings.zone_6),
            _ => None,
        };

        if let Some(target) = target {
            target.kp = settings.kp;
            target.ki = settings.ki;
            target.kd = settings.kd;
            target.zone = settings.zone;
        }
    }

    pub fn set_spool_automatic_action(&mut self, mode: SpoolAutomaticActionMode) {
        self.spool_automatic_action_state
            .spool_automatic_action_mode = mode;
    }

    pub fn set_live_values(&mut self, live_values: LiveValuesEvent) {
        self.live_values = live_values;
    }
}
