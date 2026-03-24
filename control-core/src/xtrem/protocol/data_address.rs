#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataAddress {
    SerialNumber    = 0x0000,
    DeviceId        = 0x0001,
    WeightValue     = 0x0101,
}