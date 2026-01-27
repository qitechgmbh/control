use std::time::{Duration, Instant};

use crate::{MachineAct, MachineMessage, MachineValues};

use super::new::SensorMachine;

impl MachineAct for SensorMachine {
    fn act_machine_message(&mut self, msg: MachineMessage) {
        if let MachineMessage::RequestValues(sender) = msg {
            let _ = sender.send_blocking(MachineValues {
                state: serde_json::Value::Null,
                live_values: serde_json::Value::Null,
            });
            sender.close();
        }
    }

    fn act(&mut self, now: Instant) {
        while let Ok(msg) = self.api_receiver.try_recv() {
            self.act_machine_message(msg);
        }

        // Print at 5 Hz
        if now.duration_since(self.last_print) >= Duration::from_millis(200) {
            let ai1_norm = self.ai1.get_normalized();
            
            // EL3001: normalized = voltage / 10
            // Therefore: voltage = normalized × 10
            let voltage_v = ai1_norm * 10.0;

            tracing::info!(
                "SENSOR_MACHINE EL3001: AI1={:.3}V (norm={:.4})",
                voltage_v, ai1_norm
            );
            
            self.last_print = now;
        }
    }
}