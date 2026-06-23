use crate::minimal_machines::wago_671_slot12_test_machine::{
    Wago671Slot12TestMachine, api::Wago671Slot12TestMachineNamespace,
};
use anyhow::Error;
use ethercat_hal::{
    devices::{
        EthercatDevice, downcast_device,
        wago_750_354::{WAGO_750_354_IDENTITY_A, Wago750_354},
        wago_modules::wago_750_671::Wago750_671,
    },
    io::stepper_velocity_wago_750_671::StepperVelocityWago750671,
};
use smol::block_on;
use std::time::Instant;

use crate::{
    MachineNewHardware, MachineNewParams, MachineNewTrait, get_ethercat_device,
    validate_no_role_duplicates, validate_same_machine_identification_unique,
};

fn configure_axis(stepper: &mut StepperVelocityWago750671) {
    stepper.set_motor_full_steps_per_rev(200);
    stepper.set_microsteps_per_full_step(64);
    stepper.set_direction_multiplier(1);
    stepper.set_speed_scale(1.0);
    stepper.set_freq_range_sel(2);
    stepper.set_acc_range_sel(2);
    stepper.set_acceleration(1000);
    stepper.request_speed_mode();
    stepper.clear_fast_stop();
}

impl MachineNewTrait for Wago671Slot12TestMachine {
    fn new<'maindevice>(params: &MachineNewParams) -> Result<Self, Error> {
        let device_identification = params.device_group.to_vec();
        validate_same_machine_identification_unique(&device_identification)?;
        validate_no_role_duplicates(&device_identification)?;

        let hardware = match &params.hardware {
            MachineNewHardware::Ethercat(x) => x,
            _ => {
                return Err(anyhow::anyhow!(
                    "[{}::MachineNewTrait/Wago671Slot12TestMachine::new] MachineNewHardware is not Ethercat",
                    module_path!()
                ));
            }
        };

        block_on(async {
            let wago_750_354 = get_ethercat_device::<Wago750_354>(
                hardware,
                params,
                0,
                vec![WAGO_750_354_IDENTITY_A],
            )
            .await?;

            let modules = Wago750_354::initialize_modules(wago_750_354.1).await?;
            let mut coupler = wago_750_354.0.write().await;
            for module in modules {
                coupler.set_module(module);
            }
            coupler.init_slot_modules(wago_750_354.1);

            let slot1_dev = coupler
                .slot_devices
                .get(0)
                .and_then(|entry| entry.clone())
                .ok_or_else(|| anyhow::anyhow!("slot 0 missing Wago 750-671"))?;
            let slot2_dev = coupler
                .slot_devices
                .get(1)
                .and_then(|entry| entry.clone())
                .ok_or_else(|| anyhow::anyhow!("slot 1 missing Wago 750-671"))?;

            let slot1 = downcast_device::<Wago750_671>(slot1_dev)
                .await
                .map_err(|source| anyhow::anyhow!("slot 0 expected Wago 750-671: {}", source))?;
            let slot2 = downcast_device::<Wago750_671>(slot2_dev)
                .await
                .map_err(|source| anyhow::anyhow!("slot 1 expected Wago 750-671: {}", source))?;
            drop(coupler);

            let mut slot1 = StepperVelocityWago750671::new(slot1);
            let mut slot2 = StepperVelocityWago750671::new(slot2);
            configure_axis(&mut slot1);
            configure_axis(&mut slot2);

            let (sender, receiver) = smol::channel::unbounded();
            let mut machine = Self {
                api_receiver: receiver,
                api_sender: sender,
                machine_identification_unique: params.get_machine_identification_unique(),
                namespace: Wago671Slot12TestMachineNamespace {
                    namespace: params.namespace.clone(),
                },
                last_state_emit: Instant::now(),
                last_debug_emit: Instant::now(),
                main_sender: params.main_thread_channel.clone(),
                slot1,
                slot2,
            };
            machine.emit_state();
            Ok(machine)
        })
    }
}
