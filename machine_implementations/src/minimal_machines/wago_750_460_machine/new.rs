use std::{cell::RefCell, rc::Rc, time::Instant};

use anyhow::Error;

use qitech_lib::ethercat_hal::devices::{
    DynamicEthercatDevice, EthercatDevice, downcast_rc_refcell_dynamic, wago_750_354::Wago750_354,
    wago_modules::wago_750_460::Wago750_460,
};

use crate::{MachineHardware, MachineMessage, MachineNew};

use super::{Wago750_460Machine, api::Wago750_460MachineNamespace};

impl MachineNew for Wago750_460Machine {
    fn new(hw: MachineHardware) -> Result<Self, Error> {
        let ethercat_interface = match hw.ethercat_interface.clone() {
            Some(some_ethercat_interface) => some_ethercat_interface,
            None => return Err(Error::msg("ethercat interface must not be None")),
        };

        // Acquire the WAGO 750-354 bus coupler at role 0.
        let (wago_750_354_ref, wago_750_354_addr) =
            hw.try_get_ethercat_device_and_addr_by_role::<Wago750_354>(0)?;
        let mut wago_750_354 = wago_750_354_ref.borrow_mut();

        // Discover and register modules on the coupler.
        let modules =
            Wago750_354::initialize_modules(ethercat_interface.clone(), wago_750_354_addr)?;

        for module in modules {
            wago_750_354.set_module(module);
        }

        wago_750_354.init_slot_modules(ethercat_interface, wago_750_354_addr);

        // Clone a shared handle to the 750-460 module in slot 0.
        let dev: Rc<RefCell<dyn DynamicEthercatDevice>> = match wago_750_354
            .slot_devices
            .get(0)
            .and_then(|slot| slot.clone())
        {
            Some(a) => a,
            None => return Err(Error::msg("Slot 0 should have a device")),
        };

        let wago750_460 = downcast_rc_refcell_dynamic::<Wago750_460>(dev)
            .expect("downcasting device should work");

        let (sender, receiver) = tokio::sync::mpsc::channel::<MachineMessage>(10);
        let mut machine = Self {
            receiver,
            sender,
            machine_identification_unique: hw.identification,
            namespace: Wago750_460MachineNamespace { namespace: None },
            last_state_emit: Instant::now(),
            temperature_input_device: wago750_460,
        };

        machine.emit_state();
        Ok(machine)
    }
}
