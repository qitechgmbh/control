use std::{cell::RefCell, rc::Rc, time::Instant};

use crate::{MachineHardware, MachineMessage, MachineNew};

use super::{Wago8chDigitalIOTestMachine, api::Wago8chDigitalIOTestMachineNamespace};
use anyhow::Error;
use qitech_lib::ethercat_hal::devices::{
    DynamicEthercatDevice, EthercatDevice, downcast_rc_refcell_dynamic, wago_750_354::Wago750_354,
    wago_modules::wago_750_1506::Wago750_1506,
};

impl MachineNew for Wago8chDigitalIOTestMachine {
    fn new<'maindevice>(hw: MachineHardware) -> Result<Self, Error> {
        let ethercat_interface = match hw.ethercat_interface.clone() {
            Some(some_ethercat_interface) => some_ethercat_interface,
            None => return Err(Error::msg("ethercat interface must not be None")),
        };
        let (wago_750_354_ref, device_ident) =
            hw.try_get_ethercat_device_and_addr_by_role::<Wago750_354>(0)?;
        let mut wago_750_354 = wago_750_354_ref.borrow_mut();

        let modules = Wago750_354::initialize_modules(ethercat_interface.clone(), device_ident)?;

        for module in modules {
            wago_750_354.set_module(module);
        }

        wago_750_354.init_slot_modules(ethercat_interface, device_ident);

        let dev: Rc<RefCell<dyn DynamicEthercatDevice>> = match wago_750_354
            .slot_devices
            .get(0)
            .and_then(|slot| slot.clone())
        {
            Some(a) => a,
            None => return Err(Error::msg("Slot 0 should be populated")),
        };

        let wago750_1506 = downcast_rc_refcell_dynamic::<Wago750_1506>(dev)?;

        let (sender, receiver) = tokio::sync::mpsc::channel::<MachineMessage>(10);
        let mut my_test = Self {
            receiver,
            sender,
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
