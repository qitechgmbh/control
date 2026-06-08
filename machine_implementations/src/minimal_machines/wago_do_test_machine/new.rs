use std::{cell::RefCell, rc::Rc, time::Instant};

use anyhow::Error;

use qitech_lib::ethercat_hal::devices::{
    DynamicEthercatDevice, EthercatDevice, downcast_rc_refcell_dynamic, wago_750_354::Wago750_354,
    wago_modules::wago_750_530::Wago750_530,
};

use crate::{MachineHardware, MachineMessage, MachineNew};

use super::{WagoDOTestMachine, api::WagoDOTestMachineNamespace};

impl MachineNew for WagoDOTestMachine {
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

        let wago750_530 = downcast_rc_refcell_dynamic::<Wago750_530>(dev)
            .expect("downcasting device should work");

        drop(wago_750_354);

        let (sender, receiver) = tokio::sync::mpsc::channel::<MachineMessage>(10);
        let mut machine = Self {
            receiver,
            sender,
            machine_identification_unique: hw.identification,
            namespace: WagoDOTestMachineNamespace { namespace: None },
            last_state_emit: Instant::now(),
            led_on: [false; 8],
            digital_output_device: wago750_530,
        };

        machine.emit_state();
        Ok(machine)
    }
}
