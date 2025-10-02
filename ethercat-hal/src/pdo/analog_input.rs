use bitvec::prelude::*;
use ethercat_hal_derive::PdoObject;

use super::{TxPdoObject, basic::Limit};

/// PDO Object for EL30xx devices
///
/// The value is accompanied by some metadata.
#[derive(Debug, Clone, Default, PdoObject, PartialEq, Eq)]
#[pdo_object(bits = 32)]
pub struct AiStandard {
    /// The signal voltage is over the defined operating range of the device
    pub undervoltage: bool,

    /// The signal volatge is under the defined operating range of the device
    pub overvoltage: bool,

    /// Configurable limit 1
    pub limit1: Limit,

    /// Configurable limit 2
    pub limit2: Limit,

    pub error: bool,

    pub txpdo_state: bool,

    /// If the PDO objects data has changed since the last read
    pub txpdo_toggle: bool,

    /// The 16bit analog value is unsigned here but devices could write signed value with different signing strategies.
    /// This depends on the configuration of the device and the value has to be converted to i16 with custom logic.
    pub value: u16,
}

impl TxPdoObject for AiStandard {
    fn read(&mut self, bits: &BitSlice<u8, Lsb0>) {
        // only read other values if txpdo_toggle is true
        self.txpdo_toggle = bits[8 + 7];
        if !self.txpdo_toggle {
            return;
        }

        self.undervoltage = bits[0];
        self.overvoltage = bits[1];
        self.limit1 = bits[2..4].load_le::<u8>().into();
        self.limit2 = bits[4..6].load_le::<u8>().into();
        self.error = bits[7];
        self.txpdo_state = bits[8 + 6];
        self.value = bits[16..16 + 16].load_le::<u16>();
    }
}

/// PDO Object for EL30xx devices
///
/// The value without metadata.
#[derive(Debug, Clone, Default, PdoObject, PartialEq, Eq)]
#[pdo_object(bits = 16)]
pub struct AiCompact {
    /// The 16bit analog value is unsigned here but devices could write signed value with different signing strategies.
    /// This depends on the configuration of the device and the value has to be converted to i16 with custom logic.
    pub value: u16,
}

impl TxPdoObject for AiCompact {
    fn read(&mut self, bits: &BitSlice<u8, Lsb0>) {
        self.value = bits[0..16].load_le::<u16>();
    }
}
