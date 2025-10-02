use super::RxPdoObject;
use bitvec::prelude::*;
use ethercat_hal_derive::PdoObject;

/// PDO Object for EL40xx (analog output) devices
///
/// The "Analog Output" holds the output value and status information.
#[derive(Debug, Clone, Default, PdoObject, PartialEq, Eq)]
#[pdo_object(bits = 16)]
pub struct AnalogOutput {
    /// Output value (-32768-32767 typically corresponds to -10V to +10V)
    pub value: i16,
}

impl RxPdoObject for AnalogOutput {
    fn write(&self, bits: &mut BitSlice<u8, Lsb0>) {
        // Write the output value to bits 0-15
        bits[0..16].store_le(self.value as u16);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analog_output_creation() {
        let analog_output = AnalogOutput::default();
        assert_eq!(analog_output.value, 0);
    }

    #[test]
    fn test_analog_output_with_value() {
        let analog_output = AnalogOutput { value: 12345 };
        assert_eq!(analog_output.value, 12345);
    }

    #[test]
    fn test_analog_output_negative_value() {
        let analog_output = AnalogOutput { value: -12345 };
        assert_eq!(analog_output.value, -12345);
    }

    #[test]
    fn test_analog_output_max_value() {
        let analog_output = AnalogOutput { value: i16::MAX };
        assert_eq!(analog_output.value, i16::MAX);
        assert_eq!(analog_output.value, 32767);
    }

    #[test]
    fn test_analog_output_min_value() {
        let analog_output = AnalogOutput { value: i16::MIN };
        assert_eq!(analog_output.value, i16::MIN);
        assert_eq!(analog_output.value, -32768);
    }

    #[test]
    fn test_analog_output_write_zero() {
        let analog_output = AnalogOutput { value: 0 };
        let mut buffer = [0u8; 2];

        {
            let bits = buffer.view_bits_mut::<Lsb0>();
            analog_output.write(&mut bits[0..16]);
        }

        // Now we can use buffer immutably
        assert_eq!(buffer, [0x00, 0x00]);

        // Create a new view to read back
        let bits = buffer.view_bits::<Lsb0>();
        let read_value: i16 = bits[0..16].load_le();
        assert_eq!(read_value, 0);
    }

    #[test]
    fn test_analog_output_write_positive_value() {
        let test_value = 0x1234i16; // 4660 in decimal
        let analog_output = AnalogOutput { value: test_value };
        let mut buffer = [0u8; 2];

        {
            let bits = buffer.view_bits_mut::<Lsb0>();
            analog_output.write(&mut bits[0..16]);
        }

        // Little endian: 0x1234 should be stored as [0x34, 0x12]
        assert_eq!(buffer, [0x34, 0x12]);

        // Verify the value can be read back
        let bits = buffer.view_bits::<Lsb0>();
        let read_value: i16 = bits[0..16].load_le();
        assert_eq!(read_value, test_value);
    }

    #[test]
    fn test_analog_output_write_negative_value() {
        let test_value = -1000i16;
        let analog_output = AnalogOutput { value: test_value };
        let mut buffer = [0u8; 2];

        {
            let bits = buffer.view_bits_mut::<Lsb0>();
            analog_output.write(&mut bits[0..16]);
        }

        // Verify the value can be read back correctly
        let bits = buffer.view_bits::<Lsb0>();
        let read_value: i16 = bits[0..16].load_le();
        assert_eq!(read_value, test_value);
    }

    #[test]
    fn test_analog_output_write_max_value() {
        let analog_output = AnalogOutput { value: i16::MAX };
        let mut buffer = [0u8; 2];

        {
            let bits = buffer.view_bits_mut::<Lsb0>();
            analog_output.write(&mut bits[0..16]);
        }

        // i16::MAX = 0x7FFF, little endian: [0xFF, 0x7F]
        assert_eq!(buffer, [0xFF, 0x7F]);

        let bits = buffer.view_bits::<Lsb0>();
        let read_value: i16 = bits[0..16].load_le();
        assert_eq!(read_value, i16::MAX);
    }

    #[test]
    fn test_analog_output_write_min_value() {
        let analog_output = AnalogOutput { value: i16::MIN };
        let mut buffer = [0u8; 2];

        {
            let bits = buffer.view_bits_mut::<Lsb0>();
            analog_output.write(&mut bits[0..16]);
        }

        // i16::MIN = 0x8000, little endian: [0x00, 0x80]
        assert_eq!(buffer, [0x00, 0x80]);

        let bits = buffer.view_bits::<Lsb0>();
        let read_value: i16 = bits[0..16].load_le();
        assert_eq!(read_value, i16::MIN);
    }

    #[test]
    fn test_analog_output_write_various_values() {
        let test_values = [
            0i16, 1, -1, 100, -100, 1000, -1000, 10000, -10000, 32767, -32768,
        ];

        for &test_value in &test_values {
            let analog_output = AnalogOutput { value: test_value };
            let mut buffer = [0u8; 2];

            {
                let bits = buffer.view_bits_mut::<Lsb0>();
                analog_output.write(&mut bits[0..16]);
            }

            // Verify round-trip conversion
            let bits = buffer.view_bits::<Lsb0>();
            let read_value: i16 = bits[0..16].load_le();
            assert_eq!(
                read_value, test_value,
                "Failed round-trip for value: {}",
                test_value
            );
        }
    }

    #[test]
    fn test_analog_output_bit_manipulation() {
        let analog_output = AnalogOutput {
            value: 0b1010101010101010u16 as i16,
        };
        let mut buffer = [0u8; 2];

        {
            let bits = buffer.view_bits_mut::<Lsb0>();
            analog_output.write(&mut bits[0..16]);
        }

        // Check individual bits
        let bits = buffer.view_bits::<Lsb0>();
        let read_value: i16 = bits[0..16].load_le();
        assert_eq!(read_value, 0b1010101010101010u16 as i16);

        // The pattern 0b1010101010101010 in little-endian bit order:
        // Bit 0 (LSB): 0, Bit 1: 1, Bit 2: 0, Bit 3: 1, etc.
        // So the pattern is: bit i should be 1 if (i % 2) == 1
        for i in 0..16 {
            let expected_bit = (i % 2) == 1; // Alternating pattern: bit 0=0, bit 1=1, bit 2=0, etc.
            assert_eq!(
                bits[i], expected_bit,
                "Bit {} should be {} (binary pattern: {:016b})",
                i, expected_bit, 0b1010101010101010u16
            );
        }
    }

    #[test]
    fn test_analog_output_partial_eq() {
        let output1 = AnalogOutput { value: 1000 };
        let output2 = AnalogOutput { value: 1000 };
        let output3 = AnalogOutput { value: 2000 };

        assert_eq!(output1, output2);
        assert_ne!(output1, output3);
        assert_ne!(output2, output3);
    }

    #[test]
    fn test_analog_output_clone() {
        let original = AnalogOutput { value: 12345 };
        let cloned = original.clone();

        assert_eq!(original, cloned);
        assert_eq!(original.value, cloned.value);
    }

    #[test]
    fn test_analog_output_debug() {
        let analog_output = AnalogOutput { value: 42 };
        let debug_string = format!("{:?}", analog_output);

        // Should contain the struct name and value
        assert!(debug_string.contains("AnalogOutput"));
        assert!(debug_string.contains("42"));
    }

    #[test]
    fn test_analog_output_voltage_range_simulation() {
        // Simulate typical voltage ranges for analog output

        // 0V to 10V range (0-32767 maps to 0V-10V)
        let zero_volts = AnalogOutput { value: 0 };
        let five_volts = AnalogOutput { value: 16383 }; // 5V
        let ten_volts = AnalogOutput { value: 32767 }; // 10V

        let mut buffer = [0u8; 2];

        // Test 0V
        {
            let bits = buffer.view_bits_mut::<Lsb0>();
            zero_volts.write(&mut bits[0..16]);
        }
        let bits = buffer.view_bits::<Lsb0>();
        let read_zero: i16 = bits[0..16].load_le();
        assert_eq!(read_zero, 0);

        // Test 5V
        {
            let bits = buffer.view_bits_mut::<Lsb0>();
            five_volts.write(&mut bits[0..16]);
        }
        let bits = buffer.view_bits::<Lsb0>();
        let read_five: i16 = bits[0..16].load_le();
        assert_eq!(read_five, 16383);

        // Test 10V
        {
            let bits = buffer.view_bits_mut::<Lsb0>();
            ten_volts.write(&mut bits[0..16]);
        }
        let bits = buffer.view_bits::<Lsb0>();
        let read_ten: i16 = bits[0..16].load_le();
        assert_eq!(read_ten, 32767);
    }

    #[test]
    fn test_analog_output_current_range_simulation() {
        // Simulate 4-20mA current output (typical industrial range)
        // Assuming -32768 to 32767 maps to 4mA to 20mA

        let four_ma = AnalogOutput { value: -32768 }; // 4mA (minimum)
        let twelve_ma = AnalogOutput { value: 0 }; // 12mA (middle)
        let twenty_ma = AnalogOutput { value: 32767 }; // 20mA (maximum)

        let mut buffer = [0u8; 2];

        // Test 4mA
        {
            let bits = buffer.view_bits_mut::<Lsb0>();
            four_ma.write(&mut bits[0..16]);
        }
        let bits = buffer.view_bits::<Lsb0>();
        let read_four: i16 = bits[0..16].load_le();
        assert_eq!(read_four, -32768);

        // Test 12mA
        {
            let bits = buffer.view_bits_mut::<Lsb0>();
            twelve_ma.write(&mut bits[0..16]);
        }
        let bits = buffer.view_bits::<Lsb0>();
        let read_twelve: i16 = bits[0..16].load_le();
        assert_eq!(read_twelve, 0);

        // Test 20mA
        {
            let bits = buffer.view_bits_mut::<Lsb0>();
            twenty_ma.write(&mut bits[0..16]);
        }
        let bits = buffer.view_bits::<Lsb0>();
        let read_twenty: i16 = bits[0..16].load_le();
        assert_eq!(read_twenty, 32767);
    }

    #[test]
    fn test_analog_output_write_to_larger_buffer() {
        let analog_output = AnalogOutput { value: 0x5A5A };
        let mut buffer = [0u8; 4]; // Larger buffer

        {
            let bits = buffer.view_bits_mut::<Lsb0>();
            analog_output.write(&mut bits[0..16]);
        }

        // Check that only first 16 bits are affected
        assert_eq!(buffer[0], 0x5A); // Low byte
        assert_eq!(buffer[1], 0x5A); // High byte
        assert_eq!(buffer[2], 0x00); // Unchanged
        assert_eq!(buffer[3], 0x00); // Unchanged
    }

    #[test]
    fn test_analog_output_endianness() {
        let test_value = 0x1234i16;
        let analog_output = AnalogOutput { value: test_value };
        let mut buffer = [0u8; 2];

        {
            let bits = buffer.view_bits_mut::<Lsb0>();
            analog_output.write(&mut bits[0..16]);
        }

        // Little endian: least significant byte first
        assert_eq!(buffer[0], 0x34); // LSB
        assert_eq!(buffer[1], 0x12); // MSB

        // Verify we can reconstruct the original value
        let reconstructed = u16::from_le_bytes(buffer) as i16;
        assert_eq!(reconstructed, test_value);
    }
}
