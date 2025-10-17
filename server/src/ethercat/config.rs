use ethercrab::PduStorage;

/// Maximum number of SubDevices that can be stored. This must be a power of 2 greater than 1.
pub const MAX_SUBDEVICES: usize = 16;
/// Maximum PDU data payload size - set this to the max PDI size or higher.
pub const MAX_PDU_DATA: usize = PduStorage::element_size(512);
/// Maximum number of EtherCAT frames that can be in flight at any one time.
pub const MAX_FRAMES: usize = 16;
/// Maximum total PDI length.
pub const PDI_LEN: usize = 512;
