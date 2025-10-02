use crate::helpers::ethercrab_types::EthercrabSubDevicePreoperational;
use crate::io::digital_output::{DigitalOutputDevice, DigitalOutputOutput};
use crate::pdo::{RxPdo, basic::BoolPdoObject};
use ethercat_hal_derive::{EthercatDevice, RxPdo};

use super::{EthercatDeviceProcessing, NewEthercatDevice};

/// EL2634 4-channel relay device
///
/// 250V AC / 30V DC / 4A per channel
#[derive(EthercatDevice)]
pub struct EL2634 {
    pub rxpdo: EL2634RxPdo,
    is_used: bool,
}

impl EthercatDeviceProcessing for EL2634 {}

impl std::fmt::Debug for EL2634 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EL2634")
    }
}

impl NewEthercatDevice for EL2634 {
    fn new() -> Self {
        Self {
            rxpdo: EL2634RxPdo::default(),
            is_used: false,
        }
    }
}

impl DigitalOutputDevice<EL2634Port> for EL2634 {
    fn set_output(&mut self, port: EL2634Port, value: DigitalOutputOutput) {
        let expect_text = "All channels should be Some(_)";
        match port {
            EL2634Port::R1 => self.rxpdo.channel1.as_mut().expect(expect_text).value = value.into(),
            EL2634Port::R2 => self.rxpdo.channel2.as_mut().expect(expect_text).value = value.into(),
            EL2634Port::R3 => self.rxpdo.channel3.as_mut().expect(expect_text).value = value.into(),
            EL2634Port::R4 => self.rxpdo.channel4.as_mut().expect(expect_text).value = value.into(),
        }
    }

    fn get_output(&self, port: EL2634Port) -> DigitalOutputOutput {
        let expect_text = "All channels should be Some(_)";
        DigitalOutputOutput(match port {
            EL2634Port::R1 => self.rxpdo.channel1.as_ref().expect(expect_text).value,
            EL2634Port::R2 => self.rxpdo.channel2.as_ref().expect(expect_text).value,
            EL2634Port::R3 => self.rxpdo.channel3.as_ref().expect(expect_text).value,
            EL2634Port::R4 => self.rxpdo.channel4.as_ref().expect(expect_text).value,
        })
    }
}

#[derive(Debug, Clone)]
pub enum EL2634Port {
    R1,
    R2,
    R3,
    R4,
}

#[derive(Debug, Clone, RxPdo)]
pub struct EL2634RxPdo {
    #[pdo_object_index(0x1600)]
    pub channel1: Option<BoolPdoObject>,
    #[pdo_object_index(0x1601)]
    pub channel2: Option<BoolPdoObject>,
    #[pdo_object_index(0x1602)]
    pub channel3: Option<BoolPdoObject>,
    #[pdo_object_index(0x1603)]
    pub channel4: Option<BoolPdoObject>,
}

impl Default for EL2634RxPdo {
    fn default() -> Self {
        Self {
            channel1: Some(BoolPdoObject::default()),
            channel2: Some(BoolPdoObject::default()),
            channel3: Some(BoolPdoObject::default()),
            channel4: Some(BoolPdoObject::default()),
        }
    }
}
