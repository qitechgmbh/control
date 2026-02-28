use std::{sync::Arc, time::Instant};

use anyhow::Error;
use ethercat_hal::{
    devices::{
        EthercatDevice, downcast_device,
        wago_750_354::{WAGO_750_354_IDENTITY_A, Wago750_354},
        wago_modules::wago_750_430::{Wago750_430, Wago750_430Port},
    },
    io::digital_input::DigitalInput,
};
use smol::{block_on, lock::RwLock};

use super::{Wago750_430DiMachine, api::Wago750_430DiMachineNamespace};
use crate::{
    MachineNewHardware, MachineNewParams, MachineNewTrait, get_ethercat_device,
    validate_no_role_dublicates, validate_same_machine_identification_unique,
};

impl MachineNewTrait for Wago750_430DiMachine {
    fn new<'maindevice>(params: &MachineNewParams) -> Result<Self, Error> {
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
                    "[{}::Wago750_430DiMachine::new] MachineNewHardware is not Ethercat",
                    module_path!()
                ));
            }
        };

        block_on(async {
            // Get the WAGO 750-354 coupler at role 0
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

            // Get the WAGO 750-430 8CH DI module at slot 0
            let dev = coupler.slot_devices.get(0).unwrap().clone().unwrap();
            let wago750_430: Arc<RwLock<Wago750_430>> =
                downcast_device::<Wago750_430>(dev).await?;

            let di1 = DigitalInput::new(wago750_430.clone(), Wago750_430Port::Port1);
            let di2 = DigitalInput::new(wago750_430.clone(), Wago750_430Port::Port2);
            let di3 = DigitalInput::new(wago750_430.clone(), Wago750_430Port::Port3);
            let di4 = DigitalInput::new(wago750_430.clone(), Wago750_430Port::Port4);
            let di5 = DigitalInput::new(wago750_430.clone(), Wago750_430Port::Port5);
            let di6 = DigitalInput::new(wago750_430.clone(), Wago750_430Port::Port6);
            let di7 = DigitalInput::new(wago750_430.clone(), Wago750_430Port::Port7);
            let di8 = DigitalInput::new(wago750_430.clone(), Wago750_430Port::Port8);

            drop(coupler);

            let (sender, receiver) = smol::channel::unbounded();
            let mut machine = Self {
                api_receiver: receiver,
                api_sender: sender,
                machine_identification_unique: params.get_machine_identification_unique(),
                namespace: Wago750_430DiMachineNamespace {
                    namespace: params.namespace.clone(),
                },
                last_state_emit: Instant::now(),
                inputs: [false; 8],
                main_sender: params.main_thread_channel.clone(),
                digital_input: [di1, di2, di3, di4, di5, di6, di7, di8],
            };
            machine.emit_state();
            Ok(machine)
        })
    }
}
