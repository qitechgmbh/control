use super::SubDeviceIdentityTuple;
use crate::io::digital_input::{DigitalInputDevice, DigitalInputInput, DigitalInputState};
use crate::pdo::{basic::BoolPdoObject, PredefinedPdoAssignment, TxPdo};
use crate::types::EthercrabSubDevicePreoperational;
use ethercat_hal_derive::{Device, TxPdo};

/// EL1008 8-channel digital input device
///
/// 24V DC, 3ms filter
#[derive(Clone, Device)]
pub struct EL1008 {
    pub input_ts: u64,
    pub txpdo: EL1008TxPdo,
}

impl std::fmt::Debug for EL1008 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EL1008")
    }
}

impl EL1008 {
    pub fn new() -> Self {
        Self {
            input_ts: 0,
            txpdo: EL1008TxPdo::default(),
        }
    }
}

impl DigitalInputDevice<EL1008Port> for EL1008 {
    fn digital_input_state(&self, port: EL1008Port) -> DigitalInputState {
        DigitalInputState {
            input_ts: self.input_ts,
            input: DigitalInputInput {
                value: match port {
                    EL1008Port::DI1 => self.txpdo.channel1.as_ref().unwrap().value,
                    EL1008Port::DI2 => self.txpdo.channel2.as_ref().unwrap().value,
                    EL1008Port::DI3 => self.txpdo.channel3.as_ref().unwrap().value,
                    EL1008Port::DI4 => self.txpdo.channel4.as_ref().unwrap().value,
                    EL1008Port::DI5 => self.txpdo.channel5.as_ref().unwrap().value,
                    EL1008Port::DI6 => self.txpdo.channel6.as_ref().unwrap().value,
                    EL1008Port::DI7 => self.txpdo.channel7.as_ref().unwrap().value,
                    EL1008Port::DI8 => self.txpdo.channel8.as_ref().unwrap().value,
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

#[derive(Debug, Clone, TxPdo)]
pub struct EL1008TxPdo {
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

impl Default for EL1008TxPdo {
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

#[derive(Debug, Clone)]
pub enum EL1008PdoPreset {
    All,
}

impl PredefinedPdoAssignment<EL1008TxPdo, ()> for EL1008PdoPreset {
    fn txpdo_assignment(&self) -> EL1008TxPdo {
        match self {
            EL1008PdoPreset::All => EL1008TxPdo {
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

    fn rxpdo_assignment(&self) -> () {
        unreachable!()
    }
}

pub const EL1008_VENDOR_ID: u32 = 0x2;
pub const EL1008_PRODUCT_ID: u32 = 0x03f03052;
pub const EL1008_REVISION_A: u32 = 0x00120000;
pub const EL1008_IDENTITY_A: SubDeviceIdentityTuple =
    (EL1008_VENDOR_ID, EL1008_PRODUCT_ID, EL1008_REVISION_A);
