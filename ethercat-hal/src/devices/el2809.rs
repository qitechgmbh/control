use crate::helpers::ethercrab_types::EthercrabSubDevicePreoperational;
use crate::io::digital_output::{DigitalOutputDevice, DigitalOutputOutput};
use crate::pdo::{RxPdo, basic::BoolPdoObject};
use ethercat_hal_derive::{EthercatDevice, RxPdo};

use super::{EthercatDeviceProcessing, NewEthercatDevice};

/// EL2809 16-channel digital output device
///
/// 24V DC, 0.5A per channel
#[derive(EthercatDevice)]
pub struct EL2809 {
    pub rxpdo: EL2809RxPdo,
    is_used: bool,
}

impl EthercatDeviceProcessing for EL2809 {}

impl std::fmt::Debug for EL2809 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EL2809")
    }
}

impl NewEthercatDevice for EL2809 {
    fn new() -> Self {
        Self {
            rxpdo: EL2809RxPdo::default(),
            is_used: false,
        }
    }
}

impl DigitalOutputDevice<EL2809Port> for EL2809 {
    fn set_output(&mut self, port: EL2809Port, value: DigitalOutputOutput) {
        let expect_text = "All channels should be Some(_)";
        match port {
            EL2809Port::DO1 => {
                self.rxpdo.channel1.as_mut().expect(expect_text).value = value.into()
            }
            EL2809Port::DO2 => {
                self.rxpdo.channel2.as_mut().expect(expect_text).value = value.into()
            }
            EL2809Port::DO3 => {
                self.rxpdo.channel3.as_mut().expect(expect_text).value = value.into()
            }
            EL2809Port::DO4 => {
                self.rxpdo.channel4.as_mut().expect(expect_text).value = value.into()
            }
            EL2809Port::DO5 => {
                self.rxpdo.channel5.as_mut().expect(expect_text).value = value.into()
            }
            EL2809Port::DO6 => {
                self.rxpdo.channel6.as_mut().expect(expect_text).value = value.into()
            }
            EL2809Port::DO7 => {
                self.rxpdo.channel7.as_mut().expect(expect_text).value = value.into()
            }
            EL2809Port::DO8 => {
                self.rxpdo.channel8.as_mut().expect(expect_text).value = value.into()
            }
            EL2809Port::DO9 => {
                self.rxpdo.channel9.as_mut().expect(expect_text).value = value.into()
            }
            EL2809Port::DO10 => {
                self.rxpdo.channel10.as_mut().expect(expect_text).value = value.into()
            }
            EL2809Port::DO11 => {
                self.rxpdo.channel11.as_mut().expect(expect_text).value = value.into()
            }
            EL2809Port::DO12 => {
                self.rxpdo.channel12.as_mut().expect(expect_text).value = value.into()
            }
            EL2809Port::DO13 => {
                self.rxpdo.channel13.as_mut().expect(expect_text).value = value.into()
            }
            EL2809Port::DO14 => {
                self.rxpdo.channel14.as_mut().expect(expect_text).value = value.into()
            }
            EL2809Port::DO15 => {
                self.rxpdo.channel15.as_mut().expect(expect_text).value = value.into()
            }
            EL2809Port::DO16 => {
                self.rxpdo.channel16.as_mut().expect(expect_text).value = value.into()
            }
        }
    }

    fn get_output(&self, port: EL2809Port) -> DigitalOutputOutput {
        let expect_text = "All channels should be Some(_)";
        DigitalOutputOutput(match port {
            EL2809Port::DO1 => self.rxpdo.channel1.as_ref().expect(expect_text).value,
            EL2809Port::DO2 => self.rxpdo.channel2.as_ref().expect(expect_text).value,
            EL2809Port::DO3 => self.rxpdo.channel3.as_ref().expect(expect_text).value,
            EL2809Port::DO4 => self.rxpdo.channel4.as_ref().expect(expect_text).value,
            EL2809Port::DO5 => self.rxpdo.channel5.as_ref().expect(expect_text).value,
            EL2809Port::DO6 => self.rxpdo.channel6.as_ref().expect(expect_text).value,
            EL2809Port::DO7 => self.rxpdo.channel7.as_ref().expect(expect_text).value,
            EL2809Port::DO8 => self.rxpdo.channel8.as_ref().expect(expect_text).value,
            EL2809Port::DO9 => self.rxpdo.channel9.as_ref().expect(expect_text).value,
            EL2809Port::DO10 => self.rxpdo.channel10.as_ref().expect(expect_text).value,
            EL2809Port::DO11 => self.rxpdo.channel11.as_ref().expect(expect_text).value,
            EL2809Port::DO12 => self.rxpdo.channel12.as_ref().expect(expect_text).value,
            EL2809Port::DO13 => self.rxpdo.channel13.as_ref().expect(expect_text).value,
            EL2809Port::DO14 => self.rxpdo.channel14.as_ref().expect(expect_text).value,
            EL2809Port::DO15 => self.rxpdo.channel15.as_ref().expect(expect_text).value,
            EL2809Port::DO16 => self.rxpdo.channel16.as_ref().expect(expect_text).value,
        })
    }
}

#[derive(Debug, Clone)]
pub enum EL2809Port {
    DO1,
    DO2,
    DO3,
    DO4,
    DO5,
    DO6,
    DO7,
    DO8,
    DO9,
    DO10,
    DO11,
    DO12,
    DO13,
    DO14,
    DO15,
    DO16,
}

#[derive(Debug, Clone, RxPdo)]
pub struct EL2809RxPdo {
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
    #[pdo_object_index(0x1608)]
    pub channel9: Option<BoolPdoObject>,
    #[pdo_object_index(0x1609)]
    pub channel10: Option<BoolPdoObject>,
    #[pdo_object_index(0x160A)]
    pub channel11: Option<BoolPdoObject>,
    #[pdo_object_index(0x160B)]
    pub channel12: Option<BoolPdoObject>,
    #[pdo_object_index(0x160C)]
    pub channel13: Option<BoolPdoObject>,
    #[pdo_object_index(0x160D)]
    pub channel14: Option<BoolPdoObject>,
    #[pdo_object_index(0x160E)]
    pub channel15: Option<BoolPdoObject>,
    #[pdo_object_index(0x160F)]
    pub channel16: Option<BoolPdoObject>,
}

impl Default for EL2809RxPdo {
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
            channel9: Some(BoolPdoObject::default()),
            channel10: Some(BoolPdoObject::default()),
            channel11: Some(BoolPdoObject::default()),
            channel12: Some(BoolPdoObject::default()),
            channel13: Some(BoolPdoObject::default()),
            channel14: Some(BoolPdoObject::default()),
            channel15: Some(BoolPdoObject::default()),
            channel16: Some(BoolPdoObject::default()),
        }
    }
}
