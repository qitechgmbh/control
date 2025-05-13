use super::Actor;
use crate::modbus::{
    ModbusFunctionCode, ModbusRequest, ModbusResponse, calculate_modbus_rtu_timeout,
};
use ethercat_hal::io::serial_interface::{SerialEncoding, SerialInterface};
use std::{
    collections::VecDeque,
    pin::Pin,
    time::{Duration, Instant},
};

#[derive(Debug)]
pub enum State {
    /// WaitingForResponse is set after sending a request through the serial_interface
    WaitingForResponse,
    /// After Sending a Resuest we need to wait atleast one ethercat cycle
    /// After one Cycle we check if el6021 status has transmit accepted toggled
    /// Then we can set state = ReadyToSend
    WaitingForRequestAccept,
    /// After Receiving a Response we need to wait atleast one ethercat cycle
    /// After one Cycle we check if el6021 status has received accepted toggled
    WaitingForReceiveAccept,
    /// ReadyToSend is set after receiving the response from the serial_interface
    ReadyToSend,
    /// Initial State
    Uninitialized,
}

/// Specifies all System environmet Variables
/// Register addresses are calculated as follows: Register-value 40002 -> address: 40002-40001 -> address:0x1
enum MitsubishiSystemRegister {
    /// Register 40002
    InverterReset,
    /// Register 40003
    ParameterClear,
    /// Register 40004
    AllParameterClear,
    /// Register 40006
    ParamClearNonCommunication,
    /// Register 40007
    AllParameterClearNonCommunication,
    /// Register 40009
    InverterStatusAndControl,
    /// Register 40010
    OperationModeAndSetting,
    /// Register 40014
    RunningFrequencyRAM,
    /// Register 40015
    RunningFrequencyEEPROM,
}

impl MitsubishiControlRequests {
    fn get_system_register(value: MitsubishiSystemRegister) -> u16 {
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

impl From<MitsubishiControlRequests> for ModbusRequest {
    fn from(request: MitsubishiControlRequests) -> Self {
        match request {
            MitsubishiControlRequests::WriteRunningFrequency => {
                let reg: u16 = MitsubishiControlRequests::get_system_register(
                    MitsubishiSystemRegister::RunningFrequencyRAM,
                );
                let reg_bytes = reg.to_be_bytes();
                ModbusRequest {
                    slave_id: 1,
                    function_code: ModbusFunctionCode::PresetHoldingRegister,
                    data: vec![reg_bytes[0], reg_bytes[1], 0x96, 0x96], // Special value 0x9696
                }
            }
            MitsubishiControlRequests::ReadInverterStatus => {
                let reg: u16 = MitsubishiControlRequests::get_system_register(
                    MitsubishiSystemRegister::InverterStatusAndControl,
                );
                let reg_bytes = reg.to_be_bytes();
                ModbusRequest {
                    slave_id: 1,
                    function_code: ModbusFunctionCode::ReadHoldingRegister,
                    data: vec![reg_bytes[0], reg_bytes[1], 0x00, 0x01], // Read 1 register
                }
            }
            MitsubishiControlRequests::StopMotor => {
                let reg: u16 = MitsubishiControlRequests::get_system_register(
                    MitsubishiSystemRegister::InverterStatusAndControl,
                );
                let reg_bytes = reg.to_be_bytes();
                ModbusRequest {
                    slave_id: 1,
                    function_code: ModbusFunctionCode::PresetHoldingRegister,
                    data: vec![reg_bytes[0], reg_bytes[1], 0x00, 0x01], // Value 1 to stop
                }
            }
            MitsubishiControlRequests::StartForwardRotation => {
                let reg: u16 = MitsubishiControlRequests::get_system_register(
                    MitsubishiSystemRegister::InverterStatusAndControl,
                );
                let reg_bytes = reg.to_be_bytes();
                ModbusRequest {
                    slave_id: 1,
                    function_code: ModbusFunctionCode::PresetHoldingRegister,
                    data: vec![reg_bytes[0], reg_bytes[1], 0, 0b00000010], // Value 2 for forward rotation
                }
            }
            MitsubishiControlRequests::StartReverseRotation => {
                let reg: u16 = MitsubishiControlRequests::get_system_register(
                    MitsubishiSystemRegister::InverterStatusAndControl,
                );
                let reg_bytes = reg.to_be_bytes();
                ModbusRequest {
                    slave_id: 1,
                    function_code: ModbusFunctionCode::PresetHoldingRegister,
                    data: vec![reg_bytes[0], reg_bytes[1], 0, 0b00000100], // Value 4 for reverse rotation
                }
            }
            MitsubishiControlRequests::ReadRunningFrequency => {
                let reg: u16 = MitsubishiControlRequests::get_system_register(
                    MitsubishiSystemRegister::RunningFrequencyRAM,
                );
                let reg_bytes = reg.to_be_bytes();
                ModbusRequest {
                    slave_id: 1,
                    function_code: ModbusFunctionCode::ReadHoldingRegister,
                    data: vec![reg_bytes[0], reg_bytes[1], 0x00, 0x01], // Read 1 register
                }
            }
            MitsubishiControlRequests::ResetInverter => todo!(),
            MitsubishiControlRequests::ClearAllParameters => todo!(),
            MitsubishiControlRequests::ClearNonCommunicationParameter => todo!(),
            MitsubishiControlRequests::ClearNonCommunicationParameters => todo!(),
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
pub enum RotationDirection {
    Forward,
    Backwards,
    Stopped,
}

#[derive(Debug)]
pub struct MitsubishiInverterRS485Actor {
    pub serial_interface: SerialInterface,
    pub last_ts: Instant,
    pub last_message_size: usize,
    pub state: State,

    pub baudrate: Option<u32>,
    pub encoding: Option<SerialEncoding>,

    pub request_queue: VecDeque<ModbusRequest>,
    pub response_queue: VecDeque<ModbusResponse>,

    pub forward_rotation: bool,
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
            baudrate: None,
            encoding: None,
            forward_rotation: true,
        }
    }

    /// This would get called by the api to add a new request to the inverter
    pub fn add_request(&mut self, request: ModbusRequest) {
        self.request_queue.push_front(request);
    }

    /// This is used by the Api to pop off the Response of our Request
    pub fn get_response(&mut self) -> Option<ModbusResponse> {
        if self.response_queue.len() == 0 {
            return None;
        }
        self.response_queue.pop_back()
    }

    /// This is used internally to read the receive buffer of the el6021
    fn read_modbus_response(&mut self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            if !(self.serial_interface.has_message)().await {
                //   return;
            }

            let res: Option<Vec<u8>> = (self.serial_interface.read_message)().await;
            let raw_response = match res {
                Some(res) => res,
                None => {
                    log::error!("ERROR: No Modbus Response");
                    return;
                }
            };
            let response: Result<ModbusResponse, _> = ModbusResponse::try_from(raw_response);

            match &response {
                Ok(result) => {
                    //self.response_queue.push_front(result.clone());
                    self.last_message_size = result.clone().data.len() + 4;
                    // wait one ethercat cycle
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
            let res = (self.serial_interface.write_message)(request.clone()).await;
            match res {
                Ok(_) => (),
                Err(_) => log::error!("ERROR: serial_interface.write_message has failed"),
            }
            self.state = State::WaitingForRequestAccept;
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
            let elapsed: Duration = now_ts.duration_since(self.last_ts);
            // State is uninitialized until serial interface init returns true, which takes a few cycles on the el6021
            if let State::Uninitialized = self.state {
                let res = (self.serial_interface.initialize)().await;
                if res == true {
                    self.state = State::ReadyToSend;
                    self.baudrate = (self.serial_interface.get_baudrate)().await;
                    self.encoding = (self.serial_interface.get_serial_encoding)().await;
                }
                return;
            }

            let encoding = match self.encoding {
                Some(encoding) => encoding,
                None => return,
            };

            let baudrate = match self.baudrate {
                Some(baudrate) => baudrate,
                None => return,
            };

            let timeout = calculate_modbus_rtu_timeout(
                encoding.total_bits(),
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
                // Wait one cycle
                State::WaitingForReceiveAccept => self.state = State::ReadyToSend,
                State::WaitingForRequestAccept => self.state = State::WaitingForResponse,
                _ => (),
            }
        })
    }
}
