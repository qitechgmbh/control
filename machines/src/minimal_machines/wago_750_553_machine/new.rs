use std::time::Instant;

use anyhow::Error;
use ethercat_hal::{
    devices::{
        EthercatDevice, downcast_device,
        wago_750_354::{WAGO_750_354_IDENTITY_A, Wago750_354},
        wago_modules::wago_750_553::{Wago750_553, Wago750_553Port},
    },
    io::analog_output::AnalogOutput,
};
use smol::{block_on, lock::RwLock};
use std::sync::Arc;

use super::{Wago750_553Machine, api::Wago750_553MachineNamespace};
use crate::{
    MachineNewHardware, MachineNewParams, MachineNewTrait, get_ethercat_device,
    validate_no_role_dublicates, validate_same_machine_identification_unique,
};

impl MachineNewTrait for Wago750_553Machine {
    fn new(params: &MachineNewParams) -> Result<Self, Error> {
        let device_identification = params
            .device_group
            .iter()
            .map(|d| d.clone())
            .collect::<Vec<_>>();
        validate_same_machine_identification_unique(&device_identification)?;
        validate_no_role_dublicates(&device_identification)?;

        let hardware = match &params.hardware {
            MachineNewHardware::Ethercat(x) => x,
            _ => {
                return Err(anyhow::anyhow!(
                    "[{}::MachineNewTrait/Wago750_553Machine::new] MachineNewHardware is not Ethercat",
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

            let dev = coupler
                .slot_devices
                .get(0)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "[{}::Wago750_553Machine::new] Expected Wago 750-553 module in slot 0, but slot 0 is not configured",
                        module_path!()
                    )
                })?
                .clone()
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "[{}::Wago750_553Machine::new] Expected Wago 750-553 module in slot 0, but slot 0 is empty",
                        module_path!()
                    )
                })?;

            let wago750_553: Arc<RwLock<Wago750_553>> = downcast_device::<Wago750_553>(dev).await?;

            let ao1 = AnalogOutput::new(wago750_553.clone(), Wago750_553Port::AO1);
            let ao2 = AnalogOutput::new(wago750_553.clone(), Wago750_553Port::AO2);
            let ao3 = AnalogOutput::new(wago750_553.clone(), Wago750_553Port::AO3);
            let ao4 = AnalogOutput::new(wago750_553.clone(), Wago750_553Port::AO4);

            drop(coupler);

            let (sender, receiver) = smol::channel::unbounded();
            let mut machine = Self {
                api_receiver: receiver,
                api_sender: sender,
                machine_identification_unique: params.get_machine_identification_unique(),
                main_sender: params.main_thread_channel.clone(),
                namespace: Wago750_553MachineNamespace {
                    namespace: params.namespace.clone(),
                },
                last_state_emit: Instant::now(),
                outputs: [0.0; 4],
                aouts: [ao1, ao2, ao3, ao4],
            };

            machine.emit_state();
            Ok(machine)
        })
    }
}
