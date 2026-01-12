use smol::block_on;

use super::TestMachineStepper;
use crate::{MachineAct, MachineMessage, test_machine_stepper::AxisState};
use std::time::{Duration, Instant};

impl MachineAct for TestMachineStepper {
    fn act(&mut self, now: Instant) {
        if let Ok(msg) = self.api_receiver.try_recv() {
            self.act_machine_message(msg);
        }

        if now.duration_since(self.last_state_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            self.emit_state();
            self.last_state_emit = now;
        }

        let do_move = now.duration_since(self.last_move) > Duration::from_secs(1);

        if do_move {
            self.pos += 2000; // NEW target position
            self.last_move = now;
        }

        block_on(async {
            let mut stm = self.stepper.write().await;

            match self.axis_state {
                AxisState::Init => {
                    // Always keep enable asserted
                    stm.cmd_enable_pos(false);

                    // Start reference run (one pulse)
                    stm.cmd_reference(true);

                    self.axis_state = AxisState::Referencing;
                }

                AxisState::Referencing => {
                    // Keep reference mode selected
                    stm.cmd_reference(false);

                    // Wait until reference is done
                    let s2 = stm.txpdo.b[10]; // status byte S2
                    let referenced = (s2 & (1 << 2)) != 0; // Referenced bit

                    if referenced {
                        self.axis_state = AxisState::Ready;
                    }
                }

                AxisState::Ready => {
                    // Normal positioning logic
                    stm.cmd_enable_pos(false);

                    if do_move {
                        stm.set_positioning_setpoints(10000, 5000, self.pos);
                        stm.cmd_enable_pos(true);
                    }
                }
            }
        });
    }

    fn act_machine_message(&mut self, msg: MachineMessage) {
        match msg {
            MachineMessage::SubscribeNamespace(namespace) => {
                self.namespace.namespace = Some(namespace);
                self.emit_state();
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
