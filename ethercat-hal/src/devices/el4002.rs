use crate::helpers::ethercrab_types::EthercrabSubDevicePreoperational;
use crate::pdo::RxPdo;
use crate::{
    io::analog_output::{AnalogOutputDevice, AnalogOutputOutput, AnalogOutputState},
    pdo::el40xx::AnalogOutput,
};
use ethercat_hal_derive::{EthercatDevice, RxPdo};

use super::{EthercatDeviceProcessing, NewEthercatDevice, SubDeviceIdentityTuple};

/// EL4002 2-channel analog output device
///
/// 0-10V / 0-20mA analog output
#[derive(EthercatDevice)]
pub struct EL4002 {
    pub rxpdo: EL4002RxPdo,
    is_used: bool,
}

impl EthercatDeviceProcessing for EL4002 {}

impl std::fmt::Debug for EL4002 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EL4002")
    }
}

impl NewEthercatDevice for EL4002 {
    fn new() -> Self {
        Self {
            rxpdo: EL4002RxPdo::default(),
            is_used: false,
        }
    }
}

impl AnalogOutputDevice<EL4002Port> for EL4002 {
    fn analog_output_state(&self, port: EL4002Port) -> AnalogOutputState {
        let expect_text = "All channels should be Some(_)";
        let channel = match port {
            EL4002Port::AO1 => self.rxpdo.channel1.as_ref().expect(&expect_text),
            EL4002Port::AO2 => self.rxpdo.channel2.as_ref().expect(&expect_text),
        };

        // Convert i16 value to f32 (0.0 to 1.0 range)
        // Assuming 0-32767 maps to 0.0-1.0
        let normalized_value = (channel.value as f32) / 32767.0;

        AnalogOutputState {
            output: AnalogOutputOutput(normalized_value),
        }
    }

    fn analog_output_write(&mut self, port: EL4002Port, value: AnalogOutputOutput) {
        let expect_text = "All channels should be Some(_)";
        let channel = match port {
            EL4002Port::AO1 => self.rxpdo.channel1.as_mut().expect(&expect_text),
            EL4002Port::AO2 => self.rxpdo.channel2.as_mut().expect(&expect_text),
        };

        // Convert f32 (0.0 to 1.0) to i16 (0 to 32767)
        // Clamp the value to ensure it's within valid range
        let clamped_value = value.0.clamp(0.0, 1.0);
        channel.value = (clamped_value * 32767.0) as i16;
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EL4002Port {
    AO1,
    AO2,
}

impl EL4002Port {
    pub fn to_byte_offset(&self) -> usize {
        match self {
            EL4002Port::AO1 => 0,
            EL4002Port::AO2 => 4,
        }
    }
}

#[derive(Debug, Clone, RxPdo)]
pub struct EL4002RxPdo {
    #[pdo_object_index(0x1600)]
    channel1: Option<AnalogOutput>,
    #[pdo_object_index(0x1601)]
    channel2: Option<AnalogOutput>,
}

impl Default for EL4002RxPdo {
    fn default() -> Self {
        Self {
            channel1: Some(AnalogOutput::default()),
            channel2: Some(AnalogOutput::default()),
        }
    }
}

pub const EL4002_VENDOR_ID: u32 = 0x2;
pub const EL4002_PRODUCT_ID: u32 = 0xfa23052;
pub const EL4002_REVISION_A: u32 = 0x160000;
pub const EL4002_REVISION_B: u32 = 0x150000;

pub const EL4002_IDENTITY_A: SubDeviceIdentityTuple =
    (EL4002_VENDOR_ID, EL4002_PRODUCT_ID, EL4002_REVISION_A);

pub const EL4002_IDENTITY_B: SubDeviceIdentityTuple =
    (EL4002_VENDOR_ID, EL4002_PRODUCT_ID, EL4002_REVISION_B);
