use std::time::Instant;

use anyhow::Error;
use control_core::converters::linear_step_converter::LinearStepConverter;
use ethercat_hal::{
    devices::{
        EthercatDevice, downcast_device,
        wago_750_354::{WAGO_750_354_IDENTITY_A, Wago750_354},
        wago_modules::{
            wago_750_501::{Wago750_501, Wago750_501Port},
            wago_750_671::Wago750_671,
        },
    },
    io::digital_output::DigitalOutput,
    io::stepper_velocity_wago_750_671::StepperVelocityWago750671,
};
use smol::{block_on, lock::RwLock};
use std::sync::Arc as StdArc;
use units::f64::Length;
use units::length::millimeter;

use super::{
    TestControlMode, TestMachineMode, WagoTraverseTestMachine,
    api::WagoTraverseTestMachineNamespace,
};
use crate::{
    MachineNewHardware, MachineNewParams, MachineNewTrait, get_ethercat_device,
    validate_no_role_duplicates, validate_same_machine_identification_unique,
};

async fn get_slot_device<T: EthercatDevice>(
    coupler: &Wago750_354,
    slot: usize,
    device_name: &str,
) -> Result<StdArc<RwLock<T>>, Error> {
    let dev = coupler
        .slot_devices
        .get(slot)
        .and_then(|entry| entry.clone())
        .ok_or_else(|| anyhow::anyhow!("slot {} missing {}", slot, device_name))?;

    downcast_device::<T>(dev).await
}

impl MachineNewTrait for WagoTraverseTestMachine {
    fn new<'maindevice>(params: &MachineNewParams) -> Result<Self, Error> {
        let device_identification = params.device_group.to_vec();
        validate_same_machine_identification_unique(&device_identification)?;
        validate_no_role_duplicates(&device_identification)?;

        let hardware = match &params.hardware {
            MachineNewHardware::Ethercat(x) => x,
            _ => {
                return Err(anyhow::anyhow!(
                    "[{}::WagoTraverseTestMachine::new] MachineNewHardware is not Ethercat",
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

            let wago_750_501 =
                get_slot_device::<Wago750_501>(&coupler, 0, "switch Wago 750-501").await?;
            let traverse_671 =
                get_slot_device::<Wago750_671>(&coupler, 1, "traverse Wago 750-671").await?;
            drop(coupler);

            let mut traverse = StepperVelocityWago750671::new(traverse_671);
            traverse.set_freq_range_sel(3);
            traverse.set_acc_range_sel(2);
            traverse.set_acceleration(2);
            let switch_output = DigitalOutput::new(wago_750_501, Wago750_501Port::Port1);

            let (sender, receiver) = smol::channel::unbounded();
            let mut machine = Self {
                api_receiver: receiver,
                api_sender: sender,
                machine_identification_unique: params.get_machine_identification_unique(),
                main_sender: params.main_thread_channel.clone(),
                namespace: WagoTraverseTestMachineNamespace {
                    namespace: params.namespace.clone(),
                },
                last_state_emit: Instant::now(),
                switch_output,
                switch_output_on: false,
                traverse,
                fullstep_converter: LinearStepConverter::from_circumference(
                    200,
                    Length::new::<millimeter>(35.0),
                ),
                microstep_converter: LinearStepConverter::from_circumference(
                    200 * 64,
                    Length::new::<millimeter>(35.0),
                ),
                control_mode: TestControlMode::Idle,
                mode: TestMachineMode::Standby,
                homing_state: super::BenchHomingState::NotHomed,
                homing_backoff_target_steps: None,
                homing_validate_started_at: None,
                limit_inner: Length::new::<millimeter>(22.0),
                limit_outer: Length::new::<millimeter>(92.0),
                manual_speed_mm_per_second: 0.0,
                manual_velocity_register: 5000,
            };

            machine.emit_state();
            Ok(machine)
        })
    }
}
