use bitvec::{field::BitField, order::Lsb0, slice::BitSlice};
use ethercat_hal_derive::PdoObject;

use super::{RxPdoObject, TxPdoObject};

/// PDO Object that is just a bool
///
/// Commonly sued in EL20xx devices
#[derive(Debug, Clone, Default, PdoObject)]
#[pdo_object(bits = 1)]
pub struct BoolPdoObject {
    pub value: bool,
}

impl TxPdoObject for BoolPdoObject {
    fn read(&mut self, buffer: &BitSlice<u8, Lsb0>) {
        self.value = buffer[0];
    }
}

impl RxPdoObject for BoolPdoObject {
    fn write(&self, buffer: &mut BitSlice<u8, Lsb0>) {
        buffer.set(0, self.value);
    }
}

/// PDO Object that is just a f32
///
/// Commonly used in EL30xx devices
#[derive(Debug, Clone, Default, PdoObject)]
#[pdo_object(bits = 32)]
pub struct F32PdoObject {
    pub value: f32,
}

impl TxPdoObject for F32PdoObject {
    fn read(&mut self, buffer: &BitSlice<u8, Lsb0>) {
        self.value = f32::from_bits(buffer[0..32].load_le::<u32>());
    }
}

impl RxPdoObject for F32PdoObject {
    fn write(&self, buffer: &mut BitSlice<u8, Lsb0>) {
        buffer[0..32].store_le(self.value.to_bits());
    }
}

/// Not a PDO object, but a commonly used value in PDO objects.
///
/// A u8 is used to map represent 4 states of a configured limit.
/// The type of limit depends on the device.
/// The threshhold value is configured via CoE and is commonly deactivated in the base configuration.
#[derive(Debug, Clone, PartialEq, Eq, Copy, Default)]
pub enum Limit {
    #[default]
    NotActive,
    Greater,
    Smaller,
    Equal,
}

impl From<u8> for Limit {
    fn from(value: u8) -> Self {
        match value {
            0b00 => Self::NotActive,
            0b01 => Self::Greater,
            0b10 => Self::Smaller,
            0b11 => Self::Equal,
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bool_pdo_object() {
        let mut buffer = [0u8];
        let mut object = BoolPdoObject::default();

        object.value = true;

        let mut bits = BitSlice::<_, Lsb0>::from_slice_mut(&mut buffer);
        object.write(&mut bits);

        assert_eq!(buffer[0], 1);
    }

    #[test]
    fn test_f32_pdo_object() {
        let mut buffer = [0u8; 4];
        let mut object = F32PdoObject::default();

        object.value = 3.14;

        let bits = BitSlice::<_, Lsb0>::from_slice_mut(&mut buffer);
        object.write(bits);

        assert_eq!(buffer, 3.14f32.to_le_bytes());
    }

    #[test]
    fn test_temperature_input_limit() {
        assert_eq!(Limit::from(0b00), Limit::NotActive);
        assert_eq!(Limit::from(0b01), Limit::Greater);
        assert_eq!(Limit::from(0b10), Limit::Smaller);
        assert_eq!(Limit::from(0b11), Limit::Equal);
    }
}
