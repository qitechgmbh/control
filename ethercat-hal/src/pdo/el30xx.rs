use bitvec::prelude::*;
use ethercat_hal_derive::PdoObject;

use super::{basic::Limit, TxPdoObject};

#[derive(Debug, Clone, Default, PdoObject, PartialEq)]
#[pdo_object(bits = 32)]
pub struct AiStandard {
    pub undervoltage: bool,
    pub overvoltage: bool,
    pub limit1: Limit,
    pub limit2: Limit,
    pub error: bool,
    pub txpdo_state: bool,
    pub txpdo_toggle: bool,
    pub value: u16,
}

impl TxPdoObject for AiStandard {
    fn read(&mut self, bits: &BitSlice<u8, Lsb0>) {
        // only read other values if txpdo_toggle is true
        if bits[8 + 7] {
            self.txpdo_toggle = true;
        } else {
            self.txpdo_toggle = false;
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

#[derive(Debug, Clone, Default, PdoObject, PartialEq)]
#[pdo_object(bits = 16)]
pub struct AiCompact {
    pub value: u16,
}

impl TxPdoObject for AiCompact {
    fn read(&mut self, bits: &BitSlice<u8, Lsb0>) {
        self.value = bits[0..16].load_le::<u16>();
    }
}
