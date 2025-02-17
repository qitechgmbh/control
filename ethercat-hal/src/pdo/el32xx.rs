use bitvec::prelude::*;
use ethercat_hal_derive::PdoObject;

use super::TxPdoObject;

#[derive(Debug, Clone, Default, PdoObject, PartialEq)]
#[pdo_object(bits = 32)]
pub struct RtdInput {
    pub undervoltage: bool,
    pub overvoltage: bool,
    pub limit1: TemperatureInputLimit,
    pub limit2: TemperatureInputLimit,
    pub error: bool,
    pub txpdo_state: bool,
    pub txpdo_toggle: bool,
    pub temperature: f32,
}

impl TxPdoObject for RtdInput {
    fn read(&mut self, bits: &BitSlice<u8, Lsb0>) {
        self.temperature = (bits[16..16 + 16].load_le::<i16>() as f32) / 10.0;
        self.undervoltage = bits[0];
        self.overvoltage = bits[1];
        self.limit1 = bits[2..4].load_le::<u8>().into();
        self.limit2 = bits[4..6].load_le::<u8>().into();
        self.error = bits[7];
        self.txpdo_state = bits[8 + 6];
        self.txpdo_toggle = bits[8 + 7];
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum TemperatureInputLimit {
    NotActive,
    Greater,
    Smaller,
    Equal,
}

impl From<u8> for TemperatureInputLimit {
    fn from(value: u8) -> Self {
        match value {
            0b00 => TemperatureInputLimit::NotActive,
            0b01 => TemperatureInputLimit::Greater,
            0b10 => TemperatureInputLimit::Smaller,
            0b11 => TemperatureInputLimit::Equal,
            _ => unreachable!(),
        }
    }
}

impl Default for TemperatureInputLimit {
    fn default() -> Self {
        TemperatureInputLimit::NotActive
    }
}
