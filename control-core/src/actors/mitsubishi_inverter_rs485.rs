use super::Actor;
use crate::modbus::{
    self, ModbusFunctionCode, ModbusRequest, ModbusResponse, calculate_modbus_rtu_timeout,
};
use ethercat_hal::io::serial_interface::SerialInterface;
use std::{
    collections::VecDeque,
    pin::Pin,
    time::{Duration, Instant},
};

#[derive(Debug)]
pub enum State {
    /// WaitingForResponse is set after sending a request through the serial_interface
    WaitingForResponse,
    /// ReadyToSend is set after receiving the response from the serial_interface
    ReadyToSend,
    Uninitialized,
}

/// Specifies all System environmet Variables
/// Register addresses are calculated as follows: Register-value 40002 -> address: 40002-40001 -> address:0x1
enum MitsubishiSystemRegister {
    InverterReset,                     // Register 40002
    ParameterClear,                    // Register 40003
    AllParameterClear,                 // Register 40004
    ParamClearNonCommunication,        // Register 40006
    AllParameterClearNonCommunication, // Register 40007
    InverterStatusAndControl,          // Register 40009
    OperationModeAndSetting,           // Register 40010
    RunningFrequencyRAM,               // Register 40014
    RunningFrequencyEEPROM,            // Register 40015
}

impl From<MitsubishiSystemRegister> for u16 {
    fn from(value: MitsubishiSystemRegister) -> Self {
        match value {
            MitsubishiSystemRegister::InverterReset => 0x1,
            MitsubishiSystemRegister::ParameterClear => 0x2,
            MitsubishiSystemRegister::AllParameterClear => 0x3,
            MitsubishiSystemRegister::ParamClearNonCommunication => 0x5,
            MitsubishiSystemRegister::AllParameterClearNonCommunication => 0x6,
            MitsubishiSystemRegister::InverterStatusAndControl => 0x8,
            MitsubishiSystemRegister::OperationModeAndSetting => 0x9,
            MitsubishiSystemRegister::RunningFrequencyRAM => 0x0d,
            MitsubishiSystemRegister::RunningFrequencyEEPROM => 0x0e,
        }
    }
}

impl MitsubishiControlRequests {
    fn get(&self) -> ModbusRequest {
        match self {
            MitsubishiControlRequests::ResetInverter => {
                let reg: u16 = MitsubishiSystemRegister::InverterReset.into();
                ModbusRequest {
                    slave_id: 1,
                    function_code: ModbusFunctionCode::PresetHoldingRegister,
                    data: vec![reg.to_be_bytes()[0], reg.to_be_bytes()[1], 0x00, 0x01], // Any Value
                }
            }
            MitsubishiControlRequests::ClearAllParameters => {
                let reg: u16 = MitsubishiSystemRegister::AllParameterClear.into();
                ModbusRequest {
                    slave_id: 1,
                    function_code: ModbusFunctionCode::PresetHoldingRegister,
                    data: vec![reg.to_be_bytes()[0], reg.to_be_bytes()[1], 0x96, 0x96], // Special value 0x9696
                }
            }
            MitsubishiControlRequests::ClearNonCommunicationParameter => {
                let reg: u16 = MitsubishiSystemRegister::ParamClearNonCommunication.into();
                ModbusRequest {
                    slave_id: 1,
                    function_code: ModbusFunctionCode::PresetHoldingRegister,
                    data: vec![reg.to_be_bytes()[0], reg.to_be_bytes()[1], 0x96, 0x96], // Special value 0x9696
                }
            }
            MitsubishiControlRequests::ClearNonCommunicationParameters => {
                let reg: u16 = MitsubishiSystemRegister::AllParameterClearNonCommunication.into();
                ModbusRequest {
                    slave_id: 1,
                    function_code: ModbusFunctionCode::PresetHoldingRegister,
                    data: vec![reg.to_be_bytes()[0], reg.to_be_bytes()[1], 0x96, 0x96], // Special value 0x9696
                }
            }
            MitsubishiControlRequests::ReadInverterStatus => {
                let reg: u16 = MitsubishiSystemRegister::InverterStatusAndControl.into();
                ModbusRequest {
                    slave_id: 1,
                    function_code: ModbusFunctionCode::ReadHoldingRegister,
                    data: vec![reg.to_be_bytes()[0], reg.to_be_bytes()[1], 0x00, 0x01], // Read 1 register
                }
            }
            MitsubishiControlRequests::StopMotor => {
                let reg: u16 = MitsubishiSystemRegister::InverterStatusAndControl.into();
                ModbusRequest {
                    slave_id: 1,
                    function_code: ModbusFunctionCode::PresetHoldingRegister,
                    data: vec![reg.to_be_bytes()[0], reg.to_be_bytes()[1], 0x00, 0x01], // Value 1 to stop
                }
            }
            MitsubishiControlRequests::StartForwardRotation => {
                let reg: u16 = MitsubishiSystemRegister::InverterStatusAndControl.into();
                ModbusRequest {
                    slave_id: 1,
                    function_code: ModbusFunctionCode::PresetHoldingRegister,
                    data: vec![reg.to_be_bytes()[0], reg.to_be_bytes()[1], 0, 0b00000010], // Value 2 for forward rotation
                }
            }
            MitsubishiControlRequests::StartReverseRotation => {
                let reg: u16 = MitsubishiSystemRegister::InverterStatusAndControl.into();
                ModbusRequest {
                    slave_id: 1,
                    function_code: ModbusFunctionCode::PresetHoldingRegister,
                    data: vec![reg.to_be_bytes()[0], reg.to_be_bytes()[1], 0, 0b00000100], // Value 4 for reverse rotation
                }
            }
            MitsubishiControlRequests::ReadRunningFrequency => {
                let reg: u16 = MitsubishiSystemRegister::RunningFrequencyRAM.into();
                ModbusRequest {
                    slave_id: 1,
                    function_code: ModbusFunctionCode::ReadHoldingRegister,
                    data: vec![reg.to_be_bytes()[0], reg.to_be_bytes()[1], 0x00, 0x01], // Read 1 register
                }
            }
            MitsubishiControlRequests::WriteRunningFrequency => todo!(),
        }
    }
}
/// These Requests Serve as Templates for controling the inverter
pub enum MitsubishiControlRequests {
    /// Register 40002, Reset/Restart the Inverter
    ResetInverter,
    /// Register 40004, Clear ALL parameters
    ClearAllParameters,
    /// Register 40006, Clear a non communication parameter
    ClearNonCommunicationParameter,
    /// Register 40007, Clear all Non Communication related Parameters
    ClearNonCommunicationParameters,
    /// Register 40009, Read Inverter Status
    ReadInverterStatus,
    /// Register 40009, Stops the Motor
    StopMotor,
    /// Register 40009, Starts the Motor in Forward Rotation
    StartForwardRotation,
    /// Register 40009, Starts the Motor in Reverse Rotation
    StartReverseRotation,
    /// Register 40014, Read the current frequency the motor runs at (RAM)
    ReadRunningFrequency,
    /// Register 40014, Write the frequency
    WriteRunningFrequency,
}

#[derive(Debug)]
pub struct MitsubishiInverterRS485Actor {
    pub serial_interface: SerialInterface,
    pub last_ts: Instant,
    pub last_message_size: usize,
    pub state: State,
    pub request_queue: VecDeque<ModbusRequest>,
    pub response_queue: VecDeque<ModbusResponse>,
}

impl MitsubishiInverterRS485Actor {
    pub fn new(serial_interface: SerialInterface) -> Self {
        Self {
            serial_interface,
            last_ts: Instant::now(),
            state: State::Uninitialized,
            request_queue: VecDeque::new(),
            response_queue: VecDeque::new(),
            last_message_size: 0,
        }
    }

    /// This would get called by the api to add a new request to the inverter
    pub fn add_request(&mut self, request: ModbusRequest) {
        self.request_queue.push_front(request);
    }

    /// This is used by the Api to pop off the Response of our Request
    /// Perhaps we need a transactionId to make sure that the response we got is the correct one ?
    pub fn get_response(&mut self) -> Option<ModbusResponse> {
        if self.response_queue.len() == 0 {
            return None;
        }
        self.response_queue.pop_back()
    }

    /// This is used internally to read the receive buffer of the el6021 or similiar
    fn read_modbus_response(&mut self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            if !(self.serial_interface.has_message)().await {
                return;
            }

            let res: Result<ModbusResponse, _> = (self.serial_interface.read_message)()
                .await
                .unwrap()
                .try_into();

            match res {
                Ok(result) => {
                    self.response_queue.push_front(result.clone());
                    self.last_message_size = result.clone().data.len() + 4;
                    self.state = State::ReadyToSend;
                }
                Err(_) => log::error!("Error Parsing ModbusResponse!"),
            };
        })
    }

    /// This is used internally to fill the write buffer of the el6021
    fn send_modbus_request(&mut self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            if self.request_queue.len() == 0 {
                return;
            }
            let request: Vec<u8> = self.request_queue.pop_back().unwrap().into();
            let _ = (self.serial_interface.write_message)(request.clone()).await;
            self.state = State::WaitingForResponse;
            self.last_message_size = request.len();
        })
    }
}

enum RequestType {
    /// Monitoring, Operation (start,stop etc) command, frequency setting (RAM), less than 12 milliseconds timeout for Response
    OperationCommand,
    /// Parameter Read/Write and Frequency (EEPROM), Less than 30 milliseconds timeout for Response
    ReadWrite,
    /// Less than 5 seconds timeout for Response
    ParamClear,
    /// no Timeout for Response
    Reset,
}

impl RequestType {
    fn timeout_duration(self) -> Duration {
        match self {
            RequestType::OperationCommand => Duration::from_millis(12),
            RequestType::ReadWrite => Duration::from_millis(30),
            RequestType::ParamClear => Duration::from_millis(5000),
            RequestType::Reset => Duration::from_millis(0),
        }
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
    fn act(&mut self, now_ts: Instant) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            if let State::Uninitialized = self.state {
                self.add_request(MitsubishiControlRequests::StartForwardRotation.get());
                self.add_request(MitsubishiControlRequests::StopMotor.get());
                self.state = State::ReadyToSend;
            }

            let elapsed: Duration = self.last_ts.duration_since(now_ts);
            let baudrate = (self.serial_interface.get_baudrate)().await.unwrap();
            let coding = (self.serial_interface.get_serial_encoding)().await.unwrap();
            let timeout = calculate_modbus_rtu_timeout(
                coding.total_bits(),
                RequestType::OperationCommand.timeout_duration(),
                baudrate,
                self.last_message_size,
            );

            if elapsed < timeout {
                return;
            }
            self.last_ts = now_ts;

            match self.state {
                State::WaitingForResponse => self.read_modbus_response().await,
                State::ReadyToSend => self.send_modbus_request().await,
                _ => (),
            }
        })
    }
}
