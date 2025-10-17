use super::{EthercatDeviceProcessing, NewEthercatDevice, SubDeviceIdentityTuple};
use crate::helpers::ethercrab_types::EthercrabSubDevicePreoperational;
use crate::io::digital_output::{DigitalOutputDevice, DigitalOutputOutput};
use crate::pdo::{RxPdo, basic::BoolPdoObject};
use ethercat_hal_derive::{EthercatDevice, RxPdo};

/// WAGO_730_1506 8-channel digital output device
///
/// 24V DC, 0.5A per channel
#[derive(EthercatDevice)]
pub struct WAGO_730_1506 {
    pub rxpdo: WAGO_730_1506_RxPdo,
    is_used: bool,
}

impl EthercatDeviceProcessing for WAGO_730_1506 {}

impl std::fmt::Debug for WAGO_730_1506 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WAGO_730_1506")
    }
}

impl NewEthercatDevice for WAGO_730_1506 {
    fn new() -> Self {
        Self {
            rxpdo: WAGO_730_1506_RxPdo::default(),
            is_used: false,
        }
    }
}

impl DigitalOutputDevice<WAGO_730_1506_Port> for WAGO_730_1506 {
    fn set_output(&mut self, port: WAGO_730_1506_Port, value: DigitalOutputOutput) {
        let expect_text = "All channels should be Some(_)";
        match port {
            WAGO_730_1506_Port::DO1 => {
                self.rxpdo.channel1.as_mut().expect(expect_text).value = value.into()
            }
            WAGO_730_1506_Port::DO2 => {
                self.rxpdo.channel2.as_mut().expect(expect_text).value = value.into()
            }
            WAGO_730_1506_Port::DO3 => {
                self.rxpdo.channel3.as_mut().expect(expect_text).value = value.into()
            }
            WAGO_730_1506_Port::DO4 => {
                self.rxpdo.channel4.as_mut().expect(expect_text).value = value.into()
            }
            WAGO_730_1506_Port::DO5 => {
                self.rxpdo.channel5.as_mut().expect(expect_text).value = value.into()
            }
            WAGO_730_1506_Port::DO6 => {
                self.rxpdo.channel6.as_mut().expect(expect_text).value = value.into()
            }
            WAGO_730_1506_Port::DO7 => {
                self.rxpdo.channel7.as_mut().expect(expect_text).value = value.into()
            }
            WAGO_730_1506_Port::DO8 => {
                self.rxpdo.channel8.as_mut().expect(expect_text).value = value.into()
            }
        }
    }

    fn get_output(&self, port: WAGO_730_1506_Port) -> DigitalOutputOutput {
        let expect_text = "All channels should be Some(_)";
        DigitalOutputOutput(match port {
            WAGO_730_1506_Port::DO1 => self.rxpdo.channel1.as_ref().expect(expect_text).value,
            WAGO_730_1506_Port::DO2 => self.rxpdo.channel2.as_ref().expect(expect_text).value,
            WAGO_730_1506_Port::DO3 => self.rxpdo.channel3.as_ref().expect(expect_text).value,
            WAGO_730_1506_Port::DO4 => self.rxpdo.channel4.as_ref().expect(expect_text).value,
            WAGO_730_1506_Port::DO5 => self.rxpdo.channel5.as_ref().expect(expect_text).value,
            WAGO_730_1506_Port::DO6 => self.rxpdo.channel6.as_ref().expect(expect_text).value,
            WAGO_730_1506_Port::DO7 => self.rxpdo.channel7.as_ref().expect(expect_text).value,
            WAGO_730_1506_Port::DO8 => self.rxpdo.channel8.as_ref().expect(expect_text).value,
        })
    }
}

#[derive(Debug, Clone)]
pub enum WAGO_730_1506_Port {
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
pub struct WAGO_730_1506_RxPdo {
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

impl Default for WAGO_730_1506_RxPdo {
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

pub const WAGO_730_1506_VENDOR_ID: u32 = 0x21;
pub const WAGO_730_1506_PRODUCT_ID: u32 = 0x7500354;
pub const WAGO_730_1506_REVISION: u32 = 0x2;
pub const WAGO_730_1506_IDENTITY: SubDeviceIdentityTuple = (
    WAGO_730_1506_VENDOR_ID,
    WAGO_730_1506_PRODUCT_ID,
    WAGO_730_1506_REVISION,
);
