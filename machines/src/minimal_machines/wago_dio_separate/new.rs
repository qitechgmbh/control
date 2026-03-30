use super::{WagoDioSeparate, api::WagoDioSeparateNamespace};
use crate::{
    MachineNewHardware, MachineNewParams, MachineNewTrait, get_ethercat_device,
    validate_no_role_duplicates, validate_same_machine_identification_unique,
};
use anyhow::Error;
use ethercat_hal::devices::EthercatDevice;
use ethercat_hal::{
    devices::{
        downcast_device,
        wago_750_354::{WAGO_750_354_IDENTITY_A, Wago750_354},
        wago_modules::{
            wago_750_430::{Wago750_430, Wago750_430Port},
            wago_750_530::{Wago750_530, Wago750_530Port},
        },
    },
    io::{digital_input::DigitalInput, digital_output::DigitalOutput},
};
use smol::{block_on, lock::RwLock};
use std::{sync::Arc, time::Instant};

impl MachineNewTrait for WagoDioSeparate {
    fn new<'maindevice>(params: &MachineNewParams) -> Result<Self, Error> {
        let device_identification = params
            .device_group
            .iter()
            .map(|d| d.clone())
            .collect::<Vec<_>>();
        validate_same_machine_identification_unique(&device_identification)?;
        validate_no_role_duplicates(&device_identification)?;

        let hardware = match &params.hardware {
            MachineNewHardware::Ethercat(x) => x,
            _ => {
                return Err(anyhow::anyhow!(
                    "[{}::WagoDioSeparate::new] MachineNewHardware is not Ethercat",
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

            // Get the WAGO 750-530 8CH DO module at slot 0
            let dev = coupler.slot_devices.get(0).unwrap().clone().unwrap();
            let wago750_530: Arc<RwLock<Wago750_530>> = downcast_device::<Wago750_530>(dev).await?;

            // Get the WAGO 750-430 8CH DI module at slot 1
            let dev = coupler.slot_devices.get(1).unwrap().clone().unwrap();
            let wago750_430: Arc<RwLock<Wago750_430>> = downcast_device::<Wago750_430>(dev).await?;

            let di1 = DigitalInput::new(wago750_430.clone(), Wago750_430Port::Port1);
            let di2 = DigitalInput::new(wago750_430.clone(), Wago750_430Port::Port2);
            let di3 = DigitalInput::new(wago750_430.clone(), Wago750_430Port::Port3);
            let di4 = DigitalInput::new(wago750_430.clone(), Wago750_430Port::Port4);
            let di5 = DigitalInput::new(wago750_430.clone(), Wago750_430Port::Port5);
            let di6 = DigitalInput::new(wago750_430.clone(), Wago750_430Port::Port6);
            let di7 = DigitalInput::new(wago750_430.clone(), Wago750_430Port::Port7);
            let di8 = DigitalInput::new(wago750_430.clone(), Wago750_430Port::Port8);

            let do1 = DigitalOutput::new(wago750_530.clone(), Wago750_530Port::Port1);
            let do2 = DigitalOutput::new(wago750_530.clone(), Wago750_530Port::Port2);
            let do3 = DigitalOutput::new(wago750_530.clone(), Wago750_530Port::Port3);
            let do4 = DigitalOutput::new(wago750_530.clone(), Wago750_530Port::Port4);
            let do5 = DigitalOutput::new(wago750_530.clone(), Wago750_530Port::Port5);
            let do6 = DigitalOutput::new(wago750_530.clone(), Wago750_530Port::Port6);
            let do7 = DigitalOutput::new(wago750_530.clone(), Wago750_530Port::Port7);
            let do8 = DigitalOutput::new(wago750_530.clone(), Wago750_530Port::Port8);

            drop(coupler);

            let (sender, receiver) = smol::channel::unbounded();
            let mut machine = Self {
                api_receiver: receiver,
                api_sender: sender,
                machine_identification_unique: params.get_machine_identification_unique(),
                namespace: WagoDioSeparateNamespace {
                    namespace: params.namespace.clone(),
                },
                last_state_emit: Instant::now(),
                inputs: [false; 8],
                led_on: [false; 8],
                main_sender: params.main_thread_channel.clone(),
                digital_input: [di1, di2, di3, di4, di5, di6, di7, di8],
                digital_output: [do1, do2, do3, do4, do5, do6, do7, do8],
            };
            machine.emit_state();
            Ok(machine)
        })
    }
}
