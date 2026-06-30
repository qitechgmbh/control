use std::time::Instant;

use anyhow::Error;
use smol::block_on;
use smol::lock::RwLock;
use std::sync::Arc;

use ethercat_hal::devices::{
    EthercatDevice, downcast_device,
    wago_750_354::{WAGO_750_354_IDENTITY_A, Wago750_354},
    wago_modules::wago_750_460::{Wago750_460, Wago750_460Port},
};
use ethercat_hal::io::temperature_input::TemperatureInput;

use super::{Wago750_460Machine, api::Wago750_460MachineNamespace};
use crate::{
    MachineNewHardware, MachineNewParams, MachineNewTrait, get_ethercat_device,
    validate_no_role_duplicates, validate_same_machine_identification_unique,
};

impl MachineNewTrait for Wago750_460Machine {
    fn new(params: &MachineNewParams) -> Result<Self, Error> {
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
                    "[{}::Wago750_460Machine::new] MachineNewHardware is not Ethercat",
                    module_path!()
                ));
            }
        };

        block_on(async {
            // Acquire the WAGO 750-354 bus coupler at role 0.
            let (coupler_dev, coupler_subdev) = get_ethercat_device::<Wago750_354>(
                hardware,
                params,
                0,
                [WAGO_750_354_IDENTITY_A].to_vec(),
            )
            .await?;

            // Discover and register modules on the coupler.
            let modules = Wago750_354::initialize_modules(coupler_subdev).await?;
            let mut coupler = coupler_dev.write().await;
            for module in modules {
                coupler.set_module(module);
            }
            coupler.init_slot_modules(coupler_subdev);

            // Retrieve the 750-460 module from slot 0.
            let dev = coupler
                .slot_devices
                .get(0)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "[{}::Wago750_460Machine::new] No device in slot 0",
                        module_path!()
                    )
                })?
                .clone()
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "[{}::Wago750_460Machine::new] Slot 0 is empty",
                        module_path!()
                    )
                })?;

            let wago750_460: Arc<RwLock<Wago750_460>> = downcast_device::<Wago750_460>(dev).await?;

            // Create TemperatureInput handles for all 4 channels.
            let t1 = TemperatureInput::new(wago750_460.clone(), Wago750_460Port::T1);
            let t2 = TemperatureInput::new(wago750_460.clone(), Wago750_460Port::T2);
            let t3 = TemperatureInput::new(wago750_460.clone(), Wago750_460Port::T3);
            let t4 = TemperatureInput::new(wago750_460.clone(), Wago750_460Port::T4);

            drop(coupler);

            let (sender, receiver) = smol::channel::unbounded();
            let mut machine = Self {
                api_receiver: receiver,
                api_sender: sender,
                machine_identification_unique: params.get_machine_identification_unique(),
                main_sender: params.main_thread_channel.clone(),
                namespace: Wago750_460MachineNamespace {
                    namespace: params.namespace.clone(),
                },
                last_state_emit: Instant::now(),
                temperature_inputs: [t1, t2, t3, t4],
            };

            machine.emit_state();
            Ok(machine)
        })
    }
}
