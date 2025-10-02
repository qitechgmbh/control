use super::{EthercatDeviceProcessing, NewEthercatDevice, SubDeviceIdentityTuple};
use crate::helpers::ethercrab_types::EthercrabSubDevicePreoperational;
use crate::io::digital_output::{DigitalOutputDevice, DigitalOutputOutput};
use crate::pdo::{RxPdo, basic::BoolPdoObject};
use ethercat_hal_derive::{EthercatDevice, RxPdo};
/// EL2004 4-channel digital output device
/// 24V DC, 0.5A per channel
#[derive(EthercatDevice)]
pub struct EL2004 {
    pub rxpdo: EL2004RxPdo,
    is_used: bool,
}

impl EthercatDeviceProcessing for EL2004 {}

impl std::fmt::Debug for EL2004 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EL2004")
    }
}

impl NewEthercatDevice for EL2004 {
    fn new() -> Self {
        Self {
            rxpdo: EL2004RxPdo::default(),
            is_used: false,
        }
    }
}

impl DigitalOutputDevice<EL2004Port> for EL2004 {
    fn set_output(&mut self, port: EL2004Port, value: DigitalOutputOutput) {
        let expect_text = "All channels should be Some(_)";
        match port {
            EL2004Port::DO1 => {
                self.rxpdo.channel1.as_mut().expect(expect_text).value = value.into()
            }
            EL2004Port::DO2 => {
                self.rxpdo.channel2.as_mut().expect(expect_text).value = value.into()
            }
            EL2004Port::DO3 => {
                self.rxpdo.channel3.as_mut().expect(expect_text).value = value.into()
            }
            EL2004Port::DO4 => {
                self.rxpdo.channel4.as_mut().expect(expect_text).value = value.into()
            }
        }
    }

    fn get_output(&self, port: EL2004Port) -> DigitalOutputOutput {
        let expect_text = "All channels should be Some(_)";
        DigitalOutputOutput(match port {
            EL2004Port::DO1 => self.rxpdo.channel1.as_ref().expect(expect_text).value,
            EL2004Port::DO2 => self.rxpdo.channel2.as_ref().expect(expect_text).value,
            EL2004Port::DO3 => self.rxpdo.channel3.as_ref().expect(expect_text).value,
            EL2004Port::DO4 => self.rxpdo.channel4.as_ref().expect(expect_text).value,
        })
    }
}

#[derive(Debug, Clone)]
pub enum EL2004Port {
    DO1,
    DO2,
    DO3,
    DO4,
}

#[derive(Debug, Clone, RxPdo)]
pub struct EL2004RxPdo {
    #[pdo_object_index(0x1600)]
    pub channel1: Option<BoolPdoObject>,
    #[pdo_object_index(0x1601)]
    pub channel2: Option<BoolPdoObject>,
    #[pdo_object_index(0x1602)]
    pub channel3: Option<BoolPdoObject>,
    #[pdo_object_index(0x1603)]
    pub channel4: Option<BoolPdoObject>,
}

impl Default for EL2004RxPdo {
    fn default() -> Self {
        Self {
            channel1: Some(BoolPdoObject::default()),
            channel2: Some(BoolPdoObject::default()),
            channel3: Some(BoolPdoObject::default()),
            channel4: Some(BoolPdoObject::default()),
        }
    }
}

pub const EL2004_VENDOR_ID: u32 = 0x2;
pub const EL2004_PRODUCT_ID: u32 = 131346514;
pub const EL2004_REVISION_A: u32 = 1179648;
pub const EL2004_IDENTITY_A: SubDeviceIdentityTuple =
    (EL2004_VENDOR_ID, EL2004_PRODUCT_ID, EL2004_REVISION_A);
