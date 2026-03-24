// ============================================================================
// new.rs — Machine constructor (hardware initialization)
// ============================================================================
// This file implements `MachineNewTrait` for your machine, which is called
// once at startup to acquire hardware handles and build the machine struct.
//
// Two common hardware patterns are shown:
//   A) Beckhoff EtherCAT terminal (e.g. EL2004 digital output)
//   B) WAGO 750 coupler + expansion module (e.g. 750-530 digital output)
//
// Only keep the block that matches your hardware — delete the other.
// ============================================================================

use std::time::Instant;

use anyhow::Error;
use smol::block_on;

use crate::{
    MachineNewHardware, MachineNewParams, MachineNewTrait,
    get_ethercat_device, validate_no_role_dublicates, validate_same_machine_identification_unique,
};
use super::{MyMachine, api::MyMachineNamespace};

// --- Pattern A: Beckhoff terminal imports ------------------------------------
// Uncomment and adapt for the specific terminal you need.
//
// use ethercat_hal::devices::el2004::{EL2004, EL2004_IDENTITY_A, EL2004Port};
// use ethercat_hal::io::digital_output::DigitalOutput;

// --- Pattern B: WAGO coupler + module imports ---------------------------------
// Uncomment and adapt for the specific WAGO module you need.
//
// use std::sync::Arc;
// use smol::lock::RwLock;
// use ethercat_hal::devices::{EthercatDevice, downcast_device};
// use ethercat_hal::devices::wago_750_354::{WAGO_750_354_IDENTITY_A, Wago750_354};
// use ethercat_hal::devices::wago_modules::wago_750_530::{Wago750_530, Wago750_530Port};
// use ethercat_hal::io::digital_output::DigitalOutput;

impl MachineNewTrait for MyMachine {
    fn new(params: &MachineNewParams) -> Result<Self, Error> {
        // --- Validate the device group (mandatory, keep as-is) --------------
        let device_identification = params
            .device_group
            .iter()
            .map(|d| d.clone())
            .collect::<Vec<_>>();
        validate_same_machine_identification_unique(&device_identification)?;
        validate_no_role_dublicates(&device_identification)?;

        // --- Require EtherCAT hardware (mandatory for EtherCAT machines) ----
        let hardware = match &params.hardware {
            MachineNewHardware::Ethercat(x) => x,
            _ => {
                return Err(anyhow::anyhow!(
                    "[{}::MachineNewTrait/MyMachine::new] MachineNewHardware is not Ethercat",
                    module_path!()
                ));
            }
        };

        block_on(async {
            // ----------------------------------------------------------------
            // Pattern A — Beckhoff EtherCAT terminal
            // ----------------------------------------------------------------
            // `get_ethercat_device` arguments:
            //   hardware  — from above
            //   params    — from above
            //   role      — device role index as configured in properties.ts
            //   identities — list of accepted (vendor_id, product_id, revision)
            //
            // Example: EL2004 (4× digital output), role 1
            //
            // let el2004 = get_ethercat_device::<EL2004>(
            //     hardware, params, 1, [EL2004_IDENTITY_A].to_vec(),
            // ).await?.0;
            //
            // let do1 = DigitalOutput::new(el2004.clone(), EL2004Port::DO1);
            // let do2 = DigitalOutput::new(el2004.clone(), EL2004Port::DO2);
            // let do3 = DigitalOutput::new(el2004.clone(), EL2004Port::DO3);
            // let do4 = DigitalOutput::new(el2004.clone(), EL2004Port::DO4);

            // ----------------------------------------------------------------
            // Pattern B — WAGO 750 bus coupler + expansion module
            // ----------------------------------------------------------------
            // The coupler is always role 0. Modules are discovered by reading
            // the PDO slot map from the coupler subdevice.
            //
            // Example: WAGO 750-354 coupler with a 750-530 (8× DO) in slot 0
            //
            // let (coupler_dev, coupler_subdev) = get_ethercat_device::<Wago750_354>(
            //     hardware, params, 0, [WAGO_750_354_IDENTITY_A].to_vec(),
            // ).await?;
            //
            // let modules = Wago750_354::initialize_modules(coupler_subdev).await?;
            // let mut coupler = coupler_dev.write().await;
            // for module in modules { coupler.set_module(module); }
            // coupler.init_slot_modules(coupler_subdev);
            //
            // // Get the device at slot index 0 (first expansion module).
            // let dev = coupler.slot_devices.get(0)
            //     .ok_or_else(|| anyhow::anyhow!(
            //         "[{}::MyMachine::new] slot 0 not configured", module_path!()
            //     ))?
            //     .clone()
            //     .ok_or_else(|| anyhow::anyhow!(
            //         "[{}::MyMachine::new] slot 0 is empty", module_path!()
            //     ))?;
            // let wago750_530: Arc<RwLock<Wago750_530>> =
            //     downcast_device::<Wago750_530>(dev).await?;
            //
            // let do1 = DigitalOutput::new(wago750_530.clone(), Wago750_530Port::Port1);
            // // ... repeat for do2..do8
            // drop(coupler);

            // ----------------------------------------------------------------
            // Build the machine struct (mandatory plumbing, adapt fields)
            // ----------------------------------------------------------------
            let (sender, receiver) = smol::channel::unbounded();
            let mut machine = Self {
                api_receiver: receiver,
                api_sender: sender,
                machine_identification_unique: params.get_machine_identification_unique(),
                main_sender: params.main_thread_channel.clone(),
                namespace: MyMachineNamespace {
                    namespace: params.namespace.clone(),
                },
                last_state_emit: Instant::now(),

                // TODO: add your hardware fields here, e.g.:
                // douts: [do1, do2, do3, do4],
            };

            // Emit initial state so subscribers get values immediately.
            machine.emit_state();
            Ok(machine)
        })
    }
}
