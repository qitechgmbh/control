use super::{EthercatDeviceProcessing, NewEthercatDevice, SubDeviceIdentityTuple};
use ethercat_hal_derive::EthercatDevice;

/// EL9505 power supply terminal, 5 V DC output voltage.
/// No process data — present on the bus purely to supply 5 V to field devices.
#[derive(Clone, EthercatDevice)]
pub struct EL9505 {
    is_used: bool,
}

impl EthercatDeviceProcessing for EL9505 {}

impl NewEthercatDevice for EL9505 {
    fn new() -> Self {
        Self { is_used: false }
    }
}

impl std::fmt::Debug for EL9505 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EL9505")
    }
}

pub const EL9505_VENDOR_ID: u32 = 0x2;
pub const EL9505_PRODUCT_ID: u32 = 0x25213052;
pub const EL9505_REVISION_A: u32 = 0x00120000;
pub const EL9505_IDENTITY_A: SubDeviceIdentityTuple =
    (EL9505_VENDOR_ID, EL9505_PRODUCT_ID, EL9505_REVISION_A);
