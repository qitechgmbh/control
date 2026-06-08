use std::{cell::RefCell, rc::Rc, time::Instant};

use anyhow::Error;

use qitech_lib::ethercat_hal::devices::{
    DynamicEthercatDevice, EthercatDevice, downcast_rc_refcell_dynamic, wago_750_354::Wago750_354,
    wago_modules::wago_750_501::Wago750_501,
};

use crate::{MachineHardware, MachineMessage, MachineNew};

use super::{Wago750_501TestMachine, api::Wago750_501TestMachineNamespace};

impl MachineNew for Wago750_501TestMachine {
    fn new(hw: MachineHardware) -> Result<Self, Error> {
        let ethercat_interface = hw
            .ethercat_interface
            .clone()
            .ok_or_else(|| Error::msg("ethercat interface must not be None"))?;

        let (wago_750_354_ref, wago_750_354_addr) =
            hw.try_get_ethercat_device_and_addr_by_role::<Wago750_354>(0)?;
        let mut wago_750_354 = wago_750_354_ref.borrow_mut();

        let modules =
            Wago750_354::initialize_modules(ethercat_interface.clone(), wago_750_354_addr)?;

        for module in modules {
            wago_750_354.set_module(module);
        }

        wago_750_354.init_slot_modules(ethercat_interface, wago_750_354_addr);

        let dev: Rc<RefCell<dyn DynamicEthercatDevice>> = wago_750_354
            .slot_devices
            .get(0)
            .ok_or_else(|| Error::msg("Slot 0 should have a device"))?
            .clone()
            .ok_or_else(|| Error::msg("Slot 0 should have a device"))?;

        let wago750_501 = downcast_rc_refcell_dynamic::<Wago750_501>(dev)
            .expect("downcasting device should work");

        drop(wago_750_354);

        let (sender, receiver) = tokio::sync::mpsc::channel::<MachineMessage>(10);
        let mut machine = Self {
            receiver,
            sender,
            machine_identification_unique: hw.identification,
            namespace: Wago750_501TestMachineNamespace { namespace: None },
            last_state_emit: Instant::now(),
            outputs: [false; 2],
            digital_output_device: wago750_501,
        };

        machine.emit_state();
        Ok(machine)
    }
}
