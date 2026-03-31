use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
use control_core::socketio::namespace::Namespace;
use qitech_lib::{ethercat_hal::{devices::{EthercatDevice, downcast_rc_refcell}, machine_ident_read::MachineDeviceInfo}, machines::{Machine, MachineIdentificationUnique}};
use serde::Serialize;
use tokio::sync::mpsc::Sender;
pub mod minimal_machines;
pub mod machine_identification;
pub mod registry;
/*pub mod aquapath1;
#[cfg(not(feature = "mock-machine"))]
pub mod buffer1;
pub mod extruder1;
pub mod extruder2;
pub mod laser;
pub mod machine_identification;
pub mod minimal_machines;
pub mod registry;
pub mod serial;
pub mod wago_power;*/
/*pub mod wago_serial_machine;*/
/*pub mod winder2;*/

pub const VENDOR_QITECH: u16 = 0x0001;
pub const MACHINE_WINDER_V1: u16 = 0x0002;
pub const MACHINE_EXTRUDER_V1: u16 = 0x0004;
pub const MACHINE_LASER_V1: u16 = 0x0006;
pub const MACHINE_MOCK: u16 = 0x0007;
pub const MACHINE_BUFFER_V1: u16 = 0x0008;
pub const MACHINE_AQUAPATH_V1: u16 = 0x0009;
pub const MACHINE_WAGO_POWER_V1: u16 = 0x000A;
pub const MACHINE_EXTRUDER_V2: u16 = 0x0016;
pub const TEST_MACHINE: u16 = 0x0033;
pub const IP20_TEST_MACHINE: u16 = 0x0034;
pub const ANALOG_INPUT_TEST_MACHINE: u16 = 0x0035;
pub const WAGO_AI_TEST_MACHINE: u16 = 0x0036;
pub const DIGITAL_INPUT_TEST_MACHINE: u16 = 0x0040;
pub const WAGO_8CH_IO_TEST_MACHINE: u16 = 0x0041;
pub const WAGO_750_430_DI_MACHINE: u16 = 0x0043;
pub const WAGO_750_553_MACHINE: u16 = 0x0044;
pub const TEST_MACHINE_STEPPER: u16 = 0x0037;
pub const MOTOR_TEST_MACHINE: u16 = 0x0011;
pub const WAGO_DO_TEST_MACHINE: u16 = 0x000E;
pub const WAGO_750_501_TEST_MACHINE: u16 = 0x0042;

#[derive(Serialize, Debug, Clone)]
pub struct MachineValues {
    pub state: serde_json::Value,
    pub live_values: serde_json::Value,
}

pub enum MachineMessage {
    SubscribeNamespace(Namespace),
    UnsubscribeNamespace,
    HttpApiJsonRequest(serde_json::Value),
    RequestValues(tokio::sync::oneshot::Sender<MachineValues>),
}

pub trait MachineApi {
    fn act_machine_message(&mut self, msg: MachineMessage);
    fn get_api_sender(&self) -> Sender<MachineMessage>;
    fn api_mutate(&mut self, value: serde_json::Value) -> Result<(), anyhow::Error>;
    fn api_event_namespace(&mut self) -> Option<Namespace>;
}

#[derive( Debug, Clone)]
pub struct IdentifiedEthercat {
    pub hw : Rc<RefCell<dyn EthercatDevice>>,
    pub ident : MachineDeviceInfo
}

#[derive( Debug, Clone)]
pub enum Hardware {
    Ethercat(IdentifiedEthercat),
    Serial(),
    Usb(),
    ModbusTcp(),
    ModbusRtu(),
    ModbusAscii(),
}

#[derive( Debug, Clone)]
pub struct MachineHardware {
    pub hw : Vec<Hardware>,
}

impl MachineHardware {
    pub fn try_get_ethercat_device_by_index<T>(&self, index : usize) -> Result<Rc<RefCell<T>>,anyhow::Error> 
        where T : EthercatDevice 
    {
        let hw = self.hw.get(index);
        let hw = match hw {
            Some(hw) => hw,
            None => return Err(anyhow::anyhow!("index {} not found in hardware", index)),
        };

        let identified_ethercat = match hw {
            Hardware::Ethercat(rc_ecat) => rc_ecat,
            _ => return Err(anyhow::anyhow!("index {} not an ethercat device in hardware", index)),
        };
        Ok(downcast_rc_refcell::<T>(identified_ethercat.hw.clone())?)
    }
}

pub trait MachineNew: Sized {
    fn new(hw: MachineHardware) -> Result<Self>;
}

pub trait QiTechMachine: Machine + MachineApi  {}