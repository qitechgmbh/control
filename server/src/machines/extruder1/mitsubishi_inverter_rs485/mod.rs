use bitvec::{order::Lsb0, slice::BitSlice};
use control_core::actors::serial_interface_actor::SerialInterfaceActor;
use control_core::modbus::{ModbusFunctionCode, ModbusRequest, ModbusResponse};
use ethercat_hal::io::serial_interface::SerialInterface;
use std::{
    time::{Duration, Instant},
    u16,
};
use uom::{
    ConstZero,
    si::{f64::Frequency, frequency::centihertz},
};

/// Specifies all System environmet Variables
/// Register addresses are calculated as follows: Register-value 40002 -> address: 40002-40001 -> address:0x1
enum MitsubishiSystemRegister {
    /// Register 40002
    InverterReset,
    /// Register 40003
    //ParameterClear,
    /// Register 40004
    //AllParameterClear,
    /// Register 40006
    //ParamClearNonCommunication,
    /// Register 40007
    //AllParameterClearNonCommunication,
    /// Register 40009
    InverterStatusAndControl,
    /// Register 40010
    //OperationModeAndSetting,
    /// Register 40014
    RunningFrequencyRAM,
    /// Register 40015
    //RunningFrequencyEEPROM,
    /// Register 40201
    MotorFrequency,
}

impl MitsubishiControlRequests {
    fn get_system_register(value: MitsubishiSystemRegister) -> u16 {
        match value {
            MitsubishiSystemRegister::InverterReset => 0x1,
            // MitsubishiSystemRegister::ParameterClear => 0x2,
            // MitsubishiSystemRegister::AllParameterClear => 0x3,
            // MitsubishiSystemRegister::ParamClearNonCommunication => 0x5,
            // MitsubishiSystemRegister::AllParameterClearNonCommunication => 0x6,
            MitsubishiSystemRegister::InverterStatusAndControl => 0x8,
            // MitsubishiSystemRegister::OperationModeAndSetting => 0x9,
            MitsubishiSystemRegister::RunningFrequencyRAM => 0x0d,
            //MitsubishiSystemRegister::RunningFrequencyEEPROM => 0x0e,
            MitsubishiSystemRegister::MotorFrequency => 0x00C8,
        }
    }
}

// So that the generic SerialInterfaceActor can identify the Requests
impl From<MitsubishiControlRequests> for u32 {
    fn from(request: MitsubishiControlRequests) -> Self {
        match request {
            MitsubishiControlRequests::None => 0,
            MitsubishiControlRequests::ResetInverter => 1,
            MitsubishiControlRequests::ClearAllParameters => 2,
            MitsubishiControlRequests::ClearNonCommunicationParameter => 3,
            MitsubishiControlRequests::ClearNonCommunicationParameters => 4,
            MitsubishiControlRequests::ReadInverterStatus => 5,
            MitsubishiControlRequests::StopMotor => 6,
            MitsubishiControlRequests::StartForwardRotation => 7,
            MitsubishiControlRequests::StartReverseRotation => 8,
            MitsubishiControlRequests::ReadRunningFrequency => 9,
            MitsubishiControlRequests::WriteRunningFrequency => 10,
            MitsubishiControlRequests::ReadMotorFrequency => 11,
            MitsubishiControlRequests::WriteParameter => 12,
        }
    }
}

impl TryFrom<u32> for MitsubishiControlRequests {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MitsubishiControlRequests::None),
            1 => Ok(MitsubishiControlRequests::ResetInverter),
            2 => Ok(MitsubishiControlRequests::ClearAllParameters),
            3 => Ok(MitsubishiControlRequests::ClearNonCommunicationParameter),
            4 => Ok(MitsubishiControlRequests::ClearNonCommunicationParameters),
            5 => Ok(MitsubishiControlRequests::ReadInverterStatus),
            6 => Ok(MitsubishiControlRequests::StopMotor),
            7 => Ok(MitsubishiControlRequests::StartForwardRotation),
            8 => Ok(MitsubishiControlRequests::StartReverseRotation),
            9 => Ok(MitsubishiControlRequests::ReadRunningFrequency),
            10 => Ok(MitsubishiControlRequests::WriteRunningFrequency),
            11 => Ok(MitsubishiControlRequests::ReadMotorFrequency),
            _ => Err(()),
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

    Lets say one request "A" with priority 1 and one with 2 "B" are queued up, assume that request B is frequently used
    1. Request "B" is executed due to higher priority
    2. When B is added again request A has the same priority because it was ignored. B is executed once again
    3. B is added again, now A has an effective priority of 3, which is higher then B
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
                    // Priority is -6 because we do not want to know the status as frequently as the frequency, which is why its priority is lower
                    // In essence this means for every fifth ReadMotorFrequency request an InverterStatus request is sent
                    priority: u16::MAX - 6,
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
                    request_type: RequestType::Reset,
                    priority: u16::MAX,
                    control_request_type: MitsubishiControlRequests::ResetInverter,
                }
            }
            MitsubishiControlRequests::ClearAllParameters => todo!(),
            MitsubishiControlRequests::ClearNonCommunicationParameter => todo!(),
            MitsubishiControlRequests::ClearNonCommunicationParameters => todo!(),
            MitsubishiControlRequests::None => todo!(),
            MitsubishiControlRequests::WriteParameter => MitsubishiModbusRequest {
                request: ModbusRequest {
                    slave_id: 1,
                    function_code: ModbusFunctionCode::PresetHoldingRegister,
                    data: vec![0x0, 0x0, 0x0, 0x0],
                },
                request_type: RequestType::ReadWrite,
                priority: u16::MAX,
                control_request_type: MitsubishiControlRequests::WriteParameter,
            },
        }
    }
}

/// These Requests Serve as Templates for controling the inverter
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum MitsubishiControlRequests {
    None,
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
    /// Write "Arbitrary" Parameters
    WriteParameter,
}

// We need to know from the request queue which events are of what operation type, so that the correct timeout can be used
#[derive(Debug, Clone)]
pub struct MitsubishiModbusRequest {
    request: ModbusRequest,
    control_request_type: MitsubishiControlRequests,
    request_type: RequestType,
    priority: u16,
}

#[derive(Debug, Default)]
pub struct MitsubishiInverterStatus {
    pub running: bool,
    pub forward_running: bool,
    pub reverse_running: bool,
    pub su: bool,
    pub ol: bool,
    pub no_function: bool,
    pub fu: bool,
    pub abc_: bool,
    pub fault_occurence: bool,
}

#[derive(Debug)]
pub struct MitsubishiInverterController {
    // Communication
    pub inverter_status: MitsubishiInverterStatus,
    pub serial_actor: SerialInterfaceActor,
    pub last_ts: Instant,
    pub frequency: Frequency,
}

impl MitsubishiInverterController {
    pub fn new(serial_interface: SerialInterface) -> Self {
        Self {
            serial_actor: SerialInterfaceActor::new(serial_interface),
            last_ts: Instant::now(),
            frequency: Frequency::ZERO,
            inverter_status: MitsubishiInverterStatus::default(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RequestType {
    None,
    /// Monitoring, Operation (start,stop etc) command, frequency setting (RAM), less than 12 milliseconds timeout for Response
    OperationCommand,
    /// Parameter Read/Write and Frequency (EEPROM), Less than 30 milliseconds timeout for Response
    ReadWrite,
    /// Less than 5 seconds timeout for Response
    ParamClear,
    /// Supposedly no waiting time, however inverter takes a while to start ~300ms should be more than enough
    Reset,
}

impl RequestType {
    fn timeout_duration(self) -> Duration {
        match self {
            RequestType::OperationCommand => Duration::from_millis(12),
            RequestType::ReadWrite => Duration::from_millis(30),
            RequestType::ParamClear => Duration::from_millis(5000),
            RequestType::Reset => Duration::from_millis(300),
            RequestType::None => Duration::from_millis(12),
        }
    }
}

impl MitsubishiInverterController {
    fn handle_motor_frequency(&mut self, resp: ModbusResponse) {
        let freq_bytes = &resp.data[1..3]; // bytes 1 and 2 are needed
        let raw_frequency = u16::from_be_bytes([freq_bytes[0], freq_bytes[1]]) as f64;
        self.frequency = Frequency::new::<centihertz>(raw_frequency);
    }

    fn handle_read_inverter_status(&mut self, resp: ModbusResponse) {
        let status_bytes: [u8; 2] = match resp.data[1..3].try_into() {
            Ok(res) => res,
            Err(_) => return,
        };

        let bits: &BitSlice<u8, Lsb0> = BitSlice::<_, Lsb0>::from_slice(&status_bytes);
        self.inverter_status = MitsubishiInverterStatus {
            running: bits[8],
            forward_running: bits[9],
            reverse_running: bits[10],
            su: bits[11],
            ol: bits[12],
            no_function: bits[13],
            fu: bits[14],
            abc_: bits[15],
            fault_occurence: bits[7],
        };
    }

    fn handle_response(&mut self, control_request_type: u32) {
        let response_type: Result<MitsubishiControlRequests, ()> = control_request_type.try_into();
        let result = match response_type {
            Ok(response_type) => response_type,
            Err(_) => {
                return;
            }
        };

        let response = match self.serial_actor.get_response() {
            Some(response) => response,
            None => {
                return;
            }
        };
        match result {
            MitsubishiControlRequests::None => (),
            MitsubishiControlRequests::ResetInverter => (),
            MitsubishiControlRequests::ClearAllParameters => (),
            MitsubishiControlRequests::ClearNonCommunicationParameter => (),
            MitsubishiControlRequests::ClearNonCommunicationParameters => (),
            MitsubishiControlRequests::ReadInverterStatus => {
                self.handle_read_inverter_status(response.clone())
            }
            MitsubishiControlRequests::StopMotor => (),
            MitsubishiControlRequests::StartForwardRotation => (),
            MitsubishiControlRequests::StartReverseRotation => (),
            MitsubishiControlRequests::ReadRunningFrequency => (),
            MitsubishiControlRequests::WriteRunningFrequency => (),
            MitsubishiControlRequests::ReadMotorFrequency => {
                self.handle_motor_frequency(response.clone())
            }
            MitsubishiControlRequests::WriteParameter => (),
        }
    }

    pub fn convert_hz_float_to_word(&mut self, value: Frequency) -> u16 {
        let scaled = value.get::<centihertz>(); // Convert Hz to 0.01 Hz units
        scaled.round() as u16
    }

    fn add_request(&mut self, request: MitsubishiModbusRequest) {
        self.serial_actor.add_request(
            request.control_request_type.into(),
            request.priority as u32,
            request.request,
            Some(request.request_type.timeout_duration().as_nanos() as u32),
        );
    }

    pub fn stop_motor(&mut self) {
        self.add_request(MitsubishiControlRequests::StopMotor.into());
    }

    pub fn set_frequency_target(&mut self, frequency: Frequency) {
        let mut request: MitsubishiModbusRequest =
            MitsubishiControlRequests::WriteRunningFrequency.into();

        let result: u16 = self.convert_hz_float_to_word(frequency); // convert hz float to short
        request.request.data[2] = result.to_le_bytes()[1];
        request.request.data[3] = result.to_le_bytes()[0];

        self.serial_actor.add_request(
            MitsubishiControlRequests::WriteRunningFrequency.into(),
            request.priority as u32,
            request.request,
            Some(request.request_type.timeout_duration().as_nanos() as u32),
        );
    }

    pub fn set_rotation(&mut self, forward_rotation: bool) {
        if forward_rotation {
            // Gearbox is inverted!
            self.add_request(MitsubishiControlRequests::StartReverseRotation.into());
        } else {
            self.add_request(MitsubishiControlRequests::StartForwardRotation.into());
        }
    }

    pub fn reset_inverter(&mut self) {
        self.add_request(MitsubishiControlRequests::ResetInverter.into());
    }

    pub async fn act(&mut self, now: Instant) {
        if self.serial_actor.is_initialized() == false {
            let res = self.serial_actor.initialize().await;
            if res {
                self.add_request(MitsubishiControlRequests::ResetInverter.into());
            }
            return;
        }

        self.add_request(MitsubishiControlRequests::ReadInverterStatus.into());
        self.add_request(MitsubishiControlRequests::ReadMotorFrequency.into());
        self.serial_actor.act(now).await;
        self.handle_response(self.serial_actor.last_message_id);
    }
}
