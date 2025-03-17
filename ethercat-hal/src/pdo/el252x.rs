use super::{RxPdoObject, TxPdoObject};
use bitvec::{field::BitField, order::Lsb0, slice::BitSlice};
use ethercat_hal_derive::PdoObject;

#[derive(Debug, Clone, Default, PdoObject, PartialEq)]
#[pdo_object(bits = 16)]
pub struct PtoStatus {
    pub select_end_counter: bool,
    pub ramp_active: bool,
    pub input_t: bool,
    pub input_z: bool,
    pub error: bool,
    pub sync_error: bool,
    pub txpdo_toggle: bool,
}

impl TxPdoObject for PtoStatus {
    fn read(&mut self, buffer: &BitSlice<u8, Lsb0>) {
        self.select_end_counter = buffer[0];
        self.ramp_active = buffer[1];
        self.input_t = buffer[4];
        self.input_z = buffer[5];
        self.error = buffer[6];

        self.sync_error = buffer[8 + 5];
        self.txpdo_toggle = buffer[8 + 7];
    }
}

#[derive(Debug, Clone, Default, PdoObject, PartialEq)]
#[pdo_object(bits = 48)]
pub struct EncStatus {
    pub set_counter_done: bool,
    pub counter_underflow: bool,
    pub counter_overflow: bool,
    pub sync_error: bool,
    pub txpdo_toggle: bool,
    pub counter_value: u32,
}

impl TxPdoObject for EncStatus {
    fn read(&mut self, bits: &BitSlice<u8, Lsb0>) {
        self.set_counter_done = bits[2];
        self.counter_underflow = bits[3];
        self.counter_overflow = bits[4];
        self.sync_error = bits[8 + 5];
        self.txpdo_toggle = bits[8 + 7];
        self.counter_value = bits[16..16 + 32].load_le();
    }
}

#[derive(Debug, Clone, Default, PdoObject)]
#[pdo_object(bits = 32)]
pub struct PtoControl {
    pub frequency_select: bool,
    pub disble_ramp: bool,
    pub go_counter: bool,
    pub frequency_value: i32,
}

impl RxPdoObject for PtoControl {
    fn write(&self, buffer: &mut BitSlice<u8, Lsb0>) {
        buffer.set(0, self.frequency_select);
        buffer.set(1, self.disble_ramp);
        buffer.set(2, self.go_counter);

        buffer[16..16 + 16].store_le(self.frequency_value);
    }
}

#[derive(Debug, Clone, Default, PdoObject)]
#[pdo_object(bits = 32)]
pub struct PtoTarget {
    pub target_counter_value: u32,
}

impl RxPdoObject for PtoTarget {
    fn write(&self, buffer: &mut BitSlice<u8, Lsb0>) {
        buffer[0..32].store_le(self.target_counter_value);
    }
}

#[derive(Debug, Clone, Default, PdoObject)]
#[pdo_object(bits = 48)]
pub struct EncControl {
    pub set_counter: bool,
    pub set_counter_value: u32,
}

impl RxPdoObject for EncControl {
    fn write(&self, buffer: &mut BitSlice<u8, Lsb0>) {
        buffer.set(2, self.set_counter);

        buffer[16..16 + 32].store_le(self.set_counter_value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitvec::prelude::*;

    #[test]
    fn test_pto_status() {
        // set all flags
        let buffer = vec![0b0111_0011u8, 0b1010_0000u8];
        let bits = buffer.view_bits::<Lsb0>();
        let mut pdo_status = PtoStatus::default();
        pdo_status.read(&bits);
        assert_eq!(
            pdo_status,
            PtoStatus {
                select_end_counter: true,
                ramp_active: true,
                input_t: true,
                input_z: true,
                error: true,
                sync_error: true,
                txpdo_toggle: true
            }
        )
    }

    #[test]
    fn test_enc_status() {
        // set all flags
        // set counter value to 0x12345678
        let buffer = vec![0b0001_1100u8, 0b1010_0000u8, 0x78u8, 0x56u8, 0x34u8, 0x12u8];
        let bits = buffer.view_bits::<Lsb0>();
        let mut enc_status = EncStatus::default();
        enc_status.read(&bits);

        assert_eq!(
            enc_status,
            EncStatus {
                set_counter_done: true,
                counter_underflow: true,
                counter_overflow: true,
                sync_error: true,
                txpdo_toggle: true,
                counter_value: 0x12345678
            }
        )
    }

    #[test]
    fn test_pto_control() {
        // set all flags
        // set frequency value to 0x1234
        let mut buffer = vec![0u8; 4];
        let mut bits = buffer.view_bits_mut::<Lsb0>();
        let pto_control = PtoControl {
            frequency_select: true,
            disble_ramp: true,
            go_counter: true,
            frequency_value: 0x1234,
        };
        pto_control.write(&mut bits);
        assert_eq!(buffer, vec![0b0000_0111u8, 0u8, 0x34u8, 0x12u8])
    }

    #[test]
    fn test_pto_target() {
        // set target counter value to 0x12345678
        let mut buffer = vec![0u8; 4];
        let mut bits = buffer.view_bits_mut::<Lsb0>();
        let pto_target = PtoTarget {
            target_counter_value: 0x12345678,
        };
        pto_target.write(&mut bits);
        assert_eq!(buffer, vec![0x78u8, 0x56u8, 0x34u8, 0x12u8])
    }

    #[test]
    fn test_enc_control() {
        // set all flags
        // set counter value to 0x12345678
        let mut buffer = vec![0u8; 6];
        let mut bits = buffer.view_bits_mut::<Lsb0>();
        let enc_control = EncControl {
            set_counter: true,
            set_counter_value: 0x12345678,
        };
        enc_control.write(&mut bits);
        assert_eq!(
            buffer,
            vec![0b0000_0100u8, 0u8, 0x78u8, 0x56u8, 0x34u8, 0x12u8]
        )
    }
}
