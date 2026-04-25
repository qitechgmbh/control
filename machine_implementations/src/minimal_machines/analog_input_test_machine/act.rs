use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use qitech_lib::{
    ethercat_hal::{devices::el3021::EL3021Port, io::analog_input::physical::AnalogInputValue},
    machines::{Machine, MachineDataRegistry, MachineIdentificationUnique},
};

use crate::{
    MachineApi, MachineMessage, MachineValues,
    minimal_machines::analog_input_test_machine::AnalogInputTestMachine,
};

impl Machine for AnalogInputTestMachine {
    fn act(&mut self, machine_data: Option<&mut MachineDataRegistry>) {
        let now = Instant::now();
        let recv = self.api_receiver.try_recv();
        if let Ok(msg) = recv {
            self.act_machine_message(msg);
        }
        if now.duration_since(self.last_measurement)
            > Duration::from_secs_f64(1.0 / self.measurement_rate_hz)
        {
            let analog_input_clone = self.analog_input.clone(); //Might be possible without cloning
            let analog_input = analog_input_clone.borrow();
            let measured_value = analog_input
                .get_input(0)
                .expect("analog input must exist")
                .get_physical(&analog_input.analog_input_range());
            match measured_value {
                AnalogInputValue::Potential(_quantity) => {
                    // Don't do anything
                }
                AnalogInputValue::Current(quantity) => {
                    let now_milliseconds = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("Now is expected to be after UNIX_EPOCH")
                        .as_millis();
                    self.emit_measurement(quantity.value, now_milliseconds);
                }
            }
            self.last_measurement = Instant::now();
        }
    }

    fn get_identification(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn react(&mut self, registry: &qitech_lib::machines::MachineDataRegistry) {}
}
