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
    ReadHoldingRegister,
    PresetHoldingRegister,
    DiagnoseFunction,
}

pub struct ModbusRequest {
    pub slave_id: u8,
    pub function_code: ModbusFunctionCode,
    pub data: Vec<u8>,
}

pub struct ModbusResponse {
    pub slave_id: u8,
    pub function_code: u8, // needs to be u8 because of exceptions
    pub data: Vec<u8>,
}

impl From<ModbusFunctionCode> for u8 {
    fn from(value: ModbusFunctionCode) -> Self {
        match value {
            ModbusFunctionCode::ReadHoldingRegister => 0x03,
            ModbusFunctionCode::PresetHoldingRegister => 0x06, // Preset basically just means write
            ModbusFunctionCode::DiagnoseFunction => 0x08,
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
) -> u64 {
    let nanoseconds_per_bit: u64 = (1000000 / baudrate) as u64;
    let nanoseconds_per_byte: u64 = bits as u64 * nanoseconds_per_bit as u64;
    let transmission_timeout: u64 = nanoseconds_per_byte * message_size as u64;
    let silent_time: u64 = (nanoseconds_per_byte * (35)) / 10 as u64; // silent_time is 3.5x of character length,which is 11 bit for 8E1
    let mut full_timeout: u64 = transmission_timeout as u64;
    full_timeout += machine_operation_delay_nano;
    full_timeout += silent_time;
    return full_timeout as u64;
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
}
