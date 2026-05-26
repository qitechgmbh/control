use crate::helpers::ethercrab_types::EthercrabSubDevicePreoperational;

pub use super::el70x1::{
    EncConfiguration, EL70x1InputFunction, EL70x1OperationMode, EL70x1SpeedRange, PosConfiguration,
    PosFeatures,
};

// ────────────────────────────────────────────────────────────────────────────
// STM Motor Configuration
// ────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct StmMotorConfiguration {
    /// # 0x8010:01
    /// Maximum current (unit: 1 mA)
    ///
    /// default: `0x05DC` (1500dec) = 1.5A
    pub max_current: u16,

    /// # 0x8010:02
    /// Reduced current (unit: 1 mA)
    ///
    /// default: `0x02EE` (750dec) = 0.75A
    pub reduced_current: u16,

    /// # 0x8010:03
    /// Nominal voltage (unit: 1 mV, stored; written as 10 mV to hardware)
    ///
    /// default: `0xC350` (50000dec) = 50V
    pub nominal_voltage: u16,

    /// # 0x8010:04
    /// Motor coil resistance (unit: 0.01 ohm)
    ///
    /// default: `0x0064` (100dec) = 1 ohm
    pub motor_coil_resistance: u16,

    /// # 0x8010:05
    /// Motor countervoltage (unit: 1 mV/(rad/s))
    ///
    /// default: `0x0000` (0dec)
    pub motor_emf: u16,

    /// # 0x8010:06
    /// Motor full steps (unit: 1 step)
    ///
    /// default: `0x00C8` (200dec) = 200 steps
    pub motor_full_steps: u16,

    /// # 0x8010:09
    /// Maximum possible start velocity of the motor
    ///
    /// default: `0x0000` (0dec)
    pub start_velocity: u16,

    /// # 0x8010:10
    /// Switch-on delay of the driver stage
    ///
    /// default: `0x0064` (100dec) = 0.1s
    pub drive_on_delay_time: u16,

    /// # 0x8010:11
    /// Switch-off delay of the driver stage
    ///
    /// default: `0x0064` (100dec) = 0.1s
    pub drive_off_delay_time: u16,
}

impl Default for StmMotorConfiguration {
    /// Defaults according to the datasheet
    fn default() -> Self {
        Self {
            max_current: 0x05DC,           // 1500 mA = 1.5A
            reduced_current: 0x02EE,       // 750 mA = 0.75A
            nominal_voltage: 0xC350,       // 50000 mV = 50V
            motor_coil_resistance: 0x0064, // 100 = 1 ohm
            motor_emf: 0x00C8,             // 200 mV/(rad/s)
            motor_full_steps: 0x00C8,      // 200 steps
            start_velocity: 0x0000,        // 0
            drive_on_delay_time: 0x0064,   // 100 ms = 0.1s
            drive_off_delay_time: 0x0064,  // 100 ms = 0.1s
        }
    }
}

impl StmMotorConfiguration {
    pub async fn write_config<'a>(
        &self,
        device: &EthercrabSubDevicePreoperational<'a>,
    ) -> Result<(), anyhow::Error> {
        device.sdo_write(0x8010, 0x01, self.max_current).await?;
        device.sdo_write(0x8010, 0x02, self.reduced_current).await?;
        // EL7037 uses 10 mV units for 0x8010:03; nominal_voltage is stored as 1 mV.
        // Terminal may reject values exceeding the actual supply voltage; fall back to default.
        if let Err(e) = device
            .sdo_write(0x8010, 0x03, self.nominal_voltage / 10)
            .await
        {
            tracing::debug!(
                "EL7037 0x8010:03 nominal_voltage write rejected ({e}); \
                 keeping terminal default."
            );
        }
        device
            .sdo_write(0x8010, 0x04, self.motor_coil_resistance)
            .await?;
        device.sdo_write(0x8010, 0x05, self.motor_emf).await?;
        device
            .sdo_write(0x8010, 0x06, self.motor_full_steps)
            .await?;
        device.sdo_write(0x8010, 0x09, self.start_velocity).await?;
        device
            .sdo_write(0x8010, 0x10, self.drive_on_delay_time)
            .await?;
        device
            .sdo_write(0x8010, 0x11, self.drive_off_delay_time)
            .await?;
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// STM Controller Configuration
// ────────────────────────────────────────────────────────────────────────────

/// EL7037 current controller configuration.
///
/// EL7037 object 0x8011 has max subindex 0x02 (Kp + Ki only).
/// EL7031/EL7041 have 7 subindices — use `el70x1::StmControllerConfiguration` for those.
#[derive(Debug, Clone)]
pub struct StmControllerConfiguration {
    /// # 0x8011:01
    /// Kp control factor (proportional component) for the current controller (unit: 0.001)
    ///
    /// default: `0x0190` (400dec) = 0.4
    pub kp_factor: u16,

    /// # 0x8011:02
    /// Ki control factor (integral component) for the current controller (unit: 0.001)
    ///
    /// default: `0x0004` (4dec) = 0.004
    pub ki_factor: u16,
}

impl Default for StmControllerConfiguration {
    fn default() -> Self {
        Self {
            kp_factor: 0x0190, // 400
            ki_factor: 0x0004, // 4
        }
    }
}

impl StmControllerConfiguration {
    pub async fn write_config<'a>(
        &self,
        device: &EthercrabSubDevicePreoperational<'a>,
        base_index: u16,
    ) -> Result<(), anyhow::Error> {
        device.sdo_write(base_index, 0x01, self.kp_factor).await?;
        device.sdo_write(base_index, 0x02, self.ki_factor).await?;
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// EL70x7 Info Data
// ────────────────────────────────────────────────────────────────────────────

/// Info data selection for EL7037.
///
/// Superset of `el70x1::EL70x1InfoData` — includes EL7037-specific values
/// `MotorLoad` (11) and `MotorDcCurrent` (13).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EL70x7InfoData {
    /// Status word
    StatusWord = 0,
    /// Motor voltage coil A (unit 1 mV)
    MotorVoltageCoilA = 1,
    /// Motor voltage coil B (unit 1 mV)
    MotorVoltageCoilB = 2,
    /// Motor current coil A (unit 1 mA)
    MotorCurrentCoilA = 3,
    /// Motor current coil B (unit 1 mA)
    MotorCurrentCoilB = 4,
    /// Duty-Cycle coil A (unit 1%)
    DutyCycleCoilA = 5,
    /// Duty-Cycle coil B (unit 1%)
    DutyCycleCoilB = 6,
    /// Current velocity (value range +/- 10000)
    CurrentVelocity = 7,
    /// Motor load (unit 0.01 deg)
    MotorLoad = 11,
    /// Motor DC current (unit 1 mA)
    MotorDcCurrent = 13,
    /// Internal temperature of the driver card
    InternalTemperature = 101,
    /// Control voltage
    ControlVoltage = 103,
    /// Motor supply voltage
    MotorSupplyVoltage = 104,
    /// Drive - Status word
    DriveStatusWord = 150,
    /// Drive - State
    DriveState = 151,
    /// Drive - Position lag (low word)
    DrivePositionLagLow = 152,
    /// Drive - Position lag (high word)
    DrivePositionLagHigh = 153,
}

impl std::fmt::Debug for EL70x7InfoData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StatusWord => write!(f, "StatusWord (0)"),
            Self::MotorVoltageCoilA => write!(f, "MotorVoltageCoilA (1)"),
            Self::MotorVoltageCoilB => write!(f, "MotorVoltageCoilB (2)"),
            Self::MotorCurrentCoilA => write!(f, "MotorCurrentCoilA (3)"),
            Self::MotorCurrentCoilB => write!(f, "MotorCurrentCoilB (4)"),
            Self::DutyCycleCoilA => write!(f, "DutyCycleCoilA (5)"),
            Self::DutyCycleCoilB => write!(f, "DutyCycleCoilB (6)"),
            Self::CurrentVelocity => write!(f, "CurrentVelocity (7)"),
            Self::MotorLoad => write!(f, "MotorLoad (11)"),
            Self::MotorDcCurrent => write!(f, "MotorDcCurrent (13)"),
            Self::InternalTemperature => write!(f, "InternalTemperature (101)"),
            Self::ControlVoltage => write!(f, "ControlVoltage (103)"),
            Self::MotorSupplyVoltage => write!(f, "MotorSupplyVoltage (104)"),
            Self::DriveStatusWord => write!(f, "DriveStatusWord (150)"),
            Self::DriveState => write!(f, "DriveState (151)"),
            Self::DrivePositionLagLow => write!(f, "DrivePositionLagLow (152)"),
            Self::DrivePositionLagHigh => write!(f, "DrivePositionLagHigh (153)"),
        }
    }
}

impl TryFrom<u8> for EL70x7InfoData {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::StatusWord),
            1 => Ok(Self::MotorVoltageCoilA),
            2 => Ok(Self::MotorVoltageCoilB),
            3 => Ok(Self::MotorCurrentCoilA),
            4 => Ok(Self::MotorCurrentCoilB),
            5 => Ok(Self::DutyCycleCoilA),
            6 => Ok(Self::DutyCycleCoilB),
            7 => Ok(Self::CurrentVelocity),
            11 => Ok(Self::MotorLoad),
            13 => Ok(Self::MotorDcCurrent),
            101 => Ok(Self::InternalTemperature),
            103 => Ok(Self::ControlVoltage),
            104 => Ok(Self::MotorSupplyVoltage),
            150 => Ok(Self::DriveStatusWord),
            151 => Ok(Self::DriveState),
            152 => Ok(Self::DrivePositionLagLow),
            153 => Ok(Self::DrivePositionLagHigh),
            _ => Err(anyhow::anyhow!(
                "Invalid value for EL70x7InfoData: {}",
                value
            )),
        }
    }
}

impl From<EL70x7InfoData> for u8 {
    fn from(value: EL70x7InfoData) -> Self {
        value as Self
    }
}

// ────────────────────────────────────────────────────────────────────────────
// STM Features
// ────────────────────────────────────────────────────────────────────────────

/// EL7037 STM features (object 0x8012).
///
/// Differs from `el70x1::StmFeatures` in two ways:
/// - Uses `EL70x7InfoData` for info data selection (supports `MotorLoad`/`MotorDcCurrent`)
/// - `write_config` explicitly writes `operation_mode` (0x8012:01), required on EL7037
#[derive(Debug, Clone)]
pub struct StmFeatures {
    /// # 0x8012:01
    /// Operating mode
    ///
    /// default: `0x00` (0dec) = Automatic
    pub operation_mode: EL70x1OperationMode,

    /// # 0x8012:05
    /// Preselection of the speed range
    ///
    /// default: `0x01` (1dec) = 2000 full steps/second
    pub speed_range: EL70x1SpeedRange,

    /// # 0x8012:09
    /// Activates reversal of the motor rotation direction.
    ///
    /// default: `false`
    pub invert_motor_polarity: bool,

    /// # 0x8012:11
    /// Select "Info data 1" (see 0x6010:11)
    ///
    /// default: `0x03` (3dec) = Motor current coil A
    pub select_info_data_1: EL70x7InfoData,

    /// # 0x8012:19
    /// Selection "Info data 2"
    ///
    /// default: `0x04` (4dec) = Motor current coil B
    pub select_info_data_2: EL70x7InfoData,

    /// # 0x8012:30
    /// Inversion of digital input 1
    ///
    /// default: `false`
    pub invert_digital_input_1: bool,

    /// # 0x8012:31
    /// Inversion of digital input 2
    ///
    /// default: `false`
    pub invert_digital_input_2: bool,

    /// # 0x8012:32
    /// Selection of the function for input 1
    ///
    /// default: `0x02` (2dec) = Plc cam
    pub function_for_input_1: EL70x1InputFunction,

    /// # 0x8012:36
    /// Selection of the function for input 2
    ///
    /// default: `0x02` (2dec) = Plc cam
    pub function_for_input_2: EL70x1InputFunction,
}

impl Default for StmFeatures {
    fn default() -> Self {
        Self {
            operation_mode: EL70x1OperationMode::Automatic,
            speed_range: EL70x1SpeedRange::Steps2000,
            invert_motor_polarity: false,
            select_info_data_1: EL70x7InfoData::MotorCurrentCoilA,
            select_info_data_2: EL70x7InfoData::MotorCurrentCoilB,
            invert_digital_input_1: false,
            invert_digital_input_2: false,
            function_for_input_1: EL70x1InputFunction::PlcCam,
            function_for_input_2: EL70x1InputFunction::PlcCam,
        }
    }
}

impl StmFeatures {
    pub async fn write_config<'a>(
        &self,
        device: &EthercrabSubDevicePreoperational<'a>,
    ) -> Result<(), anyhow::Error> {
        device
            .sdo_write(0x8012, 0x01, u8::from(self.operation_mode))
            .await?;
        device
            .sdo_write(0x8012, 0x05, u8::from(self.speed_range))
            .await?;
        device
            .sdo_write(0x8012, 0x09, self.invert_motor_polarity)
            .await?;
        device
            .sdo_write(0x8012, 0x11, u8::from(self.select_info_data_1))
            .await?;
        device
            .sdo_write(0x8012, 0x19, u8::from(self.select_info_data_2))
            .await?;
        device
            .sdo_write(0x8012, 0x30, self.invert_digital_input_1)
            .await?;
        device
            .sdo_write(0x8012, 0x31, self.invert_digital_input_2)
            .await?;
        device
            .sdo_write(0x8012, 0x32, u8::from(self.function_for_input_1))
            .await?;
        device
            .sdo_write(0x8012, 0x36, u8::from(self.function_for_input_2))
            .await?;
        Ok(())
    }
}
