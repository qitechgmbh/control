use crate::minimal_machines::test_machine_stepper::{
    TestMachineStepper, api::TestMachineStepperNamespace,
};
use ethercat_hal::{
    devices::{
        EthercatDevice, downcast_device,
        wago_750_354::{WAGO_750_354_IDENTITY_A, Wago750_354},
        wago_modules::wago_750_672::Wago750_672,
    },
    io::stepper_velocity_wago_750_672::StepperVelocityWago750672,
};
use smol::{block_on, lock::RwLock};
use std::{sync::Arc, time::Instant};

use crate::{
    MachineNewHardware, MachineNewParams, MachineNewTrait, get_ethercat_device,
    validate_no_role_dublicates, validate_same_machine_identification_unique,
};

use anyhow::Error;

impl MachineNewTrait for TestMachineStepper {
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
                    "[{}::MachineNewTrait/TestMachineStepper::new] MachineNewHardware is not Ethercat",
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
            let dev = coupler.slot_devices.first().unwrap().clone().unwrap();

            // change uncomment this to change to different stepper driver
            //
            // let wago_750_671: Arc<RwLock<Wago750_671>> =
            //     downcast_device::<Wago750_671>(dev).await?;
            let wago_750_672: Arc<RwLock<Wago750_672>> =
                downcast_device::<Wago750_672>(dev).await?;
            drop(coupler);

            let stepper = StepperVelocityWago750672::new(wago_750_672);

            let (sender, receiver) = smol::channel::unbounded();
            let mut my_test = Self {
                api_receiver: receiver,
                api_sender: sender,
                machine_identification_unique: params.get_machine_identification_unique(),
                namespace: TestMachineStepperNamespace {
                    namespace: params.namespace.clone(),
                },
                last_state_emit: Instant::now(),
                main_sender: params.main_thread_channel.clone(),
                stepper,
            };
            my_test.emit_state();
            Ok(my_test)
        })
    }
}
