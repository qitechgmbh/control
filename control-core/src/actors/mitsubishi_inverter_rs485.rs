// JUST TESTING STUFF HERE
use super::Actor;
use chrono::prelude::*;
use ethercat_hal::io::serial_interface::SerialInterface;
use std::{pin::Pin, thread, time::Duration};

#[derive(Debug)]
pub enum Operation {
    Send,
    Receive,
    None,
}

#[derive(Debug)]
pub struct MitsubishiInverterRS485Actor {
    pub received_response: bool,
    /// Message frames comprise the four message fields shown in the figures above.
    /// A slave recognizes message data as one message when a 3.5 character long no-data time (T1: start/end) is added before
    /// and after the data.
    pub last_bytes_sent: u16,
    pub bytes_waited: u16,
    pub init_done: bool,
    pub serial_interface: SerialInterface,
    pub last_ts: i64,
    pub last_op: Operation,
}

impl MitsubishiInverterRS485Actor {
    pub fn new(received_response: bool, serial_interface: SerialInterface) -> Self {
        Self {
            received_response,
            serial_interface,
            last_bytes_sent: 0,
            bytes_waited: 0,
            init_done: false,
            last_ts: 0,
            last_op: Operation::None,
        }
    }
}

fn convert_nanoseconds_to_milliseconds(nanoseconds: u64) -> u64 {
    if nanoseconds == 0 {
        return 0;
    }
    return nanoseconds / 1000000;
}

// For now this assumes 8E1 configuration -> 11 bits per byte
// 8N1 would mean 10 bits per byte
/*
Monitoring, operation command, frequency
setting (RAM)Less than 12 ms

Parameter read/write, frequency setting
(EEPROM)Less than 30 ms

Parameter clear / All parameter clearLess than 5 s

Reset commandNo reply
*/
enum RequestType {
    OperationCommand,
    ReadWrite,
    ParamClear,
    Reset,
}

enum ModbusFunctionCode {
    ReadHoldingRegister,
    PresetHoldingRegister,
    DiagnoseFunction,
}

impl From<ModbusFunctionCode> for u8 {
    fn from(value: ModbusFunctionCode) -> Self {
        match value {
            ModbusFunctionCode::ReadHoldingRegister => 0x03,
            ModbusFunctionCode::PresetHoldingRegister => 0x06,
            ModbusFunctionCode::DiagnoseFunction => 0x08,
        }
    }
}

enum ModbusExceptionCode {
    IllegalFunction,
    IllegalDataAddress,
    IllegalDataValue,
}

impl From<ModbusExceptionCode> for u8 {
    fn from(value: ModbusExceptionCode) -> Self {
        match value {
            ModbusExceptionCode::IllegalFunction => 1,
            ModbusExceptionCode::IllegalDataAddress => 2,
            ModbusExceptionCode::IllegalDataValue => 3,
        }
    }
}

struct ModbusRequest {
    pub slave_id: u8,
    pub function_code: ModbusFunctionCode,
    pub data: Vec<u8>,
}

struct ModbusResponse {
    pub slave_id: u8,
    pub function_code: u8, // needs to be u8 because of exceptions
    pub data: Vec<u8>,
}

fn response_is_exception(response: ModbusResponse) -> bool {
    return (response.function_code & 0b10000000) > 0; // 0x80 is set when an exception happens
}

fn response_functioncode_is_exception(function_code: u8) -> bool {
    return (function_code & 0b10000000) > 0; // 0x80 is set when an exception happens
}

impl From<ModbusRequest> for Vec<u8> {
    fn from(request: ModbusRequest) -> Self {
        let mut buffer = Vec::new();
        let X25: crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_MODBUS);

        // Add slave ID
        buffer.push(request.slave_id);

        // Add function code (assuming ModbusFunctionCode can be converted to u8)
        buffer.push(request.function_code.into());

        // Add data bytes
        buffer.extend_from_slice(&request.data);

        let length = buffer.len();

        let result = X25.checksum(&buffer[..length]);
        let high_byte = (result >> 8) as u8; // upper 8 bits
        let low_byte = (result & 0xFF) as u8; // lower 8 bits

        buffer.push(low_byte);
        buffer.push(high_byte);

        return buffer;
    }
}

fn calculate_modbus_timeout(request_type: RequestType, baudrate: u32, message_size: u32) -> u64 {
    let nanoseconds_per_bit = 1000000 / baudrate;
    let nanoseconds_per_byte = 11 * nanoseconds_per_bit;

    let transmission_timeout = nanoseconds_per_byte * message_size;
    let silent_time = (nanoseconds_per_byte * (35)) / 10; // silent_time is 3.5x of character length,which is 11 bit for 8E1
    let mut full_timeout: u64 = transmission_timeout as u64;

    match request_type {
        RequestType::OperationCommand => full_timeout += 12 * 1000000, //12ms delay extra
        RequestType::ReadWrite => full_timeout += 30 * 1000000,        //30ms delay extra
        RequestType::ParamClear => full_timeout += 5 * 1000 * 1000000, //5seconds delay
        RequestType::Reset => (),
    }

    return full_timeout + silent_time as u64;
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

impl Actor for MitsubishiInverterRS485Actor {
    fn act(&mut self, _now_ts: u64) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let now = Utc::now();
            if self.last_ts == 0 {
                self.last_ts = now.timestamp_nanos();
            }
            let diff: i64 = now.timestamp_nanos() - self.last_ts;
            let timeout = calculate_modbus_timeout(RequestType::ReadWrite, 19200, 8) as i64 * 3;
            if diff < timeout {
                return;
            }
            self.last_ts = now.timestamp_nanos();

            let mut test_data = vec![0; 8]; // Read Holding Register             
            test_data[0] = 0x01; // slave addr
            test_data[1] = 0x03; // function            

            test_data[2] = 0x03; // starting addr Reg H
            test_data[3] = 0xeb; // starting addr Reg L --> 41004 (Pr.4)

            test_data[4] = 0x0; // No of Points H                                               test_data[5] = 0x03; // No of Points L
            test_data[5] = 0x1; // No of Points H                                               test_data[5] = 0x03; // No of Points L

            test_data[6] = 244; // Crc Check L
            test_data[7] = 122; // Crc Check H

            let mut test_data1: Vec<u8> = ModbusRequest {
                slave_id: 1,
                function_code: ModbusFunctionCode::ReadHoldingRegister,
                data: vec![0x03, 0xeb, 0x0, 0x1],
            }
            .into();

            let mut reset_inverter: Vec<u8> = ModbusRequest {
                slave_id: 1,
                function_code: ModbusFunctionCode::PresetHoldingRegister,
                data: vec![0x00, 0x02, 0x0, 0x01],
            }
            .into();

            let mut set_motor_freq: Vec<u8> = ModbusRequest {
                slave_id: 1,
                function_code: ModbusFunctionCode::PresetHoldingRegister,
                data: vec![0x03, 0xeb, 0, 20],
            }
            .into();

            let mut read_motor_freq: Vec<u8> = ModbusRequest {
                slave_id: 1,
                function_code: ModbusFunctionCode::ReadHoldingRegister,
                data: vec![0x0, 0xd, 0, 1],
            }
            .into();

            let mut set_running_frequency: Vec<u8> = ModbusRequest {
                slave_id: 1,
                function_code: ModbusFunctionCode::PresetHoldingRegister,
                data: vec![0x0, 0xd, 0x00, 0x70],
            }
            .into();

            let mut read_status_control: Vec<u8> = ModbusRequest {
                slave_id: 1,
                function_code: ModbusFunctionCode::ReadHoldingRegister,
                data: vec![0x0, 0x8, 0, 1],
            }
            .into();

            let mut set_status_control: Vec<u8> = ModbusRequest {
                slave_id: 1,
                function_code: ModbusFunctionCode::PresetHoldingRegister,
                data: vec![0x0, 0x8, 00, 02],
            }
            .into();

            let mut diagnose_test: Vec<u8> = ModbusRequest {
                slave_id: 1,
                function_code: ModbusFunctionCode::DiagnoseFunction,
                data: vec![0x0, 0x0, 0x0, 0x0],
            }
            .into();

            if let Operation::Send = self.last_op {
                let res = (self.serial_interface.read_message)().await;

                println!("read result: {:x?}", res);

                if res.len() > 1 {
                    let isException = response_functioncode_is_exception(res[1]);
                    if isException {
                        println!(" is Exception {}", isException);
                    }
                }
                // self.last_op = Operation::Receive;
            }
            if self.init_done == false {
                println!("write");
                (self.serial_interface.write_message)(set_status_control.clone()).await; // keep sending to test   
                self.last_op = Operation::Send;
                self.init_done = true;
            }
        })
    }
}
