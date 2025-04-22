fn convert_nanoseconds_to_milliseconds(nanoseconds: u64) -> u64 {
    if nanoseconds == 0 {
        return 0;
    }
    return nanoseconds / 1000000;
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
