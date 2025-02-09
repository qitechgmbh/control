use crate::types::EthercrabSubDevice;

pub trait Configuration {
    async fn write_to(&self, device: &EthercrabSubDevice<'_>) -> Result<(), anyhow::Error>;
}

pub trait TxPDOAssignment {
    /// The input PDOs assignment
    ///
    /// Each u32 represents a PDO object or a PDO map.
    /// - Bit 31:24: Subindex
    /// - Bit 23:16: Length in Bits
    /// - Bit 15:0: Index
    /// `0x00_10_1A01` is a PDO mapping at index 0x1A01, subindex 0, with a length of 26 bits.
    ///
    /// The ethercat device actually saves the data as `[0x1A01, 0x0010]` (two u16 LE in reverse order)
    fn txpdo_mapping(&self) -> &[u16];

    /// The length of the PDOs in this configuration
    fn txpdo_len(&self) -> usize;

    /// The maximum length of the PDOs in any configuration
    fn txpdo_max_len() -> usize;
}

pub trait RxPDOAssignment {
    /// The output PDOs assignment
    ///
    /// Each u32 represents a PDO object or a PDO map.
    /// - Bit 31:24: Subindex
    /// - Bit 23:16: Length in Bits
    /// - Bit 15:0: Index
    /// `0x00_10_1601` is a PDO mapping at index 0x1601, subindex 0, with a length of 26 bits.
    ///
    /// The ethercat device actually saves the data as `[0x1601, 0x0010]` (two u16 LE in reverse order)
    fn rxpdo_mapping(&self) -> &[u16];

    /// The length of the PDOs in this configuration
    fn rxpdo_len(&self) -> usize;

    /// The maximum length of the PDOs in any configuration
    fn rxpdo_max_len() -> usize;
}

pub const TX_PDO_ASSIGNMENT_REG: u16 = 0x1C13;

pub const RX_PDO_ASSIGNMENT_REG: u16 = 0x1C12;
