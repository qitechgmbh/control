use control_core::socketio::namespace::NamespaceCacheingLogic;
use qitech_lib::machines::{Machine, MachineDataRegistry, MachineError};

use crate::{
    MachineApi,
    minimal_machines::digital_input_test_machine::{
        DigitalInputTestMachine,
        api::{self, StateEvent},
    },
};

impl Machine for DigitalInputTestMachine {
    fn act(&mut self, _registry: Option<&mut MachineDataRegistry>) -> Result<(), MachineError> {
        let now = std::time::Instant::now();

        let res = self.receiver.try_recv();
        match res {
            Ok(msg) => self.act_machine_message(msg),
            Err(_) => (),
        };

        let mut el2004 = self.el2004.borrow_mut();
        el2004.rxpdo.channel2 =
            Some(qitech_lib::ethercat_hal::pdo::basic::BoolPdoObject { value: true });

        let digital_input_device = self.digital_input_device.borrow_mut();
        let port_count = digital_input_device.get_port_count();
        for i in 0..port_count {
            let value = match digital_input_device.get_input(i) {
                Ok(v) => v,
                Err(_e) => false,
            };

            if i < 4 {
                self.led_on[i] = value;
            }
        }

        if now.duration_since(self.last_state_emit) > std::time::Duration::from_secs_f64(1.0 / 30.0)
        {
            self.namespace
                .emit(api::DigitalInputTestMachineEvents::State(
                    StateEvent {
                        led_on: self.led_on,
                    }
                    .build(),
                ));
            self.last_state_emit = now;
        }
        Ok(())
    }

    fn react(&mut self, _registry: &qitech_lib::machines::MachineDataRegistry) {
        // react to specific machines data
    }

    fn get_identification(&self) -> qitech_lib::machines::MachineIdentificationUnique {
        return self.machine_identification_unique.into();
    }
}
