use bitvec::{order::Lsb0, slice::BitSlice};
use control_core::modbus::{
    ModbusFunctionCode, ModbusRequest, ModbusResponse,
    modbus_serial_interface::ModbusSerialInterface,
};
use ethercat_hal::io::serial_interface::SerialInterface;
use serde::Serialize;
use std::time::{Duration, Instant};
use units::electric_current::centiampere;
use units::electric_potential::centivolt;
use units::f64::*;
use units::frequency::centihertz;

/// Specifies all System environment Variables
/// Register addresses are calculated as follows: Register-value 40002 -> address: 40002-40001 -> actual address in request:0x1
#[derive(Debug, Clone, Copy)]
enum MitsubishiCS80Register {
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
    MotorStatus,
}

impl MitsubishiCS80Register {
    const fn address(self) -> u16 {
        match self {
            Self::InverterReset => 0x1,
            Self::InverterStatusAndControl => 0x8,
            Self::RunningFrequencyRAM => 0x0d,
            Self::MotorStatus => 0x00C8, // a0x00C8 = frequency , 0x00C9 = current ,0x00C10 = voltage
        }
    }

    const fn address_be_bytes(self) -> [u8; 2] {
        self.address().to_be_bytes()
    }
}

/// These Requests Serve as Templates for controlling the inverter
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum MitsubishiCS80Requests {
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
    /// Read Register 40201, 40202 and 40203 frequency,current and voltage
    ReadMotorStatus,
    /// Write "Arbitrary" Parameters
    WriteParameter,
}

impl From<MitsubishiCS80Requests> for u32 {
    fn from(request: MitsubishiCS80Requests) -> Self {
        request as Self
    }
}

impl TryFrom<u32> for MitsubishiCS80Requests {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::ResetInverter),
            2 => Ok(Self::ClearAllParameters),
            3 => Ok(Self::ClearNonCommunicationParameter),
            4 => Ok(Self::ClearNonCommunicationParameters),
            5 => Ok(Self::ReadInverterStatus),
            6 => Ok(Self::StopMotor),
            7 => Ok(Self::StartForwardRotation),
            8 => Ok(Self::StartReverseRotation),
            9 => Ok(Self::ReadRunningFrequency),
            10 => Ok(Self::WriteRunningFrequency),
            11 => Ok(Self::ReadMotorStatus),
            12 => Ok(Self::WriteParameter),
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
impl From<MitsubishiCS80Requests> for MitsubishiCS80Request {
    fn from(request: MitsubishiCS80Requests) -> Self {
        match request {
            MitsubishiCS80Requests::WriteRunningFrequency => {
                let reg_bytes = MitsubishiCS80Register::RunningFrequencyRAM.address_be_bytes();
                Self::new(
                    ModbusRequest {
                        slave_id: 1,
                        function_code: ModbusFunctionCode::PresetHoldingRegister,
                        data: vec![reg_bytes[0], reg_bytes[1], 0x0, 0x0],
                    },
                    request,
                    RequestType::OperationCommand,
                    u16::MAX - 1,
                )
            }
            MitsubishiCS80Requests::ReadInverterStatus => {
                let reg_bytes = MitsubishiCS80Register::InverterStatusAndControl.address_be_bytes();
                Self::new(
                    ModbusRequest {
                        slave_id: 1,
                        function_code: ModbusFunctionCode::ReadHoldingRegister,
                        data: vec![reg_bytes[0], reg_bytes[1], 0x00, 0x01], // Read 1 register
                    },
                    request,
                    RequestType::OperationCommand,
                    u16::MAX - 6, // Priority is -6 because we do not want to know the status as frequently as the frequency
                )
            }
            MitsubishiCS80Requests::StopMotor => {
                let reg_bytes = MitsubishiCS80Register::InverterStatusAndControl.address_be_bytes();
                Self::new(
                    ModbusRequest {
                        slave_id: 1,
                        function_code: ModbusFunctionCode::PresetHoldingRegister,
                        data: vec![reg_bytes[0], reg_bytes[1], 0x00, 0x01], // Value 1 to stop
                    },
                    request,
                    RequestType::OperationCommand,
                    u16::MAX, // StopMotor should have highest priority
                )
            }
            MitsubishiCS80Requests::StartForwardRotation => {
                let reg_bytes = MitsubishiCS80Register::InverterStatusAndControl.address_be_bytes();
                Self::new(
                    ModbusRequest {
                        slave_id: 1,
                        function_code: ModbusFunctionCode::PresetHoldingRegister,
                        data: vec![reg_bytes[0], reg_bytes[1], 0, 0b00000010], // Value 2 for forward rotation
                    },
                    request,
                    RequestType::OperationCommand,
                    u16::MAX - 1,
                )
            }
            MitsubishiCS80Requests::StartReverseRotation => {
                let reg_bytes = MitsubishiCS80Register::InverterStatusAndControl.address_be_bytes();
                Self::new(
                    ModbusRequest {
                        slave_id: 1,
                        function_code: ModbusFunctionCode::PresetHoldingRegister,
                        data: vec![reg_bytes[0], reg_bytes[1], 0, 0b00000100], // Value 4 for reverse rotation
                    },
                    request,
                    RequestType::OperationCommand,
                    u16::MAX - 1,
                )
            }
            MitsubishiCS80Requests::ReadRunningFrequency => {
                let reg_bytes = MitsubishiCS80Register::RunningFrequencyRAM.address_be_bytes();
                Self::new(
                    ModbusRequest {
                        slave_id: 1,
                        function_code: ModbusFunctionCode::ReadHoldingRegister,
                        data: vec![reg_bytes[0], reg_bytes[1], 0x00, 0x01], // Read 1 register
                    },
                    request,
                    RequestType::OperationCommand,
                    u16::MAX - 4,
                )
            }
            MitsubishiCS80Requests::ReadMotorStatus => {
                let reg_bytes = MitsubishiCS80Register::MotorStatus.address_be_bytes();
                Self::new(
                    ModbusRequest {
                        slave_id: 1,
                        function_code: ModbusFunctionCode::ReadHoldingRegister,
                        data: vec![reg_bytes[0], reg_bytes[1], 0x0, 0x3], // read 3 registers: 0x00C8 = frequency , 0x00C9 = current ,0x00C10 = voltage
                    },
                    request,
                    RequestType::OperationCommand,
                    u16::MAX - 2,
                )
            }
            MitsubishiCS80Requests::ResetInverter => {
                let reg_bytes = MitsubishiCS80Register::InverterReset.address_be_bytes();
                Self::new(
                    ModbusRequest {
                        slave_id: 1,
                        function_code: ModbusFunctionCode::PresetHoldingRegister,
                        data: vec![reg_bytes[0], reg_bytes[1], 0x0, 0b00000001],
                    },
                    request,
                    RequestType::Reset,
                    u16::MAX,
                )
            }
            MitsubishiCS80Requests::WriteParameter => Self::new(
                ModbusRequest {
                    slave_id: 1,
                    function_code: ModbusFunctionCode::PresetHoldingRegister,
                    data: vec![0x0, 0x0, 0x0, 0x0],
                },
                request,
                RequestType::ReadWrite,
                u16::MAX,
            ),

            // For unimplemented variants, return a default request
            _ => Self::new(
                ModbusRequest {
                    slave_id: 1,
                    function_code: ModbusFunctionCode::ReadHoldingRegister,
                    data: vec![0x0, 0x0, 0x0, 0x1],
                },
                request,
                RequestType::None,
                0,
            ),
        }
    }
}

// Serialize is needed so we can hash it
#[derive(Debug, Default, Clone, Copy, Serialize)]
pub struct MitsubishiCS80Status {
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

#[derive(Debug, Default, Clone, Copy)]
pub struct MotorStatus {
    pub rpm: AngularVelocity,
    pub frequency: Frequency,
    pub current: ElectricCurrent,
    pub voltage: ElectricPotential,
}

#[derive(Debug)]
pub struct MitsubishiCS80 {
    // Communication
    pub status: MitsubishiCS80Status,
    pub motor_status: MotorStatus,
    pub modbus_serial_interface: ModbusSerialInterface,
    pub last_ts: Instant,
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
    const fn timeout_duration(self) -> Duration {
        match self {
            Self::OperationCommand => Duration::from_millis(12),
            Self::ReadWrite => Duration::from_millis(30),
            Self::ParamClear => Duration::from_millis(5000),
            Self::Reset => Duration::from_millis(900),
            Self::None => Duration::from_millis(12),
        }
    }
}

// We need to know from the request queue which events are of what operation type, so that the correct timeout can be used
#[derive(Debug, Clone)]
pub struct MitsubishiCS80Request {
    request: ModbusRequest,
    control_request_type: MitsubishiCS80Requests,
    request_type: RequestType,
    priority: u16,
}

impl MitsubishiCS80Request {
    const fn new(
        request: ModbusRequest,
        control_request_type: MitsubishiCS80Requests,
        request_type: RequestType,
        priority: u16,
    ) -> Self {
        Self {
            request,
            control_request_type,
            request_type,
            priority,
        }
    }
}

impl MitsubishiCS80 {
    pub fn new(serial_interface: SerialInterface) -> Self {
        Self {
            modbus_serial_interface: ModbusSerialInterface::new(serial_interface),
            last_ts: Instant::now(),
            motor_status: MotorStatus::default(),
            status: MitsubishiCS80Status::default(),
        }
    }

    fn handle_motor_status(&mut self, resp: &ModbusResponse) {
        if resp.data.len() >= 7 {
            let freq_bytes = &resp.data[1..3]; // bytes 1 and 2 are needed
            let raw_frequency = u16::from_be_bytes([freq_bytes[0], freq_bytes[1]]) as f64;
            self.motor_status.frequency = Frequency::new::<centihertz>(raw_frequency);

            let electric_current_bytes = &resp.data[3..5];
            let raw_current =
                u16::from_be_bytes([electric_current_bytes[0], electric_current_bytes[1]]) as f64;
            self.motor_status.current = ElectricCurrent::new::<centiampere>(raw_current);

            let voltage_current_bytes = &resp.data[5..7];
            let raw_voltage =
                u16::from_be_bytes([voltage_current_bytes[0], voltage_current_bytes[1]]) as f64;
            self.motor_status.voltage = ElectricPotential::new::<centivolt>(raw_voltage);
        }
    }

    fn handle_read_inverter_status(&mut self, resp: &ModbusResponse) {
        if resp.data.len() < 3 {
            return;
        }

        let status_bytes: [u8; 2] = match resp.data[1..3].try_into() {
            Ok(bytes) => bytes,
            Err(_) => return,
        };

        let bits: &BitSlice<u8, Lsb0> = BitSlice::<_, Lsb0>::from_slice(&status_bytes);
        if bits.len() >= 16 {
            self.status = MitsubishiCS80Status {
                fault_occurence: bits[7],
                running: bits[8],
                forward_running: bits[9],
                reverse_running: bits[10],
                su: bits[11],
                ol: bits[12],
                no_function: bits[13],
                fu: bits[14],
                abc_: bits[15],
            };
        }
    }

    fn handle_response(&mut self, control_request_type: u32) {
        let response_type = match MitsubishiCS80Requests::try_from(control_request_type) {
            Ok(request_type) => request_type,
            Err(_) => return,
        };

        let Some(response) = self.modbus_serial_interface.get_response().cloned() else {
            return;
        };

        match response_type {
            MitsubishiCS80Requests::ReadInverterStatus => {
                self.handle_read_inverter_status(&response);
            }
            MitsubishiCS80Requests::ReadMotorStatus => {
                self.handle_motor_status(&response);
            }
            // Other request types don't need response handling
            _ => {}
        }
    }

    fn convert_frequency_to_word(&self, frequency: Frequency) -> u16 {
        let scaled = frequency.get::<centihertz>(); // Convert Hz to 0.01 Hz units
        scaled.round() as u16
    }

    fn add_request(&mut self, request: MitsubishiCS80Request) {
        let no_response_expected = matches!(
            request.control_request_type,
            MitsubishiCS80Requests::None | MitsubishiCS80Requests::ResetInverter
        );

        self.modbus_serial_interface.add_request(
            request.control_request_type.into(),
            request.priority as u32,
            request.request,
            no_response_expected,
            Some(request.request_type.timeout_duration().as_nanos() as u32),
        );
    }

    pub fn stop_motor(&mut self) {
        self.add_request(MitsubishiCS80Requests::StopMotor.into());
    }

    pub fn set_frequency_target(&mut self, frequency: Frequency) {
        let mut request: MitsubishiCS80Request =
            MitsubishiCS80Requests::WriteRunningFrequency.into();
        let result = self.convert_frequency_to_word(frequency);
        let bytes = result.to_le_bytes();
        request.request.data[2] = bytes[1];
        request.request.data[3] = bytes[0];

        self.add_request(request);
    }

    pub fn set_rotation(&mut self, forward_rotation: bool) {
        let request = if forward_rotation {
            // Gearbox is inverted!
            MitsubishiCS80Requests::StartReverseRotation
        } else {
            MitsubishiCS80Requests::StartForwardRotation
        };
        self.add_request(request.into());
    }

    pub fn reset_inverter(&mut self) {
        self.add_request(MitsubishiCS80Requests::ResetInverter.into());
    }

    pub async fn act(&mut self, now: Instant) {
        if !self.modbus_serial_interface.is_initialized() {
            if self.modbus_serial_interface.initialize().await {
                self.add_request(MitsubishiCS80Requests::ResetInverter.into());
            }
            return;
        }

        self.add_request(MitsubishiCS80Requests::ReadInverterStatus.into());
        self.add_request(MitsubishiCS80Requests::ReadMotorStatus.into());
        self.modbus_serial_interface.act(now).await;
        self.handle_response(self.modbus_serial_interface.last_message_id);
    }
}
