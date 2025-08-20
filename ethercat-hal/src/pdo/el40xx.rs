use super::RxPdoObject;
use bitvec::prelude::*;
use ethercat_hal_derive::PdoObject;

/// PDO Object for EL40xx (analog output) devices
///
/// The "Analog Output" holds the output value and status information.
#[derive(Debug, Clone, Default, PdoObject, PartialEq, Eq)]
#[pdo_object(bits = 16)]
pub struct AnalogOutput {
    /// Output value (-32768-32767 typically corresponds to -10V to +10V)
    pub value: i16,
}

impl RxPdoObject for AnalogOutput {
    fn write(&self, bits: &mut BitSlice<u8, Lsb0>) {
        // Write the output value to bits 0-15
        bits[0..16].store_le(self.value as u16);
    }
}
