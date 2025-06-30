use std::time::Instant;

use anyhow::Error;
use control_core::machines::new::{validate_no_role_dublicates, validate_same_machine_identification_unique, MachineNewParams, MachineNewTrait};

use super::{
    api::{BufferedWinderNamespace, Mode}, BufferedWinder
};

impl MachineNewTrait for BufferedWinder {
    fn new<'maindevice>(params: &MachineNewParams) -> Result<Self, Error> {
        // validate general stuff
        let device_identification = params
            .device_group
            .iter()
            .map(|device_identification| device_identification.clone())
            .collect::<Vec<_>>();
        validate_same_machine_identification_unique(&device_identification)?;
        validate_no_role_dublicates(&device_identification)?;

        //TODO


        smol::block_on(async {
            //TODO

            let buffered_winder: BufferedWinder = Self {
                    namespace: BufferedWinderNamespace::new(params.socket_queue_tx.clone()),
                    last_measurement_emit: Instant::now(),
                    mode: Mode::Standby,
            };
            Ok(buffered_winder)
            })
    }
}
