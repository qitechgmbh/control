use std::{sync::Arc, time::Instant};

use anyhow::Error;
use ethercat_hal::{
    devices::{
        EthercatDevice, downcast_device,
        wago_750_354::{WAGO_750_354_IDENTITY_A, Wago750_354},
        wago_modules::wago_750_671::Wago750_671,
    },
    io::stepper_velocity_wago_750_671::StepperVelocityWago750671,
};
use smol::{block_on, lock::RwLock};

use super::{WagoWinderSmokeTestMachine, api::WagoWinderSmokeTestMachineNamespace};
use crate::{
    MachineNewHardware, MachineNewParams, MachineNewTrait, get_ethercat_device,
    validate_no_role_duplicates, validate_same_machine_identification_unique,
};

async fn get_first_671_slot(coupler: &Wago750_354) -> Result<Arc<RwLock<Wago750_671>>, Error> {
    let mut stepper: Option<Arc<RwLock<Wago750_671>>> = None;

    // Allow bench stacks that contain helper modules such as a 750-501 before the
    // stepper controller. This tester only cares about the first 750-671 it finds.
    for dev in coupler.slot_devices.iter().flatten() {
        let is_671 = {
            let guard = dev.read().await;
            guard.as_any().is::<Wago750_671>()
        };

        if is_671 {
            stepper = Some(downcast_device::<Wago750_671>(dev.clone()).await?);
            break;
        }
    }

    stepper.ok_or_else(|| {
        anyhow::anyhow!(
            "expected at least one Wago 750-671 module; other modules such as 750-501 are allowed"
        )
    })
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

            let stepper = get_first_671_slot(&coupler).await?;

            drop(coupler);

            let mut stepper = StepperVelocityWago750671::new(stepper);
            stepper.set_freq_range_sel(3);
            stepper.set_acc_range_sel(2);
            stepper.set_acceleration(1000);
            stepper.request_speed_mode();

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
                stepper,
                last_debug_snapshot: None,
            };
            machine.emit_state();
            Ok(machine)
        })
    }
}
