use std::{time::Instant};
use anyhow::Error;
use qitech_lib::ethercat_hal::{
    devices::{
        EthercatDevice,
        wago_750_354::{WAGO_750_354_IDENTITY_A, Wago750_354},
        wago_modules::wago_750_402::{Wago750_402},
    },
};
use super::{DigitalInputTestMachine, api::DigitalInputTestMachineNamespace};
use crate::{
    MachineNewHardware, MachineNewParams, MachineNewTrait, get_ethercat_device,
    validate_no_role_duplicates, validate_same_machine_identification_unique,
};

pub fn downcast_device<T: EthercatDevice>(dev : T, identity : (u32,u32,u32)) -> Result<T,anyhow::Error> {
    match identity {
        (0,0,0) => Ok(dev),        
        _ => Err(anyhow::anyhow!("error")),
    }
}

impl MachineNewTrait for DigitalInputTestMachine {
    fn new<'maindevice>(params: &MachineNewParams) -> Result<Self, Error> {
        let device_identification = params
            .device_group
            .iter()
            .map(|device_identification| device_identification.clone())
            .collect::<Vec<_>>();
            validate_same_machine_identification_unique(&device_identification)?;
            validate_no_role_duplicates(&device_identification)?;
            let hardware = match &params.hardware {
                MachineNewHardware::Ethercat(x) => x,
                _ => {
                    return Err(anyhow::anyhow!(
                        "[{}::EtherCATMachine/TestMachine::new] MachineNewHardware is not Ethercat",
                        module_path!()
                    ));
                }
            };
            let (sender, receiver) = smol::channel::unbounded();
            // Example usage of a Wago Coupler and a 750-402 in the first slot, where the Output Port 1,2,3,4 is used
            let mut _wago_750_354 = get_ethercat_device::<Wago750_354>(
                hardware,
                0,
                [WAGO_750_354_IDENTITY_A].to_vec(),
            )?;

            let modules = Wago750_354::initialize_modules(params.ethercat_thread_channel.clone().expect("Epxected EthercatThreadChannel!"),4096)?;
            for module in modules {
                _wago_750_354.set_module(module);
            }
            _wago_750_354.init_slot_modules(params.ethercat_thread_channel.clone().expect("Epxected EthercatThreadChannel!"),4096);
            
            let dev = _wago_750_354.slot_devices.get(0).unwrap().as_ref().unwrap();
            let downcasted_ref : &Wago750_402 = dev.as_any().downcast_ref::<Wago750_402>().unwrap();
            let res = downcast_device::<Wago750_402>(downcasted_ref.clone(),(0u32,0u32,0u32))?;

            let mut my_test = Self {
                api_receiver: receiver,
                api_sender: sender,
                machine_identification_unique: params.get_machine_identification_unique(),
                namespace: DigitalInputTestMachineNamespace {
                    namespace: params.namespace.clone(),
                },
                last_state_emit: Instant::now(),
                led_on: [false; 4],
                main_sender: params.main_thread_channel.clone(),
                digital_input_device: Box::new(res),
            };
            my_test.emit_state();
            Ok(my_test)
    }    
}
