use super::SubDeviceIdentityTuple;
use crate::io::digital_input::{DigitalInputDevice, DigitalInputInput, DigitalInputState};
use crate::pdo::basic::BoolPdoObject;
use crate::pdo::PdoPreset;
use crate::types::EthercrabSubDevicePreoperational;
use ethercat_hal_derive::{Device, RxPdo, TxPdo};

/// EL1008 8-channel digital input device
///
/// 24V DC, 3ms filter
#[derive(Debug, Clone, Device)]
pub struct EL1008 {
    pub inputs_ts: u64,
    txpdu: EL1008TxPdu,
}

impl EL1008 {
    pub fn new() -> Self {
        Self {
            inputs_ts: 0,
            txpdu: EL1008TxPdu::default(),
        }
    }
}

impl DigitalInputDevice<EL1008Port> for EL1008 {
    fn digital_input_state(&self, port: EL1008Port) -> DigitalInputState {
        DigitalInputState {
            input_ts: self.inputs_ts,
            input: DigitalInputInput {
                value: match port {
                    EL1008Port::DI1 => self.txpdu.channel1.as_ref().unwrap().value,
                    EL1008Port::DI2 => self.txpdu.channel2.as_ref().unwrap().value,
                    EL1008Port::DI3 => self.txpdu.channel3.as_ref().unwrap().value,
                    EL1008Port::DI4 => self.txpdu.channel4.as_ref().unwrap().value,
                    EL1008Port::DI5 => self.txpdu.channel5.as_ref().unwrap().value,
                    EL1008Port::DI6 => self.txpdu.channel6.as_ref().unwrap().value,
                    EL1008Port::DI7 => self.txpdu.channel7.as_ref().unwrap().value,
                    EL1008Port::DI8 => self.txpdu.channel8.as_ref().unwrap().value,
                },
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EL1008Port {
    DI1,
    DI2,
    DI3,
    DI4,
    DI5,
    DI6,
    DI7,
    DI8,
}

impl EL1008Port {
    pub fn to_bit_index(&self) -> usize {
        match self {
            EL1008Port::DI1 => 0,
            EL1008Port::DI2 => 1,
            EL1008Port::DI3 => 2,
            EL1008Port::DI4 => 3,
            EL1008Port::DI5 => 4,
            EL1008Port::DI6 => 5,
            EL1008Port::DI7 => 6,
            EL1008Port::DI8 => 7,
        }
    }
}

#[derive(Debug, Clone, TxPdo, Default)]
struct EL1008TxPdu {
    #[pdo_object_index(0x1A00)]
    pub channel1: Option<BoolPdoObject>,
    #[pdo_object_index(0x1A01)]
    pub channel2: Option<BoolPdoObject>,
    #[pdo_object_index(0x1A02)]
    pub channel3: Option<BoolPdoObject>,
    #[pdo_object_index(0x1A03)]
    pub channel4: Option<BoolPdoObject>,
    #[pdo_object_index(0x1A04)]
    pub channel5: Option<BoolPdoObject>,
    #[pdo_object_index(0x1A05)]
    pub channel6: Option<BoolPdoObject>,
    #[pdo_object_index(0x1A06)]
    pub channel7: Option<BoolPdoObject>,
    #[pdo_object_index(0x1A07)]
    pub channel8: Option<BoolPdoObject>,
}

#[derive(Debug, Clone, RxPdo, Default)]
struct EL1008RxPdu {}

#[derive(Debug, Clone)]
pub enum EL1008PdoPreset {
    All,
}

impl PdoPreset<EL1008TxPdu, EL1008RxPdu> for EL1008PdoPreset {
    fn txpdo_assignment(&self) -> EL1008TxPdu {
        match self {
            EL1008PdoPreset::All => EL1008TxPdu {
                channel1: Some(BoolPdoObject::default()),
                channel2: Some(BoolPdoObject::default()),
                channel3: Some(BoolPdoObject::default()),
                channel4: Some(BoolPdoObject::default()),
                channel5: Some(BoolPdoObject::default()),
                channel6: Some(BoolPdoObject::default()),
                channel7: Some(BoolPdoObject::default()),
                channel8: Some(BoolPdoObject::default()),
            },
        }
    }

    fn rxpdo_assignment(&self) -> EL1008RxPdu {
        unreachable!()
    }
}

pub const EL1008_VENDOR_ID: u32 = 0x2;
pub const EL1008_PRODUCT_ID: u32 = 0x03f03052;
pub const EL1008_REVISION_A: u32 = 0x00120000;
pub const EL1008_IDENTITY_A: SubDeviceIdentityTuple =
    (EL1008_VENDOR_ID, EL1008_PRODUCT_ID, EL1008_REVISION_A);
