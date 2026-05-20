use std::cell::RefCell;
use std::rc::Rc;

use crate::{MachineHardware, MachineMessage, MachineNew};

use super::{MotorState, MotorTestMachine, api::BeckhoffNamespace};
use anyhow::Error;

use ethercat_hal::coe::ConfigurableDevice;
use ethercat_hal::devices::ek1100::EK1100;
use ethercat_hal::devices::el7031_0030::EL7031_0030;
use ethercat_hal::devices::el7031_0030::coe::EL7031_0030Configuration;
use ethercat_hal::devices::el7031_0030::pdo::EL7031_0030PredefinedPdoAssignment;
use ethercat_hal::shared_config;
use ethercat_hal::shared_config::el70x1::{EL70x1OperationMode, StmMotorConfiguration};
use qitech_lib::ethercat_hal;

impl MachineNew for MotorTestMachine {
    fn new<'maindevice>(hw: MachineHardware) -> Result<Self, Error> {
        println!("[{}::new] Creating new MotorTestMachine", module_path!());

        // removed a check on params.hardware that returned an error if not ethercat. is that necessary here?

        // Role 0: EK1100 (Koppler)
        let _ek1100: Rc<RefCell<EK1100>> = hw.try_get_ethercat_device_by_role(0)?;

        // Role 1: EL7031 (Stepper Motor)
        let el7031 = {
            let (device, device_addr): (Rc<RefCell<EL7031_0030>>, u16) =
                hw.try_get_ethercat_device_and_addr_by_role(1)?;

            let el7031_config = EL7031_0030Configuration {
                stm_features: ethercat_hal::devices::el7031_0030::coe::StmFeatures {
                    operation_mode: EL70x1OperationMode::DirectVelocity,
                    speed_range: shared_config::el70x1::EL70x1SpeedRange::Steps1000,
                    ..Default::default()
                },
                stm_motor: StmMotorConfiguration {
                    max_current: 1500,
                    ..Default::default()
                },
                pdo_assignment: EL7031_0030PredefinedPdoAssignment::VelocityControlCompact,
                ..Default::default()
            };

            device.borrow_mut().write_config(
                match hw.ethercat_interface {
                    Some(some_ethercat_interface) => some_ethercat_interface,
                    None => return Err(Error::msg("ethercat interface must not be None")),
                },
                device_addr,
                &el7031_config,
            )?;

            device
        };

        let (sender, receiver) = tokio::sync::mpsc::channel::<MachineMessage>(10);

        Ok(Self {
            receiver,
            sender,
            machine_identification_unique: hw.identification,
            namespace: BeckhoffNamespace { namespace: None },
            motor_driver: el7031,
            motor_driver_port: 0, //@TODO Would be cool if we could use EL7031_0030StepperPort::STM1.into::<usize>() in the future and have hardcoded mappings from enum values to ports
            motor_state: MotorState {
                enabled: true,
                target_velocity: 0,
            },
        })
    }
}
