use std::time::Instant;

use crate::serial::devices::xtrem_zebra::XtremSerial;
use crate::xtrem_zebra::api::XtremZebraNamespace;
use crate::xtrem_zebra::XtremZebra;
use crate::{
    MachineNewTrait, validate_no_role_dublicates, validate_same_machine_identification_unique,
};
use anyhow::Error;

use super::api::Configuration;

impl MachineNewTrait for XtremZebra {
    fn new<'maindevice>(params: &crate::MachineNewParams) -> Result<Self, Error> {
        let device_identification = params.device_group.to_vec();
        validate_same_machine_identification_unique(&device_identification)?;
        validate_no_role_dublicates(&device_identification)?;

        // This creates the "driver" for the serial connection to the scales.
        let hardware_serial = XtremSerial::new_serial();
        let (_device_id, xtrem_serial) = hardware_serial?;

        let (sender, receiver) = smol::channel::unbounded();

        let xtrem_zebra_machine = Self {
            main_sender: params.main_thread_channel.clone(),
            xtrem_serial,
            api_receiver: receiver,
            api_sender: sender,
            machine_identification_unique: params.get_machine_identification_unique(),
            namespace: XtremZebraNamespace {
                namespace: params.namespace.clone(),
            },
            last_measurement_emit: Instant::now(),
            _emitted_default_state: false,
            total_weight: 0.0,
            current_weight: 0.0,
            last_weight: 0.0,
            cycle_max_weight: 0.0,
            in_accumulation: false,
            plate_counter: 0,
            upper_tolerance: 0.3,
            lower_tolerance: 0.3,
            tare_weight: 0.0,
            last_raw_weight: 0.0,
        };

        Ok(xtrem_zebra_machine)
    }
}
