use bitvec::prelude::*;

use crate::coe::Configuration;

pub mod basic;
pub mod el252x;
pub mod el30xx;
pub mod el32xx;

pub trait PdoObject {
    /// size in bits
    fn size(&self) -> usize;
}

pub trait TxPdoObject: PdoObject {
    fn read(&mut self, buffer: &BitSlice<u8, Lsb0>);
}

pub trait RxPdoObject: PdoObject {
    fn write(&self, buffer: &mut BitSlice<u8, Lsb0>);
}

pub trait PdoPreset<TXPDOA, RXPDOA>
where
    TXPDOA: Clone,
    RXPDOA: Clone,
{
    fn txpdo_assignment(&self) -> TXPDOA;
    fn rxpdo_assignment(&self) -> RXPDOA;
}

pub trait RxPdo: Configuration {
    fn get_objects(&self) -> &[Option<&dyn RxPdoObject>];

    fn size(&self) -> usize {
        self.get_objects()
            .iter()
            .map(|o| o.as_ref().map(|o| o.size()).unwrap_or(0))
            .sum::<usize>()
    }

    fn write(&self, buffer: &mut BitSlice<u8, Lsb0>) {
        let mut bit_offset = 0;
        for object in self.get_objects() {
            if let Some(object) = object {
                let end_bit_index = bit_offset + object.size();
                object.write(&mut buffer[bit_offset..end_bit_index]);
                bit_offset += object.size();
            }
        }
    }
}

pub trait TxPdo: Configuration {
    fn get_objects(&self) -> &[Option<&dyn TxPdoObject>];

    fn get_objects_mut(&mut self) -> &mut [Option<&mut dyn TxPdoObject>];

    fn size(&self) -> usize {
        self.get_objects()
            .iter()
            .map(|o| o.as_ref().map(|o| o.size()).unwrap_or(0))
            .sum::<usize>()
    }

    fn read(&mut self, buffer: &BitSlice<u8, Lsb0>) {
        let mut bit_offset = 0;
        for object in self.get_objects_mut().iter_mut() {
            if let Some(object) = object {
                let end_bit_index = bit_offset + object.size();
                object.read(&buffer[bit_offset..end_bit_index]);
                bit_offset += object.size();
            }
        }
    }
}
