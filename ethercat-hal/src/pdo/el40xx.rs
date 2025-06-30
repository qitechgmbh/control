use bitvec::prelude::*;
use ethercat_hal_derive::PdoObject;

use super::RxPdoObject;

/// PDO Object for EL40xx (analog output) devices
///
/// The "Analog Output" holds the output value and status information.
#[derive(Debug, Clone, Default, PdoObject, PartialEq)]
#[pdo_object(bits = 32)]
pub struct AnalogOutput {
    /// Output value (0-32767 typically corresponds to 0-10V or 0-20mA)
    pub value: i16,

    /// Error flag indicating output issues
    pub error: bool,

    /// RxPDO state
    pub rxpdo_state: bool,

    /// If the PDO objects data has changed since the last write
    pub rxpdo_toggle: bool,
}

impl RxPdoObject for AnalogOutput {
    fn write(&self, bits: &mut BitSlice<u8, Lsb0>) {
        // Write the output value to bits 0-15
        bits[0..16].store_le(self.value);

        // Write status bits using set() method
        bits.set(16, self.error);
        bits.set(17, self.rxpdo_state);
        bits.set(18, self.rxpdo_toggle);

        // Clear remaining bits
        for i in 19..32 {
            bits.set(i, false);
        }
    }
}
