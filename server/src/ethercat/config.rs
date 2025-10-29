use ethercrab::PduStorage;

/// Maximum number of SubDevices that can be stored. This must be a power of 2 greater than 1.
pub const MAX_SUBDEVICES: usize = 16;
/// Maximum PDU data payload size - set this to the max PDI size or higher.
pub const PDI_LEN: usize = 32;
pub const MAX_PDU_DATA: usize = PduStorage::element_size(PDI_LEN * 16);
pub const MAX_FRAMES: usize = 16;
