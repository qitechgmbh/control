use std::time::Instant;

use crate::MachineNewTrait;
use crate::xtrem_zebra::XtremZebra;
use crate::xtrem_zebra::api::XtremZebraNamespace;

use anyhow::Error;

impl MachineNewTrait for XtremZebra {
    fn new(params: &crate::MachineNewParams<'_, '_, '_, '_, '_, '_, '_>) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let (sender, receiver) = smol::channel::unbounded();

        let xtrem_zebra = Self {
            main_sender: params.main_thread_channel.clone(),
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
