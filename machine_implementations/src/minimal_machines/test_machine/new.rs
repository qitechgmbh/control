use std::time::Instant;

use anyhow::Error;

use qitech_lib::ethercat_hal::devices::el2004::EL2004;

use crate::{MachineHardware, MachineMessage, MachineNew};

use super::{TestMachine, api::TestMachineNamespace};

impl MachineNew for TestMachine {
    fn new(hw: MachineHardware) -> Result<Self, Error> {
        let el2004 = hw.try_get_ethercat_device_by_role::<EL2004>(1)?;

        let (sender, receiver) = tokio::sync::mpsc::channel::<MachineMessage>(10);
        let mut machine = Self {
            receiver,
            sender,
            machine_identification_unique: hw.identification,
            namespace: TestMachineNamespace { namespace: None },
            last_state_emit: Instant::now(),
            led_on: [false; 4],
            el2004,
        };

        machine.emit_state();
        Ok(machine)
    }
}
