use std::{cell::RefCell, rc::Rc, time::Instant};

use anyhow::Error;

use qitech_lib::ethercat_hal::devices::{
    DynamicEthercatDevice, EthercatDevice, downcast_rc_refcell_dynamic, wago_750_354::Wago750_354,
    wago_modules::wago_750_455::Wago750_455,
};

use crate::{MachineHardware, MachineMessage, MachineNew};

use super::{WagoAiTestMachine, api::WagoAiTestMachineNamespace};

impl MachineNew for WagoAiTestMachine {
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

        let wago750_455 = downcast_rc_refcell_dynamic::<Wago750_455>(dev)
            .expect("downcasting device should work");

        drop(wago_750_354);

        let (sender, receiver) = tokio::sync::mpsc::channel::<MachineMessage>(10);
        Ok(Self {
            receiver,
            sender,
            machine_identification_unique: hw.identification,
            namespace: WagoAiTestMachineNamespace { namespace: None },
            last_measurement: Instant::now(),
            measurement_rate_hz: 1.0,
            analog_input_device: wago750_455,
        })
    }
}
