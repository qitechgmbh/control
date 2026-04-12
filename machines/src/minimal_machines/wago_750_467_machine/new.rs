use std::time::Instant;

use anyhow::Error;
use ethercat_hal::{
    devices::{
        EthercatDevice, downcast_device,
        wago_750_354::{WAGO_750_354_IDENTITY_A, Wago750_354},
        wago_modules::wago_750_467::{Wago750_467, Wago750_467Port},
    },
    io::analog_input::AnalogInput,
};
use smol::{block_on, channel::unbounded, lock::RwLock};
use std::sync::Arc;

use super::{Wago750_467Machine, api::Wago750_467MachineNamespace};
use crate::{
    MachineNewHardware, MachineNewParams, MachineNewTrait, get_ethercat_device,
    validate_no_role_duplicates, validate_same_machine_identification_unique,
};

impl MachineNewTrait for Wago750_467Machine {
    fn new(params: &MachineNewParams) -> Result<Self, Error> {
        let device_identification = params.device_group.iter().cloned().collect::<Vec<_>>();
        validate_same_machine_identification_unique(&device_identification)?;
        validate_no_role_duplicates(&device_identification)?;

        let hardware = match &params.hardware {
            MachineNewHardware::Ethercat(x) => x,
            _ => {
                return Err(anyhow::anyhow!(
                    "[{}::Wago750_467Machine::new] MachineNewHardware is not Ethercat",
                    module_path!()
                ));
            }
        };

        block_on(async {
            let (coupler_dev, coupler_subdev) = get_ethercat_device::<Wago750_354>(
                hardware,
                params,
                0,
                [WAGO_750_354_IDENTITY_A].to_vec(),
            )
            .await?;

            let modules = Wago750_354::initialize_modules(coupler_subdev).await?;
            let mut coupler = coupler_dev.write().await;
            for module in modules {
                coupler.set_module(module);
            }
            coupler.init_slot_modules(coupler_subdev);

            let device = coupler
                .slot_devices
                .iter()
                .flatten()
                .find_map(|device| {
                    let cloned = device.clone();
                    smol::block_on(async {
                        if downcast_device::<Wago750_467>(cloned.clone()).await.is_ok() {
                            Some(cloned)
                        } else {
                            None
                        }
                    })
                })
                .ok_or_else(|| anyhow::anyhow!("No Wago 750-467 found on the coupler"))?;

            let wago750_467: Arc<RwLock<Wago750_467>> =
                downcast_device::<Wago750_467>(device).await?;
            let ai1 = AnalogInput::new(wago750_467.clone(), Wago750_467Port::AI1);
            let ai2 = AnalogInput::new(wago750_467.clone(), Wago750_467Port::AI2);

            drop(coupler);

            let (sender, receiver) = unbounded();
            let mut machine = Self {
                api_receiver: receiver,
                api_sender: sender,
                machine_identification_unique: params.get_machine_identification_unique(),
                main_sender: params.main_thread_channel.clone(),
                namespace: Wago750_467MachineNamespace {
                    namespace: params.namespace.clone(),
                },
                last_state_emit: Instant::now(),
                analog_inputs: [ai1, ai2],
                device: wago750_467,
            };

            machine.emit_state();
            Ok(machine)
        })
    }
}
