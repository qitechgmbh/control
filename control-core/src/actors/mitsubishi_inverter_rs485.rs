use super::Actor;
use common::modbus::modbus::{
    self, ModbusFunctionCode, ModbusRequest, ModbusResponse, SerialEncoding,
    calculate_modbus_rtu_timeout,
};
use ethercat_hal::io::serial_interface::SerialInterface;
use std::{
    pin::Pin,
    time::{Duration, Instant},
};

#[derive(Debug)]
pub enum Operation {
    Send,
    Receive,
    None,
}

#[derive(Debug)]
pub struct MitsubishiInverterRS485Actor {
    pub received_response: bool,
    pub init_done: bool,
    pub serial_interface: SerialInterface,
    pub last_ts: Instant,
    pub last_op: Operation,
}

impl MitsubishiInverterRS485Actor {
    pub fn new(received_response: bool, serial_interface: SerialInterface) -> Self {
        Self {
            received_response,
            serial_interface,
            init_done: false,
            last_ts: Instant::now(),
            last_op: Operation::None,
        }
    }
}

pub fn response_is_exception(response: ModbusResponse) -> bool {
    let code: u8 = response.function_code.into();
    return (code & 0b10000000) > 0; // 0x80 is set when an exception happens
}

pub fn response_function_code_is_exception(function_code: u8) -> bool {
    return (function_code & 0b10000000) > 0; // 0x80 is set when an exception happens
}

enum RequestType {
    OperationCommand,
    /// Monitoring, Operation (start,stop etc) command, frequency setting (RAM), less than 12 milliseconds timeout
    ReadWrite,
    /// Parameter Read/Write and Frequency (EEPROM), Less than 30 milliseconds timeout
    ParamClear,
    /// Less than 5 seconds timeout
    Reset,
}

impl RequestType {
    fn timeout_milliseconds(self) -> u64 {
        match self {
            RequestType::OperationCommand => 12,
            RequestType::ReadWrite => 30,
            RequestType::ParamClear => 5000,
            RequestType::Reset => 0,
        }
    }

    fn timeout_nanoseconds(self) -> u64 {
        self.timeout_milliseconds() * 1000000
    }
}

enum MitsubishiModbusExceptionCode {
    IllegalFunction,
    IllegalDataAddress,
    IllegalDataValue,
    None,
}

impl MitsubishiModbusExceptionCode {
    fn display(self) -> String {
        match self {
            MitsubishiModbusExceptionCode::IllegalFunction => "Illegal Function".to_string(),
            MitsubishiModbusExceptionCode::IllegalDataAddress => "Illegal Data Address".to_string(),
            MitsubishiModbusExceptionCode::IllegalDataValue => "Illegal Data Value".to_string(),
            MitsubishiModbusExceptionCode::None => "No Exceptions".to_string(),
        }
    }
}

impl From<MitsubishiModbusExceptionCode> for u8 {
    fn from(value: MitsubishiModbusExceptionCode) -> Self {
        match value {
            MitsubishiModbusExceptionCode::None => 0,
            MitsubishiModbusExceptionCode::IllegalFunction => 1,
            MitsubishiModbusExceptionCode::IllegalDataAddress => 2,
            MitsubishiModbusExceptionCode::IllegalDataValue => 3,
        }
    }
}

impl From<u8> for MitsubishiModbusExceptionCode {
    fn from(value: u8) -> Self {
        match value {
            1 => MitsubishiModbusExceptionCode::IllegalFunction,
            2 => MitsubishiModbusExceptionCode::IllegalDataAddress,
            3 => MitsubishiModbusExceptionCode::IllegalDataValue,
            _ => MitsubishiModbusExceptionCode::None,
        }
    }
}

impl Actor for MitsubishiInverterRS485Actor {
    fn act(&mut self, _now_ts: Instant) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let elapsed: Duration = self.last_ts.elapsed();

            // TODO: implement a message queue for the messages coming from machine api AND a response message queue maybe ?
            let start_motor: Vec<u8> = ModbusRequest {
                slave_id: 1,
                function_code: ModbusFunctionCode::PresetHoldingRegister,
                // This is the data for the Function, 0x0 0x8 is the address and 00 02 is the value, which in this case sets a bit to rotate the motor forwards
                data: vec![0x0, 0x8, 00, 02],
            }
            .into();

            // Maybe encapsulate ModbusRequest inside of MitsubishiRequest, since the timeouts and operations are specific to mitsubishi
            let request_timeout = RequestType::OperationCommand.timeout_nanoseconds();

            let timeout = calculate_modbus_rtu_timeout(
                SerialEncoding::Coding8E1.total_bits(),
                request_timeout,
                19200, // baudrate
                start_motor.len(),
            );

            if elapsed < timeout {
                return;
            }

            self.last_ts = _now_ts;

            if let Operation::Send = self.last_op {
                let res = (self.serial_interface.read_message)().await;

                if let Some(result) = res {
                    if result.len() > 2 {
                        let is_exception = response_function_code_is_exception(result[1]);
                        let exception_code = MitsubishiModbusExceptionCode::from(result[2]);
                        if is_exception {
                            println!("Mitsubishi Modbus Exception: {}", exception_code.display());
                        }
                    }
                }

                // self.last_op = Operation::Receive;
            }
            if self.init_done == false {
                println!("write");
                let _ = (self.serial_interface.write_message)(start_motor.clone()).await;
                self.last_op = Operation::Send;
                self.init_done = true;
            }
        })
    }
}
