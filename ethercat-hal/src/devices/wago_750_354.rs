use super::{EthercatDeviceProcessing, NewEthercatDevice, SubDeviceIdentityTuple};
use ethercat_hal_derive::EthercatDevice;

/// WAGO_750_354 bus coupler
#[derive(Clone, EthercatDevice)]
pub struct WAGO_750_354 {
    is_used: bool,
}

impl EthercatDeviceProcessing for WAGO_750_354 {}

impl NewEthercatDevice for WAGO_750_354 {
    fn new() -> Self {
        Self { is_used: false }
    }
}

impl std::fmt::Debug for WAGO_750_354 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WAGO_750_354")
    }
}
pub const WAGO_750_354_VENDOR_ID: u32 = 0x21;
pub const WAGO_750_354_PRODUCT_ID: u32 = 0x7500354;
pub const WAGO_750_354_REVISION: u32 = 0x2;
pub const WAGO_750_354_IDENTITY: SubDeviceIdentityTuple = (
    WAGO_750_354_VENDOR_ID,
    WAGO_750_354_PRODUCT_ID,
    WAGO_750_354_REVISION,
);
