use crate::minimal_machines::wago_750_531_machine::Wago750_531Machine;
use crate::minimal_machines::wago_750_531_machine::api::Wago750_531MachineNamespace;
use smol::block_on;
use std::time::Instant;

use crate::{
    MachineNewHardware, MachineNewParams, MachineNewTrait, get_ethercat_device,
    validate_no_role_duplicates, validate_same_machine_identification_unique,
};

use anyhow::Error;
use ethercat_hal::devices::wago_750_354::{WAGO_750_354_IDENTITY_A, Wago750_354};
use ethercat_hal::devices::wago_modules::wago_750_531::{Wago750_531, Wago750_531OutputPort};
use ethercat_hal::devices::{EthercatDevice, downcast_device};
use ethercat_hal::io::digital_output::DigitalOutput;
use smol::lock::RwLock;
use std::sync::Arc;

impl MachineNewTrait for Wago750_531Machine {
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
                    "[{}::MachineNewTrait/Wago750_531Machine::new] MachineNewHardware is not Ethercat",
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
                        "[{}::MachineNewTrait/Wago750_531Machine::new] Expected Wago 750-531 module in slot 0, but slot 0 is not configured",
                        module_path!()
                    )
                })?
                .clone()
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "[{}::MachineNewTrait/Wago750_531Machine::new] Expected Wago 750-531 module in slot 0, but slot 0 is empty or no device is present",
                        module_path!()
                    )
                })?;
            let wago750_531: Arc<RwLock<Wago750_531>> =
                downcast_device::<Wago750_531>(dev).await?;

            let do1 = DigitalOutput::new(wago750_531.clone(), Wago750_531OutputPort::DO1);
            let do2 = DigitalOutput::new(wago750_531.clone(), Wago750_531OutputPort::DO2);
            let do3 = DigitalOutput::new(wago750_531.clone(), Wago750_531OutputPort::DO3);
            let do4 = DigitalOutput::new(wago750_531.clone(), Wago750_531OutputPort::DO4);
            drop(coupler);

            let (sender, receiver) = smol::channel::unbounded();
            let mut machine = Self {
                api_receiver: receiver,
                api_sender: sender,
                machine_identification_unique: params.get_machine_identification_unique(),
                namespace: Wago750_531MachineNamespace {
                    namespace: params.namespace.clone(),
                },
                last_state_emit: Instant::now(),
                outputs_on: [false; 4],
                main_sender: params.main_thread_channel.clone(),
                douts: [do1, do2, do3, do4],
            };
            machine.emit_state();
            Ok(machine)
        })
    }
}
