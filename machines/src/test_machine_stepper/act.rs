use smol::block_on;

use super::TestMachineStepper;
use crate::{MachineAct, MachineMessage, test_machine_stepper::SpeedCtlState};
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

        block_on(async {
            let mut stm = self.stepper.write().await;

            // Desired setpoints (example)
            let cmd_vel: i16 = 25000; // steps/s (sign = direction)
            let cmd_acc: u16 = 10000;  // steps/s^2 (must be > 0)

            // Always write setpoints continuously (safe)
            stm.set_speed_setpoint(cmd_vel, cmd_acc);

            // Reset ack if module reports reset
            if stm.reset_active() && !self.reset_quit_pulsed {
                stm.apply_reset_quit(true); // one-cycle pulse
                self.reset_quit_pulsed = true;
            } else {
                stm.apply_reset_quit(false);
                if !stm.reset_active() {
                    self.reset_quit_pulsed = false;
                }
            }

            // If error is active, go to error ack state
            if stm.error_active() {
                self.speed_state = SpeedCtlState::ErrorAck;
            }

            match self.speed_state {
                SpeedCtlState::Init => {
                    // Ensure clean edges
                    self.start_pulsed = false;
                    self.error_quit_pulsed = false;

                    // Request enable + speed mode (no start yet)
                    stm.apply_speed_control_state(true, true, false);

                    self.speed_state = SpeedCtlState::WaitReady;
                }

                SpeedCtlState::WaitReady => {
                    stm.apply_speed_control_state(true, true, false);

                    // Need Ready + Stop_N_ACK before start edge is accepted
                    if stm.ready() && stm.stop_n_ack() {
                        self.speed_state = SpeedCtlState::SelectMode;
                    }
                }

                SpeedCtlState::SelectMode => {
                    // Keep requesting speed mode until ACK is set
                    stm.apply_speed_control_state(true, true, false);

                    if stm.speed_mode_ack() {
                        self.speed_state = SpeedCtlState::StartSpeed;
                    }
                }

                SpeedCtlState::StartSpeed => {
                    // Pulse Start once (rising edge)
                    let start_pulse = !self.start_pulsed;
                    stm.apply_speed_control_state(true, true, start_pulse);

                    if start_pulse {
                        self.start_pulsed = true;
                    } else if stm.start_ack() {
                        // Once accepted, go running
                        self.speed_state = SpeedCtlState::Running;
                    }
                }

                SpeedCtlState::Running => {
                    // Keep enabled and in speed mode, no start pulse
                    stm.apply_speed_control_state(true, true, false);

                    // If you want to update speed "on the fly", pulse Start again
                    // when you change setpoints (same mechanism).
                }

                SpeedCtlState::ErrorAck => {
                    // Keep enable request on; ACK error with a pulse on C2.7 (Error_Quit)
                    stm.apply_speed_control_state(true, true, false);

                    if !self.error_quit_pulsed {
                        stm.apply_error_quit(true);
                        self.error_quit_pulsed = true;
                    } else {
                        stm.apply_error_quit(false);
                    }

                    // Once error clears, restart sequence
                    if !stm.error_active() {
                        self.error_quit_pulsed = false;
                        self.start_pulsed = false;
                        self.speed_state = SpeedCtlState::WaitReady;
                    }
                }
            }

            // Debug prints
            println!(
                "S1={:08b} S2={:08b} S3={:08b} | C1_OUT={:08b} C2_OUT={:08b} C3_OUT={:08b} | v_act={} | pos={}",
                stm.txpdo.b[11],
                stm.txpdo.b[10],
                stm.txpdo.b[9],
                stm.rxpdo.b[11],
                stm.rxpdo.b[10],
                stm.rxpdo.b[9],
                stm.actual_velocity(),
                stm.position(),
            );
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
            MachineMessage::ConnectToMachine(_machine_connection) => {}
            MachineMessage::DisconnectMachine(_machine_connection) => {}
        }
    }
}

