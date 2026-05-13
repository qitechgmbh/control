use std::time::Instant;

use anyhow::Error;

use qitech_lib::ethercat_hal::devices::{
    DynamicEthercatDevice, EthercatDevice, downcast_subdevice, wago_750_354::Wago750_354,
    wago_modules::wago_750_460::Wago750_460,
};

use crate::{MachineHardware, MachineMessage, MachineNew};

use super::{Wago750_460Machine, api::Wago750_460MachineNamespace};

impl MachineNew for Wago750_460Machine {
    fn new(hw: MachineHardware) -> Result<Self, Error> {
        let ethercat_interface = hw
            .ethercat_interface
            .clone()
            .expect("ethercat interface should exist");

        // Acquire the WAGO 750-354 bus coupler at role 0.
        let (wago_750_354_ref, wago_750_354_ident) = hw
            .try_get_ethercat_device_and_addr_by_role::<Wago750_354>(0)
            .expect("should have device with role 0");
        let mut wago_750_354 = wago_750_354_ref.borrow_mut();

        // Discover and register modules on the coupler.
        let modules =
            Wago750_354::initialize_modules(ethercat_interface.clone(), wago_750_354_ident)?;

        for module in modules {
            wago_750_354.set_module(module);
        }

        wago_750_354.init_slot_modules(ethercat_interface, wago_750_354_ident);

        // Retrieve the 750-460 module from slot 0.
        let dev: Box<dyn DynamicEthercatDevice> = wago_750_354.slot_devices[0]
            .take()
            .expect("slot 0 device should exist");
        let dev: Box<dyn EthercatDevice> = dev;

        let wago750_460 =
            downcast_subdevice::<Wago750_460>(dev).expect("downcasting device should work");

        let (sender, receiver) = tokio::sync::mpsc::channel::<MachineMessage>(2);
        let mut machine = Self {
            api_receiver: receiver,
            api_sender: sender,
            machine_identification_unique: hw.identification,
            namespace: Wago750_460MachineNamespace { namespace: None },
            last_state_emit: Instant::now(),
            temperature_input_device: wago750_460,
        };

        machine.emit_state();
        Ok(machine)
    }
}
