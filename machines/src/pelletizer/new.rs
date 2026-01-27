use std::time::Instant;

use crate::pelletizer::MutationRequests;
use crate::serial::devices::us_3202510::US3202510;
use crate::serial::{registry::SERIAL_DEVICE_REGISTRY};
use crate::{MachineNewHardware, MachineNewTrait};

use super::{Pelletizer, api::PelletMachineNamespace};

use anyhow::Error;

impl MachineNewTrait for Pelletizer 
{
    fn new(params: &crate::MachineNewParams<'_, '_, '_, '_, '_, '_, '_>) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let hardware_serial = match params.hardware 
        {
            MachineNewHardware::Serial(serial) => *serial,
            _ => return Err(Error::msg("Invalid hardware type for Pelletizer")),
        };

        let inverter = match smol::block_on(
            SERIAL_DEVICE_REGISTRY.downcast_arc_rwlock::<US3202510>(hardware_serial.device.clone()),
        ) {
            Ok(inverter) => inverter,
            Err(_) => return Err(Error::msg("Failed to downcast to US3202510")),
        };
        
        let (sender, receiver) = smol::channel::unbounded();

        let pelletizer = Self {
            main_sender: params.main_thread_channel.clone(),
            api_receiver: receiver,
            api_sender: sender,
            machine_identification_unique: params.get_machine_identification_unique(),
            
            namespace: PelletMachineNamespace {
                namespace: params.namespace.clone(),
            },
            
            emitted_default_state: false,
            last_measurement_emit: Instant::now(),
            
            inverter,
            
            inverter_snapshot_id: 0,

            mutation_request: MutationRequests {
                running: None,
                direction: None,
                frequency: None,
                accleration_level: None,
                decleration_level: None,
            }
        };

        Ok(pelletizer)
    }
}