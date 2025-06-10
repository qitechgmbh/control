use super::Actor;
use crate::modbus::{
    ModbusFunctionCode, ModbusRequest, ModbusResponse, calculate_modbus_rtu_timeout,
};
use ethercat_hal::io::serial_interface::{SerialEncoding, SerialInterface};
use std::{
    collections::HashMap,
    pin::Pin,
    time::{Duration, Instant},
    u16,
};
use uom::{
    ConstZero,
    si::{f64::Frequency, frequency::centihertz},
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

#[derive(Debug)]
pub enum OperationMode {
    PU,
    EXT,
    NET,
}

impl MitsubishiInverterRS485Actor {
    pub fn convert_hz_float_to_word(&mut self, value: Frequency) -> u16 {
        let scaled = value.get::<centihertz>(); // Convert Hz to 0.01 Hz units
        scaled.round() as u16
    }

    pub fn set_frequency_target(&mut self, frequency: Frequency) {
        let mut request: MitsubishiModbusRequest =
            MitsubishiControlRequests::WriteRunningFrequency.into();
        let result: u16 = self.convert_hz_float_to_word(frequency); // convert hz float to short
        request.request.data[2] = result.to_le_bytes()[1];
        request.request.data[3] = result.to_le_bytes()[0];
        self.add_request(request);
    }

    pub fn read_running_frequency() {
        // Check if the current element pushed to front is freq event
    }

    pub fn switch_operation_mode() {}
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
    /// Register 40201
    MotorFrequency,
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
            MitsubishiSystemRegister::MotorFrequency => 0x00C8,
        }
    }
}

/*
    MitsubishiModbusRequest get executed by their priority
    Start and StopMotor are highest priority while writeRunningFrequency and readMotorFrequency are one lower
    lets say we had StartMotor and readMotorFrequency the order of execution is:
    1. StartMotor
    2. readMotorFrequency

    this is because StartMotor is higher priority
    Since the events do not need to be pushed into a queue this makes the inverter operation more stable
*/
impl From<MitsubishiControlRequests> for MitsubishiModbusRequest {
    fn from(request: MitsubishiControlRequests) -> Self {
        match request {
            MitsubishiControlRequests::WriteRunningFrequency => {
                let reg: u16 = MitsubishiControlRequests::get_system_register(
                    MitsubishiSystemRegister::RunningFrequencyRAM,
                );
                let reg_bytes = reg.to_be_bytes();
                MitsubishiModbusRequest {
                    request: ModbusRequest {
                        slave_id: 1,
                        function_code: ModbusFunctionCode::PresetHoldingRegister,
                        data: vec![reg_bytes[0], reg_bytes[1], 0x0, 0x0],
                    },
                    request_type: RequestType::OperationCommand,
                    expected_response_type: ResponseType::WriteFrequency,
                    priority: u16::MAX - 1,
                    control_request_type: MitsubishiControlRequests::WriteRunningFrequency,
                }
            }
            MitsubishiControlRequests::ReadInverterStatus => {
                let reg: u16 = MitsubishiControlRequests::get_system_register(
                    MitsubishiSystemRegister::InverterStatusAndControl,
                );
                let reg_bytes = reg.to_be_bytes();
                MitsubishiModbusRequest {
                    request: ModbusRequest {
                        slave_id: 1,
                        function_code: ModbusFunctionCode::ReadHoldingRegister,
                        data: vec![reg_bytes[0], reg_bytes[1], 0x00, 0x01], // Read 1 register
                    },
                    request_type: RequestType::OperationCommand,
                    expected_response_type: ResponseType::InverterStatus,
                    priority: u16::MAX - 3,
                    control_request_type: MitsubishiControlRequests::ReadInverterStatus,
                }
            }
            MitsubishiControlRequests::StopMotor => {
                let reg: u16 = MitsubishiControlRequests::get_system_register(
                    MitsubishiSystemRegister::InverterStatusAndControl,
                );
                let reg_bytes = reg.to_be_bytes();
                MitsubishiModbusRequest {
                    request: ModbusRequest {
                        slave_id: 1,
                        function_code: ModbusFunctionCode::PresetHoldingRegister,
                        data: vec![reg_bytes[0], reg_bytes[1], 0x00, 0x01], // Value 1 to stop
                    },
                    request_type: RequestType::OperationCommand,
                    expected_response_type: ResponseType::InverterControl,
                    priority: u16::MAX, // StopMotor should have highest priority
                    control_request_type: MitsubishiControlRequests::StopMotor,
                }
            }
            MitsubishiControlRequests::StartForwardRotation => {
                let reg: u16 = MitsubishiControlRequests::get_system_register(
                    MitsubishiSystemRegister::InverterStatusAndControl,
                );
                let reg_bytes = reg.to_be_bytes();
                MitsubishiModbusRequest {
                    request: ModbusRequest {
                        slave_id: 1,
                        function_code: ModbusFunctionCode::PresetHoldingRegister,
                        data: vec![reg_bytes[0], reg_bytes[1], 0, 0b00000010], // Value 2 for forward rotation
                    },
                    request_type: RequestType::OperationCommand,
                    expected_response_type: ResponseType::InverterControl,
                    priority: u16::MAX - 1,
                    control_request_type: MitsubishiControlRequests::StartForwardRotation,
                }
            }
            MitsubishiControlRequests::StartReverseRotation => {
                let reg: u16 = MitsubishiControlRequests::get_system_register(
                    MitsubishiSystemRegister::InverterStatusAndControl,
                );
                let reg_bytes = reg.to_be_bytes();
                MitsubishiModbusRequest {
                    request: ModbusRequest {
                        slave_id: 1,
                        function_code: ModbusFunctionCode::PresetHoldingRegister,
                        data: vec![reg_bytes[0], reg_bytes[1], 0, 0b00000100], // Value 4 for reverse rotation
                    },
                    request_type: RequestType::OperationCommand,
                    expected_response_type: ResponseType::InverterControl,
                    priority: u16::MAX - 1,
                    control_request_type: MitsubishiControlRequests::StartReverseRotation,
                }
            }
            MitsubishiControlRequests::ReadRunningFrequency => {
                let reg: u16 = MitsubishiControlRequests::get_system_register(
                    MitsubishiSystemRegister::RunningFrequencyRAM,
                );
                let reg_bytes = reg.to_be_bytes();
                MitsubishiModbusRequest {
                    request: ModbusRequest {
                        slave_id: 1,
                        function_code: ModbusFunctionCode::ReadHoldingRegister,
                        data: vec![reg_bytes[0], reg_bytes[1], 0x00, 0x01], // Read 1 register
                    },
                    request_type: RequestType::OperationCommand,
                    expected_response_type: ResponseType::ReadFrequency,
                    priority: u16::MAX - 4,
                    control_request_type: MitsubishiControlRequests::ReadRunningFrequency,
                }
            }
            MitsubishiControlRequests::ReadMotorFrequency => {
                let reg: u16 = MitsubishiControlRequests::get_system_register(
                    MitsubishiSystemRegister::MotorFrequency,
                );
                let reg_bytes = reg.to_be_bytes();
                MitsubishiModbusRequest {
                    request: ModbusRequest {
                        slave_id: 1,
                        function_code: ModbusFunctionCode::ReadHoldingRegister,
                        data: vec![reg_bytes[0], reg_bytes[1], 0x0, 0x1],
                    },
                    request_type: RequestType::OperationCommand,
                    expected_response_type: ResponseType::ReadMotorFrequency,
                    priority: u16::MAX - 2,
                    control_request_type: MitsubishiControlRequests::ReadMotorFrequency,
                }
            }
            MitsubishiControlRequests::ResetInverter => {
                let reg: u16 = MitsubishiControlRequests::get_system_register(
                    MitsubishiSystemRegister::InverterReset,
                );
                let reg_bytes = reg.to_be_bytes();
                MitsubishiModbusRequest {
                    request: ModbusRequest {
                        slave_id: 1,
                        function_code: ModbusFunctionCode::PresetHoldingRegister,
                        data: vec![reg_bytes[0], reg_bytes[1], 0x0, 0b00000001],
                    },
                    request_type: RequestType::OperationCommand,
                    expected_response_type: ResponseType::NoResponse,
                    priority: u16::MAX,
                    control_request_type: MitsubishiControlRequests::ResetInverter,
                }
            }
            MitsubishiControlRequests::ClearAllParameters => todo!(),
            MitsubishiControlRequests::ClearNonCommunicationParameter => todo!(),
            MitsubishiControlRequests::ClearNonCommunicationParameters => todo!(),
        }
    }
}

/// These Requests Serve as Templates for controling the inverter
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
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
    /// Read Register 40201, This contains the actual output frequency
    ReadMotorFrequency,
}

// We need to know from the request queue which events are of what operation type, so that the correct timeout can be used
#[derive(Debug)]
pub struct MitsubishiModbusRequest {
    request: ModbusRequest,
    control_request_type: MitsubishiControlRequests,
    request_type: RequestType,
    expected_response_type: ResponseType,
    priority: u16,
}

#[derive(Debug)]
pub enum RotationDirection {
    Forward,
    Backwards,
    Stopped,
}

#[derive(Debug, Clone, Copy)]
pub enum ResponseType {
    NoResponse,
    ReadFrequency,
    ReadMotorFrequency, // Motor Frequency is the actual frequency, that the motor is running at right now
    ReadInverterStatus,
    WriteFrequency,
    InverterStatus,
    InverterControl,
}

#[derive(Debug)]
pub struct MitsubishiInverterRS485Actor {
    // Communication
    pub serial_interface: SerialInterface,
    pub baudrate: Option<u32>,
    pub encoding: Option<SerialEncoding>,
    pub request_map: HashMap<MitsubishiControlRequests, MitsubishiModbusRequest>,
    pub response: Option<ModbusResponse>,

    // State
    pub last_ts: Instant,
    pub last_message_size: usize,
    pub last_request_type: RequestType,
    pub last_control_request_type: MitsubishiControlRequests,
    pub state: State,
    pub next_response_type: ResponseType,
    pub frequency: Frequency,
}

impl MitsubishiInverterRS485Actor {
    pub fn new(serial_interface: SerialInterface) -> Self {
        Self {
            serial_interface,
            last_ts: Instant::now(),
            state: State::Uninitialized,
            request_map: HashMap::new(),
            response: None,
            next_response_type: ResponseType::ReadMotorFrequency,
            last_request_type: RequestType::OperationCommand,
            last_message_size: 0,
            baudrate: None,
            encoding: None,
            frequency: Frequency::ZERO,
            last_control_request_type: MitsubishiControlRequests::ResetInverter,
        }
    }

    /// This would get called by the api to add a new request to the inverter
    pub fn add_request(&mut self, request: MitsubishiModbusRequest) {
        self.request_map
            .insert(request.control_request_type.clone(), request);
    }

    /// This is used internally to read the receive buffer of the el6021
    fn read_modbus_response(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<ModbusResponse, anyhow::Error>> + Send + '_>> {
        Box::pin(async move {
            if !(self.serial_interface.has_message)().await {
                //   return;
            }

            let res: Option<Vec<u8>> = (self.serial_interface.read_message)().await;
            let raw_response = match res {
                Some(res) => res,
                None => {
                    vec![]
                }
            };

            let response: Result<ModbusResponse, _> =
                ModbusResponse::try_from(raw_response.clone());
            match response {
                Ok(result) => {
                    self.last_message_size = result.clone().data.len() + 4;
                    self.state = State::ReadyToSend;
                    Ok(result)
                }
                Err(_) => {
                    tracing::error!("Error Parsing ModbusResponse!");
                    self.state = State::ReadyToSend;
                    Err(anyhow::anyhow!("error"))
                }
            }
        })
    }

    /// This is used internally to fill the write buffer of the el6021 with the modbus request
    /// Decides what requests to send first by finding the one with the highest priority
    /// For example Highest Priority requests: ResetInverter StopMotor    
    fn send_modbus_request(&mut self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            if self.request_map.len() == 0 {
                return;
            };

            let mut highest_prio_request: Option<&mut MitsubishiModbusRequest> = None;
            let mut highest_priority: u16 = 0;

            for (_, value) in self.request_map.iter_mut() {
                // borrowchecker complaining
                let priority = value.priority;
                if priority > highest_priority {
                    highest_prio_request = Some(value);
                    highest_priority = priority;
                }
            }

            let request = match highest_prio_request {
                Some(request) => request,
                None => return,
            };

            let modbus_request: Vec<u8> = request.request.clone().into();
            let res = (self.serial_interface.write_message)(modbus_request.clone()).await;

            match res {
                Ok(_) => {
                    self.next_response_type = request.expected_response_type;
                    self.last_request_type = request.request_type;
                    self.last_control_request_type = request.control_request_type.clone();
                }
                Err(_) => tracing::error!("ERROR: serial_interface.write_message has failed"),
            }

            self.state = State::WaitingForRequestAccept;
            self.last_message_size = modbus_request.len();
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RequestType {
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
            RequestType::Reset => Duration::from_millis(30),
        }
    }
}

pub enum MitsubishiModbusExceptionCode {
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

// Handle different Response types
impl MitsubishiInverterRS485Actor {
    // When we get respone from Pr. 40014 (Running Frequency) Convert to rpm and save it
    fn handle_motor_frequency(&mut self, resp: ModbusResponse) {
        let freq_bytes = &resp.data[1..3]; // bytes 1 and 2 are needed
        let raw_frequency = u16::from_be_bytes([freq_bytes[0], freq_bytes[1]]) as f64;
        self.frequency = Frequency::new::<centihertz>(raw_frequency);
    }

    // Technically we could verify that every request also was successful with this match and return an Error, or not
    fn handle_response(&mut self, resp: ModbusResponse) {
        match self.next_response_type {
            ResponseType::ReadFrequency => (),
            ResponseType::ReadInverterStatus => (),
            ResponseType::WriteFrequency => (),
            ResponseType::ReadMotorFrequency => self.handle_motor_frequency(resp),
            ResponseType::InverterStatus => (),
            ResponseType::InverterControl => (),
            ResponseType::NoResponse => (),
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
                    // every time when our inverter is "Uninitialzed" reset it first to clear any error states it may have
                    self.add_request(MitsubishiControlRequests::ResetInverter.into());
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
                self.last_request_type.timeout_duration(),
                baudrate,
                self.last_message_size,
            );

            self.add_request(MitsubishiControlRequests::ReadMotorFrequency.into());

            if elapsed < timeout {
                return;
            }

            self.last_ts = now_ts;

            match self.state {
                State::WaitingForResponse => {
                    let ret = self.read_modbus_response().await;
                    match ret {
                        Ok(ret) => self.handle_response(ret),
                        Err(_) => (), // Do nothing for now
                    }
                }
                State::ReadyToSend => self.send_modbus_request().await,
                State::WaitingForReceiveAccept => self.state = State::ReadyToSend,
                State::WaitingForRequestAccept => self.state = State::WaitingForResponse,
                _ => (),
            }

            self.request_map.remove(&self.last_control_request_type);
            self.response = None;
        })
    }
}
