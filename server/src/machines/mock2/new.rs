use std::time::Instant;

use super::{
    Mock2Machine,
    api::{Mock2MachineNamespace, Mode},
};
use anyhow::Error;
use control_core::machines::new::{MachineNewHardware, MachineNewTrait};
use uom::si::{f64::Frequency, frequency::hertz};

impl MachineNewTrait for Mock2Machine {
    fn new<'maindevice, 'subdevices>(
        params: &control_core::machines::new::MachineNewParams<
            'maindevice,
            'subdevices,
            '_,
            '_,
            '_,
            '_,
            '_,
        >,
    ) -> Result<Self, Error>
    where
        Self: Sized,
    {
        // Mock2 machine can work with either Serial or Ethercat hardware
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

        let mut mock_machine = Self {
            namespace: Mock2MachineNamespace::new(params.socket_queue_tx.clone()),
            last_measurement_emit: now,
            t_0: now,                                // Initialize start time to current time
            frequency: Frequency::new::<hertz>(0.5), // Default frequency of 500 mHz
            mode: Mode::Standby,                     // Start in standby mode
            last_emitted_state: None,                // No previous state emissions
            emitted_default_state: false,
            connected_mock: None,
            machine_manager: params.machine_manager.clone(),
        };

        mock_machine.emit_state();

        Ok(mock_machine)
    }
}
