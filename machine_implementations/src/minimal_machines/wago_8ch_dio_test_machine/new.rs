use std::time::Instant;

use crate::{MachineHardware, MachineMessage, MachineNew};

use super::{Wago8chDigitalIOTestMachine, api::Wago8chDigitalIOTestMachineNamespace};
use anyhow::Error;
use qitech_lib::ethercat_hal::devices::{
    DynamicEthercatDevice, EthercatDevice, downcast_subdevice, wago_750_354::Wago750_354,
    wago_modules::wago_750_1506::Wago750_1506,
};

impl MachineNew for Wago8chDigitalIOTestMachine {
    fn new<'maindevice>(hw: MachineHardware) -> Result<Self, Error> {
        let ethercat_interface = hw
            .ethercat_interface
            .clone()
            .expect("ethercat interface should exist");
        let wago_750_354_ref = hw
            .try_get_ethercat_device_by_role::<Wago750_354>(0)
            .expect("should have device with role 0");
        let mut wago_750_354 = wago_750_354_ref.borrow_mut();
        let device_ident = hw.try_get_ethercat_meta_by_role(0)?;

        let modules = Wago750_354::initialize_modules(ethercat_interface.clone(), device_ident)?;

        for module in modules {
            wago_750_354.set_module(module);
        }

        wago_750_354.init_slot_modules(ethercat_interface, device_ident);

        let dev: Box<dyn DynamicEthercatDevice> = wago_750_354.slot_devices[0]
            .take()
            .expect("slot 0 device should exist");
        let dev: Box<dyn EthercatDevice> = dev;

        let wago750_1506 =
            downcast_subdevice::<Wago750_1506>(dev).expect("downcasting device should work");

        let (tx, rx) = tokio::sync::mpsc::channel::<MachineMessage>(2);
        let mut my_test = Self {
            api_receiver: rx,
            api_sender: tx,
            machine_identification_unique: hw.identification.clone(),
            namespace: Wago8chDigitalIOTestMachineNamespace { namespace: None },
            last_state_emit: Instant::now(),
            digital_input_output_device: wago750_1506,
            last_output_state: [false; 8],
        };
        my_test.emit_state();
        Ok(my_test)
    }
}
