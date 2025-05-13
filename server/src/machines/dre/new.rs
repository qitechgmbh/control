use std::time::Instant;

use crate::serial::{devices::dre::Dre, registry::SERIAL_DEVICE_REGISTRY};

use super::{DreMachine, api::DreMachineNamespace};
use anyhow::Error;
use control_core::machines::new::MachineNewTrait;

impl MachineNewTrait for DreMachine {
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
        let hardware_serial = match params.hardware {
            control_core::machines::new::MachineNewHardware::Serial(serial) => *serial,
            _ => return Err(Error::msg("Invalid hardware type for DreMachine")),
        };

        // downcast the hardware_serial to Arc<RwLock<Dre>>

        let dre = match smol::block_on(
            SERIAL_DEVICE_REGISTRY.downcast_arc_rwlock::<Dre>(hardware_serial.device.clone()),
        ) {
            Ok(dre) => dre,
            Err(_) => return Err(Error::msg("Failed to downcast to Dre")),
        };

        Ok(Self {
            dre,
            namespace: DreMachineNamespace::new(),
            last_measurement_emit: Instant::now(),
        })
    }
}
