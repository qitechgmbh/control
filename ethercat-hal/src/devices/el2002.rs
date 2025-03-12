use super::SubDeviceIdentityTuple;
use crate::io::digital_output::{DigitalOutputDevice, DigitalOutputOutput, DigitalOutputState};
use crate::pdo::basic::BoolPdoObject;
use crate::types::EthercrabSubDevicePreoperational;
use ethercat_hal_derive::{Device, RxPdo, TxPdo};

/// EL2002 8-channel digital output device
///
/// 24V DC, 0.5A per channel
#[derive(Device)]
pub struct EL2002 {
    pub output_ts: u64,
    rxpdu: EL2002RxPdu,
}

impl std::fmt::Debug for EL2002 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EL2002")
    }
}

impl EL2002 {
    pub fn new() -> Self {
        Self {
            output_ts: 0,
            rxpdu: EL2002RxPdu::default(),
        }
    }
}

impl DigitalOutputDevice<EL2002Port> for EL2002 {
    fn digital_output_write(&mut self, port: EL2002Port, value: bool) {
        match port {
            EL2002Port::DO1 => self.rxpdu.channel1.as_mut().unwrap().value = value,
            EL2002Port::DO2 => self.rxpdu.channel2.as_mut().unwrap().value = value,
        }
    }

    fn digital_output_state(&self, port: EL2002Port) -> DigitalOutputState {
        DigitalOutputState {
            output_ts: self.output_ts,
            output: DigitalOutputOutput {
                value: match port {
                    EL2002Port::DO1 => self.rxpdu.channel1.as_ref().unwrap().value,
                    EL2002Port::DO2 => self.rxpdu.channel2.as_ref().unwrap().value,
                },
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum EL2002Port {
    DO1,
    DO2,
}

#[derive(Debug, Clone, RxPdo, Default)]
struct EL2002RxPdu {
    #[pdo_object_index(0x1600)]
    pub channel1: Option<BoolPdoObject>,
    #[pdo_object_index(0x1601)]
    pub channel2: Option<BoolPdoObject>,
}

#[derive(Debug, Clone, TxPdo, Default)]
pub struct EL2002TxPdu {}

pub const EL2002_VENDOR_ID: u32 = 0x2;
pub const EL2002_PRODUCT_ID: u32 = 0x07d23052;
pub const EL2002_REVISION_A: u32 = 0x00110000;
pub const EL2002_IDENTITY_A: SubDeviceIdentityTuple =
    (EL2002_VENDOR_ID, EL2002_PRODUCT_ID, EL2002_REVISION_A);
