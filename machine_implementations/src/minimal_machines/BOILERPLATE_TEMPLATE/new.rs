// ============================================================================
// new.rs — Hardware initialization (called once at startup)
// ============================================================================
// Two common patterns:
//   Pattern A — Beckhoff EtherCAT terminal (e.g. EL2004, EL3021, EL7031_0030)
//   Pattern B — WAGO 750 bus coupler + expansion modules (e.g. 750-460)
//
// Keep ONLY the pattern you need — delete the other block.
// ============================================================================

use std::time::Instant;

use anyhow::Error;

use super::{MyMachine, api::MyMachineNamespace};
use crate::{MachineHardware, MachineMessage, MachineNew};

// --- Pattern A imports (Beckhoff terminal) ----------------------------------
// use std::{cell::RefCell, rc::Rc};
// use qitech_lib::ethercat_hal::devices::el2004::EL2004;
// use qitech_lib::ethercat_hal::io::digital_output::DigitalOutputDevice;

// --- Pattern B imports (WAGO coupler + module) ------------------------------
// use qitech_lib::ethercat_hal::devices::{
//     DynamicEthercatDevice, EthercatDevice, downcast_subdevice,
//     wago_750_354::Wago750_354,
//     wago_modules::wago_750_530::Wago750_530,
// };

impl MachineNew for MyMachine {
    fn new(hw: MachineHardware) -> Result<Self, Error> {
        // --------------------------------------------------------------------
        // Pattern A — Beckhoff terminal
        // --------------------------------------------------------------------
        // Role index matches `device_roles` in the machine's properties.ts.
        // Use `try_get_ethercat_device_by_role::<T>` for a concrete type
        // or to a `dyn TraitDevice` (e.g. `dyn DigitalOutputDevice`).
        //
        // let el2004: Rc<RefCell<EL2004>> = hw.try_get_ethercat_device_by_role(1)?;
        //
        // If you need to write CoE config to the device, also get the address:
        // let (device, device_addr) =
        //     hw.try_get_ethercat_device_and_addr_by_role::<EL7031_0030>(1)?;
        // device.borrow_mut().write_config(
        //     hw.ethercat_interface
        //         .clone()
        //         .ok_or_else(|| Error::msg("ethercat interface must not be None"))?,
        //     device_addr,
        //     &config,
        // )?;

        // --------------------------------------------------------------------
        // Pattern B — WAGO 750 coupler + expansion module
        // --------------------------------------------------------------------
        // The coupler is always role 0. After acquiring it, discover modules,
        // attach them to the coupler, init the slots, then pluck the subdevice
        // out of the slot you care about and downcast it.
        //
        // let ethercat_interface = hw
        //     .ethercat_interface
        //     .clone()
        //     .ok_or_else(|| Error::msg("ethercat interface must not be None"))?;
        //
        // let (wago_750_354_ref, coupler_addr) =
        //     hw.try_get_ethercat_device_and_addr_by_role::<Wago750_354>(0)?;
        // let mut wago_750_354 = wago_750_354_ref.borrow_mut();
        //
        // let modules =
        //     Wago750_354::initialize_modules(ethercat_interface.clone(), coupler_addr)?;
        // for module in modules {
        //     wago_750_354.set_module(module);
        // }
        // wago_750_354.init_slot_modules(ethercat_interface, coupler_addr);
        //
        // // Take ownership of the module from slot 0 (first expansion slot).
        // let dev: Box<dyn DynamicEthercatDevice> = wago_750_354.slot_devices[0]
        //     .take()
        //     .ok_or_else(|| Error::msg("Slot 0 should have a device"))?;
        // let dev: Box<dyn EthercatDevice> = dev;
        // let wago750_530 = downcast_subdevice::<Wago750_530>(dev)?;
        // drop(wago_750_354);

        // --------------------------------------------------------------------
        // Optional: hard-check the machine identification matches.
        // Useful when developing — catches wiring/registry mistakes early.
        // --------------------------------------------------------------------
        // if hw.identification.machine_ident != MyMachine::MACHINE_IDENTIFICATION {
        //     return Err(anyhow::anyhow!(
        //         "MyMachine: hardware ident {:?} != expected {:?}",
        //         hw.identification.machine_ident,
        //         MyMachine::MACHINE_IDENTIFICATION,
        //     ));
        // }

        // --------------------------------------------------------------------
        // Build the machine struct (mandatory plumbing).
        // Channel buffer of 10 is the convention across minimal machines.
        // --------------------------------------------------------------------
        let (sender, receiver) = tokio::sync::mpsc::channel::<MachineMessage>(10);
        let mut machine = Self {
            receiver,
            sender,
            machine_identification_unique: hw.identification.clone(),
            namespace: MyMachineNamespace { namespace: None },
            last_state_emit: Instant::now(),
            // TODO: hardware + state fields, e.g.:
            // digital_output_device: el2004,
            // last_output_state: [false; 4],
        };

        // Emit initial state so the first subscriber doesn't see an empty UI.
        machine.emit_state();
        Ok(machine)
    }
}
