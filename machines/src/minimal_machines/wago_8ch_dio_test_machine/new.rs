use std::{sync::Arc, time::Instant};

use anyhow::Error;
use ethercat_hal::{
    devices::{
        EthercatDevice, downcast_device,
        wago_750_354::{WAGO_750_354_IDENTITY_A, Wago750_354},
        wago_modules::wago_750_1506::{
            Wago750_1506, Wago750_1506InputPort, Wago750_1506OutputPort,
        },
    },
    io::{digital_input::DigitalInput, digital_output::DigitalOutput},
};
use smol::{block_on, lock::RwLock};

use crate::{
    MachineNewHardware, MachineNewParams, MachineNewTrait, get_ethercat_device,
    validate_no_role_dublicates, validate_same_machine_identification_unique,
};
use super::{Wago8chDigitalIOTestMachine, api::Wago8chDigitalIOTestMachineNamespace};

impl MachineNewTrait for Wago8chDigitalIOTestMachine {
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
            // Example usage of a Wago Coupler and a 750-402 in the first slot, where the Output Port 1,2,3,4 is used
            let _wago_750_354 = get_ethercat_device::<Wago750_354>(
                hardware,
                params,
                0,
                [WAGO_750_354_IDENTITY_A].to_vec(),
            )
            .await?;

            let modules = Wago750_354::initialize_modules(_wago_750_354.1).await?;

            let mut coupler = _wago_750_354.0.write().await;

            for module in modules {
                coupler.set_module(module);
            }

            coupler.init_slot_modules(_wago_750_354.1);
            let dev = coupler.slot_devices.get(0).unwrap().clone().unwrap();
            let wago750_1506: Arc<RwLock<Wago750_1506>> =
                downcast_device::<Wago750_1506>(dev).await?;

            let di1 = DigitalInput::new(wago750_1506.clone(), Wago750_1506InputPort::DI1);
            let di2 = DigitalInput::new(wago750_1506.clone(), Wago750_1506InputPort::DI2);
            let di3 = DigitalInput::new(wago750_1506.clone(), Wago750_1506InputPort::DI3);
            let di4 = DigitalInput::new(wago750_1506.clone(), Wago750_1506InputPort::DI4);
            let di5 = DigitalInput::new(wago750_1506.clone(), Wago750_1506InputPort::DI5);
            let di6 = DigitalInput::new(wago750_1506.clone(), Wago750_1506InputPort::DI6);
            let di7 = DigitalInput::new(wago750_1506.clone(), Wago750_1506InputPort::DI7);
            let di8 = DigitalInput::new(wago750_1506.clone(), Wago750_1506InputPort::DI8);

            let do1 = DigitalOutput::new(wago750_1506.clone(), Wago750_1506OutputPort::DO1);
            let do2 = DigitalOutput::new(wago750_1506.clone(), Wago750_1506OutputPort::DO2);
            let do3 = DigitalOutput::new(wago750_1506.clone(), Wago750_1506OutputPort::DO3);
            let do4 = DigitalOutput::new(wago750_1506.clone(), Wago750_1506OutputPort::DO4);
            let do5 = DigitalOutput::new(wago750_1506.clone(), Wago750_1506OutputPort::DO5);
            let do6 = DigitalOutput::new(wago750_1506.clone(), Wago750_1506OutputPort::DO6);
            let do7 = DigitalOutput::new(wago750_1506.clone(), Wago750_1506OutputPort::DO7);
            let do8 = DigitalOutput::new(wago750_1506.clone(), Wago750_1506OutputPort::DO8);

            drop(coupler);

            let (sender, receiver) = smol::channel::unbounded();
            let mut my_test = Self {
                api_receiver: receiver,
                api_sender: sender,
                machine_identification_unique: params.get_machine_identification_unique(),
                namespace: Wago8chDigitalIOTestMachineNamespace {
                    namespace: params.namespace.clone(),
                },
                last_state_emit: Instant::now(),
                main_sender: params.main_thread_channel.clone(),
                digital_output: [do1, do2, do3, do4, do5, do6, do7, do8],
                digital_input: [di1, di2, di3, di4, di5, di6, di7, di8],
            };
            my_test.emit_state();
            Ok(my_test)
        })
    }
}
