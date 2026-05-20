use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use anyhow::Error;

use qitech_lib::ethercat_hal::{
    devices::{
        DynamicEthercatDevice, EthercatDevice, downcast_subdevice,
        wago_750_354::Wago750_354,
        wago_modules::{
            wago_750_671::{WAGO_750_671_MODULE_IDENT, Wago750_671},
            wago_750_672::{WAGO_750_672_MODULE_IDENT, Wago750_672},
        },
    },
    io::{
        stepper_velocity_wago_750_671::StepperVelocityWago750671,
        stepper_velocity_wago_750_672::StepperVelocityWago750672,
    },
};

use crate::{
    MachineHardware, MachineMessage, MachineNew,
    minimal_machines::test_machine_stepper::{
        Stepper, TestMachineStepper, api::TestMachineStepperNamespace,
    },
};

impl MachineNew for TestMachineStepper {
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

        // Identify slot 0 module via its (vendor_id, product_id) tuple so we can
        // pick the right concrete stepper type without a fail-and-retry downcast.
        let slot_ident = match wago_750_354.slots[0]
            .as_ref()
            .map(|m| (m.vendor_id, m.product_id))
        {
            Some(a) => a,
            None => return Err(Error::msg("Slot 0 should be populated")),
        };

        let dev: Box<dyn DynamicEthercatDevice> = match wago_750_354.slot_devices[0].take() {
            Some(a) => a,
            None => return Err(Error::msg("Slot 0 should be populated")),
        };
        let dev: Box<dyn EthercatDevice> = dev;

        let stepper = match slot_ident {
            WAGO_750_672_MODULE_IDENT => {
                let inner = *downcast_subdevice::<Wago750_672>(dev)?;
                Stepper::Wago750_672(StepperVelocityWago750672::new(Rc::new(RefCell::new(inner))))
            }
            WAGO_750_671_MODULE_IDENT => {
                let inner = *downcast_subdevice::<Wago750_671>(dev)?;
                Stepper::Wago750_671(StepperVelocityWago750671::new(Rc::new(RefCell::new(inner))))
            }
            (vendor_id, product_id) => {
                return Err(anyhow::anyhow!(
                    "slot 0 module is not a supported stepper (vendor_id={:#x}, product_id={})",
                    vendor_id,
                    product_id
                ));
            }
        };

        let (sender, receiver) = tokio::sync::mpsc::channel::<MachineMessage>(10);
        let mut machine = Self {
            receiver,
            sender,
            machine_identification_unique: hw.identification,
            namespace: TestMachineStepperNamespace { namespace: None },
            last_state_emit: Instant::now(),
            stepper,
        };
        machine.emit_state();
        Ok(machine)
    }
}
