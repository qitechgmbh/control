use crate::test_machine::TestMachine;
use crate::test_machine::api::TestMachineNamespace;
use smol::block_on;
use std::time::Instant;

use crate::{
    MachineNewHardware, MachineNewParams, MachineNewTrait, get_ethercat_device,
    validate_no_role_dublicates, validate_same_machine_identification_unique,
};

use anyhow::Error;
use ethercat_hal::devices::el2004::{EL2004, EL2004_IDENTITY_A, EL2004Port};
use ethercat_hal::io::digital_output::DigitalOutput;
use smol::lock::RwLock;
use std::sync::Arc;


impl MachineNewTrait for TestMachine {
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
                    "[{}::MachineNewTrait/TestMachine::new] MachineNewHardware is not Ethercat",
                    module_path!()
                ));
            }
        };
        block_on(async {
            let el2004 = get_ethercat_device::<EL2004>(
                hardware,
                params,
                0,
                [EL2004_IDENTITY_A].to_vec(),
            )
            .await?
            .0;

            let do1 = DigitalOutput::new(el2004.clone(), EL2004Port::DO1);
            let do2 = DigitalOutput::new(el2004.clone(), EL2004Port::DO2);
            let do3 = DigitalOutput::new(el2004.clone(), EL2004Port::DO3);
            let do4 = DigitalOutput::new(el2004.clone(), EL2004Port::DO4);

            let (sender, receiver) = smol::channel::unbounded();
            let mut my_test = Self {
                api_receiver: receiver,
                api_sender: sender,
                machine_identification_unique: params.get_machine_identification_unique(),
                namespace: TestMachineNamespace {
                    namespace: params.namespace.clone(),
                },
                last_state_emit: Instant::now(),
                led_on: [false; 4],
                main_sender: params.main_thread_channel.clone(),
                douts: [do1, do2, do3, do4],
            };
            my_test.emit_state();
            Ok(my_test)
        })
    }
}