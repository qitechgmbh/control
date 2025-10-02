use super::{RxPdoObject, TxPdoObject};
use bitvec::{field::BitField, order::Lsb0, slice::BitSlice};
use ethercat_hal_derive::PdoObject;

/// PDO Object for EL252x devices
///
/// "PTO Status" contains status information about the pulse train output.
#[derive(Debug, Clone, Default, PdoObject, PartialEq, Eq)]
#[pdo_object(bits = 16)]
pub struct PtoStatus {
    pub select_end_counter: bool,

    /// Device is currenly ramping (accelerating/decelerating)
    pub ramp_active: bool,

    /// The input pin T is high
    pub input_t: bool,

    /// The input pin Z is high
    pub input_z: bool,

    pub error: bool,

    pub sync_error: bool,

    /// If the PDO objects data has changed since the last read
    pub txpdo_toggle: bool,
}

impl TxPdoObject for PtoStatus {
    fn read(&mut self, bits: &BitSlice<u8, Lsb0>) {
        // only read other values if txpdo_toggle is true
        self.txpdo_toggle = bits[8 + 7];
        if !self.txpdo_toggle {
            return;
        }

        self.select_end_counter = bits[0];
        self.ramp_active = bits[1];
        self.input_t = bits[4];
        self.input_z = bits[5];
        self.error = bits[6];

        self.sync_error = bits[8 + 5];
    }
}

/// PDO Object for EL252x devices
///
/// "Encoder Status" contains the encoder status information.
#[derive(Debug, Clone, Default, PdoObject, PartialEq, Eq)]
#[pdo_object(bits = 48)]
pub struct EncStatus {
    /// Acknowedges the set counter command of the last cycle
    pub set_counter_done: bool,

    /// If the real positon is less than the u32 position and the counter underflowed
    pub counter_underflow: bool,

    /// If the real positon is greater than the u32 position and the counter overflowed
    pub counter_overflow: bool,

    pub sync_error: bool,

    /// If the PDO objects data has changed since the last read
    pub txpdo_toggle: bool,

    /// The counted position/pulses by the encoder
    pub counter_value: u32,
}

impl TxPdoObject for EncStatus {
    fn read(&mut self, bits: &BitSlice<u8, Lsb0>) {
        // only read other values if txpdo_toggle is true
        self.txpdo_toggle = bits[8 + 7];
        if !self.txpdo_toggle {
            return;
        }

        self.set_counter_done = bits[2];
        self.counter_underflow = bits[3];
        self.counter_overflow = bits[4];
        self.sync_error = bits[8 + 5];
        self.counter_value = bits[16..16 + 32].load_le();
    }
}

/// PDO Object for EL252x devices
///
/// "PTO Control" is used to control the pulse train output.
#[derive(Debug, Clone, Default, PdoObject)]
#[pdo_object(bits = 32)]
pub struct PtoControl {
    pub frequency_select: bool,

    /// Disable ramping (acceleration/deceleration algorithm by the device)
    pub disble_ramp: bool,

    pub go_counter: bool,

    /// Pulse frequency value in Hz
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

/// PDO Object for EL252x devices
///
/// "PTO Target" is used to set the target position of the pulse train output.
#[derive(Debug, Clone, Default, PdoObject)]
#[pdo_object(bits = 32)]
pub struct PtoTarget {
    /// Target position in pulses
    ///
    /// Target of the [`EncStatus::counter_value`] field
    pub target_counter_value: u32,
}

impl RxPdoObject for PtoTarget {
    fn write(&self, buffer: &mut BitSlice<u8, Lsb0>) {
        buffer[0..32].store_le(self.target_counter_value);
    }
}

/// PDO Object for EL252x devices
///
/// "Encoder Control" is used to control the encoder.
#[derive(Debug, Clone, Default, PdoObject)]
#[pdo_object(bits = 48)]
pub struct EncControl {
    /// Set to `true` when wanting to override the encoder position
    pub set_counter: bool,
    /// Value to set the encoder to
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
