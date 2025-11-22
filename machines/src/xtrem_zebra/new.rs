use std::time::Instant;

use crate::MachineNewTrait;
use crate::serial::registry::SERIAL_DEVICE_REGISTRY;
use crate::xtrem_zebra::XtremZebra;
use crate::xtrem_zebra::api::XtremZebraNamespace;

use anyhow::Error;

impl MachineNewTrait for XtremZebra {
    fn new(params: &crate::MachineNewParams<'_, '_, '_, '_, '_, '_, '_>) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let (sender, receiver) = smol::channel::unbounded();

        let xtrem_zebra = match smol::block_on(
            SERIAL_DEVICE_REGISTRY.downcast_arc_rwlock::<XtremZebra>(hardware_serial.device.clone()),
        ) {
            Ok(xtrem_zebra) => xtrem_zebra,
            Err(_) => return Err(Error::msg("Failed to downcast to XtremZebra")),
        };

        let xtrem_zebra = Self {
            main_sender: params.main_thread_channel.clone(),
            xtrem_zebra:
            api_receiver: receiver,
            api_sender: sender,
            machine_identification_unique: params.get_machine_identification_unique(),
            namespace: XtremZebraNamespace {
                namespace: params.namespace.clone(),
            },
            last_measurement_emit: Instant::now(),
            emitted_default_state: false,
            weight: 0.0,
        };

        Ok(xtrem_zebra)
    }
}
