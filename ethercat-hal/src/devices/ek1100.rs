use super::SubDeviceIdentityTuple;

pub const EK1100_VENDOR_ID: u32 = 0x2;
pub const EK1100_PRODUCT_ID: u32 = 0x044c2c52;
pub const EK1100_REVISION_A: u32 = 0x00120000;
pub const EK1100_IDENTITY_A: SubDeviceIdentityTuple =
    (EK1100_VENDOR_ID, EK1100_PRODUCT_ID, EK1100_REVISION_A);
