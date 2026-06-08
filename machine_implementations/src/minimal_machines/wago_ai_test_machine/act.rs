use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use qitech_lib::{
    ethercat_hal::io::analog_input::{AnalogInputDevice, physical::AnalogInputValue},
    machines::{Machine, MachineDataRegistry, MachineError},
};
use qitech_lib::units::electric_current::milliampere;

use super::WagoAiTestMachine;
use crate::MachineApi;

impl Machine for WagoAiTestMachine {
    fn act(
        &mut self,
        _machine_data: Option<&mut MachineDataRegistry>,
    ) -> Result<(), MachineError> {
        let now = Instant::now();

        if let Ok(msg) = self.receiver.try_recv() {
            self.act_machine_message(msg);
        }

        if now.duration_since(self.last_measurement)
            > Duration::from_secs_f64(1.0 / self.measurement_rate_hz)
        {
            let analog_input_device = self.analog_input_device.borrow();
            let range = analog_input_device.analog_input_range();
            let mut values = [0.0f64; 4];
            let mut wiring_errors = [false; 4];

            for i in 0..4 {
                if let Ok(input) = analog_input_device.get_input(i) {
                    wiring_errors[i] = input.wiring_error;
                    match input.get_physical(&range) {
                        AnalogInputValue::Current(quantity) => {
                            values[i] = quantity.get::<milliampere>();
                        }
                        AnalogInputValue::Potential(_) => {}
                    }
                }
            }
            drop(analog_input_device);

            let now_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Now is expected to be after UNIX_EPOCH")
                .as_millis();

            self.emit_analog_inputs(values, now_ms);
            self.emit_wiring_errors(wiring_errors);
            self.last_measurement = Instant::now();
        }

        Ok(())
    }

    fn react(&mut self, _registry: &MachineDataRegistry) {}

    fn get_identification(&self) -> qitech_lib::machines::MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }
}
