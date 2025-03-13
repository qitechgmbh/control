use super::SubDeviceIdentityTuple;
use crate::io::digital_output::{DigitalOutputDevice, DigitalOutputOutput, DigitalOutputState};
use crate::pdo::{basic::BoolPdoObject, RxPdo};
use crate::types::EthercrabSubDevicePreoperational;
use ethercat_hal_derive::{Device, RxPdo};

/// EL2008 8-channel digital output device
///
/// 24V DC, 0.5A per channel
#[derive(Device)]
pub struct EL2008 {
    pub output_ts: u64,
    pub rxpdo: EL2008RxPdo,
}

impl std::fmt::Debug for EL2008 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EL2003")
    }
}

impl EL2008 {
    pub fn new() -> Self {
        Self {
            output_ts: 0,
            rxpdo: EL2008RxPdo::default(),
        }
    }
}

impl DigitalOutputDevice<EL2008Port> for EL2008 {
    fn digital_output_write(&mut self, port: EL2008Port, value: bool) {
        match port {
            EL2008Port::DO1 => self.rxpdo.channel1.as_mut().unwrap().value = value,
            EL2008Port::DO2 => self.rxpdo.channel2.as_mut().unwrap().value = value,
            EL2008Port::DO3 => self.rxpdo.channel3.as_mut().unwrap().value = value,
            EL2008Port::DO4 => self.rxpdo.channel4.as_mut().unwrap().value = value,
            EL2008Port::DO5 => self.rxpdo.channel5.as_mut().unwrap().value = value,
            EL2008Port::DO6 => self.rxpdo.channel6.as_mut().unwrap().value = value,
            EL2008Port::DO7 => self.rxpdo.channel7.as_mut().unwrap().value = value,
            EL2008Port::DO8 => self.rxpdo.channel8.as_mut().unwrap().value = value,
        }
    }

    fn digital_output_state(&self, port: EL2008Port) -> DigitalOutputState {
        DigitalOutputState {
            output_ts: self.output_ts,
            output: DigitalOutputOutput {
                value: match port {
                    EL2008Port::DO1 => self.rxpdo.channel1.as_ref().unwrap().value,
                    EL2008Port::DO2 => self.rxpdo.channel2.as_ref().unwrap().value,
                    EL2008Port::DO3 => self.rxpdo.channel3.as_ref().unwrap().value,
                    EL2008Port::DO4 => self.rxpdo.channel4.as_ref().unwrap().value,
                    EL2008Port::DO5 => self.rxpdo.channel5.as_ref().unwrap().value,
                    EL2008Port::DO6 => self.rxpdo.channel6.as_ref().unwrap().value,
                    EL2008Port::DO7 => self.rxpdo.channel7.as_ref().unwrap().value,
                    EL2008Port::DO8 => self.rxpdo.channel8.as_ref().unwrap().value,
                },
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum EL2008Port {
    DO1,
    DO2,
    DO3,
    DO4,
    DO5,
    DO6,
    DO7,
    DO8,
}

#[derive(Debug, Clone, RxPdo)]
pub struct EL2008RxPdo {
    #[pdo_object_index(0x1600)]
    pub channel1: Option<BoolPdoObject>,
    #[pdo_object_index(0x1601)]
    pub channel2: Option<BoolPdoObject>,
    #[pdo_object_index(0x1602)]
    pub channel3: Option<BoolPdoObject>,
    #[pdo_object_index(0x1603)]
    pub channel4: Option<BoolPdoObject>,
    #[pdo_object_index(0x1604)]
    pub channel5: Option<BoolPdoObject>,
    #[pdo_object_index(0x1605)]
    pub channel6: Option<BoolPdoObject>,
    #[pdo_object_index(0x1606)]
    pub channel7: Option<BoolPdoObject>,
    #[pdo_object_index(0x1607)]
    pub channel8: Option<BoolPdoObject>,
}

impl Default for EL2008RxPdo {
    fn default() -> Self {
        Self {
            channel1: Some(BoolPdoObject::default()),
            channel2: Some(BoolPdoObject::default()),
            channel3: Some(BoolPdoObject::default()),
            channel4: Some(BoolPdoObject::default()),
            channel5: Some(BoolPdoObject::default()),
            channel6: Some(BoolPdoObject::default()),
            channel7: Some(BoolPdoObject::default()),
            channel8: Some(BoolPdoObject::default()),
        }
    }
}

pub const EL2008_VENDOR_ID: u32 = 0x2;
pub const EL2008_PRODUCT_ID: u32 = 0x07d83052;
pub const EL2008_REVISION_A: u32 = 0x00110000;
pub const EL2008_IDENTITY_A: SubDeviceIdentityTuple =
    (EL2008_VENDOR_ID, EL2008_PRODUCT_ID, EL2008_REVISION_A);
