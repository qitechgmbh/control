use crate::io::digital_output::{DigitalOutputDevice, DigitalOutputOutput, DigitalOutputState};
use crate::pdo::{basic::BoolPdoObject, RxPdo};
use crate::types::EthercrabSubDevicePreoperational;
use ethercat_hal_derive::{Device, RxPdo};

use super::NewDevice;

/// EL2634 4-channel relay device
///
/// 250V AC / 30V DC / 4A per channel
#[derive(Device)]
pub struct EL2634 {
    pub output_ts: u64,
    pub rxpdo: EL2634RxPdo,
}

impl std::fmt::Debug for EL2634 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EL2634")
    }
}

impl NewDevice for EL2634 {
    fn new() -> Self {
        Self {
            output_ts: 0,
            rxpdo: EL2634RxPdo::default(),
        }
    }
}

impl DigitalOutputDevice<EL2634Port> for EL2634 {
    fn digital_output_write(&mut self, port: EL2634Port, value: DigitalOutputOutput) {
        match port {
            EL2634Port::R1 => self.rxpdo.channel1.as_mut().unwrap().value = value.into(),
            EL2634Port::R2 => self.rxpdo.channel2.as_mut().unwrap().value = value.into(),
            EL2634Port::R3 => self.rxpdo.channel3.as_mut().unwrap().value = value.into(),
            EL2634Port::R4 => self.rxpdo.channel4.as_mut().unwrap().value = value.into(),
        }
    }

    fn digital_output_state(&self, port: EL2634Port) -> DigitalOutputState {
        DigitalOutputState {
            output_ts: self.output_ts,
            output: DigitalOutputOutput(match port {
                EL2634Port::R1 => self.rxpdo.channel1.as_ref().unwrap().value,
                EL2634Port::R2 => self.rxpdo.channel2.as_ref().unwrap().value,
                EL2634Port::R3 => self.rxpdo.channel3.as_ref().unwrap().value,
                EL2634Port::R4 => self.rxpdo.channel4.as_ref().unwrap().value,
            }),
        }
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
