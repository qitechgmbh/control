use std::time::{Duration, Instant};

use crate::{MachineAct, MachineMessage, MachineValues};

use super::new::SensorMachine;

impl MachineAct for SensorMachine {
    fn act_machine_message(&mut self, msg: MachineMessage) {
        // ignore frontend; respond to RequestValues if system asks
        if let MachineMessage::RequestValues(sender) = msg {
            let _ = sender.send_blocking(MachineValues {
                state: serde_json::Value::Null,
                live_values: serde_json::Value::Null,
            });
            sender.close();
        }
    }

    fn act(&mut self, now: Instant) {
        // drain messages
        while let Ok(msg) = self.api_receiver.try_recv() {
            self.act_machine_message(msg);
        }

        // Print at 5 Hz
        if now.duration_since(self.last_print) >= Duration::from_millis(200) {
            let ai1_norm = self.ai1.get_normalized();
            let ai2_norm = self.ai2.get_normalized();
            let ai1_phys = self.ai1.get_physical(); // AnalogInputValue (e.g., volts/mA)
            let ai2_phys = self.ai2.get_physical();

            println!(
                "SENSOR_MACHINE EL3062 PDO: AI1 norm={:.4} phys={:?} | AI2 norm={:.4} phys={:?}",
                ai1_norm, ai1_phys, ai2_norm, ai2_phys
            );
            self.last_print = now;
        }
    }
}
