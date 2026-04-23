#[cfg(not(feature = "mock-machine"))]
mod winder2_imports {
    pub use super::super::api::Winder2Namespace;
    pub use super::super::tension_arm::TensionArm;
    pub use super::super::{WagoWinder, Winder2Mode};
    pub use crate::wago_winder::puller_speed_controller::PullerSpeedController;
    pub use crate::wago_winder::spool_speed_controller::SpoolSpeedController;
    pub use crate::wago_winder::traverse_controller::TraverseController;
    pub use crate::{
        MachineNewHardware, MachineNewParams, MachineNewTrait, validate_no_role_duplicates,
        validate_same_machine_identification_unique,
    };
    pub use anyhow::Error;
    pub use control_core::converters::angular_step_converter::AngularStepConverter;
    pub use control_core::converters::linear_step_converter::LinearStepConverter;
    pub use ethercat_hal::devices::wago_750_354::{WAGO_750_354_IDENTITY_A, Wago750_354};
    pub use ethercat_hal::devices::wago_modules::wago_750_467::{Wago750_467, Wago750_467Port};
    pub use ethercat_hal::devices::wago_modules::wago_750_501::{Wago750_501, Wago750_501Port};
    pub use ethercat_hal::devices::wago_modules::wago_750_671::Wago750_671;
    pub use ethercat_hal::devices::wago_modules::wago_750_672::Wago750_672;
    pub use ethercat_hal::devices::{EthercatDevice, downcast_device};
    pub use ethercat_hal::io::analog_input::AnalogInput;
    pub use ethercat_hal::io::digital_output::DigitalOutput;
    pub use ethercat_hal::io::stepper_velocity_wago_750_671::StepperVelocityWago750671;
    pub use ethercat_hal::io::stepper_velocity_wago_750_671_traverse::StepperVelocityWago750671Traverse;
    pub use ethercat_hal::io::stepper_velocity_wago_750_672::StepperVelocityWago750672;
    pub use smol::lock::RwLock;
    pub use std::sync::Arc;
    pub use std::time::Instant;
    pub use units::ConstZero;
    pub use units::f64::*;
    pub use units::length::{centimeter, meter, millimeter};
    pub use units::velocity::meter_per_minute;
}

#[cfg(not(feature = "mock-machine"))]
pub use winder2_imports::*;

#[cfg(not(feature = "mock-machine"))]
use crate::get_ethercat_device;

#[cfg(not(feature = "mock-machine"))]
async fn get_slot_device<T: EthercatDevice>(
    coupler: &Wago750_354,
    slot: usize,
    device_name: &str,
) -> Result<Arc<RwLock<T>>, Error> {
    let dev = coupler
        .slot_devices
        .get(slot)
        .and_then(|entry| entry.clone())
        .ok_or_else(|| anyhow::anyhow!("slot {} missing {}", slot, device_name))?;

    downcast_device::<T>(dev).await
}

#[cfg(not(feature = "mock-machine"))]
impl MachineNewTrait for WagoWinder {
    fn new<'maindevice>(params: &MachineNewParams) -> Result<Self, Error> {
        // validate general stuff

        let device_identification = params.device_group.to_vec();

        validate_same_machine_identification_unique(&device_identification)?;
        validate_no_role_duplicates(&device_identification)?;

        let hardware = match &params.hardware {
            MachineNewHardware::Ethercat(x) => x,
            _ => {
                return Err(anyhow::anyhow!(
                    "[{}::MachineNewTrait/WagoWinder::new] MachineNewHardware is not Ethercat",
                    module_path!()
                ));
            }
        };

        smol::block_on(async {
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

            let wago_750_501 = get_slot_device::<Wago750_501>(&coupler, 0, "Wago 750-501").await?;
            let traverse_671 =
                get_slot_device::<Wago750_671>(&coupler, 1, "traverse Wago 750-671").await?;
            let spool_672 =
                get_slot_device::<Wago750_672>(&coupler, 2, "spool Wago 750-672").await?;
            let puller_672 =
                get_slot_device::<Wago750_672>(&coupler, 3, "puller Wago 750-672").await?;
            let wago_750_467 = get_slot_device::<Wago750_467>(&coupler, 4, "Wago 750-467").await?;
            drop(coupler);

            let mode = Winder2Mode::Standby;

            let machine_id = params
                .device_group
                .first()
                .expect("device group must have at least one device")
                .device_machine_identification
                .machine_identification_unique
                .clone();
            let (sender, receiver) = smol::channel::unbounded();
            let tension_arm_raw = AnalogInput::new(wago_750_467.clone(), Wago750_467Port::AI1);
            let mut new = Self {
                main_sender: params.main_thread_channel.clone(),
                api_receiver: receiver,
                api_sender: sender,
                traverse: StepperVelocityWago750671Traverse::new(StepperVelocityWago750671::new(
                    traverse_671,
                )),
                puller: StepperVelocityWago750672::new(puller_672),
                spool: StepperVelocityWago750672::new(spool_672),
                tension_arm: TensionArm::new(AnalogInput::new(wago_750_467, Wago750_467Port::AI1)),
                tension_arm_raw,
                laser: DigitalOutput::new(wago_750_501, Wago750_501Port::Port1),
                namespace: Winder2Namespace {
                    namespace: params.namespace.clone(),
                },
                mode: mode.clone(),
                spool_step_converter: AngularStepConverter::new(200),
                spool_speed_controller: SpoolSpeedController::new(),
                last_measurement_emit: Instant::now(),
                last_debug_signature: None,
                last_axis_status_signature: None,
                last_traverse_debug_raw_position: None,
                last_traverse_debug_raw_delta: 0,
                last_control_loop_debug_emit: Instant::now(),
                spool_mode: mode.clone().into(),
                traverse_mode: mode.clone().into(),
                puller_mode: mode.into(),
                puller_speed_controller: PullerSpeedController::new(
                    Velocity::new::<meter_per_minute>(1.0),
                    LinearStepConverter::from_diameter(
                        200,                            // Assuming 200 steps per revolution for the puller stepper,
                        Length::new::<centimeter>(8.0), // 8cm diameter of the puller wheel
                    ),
                ),
                puller_reference_machine: None,
                traverse_controller: TraverseController::new(
                    Length::new::<millimeter>(22.0), // Default inner limit
                    Length::new::<millimeter>(92.0), // Default outer limit
                    64,                              // Microsteps
                ),
                emitted_default_state: false,
                spool_automatic_action: super::SpoolAutomaticAction {
                    progress: Length::ZERO,
                    progress_last_check: Instant::now(),
                    target_length: Length::new::<meter>(250.0),
                    mode: super::api::SpoolAutomaticActionMode::NoAction,
                },
                spool_tension_blocked: false,
                machine_identification_unique: machine_id,
            };

            new.spool.set_motor_full_steps_per_rev(200);
            new.spool.set_microsteps_per_full_step(64);
            new.spool.set_direction_multiplier(1);
            new.spool.set_speed_scale(1.0);
            new.spool.set_restart_on_velocity_change(false);
            new.spool.set_freq_range_sel(2);
            new.spool.set_acc_range_sel(2);
            new.spool.set_acceleration(1600);
            new.spool.request_speed_mode();
            new.spool.clear_fast_stop();
            new.spool.request_set_nominal_current_tenths_amp(50);
            new.spool.request_set_current_mailbox(100, 0x0F);
            new.traverse.configure_for_traverse_contract(3, 2, 1000);
            new.traverse
                .inner_mut()
                .request_set_current_mailbox(150, 0x0F);
            new.puller.set_motor_full_steps_per_rev(200);
            new.puller.set_microsteps_per_full_step(64);
            new.puller.set_direction_multiplier(-1);
            new.puller.set_speed_scale(1.0);
            new.puller.set_restart_on_velocity_change(false);
            new.puller.set_freq_range_sel(2);
            new.puller.set_acc_range_sel(2);
            new.puller.set_acceleration(1600);
            new.puller.request_speed_mode();
            new.puller.clear_fast_stop();
            new.puller.request_set_nominal_current_tenths_amp(28);
            new.puller.request_set_current_mailbox(100, 0x0F);
            new.traverse.set_acceleration(1000);

            // initalize events
            new.emit_state();
            Ok(new)
        })
    }
}
