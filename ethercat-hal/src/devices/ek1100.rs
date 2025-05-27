use super::{EthercatDeviceProcessing, NewEthercatDevice, SubDeviceIdentityTuple};
use ethercat_hal_derive::EthercatDevice;

/// EK1100 bus coupler
#[derive(Clone, EthercatDevice)]
pub struct EK1100 {
    is_used: bool,
}

impl EthercatDeviceProcessing for EK1100 {}

impl NewEthercatDevice for EK1100 {
    fn new() -> Self {
        Self { is_used: false }
    }
}

impl std::fmt::Debug for EK1100 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EK1100")
    }
}

pub const EK1100_VENDOR_ID: u32 = 0x2;
pub const EK1100_PRODUCT_ID: u32 = 0x044c2c52;
pub const EK1100_REVISION_A: u32 = 0x00120000;
pub const EK1100_IDENTITY_A: SubDeviceIdentityTuple =
    (EK1100_VENDOR_ID, EK1100_PRODUCT_ID, EK1100_REVISION_A);
