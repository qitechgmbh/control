use std::time::Instant;

use anyhow::Error;
use ethercat_hal::{
    devices::el3021::{EL3021, EL3021_IDENTITY_A, EL3021Port},
    io::analog_input::AnalogInput,
};
use smol::{block_on, channel::unbounded};

use crate::{
    MachineNewHardware, MachineNewParams, MachineNewTrait,
    analog_input_test_machine::{AnalogInputTestMachine, api::AnalogInputTestMachineNamespace},
    get_ethercat_device, validate_no_role_dublicates, validate_same_machine_identification_unique,
};

impl MachineNewTrait for AnalogInputTestMachine {
    fn new<'maindevice>(params: &MachineNewParams) -> Result<Self, Error> {
        // validate general stuff
        let device_identification = params
            .device_group
            .iter()
            .map(|device_identification| device_identification.clone())
            .collect::<Vec<_>>();
        validate_same_machine_identification_unique(&device_identification)?;
        validate_no_role_dublicates(&device_identification)?;

        let hardware = match &params.hardware {
            MachineNewHardware::Ethercat(x) => x,
            _ => {
                return Err(anyhow::anyhow!(
                    "[{}::EtherCATMachine/TestMachine::new] MachineNewHardware is not Ethercat",
                    module_path!()
                ));
            }
        };

        block_on(async {
            let el3021 =
                get_ethercat_device::<EL3021>(hardware, params, 1, [EL3021_IDENTITY_A].to_vec())
                    .await?
                    .0;

            let ai1 = AnalogInput::new(el3021.clone(), EL3021Port::AI1);

            let (sender, receiver) = unbounded();
            let namespace = AnalogInputTestMachineNamespace {
                namespace: params.namespace.clone(),
            };
            let new_analog_input_test_machine = Self {
                api_receiver: receiver,
                api_sender: sender,
                machine_identification_unique: params.get_machine_identification_unique(),
                main_sender: params.main_thread_channel.clone(),
                namespace,

                last_measurement: Instant::now(),
                measurement_rate_hz: 1.0,

                analog_input: ai1,
            };
            Ok(new_analog_input_test_machine)
        })
    }
}
