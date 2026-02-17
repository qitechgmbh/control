use crate::wago_do_test_machine::WagoDOTestMachine;
use crate::wago_do_test_machine::api::WagoDOTestMachineNamespace;
use smol::block_on;
use std::time::Instant;

use crate::{
    MachineNewHardware, MachineNewParams, MachineNewTrait, get_ethercat_device,
    validate_no_role_dublicates, validate_same_machine_identification_unique,
};

use anyhow::Error;
use ethercat_hal::devices::wago_750_354::{WAGO_750_354_IDENTITY_A, Wago750_354};
use ethercat_hal::devices::wago_modules::wago_750_530::{Wago750_530, Wago750_530Port};
use ethercat_hal::devices::{EthercatDevice, downcast_device};
use ethercat_hal::io::digital_output::DigitalOutput;
use smol::lock::RwLock;
use std::sync::Arc;

impl MachineNewTrait for WagoDOTestMachine {
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
                    "[{}::MachineNewTrait/WagoDOTestMachine::new] MachineNewHardware is not Ethercat",
                    module_path!()
                ));
            }
        };

        block_on(async {
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
            let dev = coupler
                .slot_devices
                .get(0)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "[{}::MachineNewTrait/WagoDOTestMachine::new] Expected Wago 750-530 module in slot 0, but slot 0 is not configured",
                        module_path!()
                    )
                })?
                .clone()
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "[{}::MachineNewTrait/WagoDOTestMachine::new] Expected Wago 750-530 module in slot 0, but slot 0 is empty or no device is present",
                        module_path!()
                    )
                })?;
            let wago750_530: Arc<RwLock<Wago750_530>> = downcast_device::<Wago750_530>(dev).await?;

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
                namespace: WagoDOTestMachineNamespace {
                    namespace: params.namespace.clone(),
                },
                last_state_emit: Instant::now(),
                led_on: [false; 8],
                main_sender: params.main_thread_channel.clone(),
                douts: [do1, do2, do3, do4, do5, do6, do7, do8],
            };
            machine.emit_state();
            Ok(machine)
        })
    }
}
