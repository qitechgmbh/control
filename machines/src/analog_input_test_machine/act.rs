use std::time::{Duration, Instant};

use crate::{MachineAct, MachineApi, analog_input_test_machine::AnalogInputTestMachine};

impl MachineAct for AnalogInputTestMachine {
    fn act_machine_message(&mut self, msg: crate::MachineMessage) {
        match msg {
            crate::MachineMessage::SubscribeNamespace(namespace) => todo!(),
            crate::MachineMessage::UnsubscribeNamespace => todo!(),
            crate::MachineMessage::HttpApiJsonRequest(value) => {
                use crate::MachineApi;
                let _res = self.api_mutate(value);
            }
            crate::MachineMessage::ConnectToMachine(machine_connection) => todo!(),
            crate::MachineMessage::DisconnectMachine(machine_connection) => todo!(),
        }
    }

    fn act(&mut self, now: std::time::Instant) {
        let recv = self.api_receiver.try_recv();
        if let Ok(msg) = recv {
            self.act_machine_message(msg);
        }
        if now.duration_since(self.last_measurement)
            > Duration::from_secs_f64(1.0 / self.measurement_rate_hz)
        {
            let measured_value = self.analog_input.get_physical();
            match measured_value {
                ethercat_hal::io::analog_input::physical::AnalogInputValue::Potential(quantity) => {
                    todo!()
                }
                ethercat_hal::io::analog_input::physical::AnalogInputValue::Current(quantity) => {
                    println!("{quantity:?}");
                }
            }
            self.last_measurement = Instant::now();
        }
    }
}
