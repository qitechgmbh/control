use std::time::Instant;

use super::{
    MockMachine,
    api::{MockMachineNamespace, Mode},
};
use crate::{MachineNewHardware, MachineNewParams, MachineNewTrait};
use anyhow::Error;
use units::f64::Frequency;
use units::frequency::hertz;

impl MachineNewTrait for MockMachine {
    fn new<'maindevice, 'subdevices>(
        params: &MachineNewParams<'maindevice, 'subdevices, '_, '_, '_, '_, '_>,
    ) -> Result<Self, Error>
    where
        Self: Sized,
    {
        // Mock machine can work with either Serial or Ethercat hardware
        // For the mock machine, we don't need to actually use the hardware
        // We just validate that we have the expected hardware type
        match params.hardware {
            MachineNewHardware::Serial(_) => {
                // For serial mode, we could potentially use the serial device if needed
                // but for a mock machine, we'll just note it and proceed
            }
            MachineNewHardware::Ethercat(_) => {
                // For ethercat mode, we could potentially use the ethercat devices
                // but for a mock machine, we'll just note it and proceed
            }
        }

        let now = Instant::now();
        let (sender, receiver) = smol::channel::unbounded();
        let mut mock_machine = Self {
            main_sender: params.main_thread_channel.clone(),
            api_receiver: receiver,
            api_sender: sender,
            machine_identification_unique: params.get_machine_identification_unique(),
            namespace: MockMachineNamespace {
                namespace: params.namespace.clone(),
            },
            last_measurement_emit: now,
            t_0: now, // Initialize start time to current time
            frequency1: Frequency::new::<hertz>(0.1), // Default frequency1 of 100 mHz
            frequency2: Frequency::new::<hertz>(0.2), // Default frequency2 of 200 mHz
            frequency3: Frequency::new::<hertz>(0.5), // Default frequency3 of 500 mHz
            mode: Mode::Standby, // Start in standby mode
            emitted_default_state: false,
            last_emitted_event: None,
        };

        mock_machine.emit_state();

        Ok(mock_machine)
    }
}
