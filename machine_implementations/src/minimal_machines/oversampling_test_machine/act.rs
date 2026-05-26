use qitech_lib::machines::{Machine, MachineDataRegistry, MachineIdentificationUnique};
use std::time::{Duration, Instant};

use crate::{MachineApi, minimal_machines::oversampling_test_machine::AnalogOutOversamplingMachine, };

impl Machine for AnalogOutOversamplingMachine {
    fn get_identification(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn act(&mut self, _machine_data: Option<&mut MachineDataRegistry>) {
        let now = Instant::now();

        if let Ok(msg) = self.api_receiver.try_recv() {
            self.act_machine_message(msg);
        }

        // Generate samples for both channels and push them to the device.
        // Called every main-loop tick (≥ EtherCAT cycle rate) so the device
        // always has fresh samples ready for the next PDI exchange.
        let samples_ch0 = self.generate_samples(0);
        let samples_ch1 = self.generate_samples(1);
        self.last_samples = [samples_ch0, samples_ch1];

        if let Ok(mut device) = self.el4732.try_borrow_mut() {
            device.set_output_samples(0, &samples_ch0);
            device.set_output_samples(1, &samples_ch1);
        }

        if now.duration_since(self.last_state_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            self.emit_state();
            self.last_state_emit = now;
        }

        if now.duration_since(self.last_live_values_emit) > Duration::from_secs_f64(1.0 / 10.0) {
            self.emit_live_values();
            self.last_live_values_emit = now;
        }
    }

    fn react(&mut self, _registry: &MachineDataRegistry) {}
}
