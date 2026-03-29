use std::{sync::Arc, time::Instant};

use anyhow::Error;
use ethercat_hal::{
    devices::{
        EthercatDevice, downcast_device,
        wago_750_354::{WAGO_750_354_IDENTITY_A, Wago750_354},
        wago_modules::wago_750_671::Wago750_671,
        wago_modules::wago_750_501::Wago750_501,
    },
    io::{digital_output::DigitalOutput, stepper_velocity_wago_750_671::StepperVelocityWago750671},
};
use smol::{block_on, lock::RwLock};

use super::{WagoWinderSmokeTestMachine, api::WagoWinderSmokeTestMachineNamespace};
use crate::{
    MachineNewHardware, MachineNewParams, MachineNewTrait, get_ethercat_device,
    validate_no_role_duplicates, validate_same_machine_identification_unique,
};

async fn get_first_n_671_slots(
    coupler: &Wago750_354,
    count: usize,
) -> Result<Vec<Arc<RwLock<Wago750_671>>>, Error> {
    let mut steppers = Vec::with_capacity(count);

    for dev in coupler.slot_devices.iter().flatten() {
        let is_671 = {
            let guard = dev.read().await;
            guard.as_any().is::<Wago750_671>()
        };

        if is_671 {
            steppers.push(downcast_device::<Wago750_671>(dev.clone()).await?);
            if steppers.len() == count {
                break;
            }
        }
    }

    if steppers.len() != count {
        return Err(anyhow::anyhow!(
            "expected {} Wago 750-671 modules, found {}",
            count,
            steppers.len()
        ));
    }

    Ok(steppers)
}

async fn get_first_device<T: EthercatDevice>(
    coupler: &Wago750_354,
    device_name: &str,
) -> Result<Arc<RwLock<T>>, Error> {
    for dev in coupler.slot_devices.iter().flatten() {
        let is_target = {
            let guard = dev.read().await;
            guard.as_any().is::<T>()
        };

        if is_target {
            return downcast_device::<T>(dev.clone()).await;
        }
    }

    Err(anyhow::anyhow!("no {} module found", device_name))
}

impl MachineNewTrait for WagoWinderSmokeTestMachine {
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
                    "[{}::WagoWinderSmokeTestMachine::new] MachineNewHardware is not Ethercat",
                    module_path!()
                ));
            }
        };

        block_on(async {
            let wago_750_354 = get_ethercat_device::<Wago750_354>(
                hardware,
                params,
                0,
                [WAGO_750_354_IDENTITY_A].to_vec(),
            )
            .await?;

            let modules = Wago750_354::initialize_modules(wago_750_354.1).await?;
            let mut coupler = wago_750_354.0.write().await;

            for module in modules {
                coupler.set_module(module);
            }

            coupler.init_slot_modules(wago_750_354.1);

            let steppers = get_first_n_671_slots(&coupler, 1).await?;
            let wago_750_501 = get_first_device::<Wago750_501>(&coupler, "Wago 750-501").await?;

            drop(coupler);

            let steppers = [StepperVelocityWago750671::new(steppers[0].clone())];
            let digital_output1 = DigitalOutput::new(
                wago_750_501.clone(),
                ethercat_hal::devices::wago_modules::wago_750_501::Wago750_501Port::Port1,
            );
            let digital_output2 = DigitalOutput::new(
                wago_750_501.clone(),
                ethercat_hal::devices::wago_modules::wago_750_501::Wago750_501Port::Port2,
            );

            let (sender, receiver) = smol::channel::unbounded();
            let mut machine = Self {
                api_receiver: receiver,
                api_sender: sender,
                machine_identification_unique: params.get_machine_identification_unique(),
                main_sender: params.main_thread_channel.clone(),
                namespace: WagoWinderSmokeTestMachineNamespace {
                    namespace: params.namespace.clone(),
                },
                last_state_emit: Instant::now(),
                steppers,
                digital_output1,
                digital_output2,
            };
            machine.emit_state();
            Ok(machine)
        })
    }
}
