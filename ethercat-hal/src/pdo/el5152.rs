use super::{RxPdoObject, TxPdoObject};
use bitvec::prelude::*;
use ethercat_hal_derive::PdoObject;

/// PDO Object for EL5152 encoder control (RxPDO)
/// Based on 1600/1602 mapping: Set counter (1 bit) + alignment + Set counter value (32/16 bit)
#[derive(Debug, Clone, Default, PdoObject, PartialEq, Eq)]
#[pdo_object(bits = 48)] // Total 48 bits for full mapping
pub struct El5152EncoderControl {
    /// Set counter - execute counter setting (1 bit) - 0x7000:03
    pub set_counter: bool,
    /// Set counter value (32-bit) - 0x7000:11
    pub set_counter_value: u32,
}

impl RxPdoObject for El5152EncoderControl {
    fn write(&self, bits: &mut BitSlice<u8, Lsb0>) {
        // bit 2: Set counter (0x7000:03)
        bits.set(2, self.set_counter);
        // Set counter value (bits 16-47, 32-bit value) - 0x7000:11
        bits[16..48].store_le(self.set_counter_value);
    }
}

/// PDO Object for EL5152 encoder status (TxPDO)
/// Based on 1A00/1A04 mapping: Status bits + Counter value (32-bit)
#[derive(Debug, Clone, Default, PdoObject, PartialEq, Eq)]
#[pdo_object(bits = 48)] // 16 bits status + 32 bits counter
pub struct El5152EncoderStatus {
    /// Set counter done - counter was set (1 bit) - 0x6000:03
    pub set_counter_done: bool,
    /// Extrapolation stall - extrapolated counter invalid (1 bit) - 0x6000:08
    pub extrapolation_stall: bool,
    /// Status of input A (1 bit) - 0x6000:09
    pub status_input_a: bool,
    /// Status of input B (1 bit) - 0x6000:0A
    pub status_input_b: bool,
    /// Sync Error - synchronization error occurred (1 bit) - 0x1C32:20
    pub sync_error: bool,
    /// TxPDO Toggle - toggled when TxPDO data is updated (1 bit) - 0x1800:09
    pub txpdo_toggle: bool,
    /// Counter value (32-bit) - 0x6000:11
    pub counter_value: u32,
}

impl TxPdoObject for El5152EncoderStatus {
    fn read(&mut self, bits: &BitSlice<u8, Lsb0>) {
        // bit 2: Set counter done (0x6000:03)
        self.set_counter_done = bits[2];
        // bit 7: Extrapolation stall (0x6000:08)
        self.extrapolation_stall = bits[7];
        // bit 8: Status of input A (0x6000:09)
        self.status_input_a = bits[8];
        // bit 9: Status of input B (0x6000:0A)
        self.status_input_b = bits[9];
        // bit 13: Sync error (0x1C32:20)
        self.sync_error = bits[13];
        // bit 15: TxPDO Toggle (0x1800:09)
        self.txpdo_toggle = bits[15];

        // Counter value (bits 16-47, 32-bit value) - 0x6000:11
        self.counter_value = bits[16..48].load_le::<u32>();
    }
}

/// PDO Object for EL5152 encoder frequency measurement (TxPDO)
/// Based on 1A03/1A07 mapping: 32-bit frequency value only
#[derive(Debug, Clone, Default, PdoObject, PartialEq, Eq)]
#[pdo_object(bits = 32)]
pub struct El5152EncoderFrequency {
    /// Frequency value (32-bit) - 0x6000:13 / 0x6010:13
    pub frequency_value: u32,
}

impl TxPdoObject for El5152EncoderFrequency {
    fn read(&mut self, bits: &BitSlice<u8, Lsb0>) {
        // Frequency value (bits 0-31) - 0x6000:13 / 0x6010:13
        self.frequency_value = bits[0..32].load_le::<u32>();
    }
}

/// PDO Object for EL5152 encoder period measurement (TxPDO)
/// Based on 1A02/1A06 mapping: 32-bit period value only
#[derive(Debug, Clone, Default, PdoObject, PartialEq, Eq)]
#[pdo_object(bits = 32)]
pub struct El5152EncoderPeriod {
    /// Period value (32-bit) - 0x6000:14 / 0x6010:14
    pub period_value: u32,
}

impl TxPdoObject for El5152EncoderPeriod {
    fn read(&mut self, bits: &BitSlice<u8, Lsb0>) {
        // Period value (bits 0-31) - 0x6000:14 / 0x6010:14
        self.period_value = bits[0..32].load_le::<u32>();
    }
}
