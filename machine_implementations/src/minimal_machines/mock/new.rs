use std::time::Instant;

use anyhow::Error;
use units::f64::Frequency;
use units::frequency::hertz;

use crate::{MachineHardware, MachineMessage, MachineNew};

use super::{MockMachine, api::{MockMachineNamespace, Mode}};

impl MachineNew for MockMachine {
    fn new(hw: MachineHardware) -> Result<Self, Error> {
        let now = Instant::now();
        let (sender, receiver) = tokio::sync::mpsc::channel::<MachineMessage>(10);
        let mut machine = Self {
            receiver,
            sender,
            machine_identification_unique: hw.identification,
            namespace: MockMachineNamespace { namespace: None },
            last_measurement_emit: now,
            t_0: now,
            frequency1: Frequency::new::<hertz>(0.1),
            frequency2: Frequency::new::<hertz>(0.2),
            frequency3: Frequency::new::<hertz>(0.5),
            mode: Mode::Standby,
            emitted_default_state: false,
            last_emitted_event: None,
        };

        machine.emit_state();
        Ok(machine)
    }
}
