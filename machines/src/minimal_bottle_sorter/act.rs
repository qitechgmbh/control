use super::MinimalBottleSorter;
use crate::{MachineAct, MachineMessage};
use std::time::{Duration, Instant};

impl MachineAct for MinimalBottleSorter {
    fn act(&mut self, now: Instant) {
        if let Ok(msg) = self.api_receiver.try_recv() {
            self.act_machine_message(msg);
        }

        // Update output pulses (check every ~10ms)
        static mut LAST_PULSE_UPDATE: Option<Instant> = None;
        unsafe {
            let should_update = match LAST_PULSE_UPDATE {
                Some(last) => now.duration_since(last) > Duration::from_millis(10),
                None => true,
            };

            if should_update {
                let delta_ms = match LAST_PULSE_UPDATE {
                    Some(last) => now.duration_since(last).as_millis() as u32,
                    None => 10,
                };
                self.update_pulses(delta_ms);
                LAST_PULSE_UPDATE = Some(now);
            }
        }

        // Emit state at 30 Hz
        if now.duration_since(self.last_state_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            self.emit_state();
            self.last_state_emit = now;
        }

        // Emit live values at 10 Hz
        if now.duration_since(self.last_live_values_emit) > Duration::from_secs_f64(1.0 / 10.0) {
            self.emit_live_values();
            self.last_live_values_emit = now;
        }
    }

    fn act_machine_message(&mut self, msg: MachineMessage) {
        match msg {
            MachineMessage::SubscribeNamespace(namespace) => {
                self.namespace.namespace = Some(namespace);
                self.emit_state();
                self.emit_live_values();
            }
            MachineMessage::UnsubscribeNamespace => self.namespace.namespace = None,
            MachineMessage::HttpApiJsonRequest(value) => {
                use crate::MachineApi;
                let _res = self.api_mutate(value);
            }
            MachineMessage::ConnectToMachine(_machine_connection) => {
                // Does not connect to any Machine; do nothing
            }
            MachineMessage::DisconnectMachine(_machine_connection) => {
                // Does not connect to any Machine; do nothing
            }
        }
    }
}
