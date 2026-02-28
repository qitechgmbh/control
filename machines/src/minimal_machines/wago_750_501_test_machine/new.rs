use crate::minimal_machines::wago_750_501_test_machine::Wago750_501TestMachine;
use crate::minimal_machines::wago_750_501_test_machine::api::Wago750_501TestMachineNamespace;
use smol::block_on;
use std::time::Instant;

use crate::{
    MachineNewHardware, MachineNewParams, MachineNewTrait, get_ethercat_device,
    validate_no_role_dublicates, validate_same_machine_identification_unique,
};

use anyhow::Error;
use ethercat_hal::devices::wago_750_354::{WAGO_750_354_IDENTITY_A, Wago750_354};
use ethercat_hal::devices::wago_modules::wago_750_501::{Wago750_501, Wago750_501Port};
use ethercat_hal::devices::{EthercatDevice, downcast_device};
use ethercat_hal::io::digital_output::DigitalOutput;
use smol::lock::RwLock;
use std::sync::Arc;

impl MachineNewTrait for Wago750_501TestMachine {
    fn new(params: &MachineNewParams) -> Result<Self, Error> {
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
                    "[{}::MachineNewTrait/Wago750_501TestMachine::new] MachineNewHardware is not Ethercat",
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
                        "[{}::MachineNewTrait/Wago750_501TestMachine::new] Expected Wago 750-501 module in slot 0, but slot 0 is not configured",
                        module_path!()
                    )
                })?
                .clone()
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "[{}::MachineNewTrait/Wago750_501TestMachine::new] Expected Wago 750-501 module in slot 0, but slot 0 is empty or no device is present",
                        module_path!()
                    )
                })?;
            let wago750_501: Arc<RwLock<Wago750_501>> = downcast_device::<Wago750_501>(dev).await?;

            let do1 = DigitalOutput::new(wago750_501.clone(), Wago750_501Port::Port1);
            let do2 = DigitalOutput::new(wago750_501.clone(), Wago750_501Port::Port2);
            drop(coupler);

            let (sender, receiver) = smol::channel::unbounded();
            let mut machine = Self {
                api_receiver: receiver,
                api_sender: sender,
                machine_identification_unique: params.get_machine_identification_unique(),
                namespace: Wago750_501TestMachineNamespace {
                    namespace: params.namespace.clone(),
                },
                last_state_emit: Instant::now(),
                outputs: [false; 2],
                main_sender: params.main_thread_channel.clone(),
                douts: [do1, do2],
            };
            machine.emit_state();
            Ok(machine)
        })
    }
}
