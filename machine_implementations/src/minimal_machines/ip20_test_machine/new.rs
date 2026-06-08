use std::time::Instant;

use anyhow::Error;

use qitech_lib::ethercat_hal::devices::wago_modules::ip20_ec_di8_do8::IP20EcDi8Do8;

use crate::{MachineHardware, MachineMessage, MachineNew};

use super::{IP20TestMachine, api::IP20TestMachineNamespace};

impl MachineNew for IP20TestMachine {
    fn new(hw: MachineHardware) -> Result<Self, Error> {
        let device = hw.try_get_ethercat_device_by_role::<IP20EcDi8Do8>(0)?;

        let (sender, receiver) = tokio::sync::mpsc::channel::<MachineMessage>(10);
        let mut machine = Self {
            receiver,
            sender,
            machine_identification_unique: hw.identification,
            namespace: IP20TestMachineNamespace { namespace: None },
            last_state_emit: Instant::now(),
            last_live_values_emit: Instant::now(),
            outputs: [false; 8],
            inputs: [false; 8],
            device,
        };

        machine.emit_state();
        machine.emit_live_values();
        Ok(machine)
    }
}
