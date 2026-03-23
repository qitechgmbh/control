#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataAddress {
    Serial   = 0x0000,
    DeviceID = 0x0001,
    Weight   = 0x0101,
}

impl DataAddress {
    pub const fn as_hex(&self) -> u16 {
        match self {
            Self::Serial => 0x0000,
            Self::DeviceID => 0x0001,
            Self::Weight => 0x0101,
        }
    }
}