use crate::coe::Configuration;

#[allow(non_snake_case)]
pub mod EL252X;

pub trait PdoObject {
    /// size in bits
    fn size(&self) -> usize;
}

pub trait TxPdoObject: PdoObject {
    fn read(&mut self, buffer: &[u8]);
}

pub trait RxPdoObject: PdoObject {
    fn write(&self, buffer: &mut [u8]);
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

    fn write(&self, buffer: &mut [u8]) {
        let mut bit_offset = 0;
        for object in self.get_objects() {
            if let Some(object) = object {
                let start_byte_index = bit_offset / 8;
                let end_byte_index = (bit_offset + object.size()) / 8;
                object.write(&mut buffer[start_byte_index..=end_byte_index]);
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

    fn read(&mut self, buffer: &[u8]) {
        let mut bit_offset = 0;
        for object in self.get_objects_mut().iter_mut() {
            if let Some(object) = object {
                let start_byte_index = bit_offset / 8;
                let end_byte_index = (bit_offset + object.size()) / 8;
                bit_offset += object.size();
                object.read(&buffer[start_byte_index..=end_byte_index]);
            }
        }
    }
}
