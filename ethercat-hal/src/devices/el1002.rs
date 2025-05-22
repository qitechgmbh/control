use super::{NewEthercatDevice, SubDeviceIdentityTuple};
use crate::devices::EthercatDeviceProcessing;
use crate::helpers::ethercrab_types::EthercrabSubDevicePreoperational;
use crate::io::digital_input::{DigitalInputDevice, DigitalInputInput, DigitalInputState};
use crate::pdo::{PredefinedPdoAssignment, TxPdo, basic::BoolPdoObject};
use ethercat_hal_derive::{EthercatDevice, TxPdo};
/// EL1002 8-channel digital input device
///
/// 24V DC, 3ms filter
#[derive(Clone, EthercatDevice)]
pub struct EL1002 {
    pub txpdo: EL1002TxPdo,
}

impl EthercatDeviceProcessing for EL1002 {}

impl std::fmt::Debug for EL1002 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EL1002")
    }
}

impl NewEthercatDevice for EL1002 {
    fn new() -> Self {
        Self {
            txpdo: EL1002TxPdo::default(),
        }
    }
}

impl DigitalInputDevice<EL1002Port> for EL1002 {
    fn digital_input_state(&self, port: EL1002Port) -> Result<DigitalInputState, anyhow::Error> {
        let error = anyhow::anyhow!(
            "[{}::Device::digital_input_state] Port {:?} is not available",
            module_path!(),
            port
        );
        Ok(DigitalInputState {
            input: DigitalInputInput {
                value: match port {
                    EL1002Port::DI1 => self.txpdo.channel1.as_ref().ok_or(error)?.value,
                    EL1002Port::DI2 => self.txpdo.channel2.as_ref().ok_or(error)?.value,
                },
            },
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EL1002Port {
    DI1,
    DI2,
}

impl EL1002Port {
    pub fn to_bit_index(&self) -> usize {
        match self {
            EL1002Port::DI1 => 0,
            EL1002Port::DI2 => 1,
        }
    }
}

#[derive(Debug, Clone, TxPdo)]
pub struct EL1002TxPdo {
    #[pdo_object_index(0x1A00)]
    pub channel1: Option<BoolPdoObject>,
    #[pdo_object_index(0x1A01)]
    pub channel2: Option<BoolPdoObject>,
}

impl Default for EL1002TxPdo {
    fn default() -> Self {
        Self {
            channel1: Some(BoolPdoObject::default()),
            channel2: Some(BoolPdoObject::default()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum EL1002PredefinedPdoAssignment {
    All,
}

impl PredefinedPdoAssignment<EL1002TxPdo, ()> for EL1002PredefinedPdoAssignment {
    fn txpdo_assignment(&self) -> EL1002TxPdo {
        match self {
            EL1002PredefinedPdoAssignment::All => EL1002TxPdo {
                channel1: Some(BoolPdoObject::default()),
                channel2: Some(BoolPdoObject::default()),
            },
        }
    }

    fn rxpdo_assignment(&self) -> () {
        unreachable!()
    }
}

pub const EL1002_VENDOR_ID: u32 = 0x2;
pub const EL1002_PRODUCT_ID: u32 = 65679442;
pub const EL1002_REVISION_A: u32 = 1179648;
pub const EL1002_IDENTITY_A: SubDeviceIdentityTuple =
    (EL1002_VENDOR_ID, EL1002_PRODUCT_ID, EL1002_REVISION_A);
