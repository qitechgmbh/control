use std::{cell::RefCell, rc::Rc, time::Instant};

use anyhow::Error;

use qitech_lib::ethercat_hal::devices::{
    DynamicEthercatDevice, EthercatDevice, downcast_rc_refcell_dynamic, wago_750_354::Wago750_354,
    wago_modules::wago_750_553::Wago750_553,
};

use crate::{MachineHardware, MachineMessage, MachineNew};

use super::{Wago750_553Machine, api::Wago750_553MachineNamespace};

impl MachineNew for Wago750_553Machine {
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

        let dev: Rc<RefCell<dyn DynamicEthercatDevice>> = wago_750_354.slot_devices
        .get(0)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "[{}::Wago750_553Machine::new] Expected Wago 750-553 module in slot 0, but slot 0 is not configured",
                module_path!()
            )
        })?
        .clone()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "[{}::Wago750_553Machine::new] Expected Wago 750-553 module in slot 0, but slot 0 is empty",
                module_path!()
            )
        })?;
        let wago750_553 = downcast_rc_refcell_dynamic::<Wago750_553>(dev)
            .expect("downcasting device should work");

        drop(wago_750_354);

        let (sender, receiver) = tokio::sync::mpsc::channel::<MachineMessage>(10);
        let mut machine = Self {
            receiver,
            sender,
            machine_identification_unique: hw.identification,
            namespace: Wago750_553MachineNamespace { namespace: None },
            last_state_emit: Instant::now(),
            outputs: [0.0; 4],
            analog_output_device: wago750_553,
        };

        machine.emit_state();
        Ok(machine)
    }
}
