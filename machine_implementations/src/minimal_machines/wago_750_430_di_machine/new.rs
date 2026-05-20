use std::time::Instant;

use anyhow::Error;

use qitech_lib::ethercat_hal::devices::{
    DynamicEthercatDevice, EthercatDevice, downcast_subdevice, wago_750_354::Wago750_354,
    wago_modules::wago_750_430::Wago750_430,
};

use crate::{MachineHardware, MachineMessage, MachineNew};

use super::{Wago750_430DiMachine, api::Wago750_430DiMachineNamespace};

impl MachineNew for Wago750_430DiMachine {
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

        let dev: Box<dyn DynamicEthercatDevice> = wago_750_354.slot_devices[0]
            .take()
            .ok_or_else(|| Error::msg("Slot 0 should have a device"))?;
        let dev: Box<dyn EthercatDevice> = dev;

        let wago750_430 =
            downcast_subdevice::<Wago750_430>(dev).expect("downcasting device should work");

        drop(wago_750_354);

        let (sender, receiver) = tokio::sync::mpsc::channel::<MachineMessage>(10);
        let mut machine = Self {
            receiver,
            sender,
            machine_identification_unique: hw.identification,
            namespace: Wago750_430DiMachineNamespace { namespace: None },
            last_state_emit: Instant::now(),
            digital_input_device: wago750_430,
        };

        machine.emit_state();
        Ok(machine)
    }
}
