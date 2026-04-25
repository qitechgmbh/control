use std::{cell::RefCell, rc::Rc, time::Instant};

use anyhow::Error;
use qitech_lib::ethercat_hal::{devices::el3021::EL3021, io::analog_input::AnalogInputDevice};

use super::{AnalogInputTestMachine, api::AnalogInputTestMachineNamespace};
use crate::{MachineHardware, MachineMessage, MachineNew};

impl MachineNew for AnalogInputTestMachine {
    fn new<'maindevice>(hw: MachineHardware) -> Result<Self, Error> {
        let ai1: Rc<RefCell<dyn AnalogInputDevice>> =
            hw.try_get_ethercat_device_by_role::<EL3021>(1)?;
        let (tx, rx) = tokio::sync::mpsc::channel::<MachineMessage>(2);

        Ok(Self {
            api_receiver: rx,
            api_sender: tx,
            machine_identification_unique: hw.identification,
            namespace: AnalogInputTestMachineNamespace { namespace: None },

            last_measurement: Instant::now(),
            measurement_rate_hz: 1.0,

            analog_input: ai1,
        })
    }
}
