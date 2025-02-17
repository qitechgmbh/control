use bitvec::{order::Lsb0, slice::BitSlice};
use ethercat_hal_derive::PdoObject;

use super::{RxPdoObject, TxPdoObject};

#[derive(Debug, Clone, Default, PdoObject)]
#[pdo_object(bits = 1)]
pub struct BoolPdoObject {
    pub value: bool,
}

impl TxPdoObject for BoolPdoObject {
    fn read(&mut self, buffer: &BitSlice<u8, Lsb0>) {
        self.value = buffer[0].into();
    }
}

impl RxPdoObject for BoolPdoObject {
    fn write(&self, buffer: &mut BitSlice<u8, Lsb0>) {
        buffer.set(0, self.value);
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
}
