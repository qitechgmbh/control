#[cfg(not(feature = "mock-machine"))]
use std::time::Instant;

#[cfg(not(feature = "mock-machine"))]
use crate::{MachineAct, MachineMessage};

#[cfg(not(feature = "mock-machine"))]
use super::ExtruderV3;

#[cfg(not(feature = "mock-machine"))]
impl MachineAct for ExtruderV3 {
    fn act(&mut self, now: Instant) {
        use std::time::Duration;

        let msg = self.api_receiver.try_recv();
        match msg {
            Ok(msg) => {
                let _res = self.act_machine_message(msg);
            }
            Err(_) => (),
        };

        self.temperature_controller_back.update(now);
        self.temperature_controller_nozzle.update(now);
        self.temperature_controller_front.update(now);
        self.temperature_controller_middle.update(now);

        if self.mode == super::ExtruderV3Mode::Extrude {
            self.screw_speed_controller.update(now, true);
        } else {
            self.screw_speed_controller.update(now, false);
        }

        if self.mode == super::ExtruderV3Mode::Standby {
            self.turn_heating_off();
        }

        if self.mode == super::ExtruderV3Mode::Extrude
            && !self.screw_speed_controller.get_motor_enabled()
        {
            self.switch_to_heat();
        }

        let now = Instant::now();

        // more than 33ms have passed since last emit (30 "fps" target)
        if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            self.maybe_emit_state_event();
            // Emit live values at 30 FPS
            self.emit_live_values();
            self.last_measurement_emit = now;
        }
    }

    fn act_machine_message(&mut self, msg: MachineMessage) {
        match msg {
            MachineMessage::SubscribeNamespace(namespace) => {
                self.namespace.namespace = Some(namespace);
                self.emit_state();
                tracing::info!("extruder1 received subscribe");
            }
            MachineMessage::UnsubscribeNamespace => self.namespace.namespace = None,
            MachineMessage::HttpApiJsonRequest(value) => {
                use crate::MachineApi;

                let _res = self.api_mutate(value);
            }
            MachineMessage::ConnectToMachine(_machine_connection) => (),
            MachineMessage::DisconnectMachine(_machine_connection) =>
            /*Doesnt connect to any Machine so do nothing*/
            {
                ()
            }
        }
    }
}
