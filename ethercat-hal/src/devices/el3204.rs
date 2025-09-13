use crate::helpers::ethercrab_types::EthercrabSubDevicePreoperational;
use crate::pdo::TxPdo;
use crate::{
    io::temperature_input::{TemperatureInputDevice, TemperatureInputInput},
    pdo::el32xx::RtdInput,
};
use ethercat_hal_derive::{EthercatDevice, TxPdo};

use super::{EthercatDeviceProcessing, NewEthercatDevice, SubDeviceIdentityTuple};

/// EL3204 4-channel temperature input device
///
/// PT100 / Ni100 (RTD) / (2 wire)
#[derive(EthercatDevice)]
pub struct EL3204 {
    pub txpdo: EL3204TxPdo,
    is_used: bool,
}

impl EthercatDeviceProcessing for EL3204 {}

impl std::fmt::Debug for EL3204 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EL3204")
    }
}

impl NewEthercatDevice for EL3204 {
    fn new() -> Self {
        Self {
            txpdo: EL3204TxPdo::default(),
            is_used: false,
        }
    }
}

impl TemperatureInputDevice<EL3204Port> for EL3204 {
    fn get_input(&self, port: EL3204Port) -> TemperatureInputInput {
        let expect_text = "All channels should be Some(_)";
        let channel = match port {
            EL3204Port::T1 => self.txpdo.channel1.as_ref().expect(expect_text),
            EL3204Port::T2 => self.txpdo.channel2.as_ref().expect(expect_text),
            EL3204Port::T3 => self.txpdo.channel3.as_ref().expect(expect_text),
            EL3204Port::T4 => self.txpdo.channel4.as_ref().expect(expect_text),
        };
        TemperatureInputInput {
            temperature: channel.temperature,
            undervoltage: channel.undervoltage,
            overvoltage: channel.overvoltage,
            limit1: channel.limit1,
            limit2: channel.limit2,
            error: channel.error,
            txpdo_state: channel.txpdo_state,
            txpdo_toggle: channel.txpdo_toggle,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EL3204Port {
    T1,
    T2,
    T3,
    T4,
}

impl EL3204Port {
    pub const fn to_byte_offset(&self) -> usize {
        match self {
            Self::T1 => 0,
            Self::T2 => 4,
            Self::T3 => 8,
            Self::T4 => 12,
        }
    }
}

#[derive(Debug, Clone, TxPdo)]
pub struct EL3204TxPdo {
    #[pdo_object_index(0x1A00)]
    channel1: Option<RtdInput>,
    #[pdo_object_index(0x1A01)]
    channel2: Option<RtdInput>,
    #[pdo_object_index(0x1A02)]
    channel3: Option<RtdInput>,
    #[pdo_object_index(0x1A03)]
    channel4: Option<RtdInput>,
}

impl Default for EL3204TxPdo {
    fn default() -> Self {
        Self {
            channel1: Some(RtdInput::default()),
            channel2: Some(RtdInput::default()),
            channel3: Some(RtdInput::default()),
            channel4: Some(RtdInput::default()),
        }
    }
}

pub const EL3204_VENDOR_ID: u32 = 0x2;
pub const EL3204_PRODUCT_ID: u32 = 0xc843052;
pub const EL3204_REVISION_A: u32 = 0x160000;
pub const EL3204_REVISION_B: u32 = 0x150000;

pub const EL3204_IDENTITY_A: SubDeviceIdentityTuple =
    (EL3204_VENDOR_ID, EL3204_PRODUCT_ID, EL3204_REVISION_A);

pub const EL3204_IDENTITY_B: SubDeviceIdentityTuple =
    (EL3204_VENDOR_ID, EL3204_PRODUCT_ID, EL3204_REVISION_B);
