use std::time::Instant;

use anyhow::Error;
use ethercat_hal::{
    devices::{
        EthercatDevice, downcast_device,
        wago_750_354::{WAGO_750_354_IDENTITY_A, Wago750_354},
        wago_modules::wago_750_455::{Wago750_455, Wago750_455Port},
    },
    io::analog_input::AnalogInput,
};
use smol::{block_on, channel::unbounded, lock::RwLock};
use std::sync::Arc;

use crate::{
    MachineNewHardware, MachineNewParams, MachineNewTrait, get_ethercat_device,
    validate_no_role_dublicates, validate_same_machine_identification_unique,
};
use super::{WagoAiTestMachine, api::WagoAiTestMachineNamespace};

impl MachineNewTrait for WagoAiTestMachine {
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
                    "[{}::WagoAiTestMachine::new] MachineNewHardware is not Ethercat",
                    module_path!()
                ));
            }
        };

        block_on(async {
            // Get the Wago 750-354 bus coupler
            let wago_750_354 = get_ethercat_device::<Wago750_354>(
                hardware,
                params,
                0,
                [WAGO_750_354_IDENTITY_A].to_vec(),
            )
            .await?;

            // Initialize modules on the bus coupler
            let modules = Wago750_354::initialize_modules(wago_750_354.1).await?;

            let mut coupler = wago_750_354.0.write().await;

            for module in modules {
                coupler.set_module(module);
            }

            coupler.init_slot_modules(wago_750_354.1);

            // Get the 750-455 analog input module from slot 0 (first module)
            let dev = coupler
                .slot_devices
                .get(0)
                .ok_or_else(|| anyhow::anyhow!("No device in slot 0"))?
                .clone()
                .ok_or_else(|| anyhow::anyhow!("Slot 0 is empty"))?;

            let wago750_455: Arc<RwLock<Wago750_455>> = downcast_device::<Wago750_455>(dev).await?;

            // Create AnalogInput instances for all 4 channels
            let ai1 = AnalogInput::new(wago750_455.clone(), Wago750_455Port::AI1);
            let ai2 = AnalogInput::new(wago750_455.clone(), Wago750_455Port::AI2);
            let ai3 = AnalogInput::new(wago750_455.clone(), Wago750_455Port::AI3);
            let ai4 = AnalogInput::new(wago750_455.clone(), Wago750_455Port::AI4);

            drop(coupler);

            let (sender, receiver) = unbounded();
            let namespace = WagoAiTestMachineNamespace {
                namespace: params.namespace.clone(),
            };

            let new_wago_ai_test_machine = Self {
                api_receiver: receiver,
                api_sender: sender,
                machine_identification_unique: params.get_machine_identification_unique(),
                main_sender: params.main_thread_channel.clone(),
                namespace,

                last_measurement: Instant::now(),
                measurement_rate_hz: 1.0,

                analog_inputs: [ai1, ai2, ai3, ai4],
            };

            Ok(new_wago_ai_test_machine)
        })
    }
}
