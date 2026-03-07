use ethercat_hal::devices::wago_modules::wago_750_652::{Wago750_652, Wago750_652Port};
use ethercat_hal::io::serial_interface::SerialInterface;
use smol::block_on;
use std::time::Instant;

use crate::{
    MachineNewHardware, MachineNewParams, MachineNewTrait, get_ethercat_device,
    validate_no_role_dublicates, validate_same_machine_identification_unique,
};
use anyhow::Error;
use ethercat_hal::devices::wago_750_354::{WAGO_750_354_IDENTITY_A, Wago750_354};
use ethercat_hal::devices::{EthercatDevice, downcast_device};
use smol::lock::RwLock;
use std::sync::Arc;

use super::WagoSerialMachine;
use super::api::WagoSerialMachineNamespace;

impl MachineNewTrait for WagoSerialMachine {
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
                        "[{}::MachineNewTrait/WagoSerialMachine::new] Expected Wago 750-652 module in slot 0, but slot 0 is not configured",
                        module_path!()
                    )
                })?
                .clone()
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "[{}::MachineNewTrait/WagoSerialMachine::new] Expected Wago 750-652 module in slot 0, but slot 0 is empty or no device is present",
                        module_path!()
                    )
                })?;
            let wago750_652: Arc<RwLock<Wago750_652>> = downcast_device::<Wago750_652>(dev).await?;
            drop(coupler);
            let serial_interface = SerialInterface::new(wago750_652, Wago750_652Port::SI1);

            let (sender, receiver) = smol::channel::unbounded();
            let mut machine = Self {
                api_receiver: receiver,
                api_sender: sender,
                machine_identification_unique: params.get_machine_identification_unique(),
                namespace: WagoSerialMachineNamespace {
                    namespace: params.namespace.clone(),
                },
                last_state_emit: Instant::now(),
                main_sender: params.main_thread_channel.clone(),
                serial_device: serial_interface,
                current_message: None,
                serial_init_is_complete: false,
            };
            machine.emit_state();
            Ok(machine)
        })
    }
}
