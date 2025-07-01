use bitvec::prelude::*;
use ethercat_hal_derive::PdoObject;

use super::RxPdoObject;

/// PDO Object for EL40xx (analog output) devices
///
/// The "Analog Output" holds the output value and status information.
#[derive(Debug, Clone, Default, PdoObject, PartialEq)]
#[pdo_object(bits = 16)]
pub struct AnalogOutput {
    /// Output value (-32768-32767 typically corresponds to -10 -10V or 0mA-20mA (from 4-20mA for EL402x))
    pub value: i16,
}

impl RxPdoObject for AnalogOutput {
    fn write(&self, bits: &mut BitSlice<u8, Lsb0>) {
        // Write the output value to bits 0-15
        bits[0..16].store_le(self.value);
    }
}
