use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerialEncoding {
    Coding7E1, // 7 data, even parity, 1 stop
    Coding7O1, // 7 data, odd parity, 1 stop
    Coding7E2, // 7 data, even parity, 2 stop
    Coding7O2, // 7 data, odd parity, 2 stop
    Coding8N1, // 8 data, no parity, 1 stop
    Coding8E1, // 8 data, even parity, 1 stop
    Coding8O1, // 8 data, odd parity, 1 stop
    Coding8N2, // 8 data, no parity, 2 stop
    Coding8E2, // 8 data, even parity, 2 stop
    Coding8O2, // 8 data, odd parity, 2 stop
    Coding8S1, // 8 data, space parity, 1 stop
    Coding8M1, // 8 data, mark parity, 1 stop
}

impl SerialEncoding {
    /// Get the number of data bits
    pub fn data_bits(&self) -> u8 {
        match self {
            Self::Coding7E1 | Self::Coding7O1 | Self::Coding7E2 | Self::Coding7O2 => 7,
            _ => 8,
        }
    }

    /// Get the number of parity bits (0 or 1)
    pub fn parity_bits(&self) -> u8 {
        match self {
            Self::Coding8N1 | Self::Coding8N2 => 0,
            _ => 1,
        }
    }

    /// Get the parity type
    pub fn parity_type(&self) -> Option<ParityType> {
        match self {
            Self::Coding7E1 | Self::Coding7E2 | Self::Coding8E1 | Self::Coding8E2 => {
                Some(ParityType::Even)
            }
            Self::Coding7O1 | Self::Coding7O2 | Self::Coding8O1 | Self::Coding8O2 => {
                Some(ParityType::Odd)
            }
            Self::Coding8S1 => Some(ParityType::Space),
            Self::Coding8M1 => Some(ParityType::Mark),
            Self::Coding8N1 | Self::Coding8N2 => None,
        }
    }

    /// Get the number of stop bits
    pub fn stop_bits(&self) -> u8 {
        match self {
            Self::Coding7E1
            | Self::Coding7O1
            | Self::Coding8N1
            | Self::Coding8E1
            | Self::Coding8O1
            | Self::Coding8S1
            | Self::Coding8M1 => 1,
            Self::Coding7E2
            | Self::Coding7O2
            | Self::Coding8N2
            | Self::Coding8E2
            | Self::Coding8O2 => 2,
        }
    }

    /// Get the total number of bits sent per byte according to the SerialEncoding (including start bit)
    /// For Example: With 8n1 transferring 1 byte over Serial actually transfers 10 bits -> 8 data bits, 0 parity, 1 start bit and 1 stop bit
    pub fn total_bits(&self) -> u8 {
        // Start bit is always 1
        1 + self.data_bits() + self.parity_bits() + self.stop_bits()
    }

    /// Get a human-readable description
    pub fn description(&self) -> String {
        let data = self.data_bits();
        let parity = match self.parity_type() {
            Some(ParityType::Even) => "E",
            Some(ParityType::Odd) => "O",
            Some(ParityType::Space) => "S",
            Some(ParityType::Mark) => "M",
            None => "N",
        };
        let stop = self.stop_bits();

        format!(
            "{}{}{}({} bits total)",
            data,
            parity,
            stop,
            self.total_bits()
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParityType {
    Even,
    Odd,
    Space,
    Mark,
}

fn convert_nanoseconds_to_milliseconds(nanoseconds: u64) -> u128 {
    let duration: Duration = Duration::from_nanos(nanoseconds);
    return duration.as_millis();
}

fn convert_milliseconds_to_nanoseconds(milliseconds: u64) -> u128 {
    let duration: Duration = Duration::from_millis(milliseconds);
    return duration.as_nanos();
}

pub enum ModbusFunctionCode {
    ReadHoldingRegister,   // Read one or more Registers
    PresetHoldingRegister, // write one Register Value
    DiagnoseFunction,      // The response should echo back your request
    Unknown(u8),           //possibly an exception or a user defined Function
}

pub struct ModbusRequest {
    pub slave_id: u8,
    pub function_code: ModbusFunctionCode,
    pub data: Vec<u8>,
}
// could just make a ModbusFrame instead?
pub struct ModbusResponse {
    pub slave_id: u8,
    pub function_code: ModbusFunctionCode, // needs to be u8 because of exceptions
    pub data: Vec<u8>,
}

impl From<ModbusFunctionCode> for u8 {
    fn from(value: ModbusFunctionCode) -> Self {
        match value {
            ModbusFunctionCode::ReadHoldingRegister => 0x03,
            ModbusFunctionCode::PresetHoldingRegister => 0x06,
            ModbusFunctionCode::DiagnoseFunction => 0x08,
            ModbusFunctionCode::Unknown(value) => value,
        }
    }
}

impl From<ModbusRequest> for Vec<u8> {
    fn from(request: ModbusRequest) -> Self {
        let mut buffer = Vec::new();
        let crc16_modbus: crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_MODBUS);
        buffer.push(request.slave_id);
        buffer.push(request.function_code.into());
        buffer.extend_from_slice(&request.data);
        let length = buffer.len();
        let result = crc16_modbus.checksum(&buffer[..length]);
        let high_byte = (result >> 8) as u8; // upper 8 bits
        let low_byte = (result & 0xFF) as u8; // lower 8 bits
        buffer.push(low_byte);
        buffer.push(high_byte);
        return buffer;
    }
}

/// bits: amount of bits sent for a 8n1 coding: 8 data bits, 0 parity, 1 stop bit (1 start,1 stop) -> 10 bits
/// machine_operation_delay_nano: Delay for the given operation in nanoseconds as specified by the slaves datasheet (example: mitsubishi csfr84 has 12ms for read write in RAM)
/// baudrate: bits per second
/// message_size: size of original message in bytes
pub fn calculate_modbus_timeout(
    bits: u8,
    machine_operation_delay_nano: u64,
    baudrate: u32,
    message_size: u32,
) -> Duration {
    let nanoseconds_per_bit: u64 = (1000000 / baudrate) as u64;
    let nanoseconds_per_byte: u64 = bits as u64 * nanoseconds_per_bit as u64;
    let transmission_timeout: u64 = nanoseconds_per_byte * message_size as u64;
    let silent_time: u64 = (nanoseconds_per_byte * (35)) / 10 as u64; // silent_time is 3.5x of character length,which is 11 bit for 8E1
    let mut full_timeout: u64 = transmission_timeout as u64;
    full_timeout += machine_operation_delay_nano;
    full_timeout += silent_time;
    let full_timeout_duration = Duration::from_nanos(full_timeout);
    return full_timeout_duration;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modbus_request_to_vec() {
        // Create a ModbusRequest for reading holding registers
        let request = ModbusRequest {
            slave_id: 0x01,
            function_code: ModbusFunctionCode::ReadHoldingRegister,
            data: vec![0x03, 0xeb, 0x00, 0x01], // Starting address 0x03EB (1003), read 1 register
        };

        // Convert the request to Vec<u8>
        let result: Vec<u8> = request.into();

        // Expected result based on provided test data
        let expected = vec![
            0x01, // slave addr
            0x03, // function code (ReadHoldingRegister)
            0x03, // starting addr Reg H
            0xeb, // starting addr Reg L
            0x00, // No of Points H
            0x01, // No of Points L
            244,  // CRC low byte (0xF4)
            122,  // CRC high byte (0x7A)
        ];

        assert_eq!(
            result, expected,
            "ModbusRequest conversion failed. Expected: {:?}, Got: {:?}",
            expected, result
        );
    }

    #[test]
    fn test_basic_timeout_calculation() {
        let res = calculate_modbus_timeout(10, 0, 9600, 10);
        let nanoseconds = res.as_nanos();
        assert_eq!(nanoseconds, 14040);
    }

    #[test]
    fn test_machine_op_timeout_calculation() {
        let res = calculate_modbus_timeout(10, 1200000, 9600, 10);
        let nanoseconds = res.as_nanos();
        let machine_delay = 1200000;
        assert_eq!(nanoseconds, 14040 + machine_delay);
    }

    #[test]
    fn test_bits_timeout_calculation() {
        let result_11_bits = calculate_modbus_timeout(11, 0, 9600, 10);
        // nanoseconds_per_byte = 11 * 104 = 1144 ns
        // transmission_timeout = 1144 * 10 = 11440 ns
        // silent_time = (1144 * 35) / 10 = 4004 ns
        // full_timeout = 11440 + 0 + 4004 = 15444
        assert_eq!(result_11_bits.as_nanos(), 15444);

        let result_9_bits = calculate_modbus_timeout(9, 0, 9600, 10);
        // nanoseconds_per_byte = 9 * 104 = 936 ns
        // transmission_timeout = 936 * 10 = 9360 ns
        // silent_time = (936 * 35) / 10 = 3276 ns
        // full_timeout = 9360 + 0 + 3276 = 12636 ns
        assert_eq!(result_9_bits.as_nanos(), 12636);
    }

    #[test]
    fn test_edge_cases() {
        // Test with zero message size
        let result_zero_size = calculate_modbus_timeout(10, 0, 9600, 0);
        // transmission_timeout = 1040 * 0 = 0 ns
        // silent_time = 3640 ns
        // full_timeout = 0 + 0 + 3640 = 3640 ns
        assert_eq!(result_zero_size.as_nanos(), 3640);

        // Test with very high baudrate (edge case, not realistic)
        let result_high_baud = calculate_modbus_timeout(10, 0, 10_000_000, 10);
        // nanoseconds_per_bit = 1000000 / 10_000_000 = 0 ns (integer division truncates)
        // This will result in zero timeout which might not be realistic
        // The function should handle this gracefully
        assert_eq!(result_high_baud.as_nanos(), 0);

        // Test with very large message (edge case)
        let result_large_msg = calculate_modbus_timeout(10, 0, 9600, 1_000_000);
        // transmission_timeout = 1040 * 1_000_000 = 1,040,000,000 ns
        // silent_time = 3640 ns
        // full_timeout = 1,040,000,000 + 0 + 3640 = 1,040,003,640 ns
        assert_eq!(result_large_msg.as_nanos(), 1_040_003_640);
    }
}
