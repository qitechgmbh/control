use anyhow;

use crate::helpers::ethercrab_types::EthercrabSubDevicePreoperational;

#[derive(Debug, Clone)]
pub struct EncConfiguration {
    /// # 8000:0E
    /// Activates reversion of rotation of the encoder.
    ///
    /// default: `false`
    pub reversion_of_rotation: bool,
}

impl Default for EncConfiguration {
    /// Defaults according to the datasheet
    fn default() -> Self {
        Self {
            reversion_of_rotation: false,
        }
    }
}

impl EncConfiguration {
    pub async fn write_config<'a>(
        &self,
        device: &EthercrabSubDevicePreoperational<'a>,
    ) -> Result<(), anyhow::Error> {
        device
            .sdo_write(0x8000, 0x0E, self.reversion_of_rotation)
            .await?;
        Ok(())
    }
}

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
    /// Nominal voltage (unit: 1 mV)
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
    /// default: `0x0000` (0dec) = 0 mH
    pub motor_emf: u16,

    /// # 0x8010:06
    /// Motor full steps (unit: 1 step)
    ///
    /// default: `0x00C8` (200dec) = 200 steps
    pub motor_full_steps: u16,

    /// # 0x8010:09
    /// Maximum possible start velocity of the motor
    ///
    /// default: `0x0000` (0dec) = 0
    pub start_velocity: u16,

    /// # 0x8010:10
    /// Switch-on delay of the driver stage
    ///
    /// default: `0064` (100dec) = 0.1s
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
        device.sdo_write(0x8010, 0x03, self.nominal_voltage).await?;
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

#[derive(Debug, Clone)]
pub struct StmControllerConfiguration {
    /// # 8011:01 / 8013:01
    /// Kp control factor (proportional component) for the current controll (unit: 0.001)
    ///
    /// default: `0x0190` (400dec) = 0.4
    pub kp_factor: u16,

    /// # 8011:02 / 8013:02
    /// Ki control factor (integral component) for the current controll (unit: 0.001)
    ///
    /// default: `0x0004` (4dec) = 0.004
    pub ki_factor: u16,

    /// # 8011:03 / 8013:03
    /// Inner window for the I component of the current controller (unit: 1%)
    ///
    /// default: `0x0000` (0dec) = 0%
    pub inner_window: u8,

    /// # 8011:05 / 8013:05
    /// Outer window for the I component of the current controller (unit: 1%)
    ///
    /// default: `0x0000` (0dec) = 0%
    pub outer_window: u8,

    /// # 8011:06 / 8013:06
    /// Filter limit frequency of the current controller (low-pass, unit: 1 Hz)
    ///
    /// default (0x0000) = 0 Hz
    pub filter_cutoff_frequency: u16,

    /// # 8011:07 / 8013:07
    /// Ka control factor (acceleration component) for the current controller(unit: 0.001)
    ///
    /// default: `0x0064` (100dec) = 0.1
    pub ka_factor: u16,

    /// # 8011:08 / 8013:08
    /// Kd control factor (deceleration component) for the current controller(unit: 0.001)
    ///
    /// default: `0x0064` (100dec) = 0.1
    pub kd_factor: u16,
}

impl Default for StmControllerConfiguration {
    /// Defaults according to the datasheet
    fn default() -> Self {
        Self {
            kp_factor: 0x0190,               // 400 mA = 0.4
            ki_factor: 0x0004,               // 4 mA = 0.004
            inner_window: 0x0000,            // 0%
            outer_window: 0x0000,            // 0%
            filter_cutoff_frequency: 0x0000, // 0 Hz
            ka_factor: 0x0064,               // 100 mA = 0.1
            kd_factor: 0x0064,               // 100 mA = 0.1
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
        device
            .sdo_write(base_index, 0x03, self.inner_window)
            .await?;
        // device
        //     .sdo_write(base_index, 0x05, self.outer_window)
        //     .await?;
        // device
        //     .sdo_write(base_index, 0x06, self.filter_cutoff_frequency)
        //     .await?;
        // device.sdo_write(base_index, 0x07, self.ka_factor).await?;
        // device.sdo_write(base_index, 0x08, self.kd_factor).await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct StmFeatures {
    /// # 0x8012:01
    /// Operating mode
    /// - `0` = Automatic
    /// - `1` = Direct velocity
    /// - `2` = Velocity controller
    /// - `3` = Position controller
    ///
    /// default: `0x00` (0dec) = Automatic
    pub operation_mode: EL70x1OperationMode,

    /// # 0x8012:05
    /// Preselection of the speed range
    /// - `0` = 1000 full steps/second
    /// - `1` = 2000 full steps/second
    /// - `2` = 4000 full steps/second
    /// - `3` = 8000 full steps/second
    /// - `4` = 16000 full steps/second
    /// - `5` = 32000 full steps/second
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
    pub select_info_data_1: EL70x1InfoData,

    /// # 0x8012:19
    /// Selection "Info data 2"
    ///
    /// default: `0x04` (4dec) = Motor current coil B
    pub select_info_data_2: EL70x1InfoData,

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
    /// - `0` = Normal input
    /// - `1` = Hardware Enable
    /// - `2` = Plc cam
    /// - `3` = Auto start
    ///
    /// default: `0x02` (2dec) = Plc cam
    pub function_for_input_1: EL70x1InputFunction,

    /// # 0x8012:36
    /// Selection of the function for input 2
    /// - `0` = Normal input
    /// - `1` = Hardware Enable
    /// - `2` = Plc cam
    /// - `3` = Auto start
    ///
    /// default: `0x02` (2dec) = Plc cam
    pub function_for_input_2: EL70x1InputFunction,
}

impl Default for StmFeatures {
    /// Defaults according to the datasheet
    fn default() -> Self {
        Self {
            operation_mode: EL70x1OperationMode::Automatic,
            speed_range: EL70x1SpeedRange::Steps2000,
            invert_motor_polarity: false,
            select_info_data_1: EL70x1InfoData::MotorCurrentCoilA,
            select_info_data_2: EL70x1InfoData::MotorCurrentCoilB,
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

#[derive(Debug, Clone)]
pub struct PosConfiguration {
    /// # 0x8020:01
    /// Minimum set velocity
    /// (range: 0-10000)
    ///
    /// default: `0x0064` (100dec)
    pub velocity_min: i16,

    /// # 0x8020:02
    /// Maximum set velocity
    /// (range: 0-10000)
    ///
    /// default: `0x2710` (10000dec)
    pub velocity_max: i16,

    /// # 0x8020:03
    /// Acceleration in positive direction of rotation
    /// (unit: 1 ms)
    ///
    /// default: `0x03E8` (1000dec)
    pub acceleration_pos: u16,

    /// # 0x8020:04
    /// Acceleration in negative direction of rotation
    /// (unit: 1 ms)
    ///
    /// default: `0x03E8` (1000dec)
    pub acceleration_neg: u16,

    /// # 0x8020:05
    /// Deceleration in positive direction of rotation
    /// (unit: 1 ms)
    ///
    /// default: `0x03E8` (1000dec)
    pub deceleration_pos: u16,

    /// # 0x8020:06
    /// Deceleration in negative direction of rotation
    /// (unit: 1 ms)
    ///
    /// default: `0x03E8` (1000dec)
    pub deceleration_neg: u16,

    /// # 0x8020:07
    /// Emergency deceleration (both directions of rotation,
    /// unit: 1 ms)
    ///
    /// default: `0x0064` (100dec)
    pub emergency_deceleration: u16,

    /// # 0x8020:08
    /// Calibration position
    ///
    /// default: `0x00000000` (0dec)
    pub calibration_position: u32,

    /// # 0x8020:09
    /// Calibration velocity towards the cam
    /// (range: 0-10000)
    ///
    /// default: `0x0064` (100dec)
    pub calibration_velocity_towards_cam: i16,

    /// # 0x8020:0A
    /// Calibration velocity away from the cam
    /// (range: 0-10000)
    ///
    /// default: `0x000A` (10dec)
    pub calibration_velocity_off_cam: i16,

    /// # 0x8020:0B
    /// Target window
    ///
    /// default: `0x000A` (10dec)
    pub target_window: u16,

    /// # 0x8020:0C
    /// Timeout at target position
    /// (unit: 1 ms)
    ///
    /// default: `0x03E8` (1000dec)
    pub in_target_timeout: u16,

    /// # 0x8020:0D
    /// Dead time compensation
    /// (unit: 1 μs)
    ///
    /// default: `0x0032` (50dec)
    pub dead_time_compensation: i16,

    /// # 0x8020:0E
    /// Modulo factor/position
    ///
    /// default: `0x00000000` (0dec)
    pub modulo_factor: u32,

    /// # 0x8020:0F
    /// Tolerance window for modulo positioning
    ///
    /// default: `0x00000000` (0dec)
    pub modulo_tolerance_window: u32,

    /// # 0x8020:10
    /// max. position lag
    ///
    /// default: `0x0000` (0dec)
    pub position_lag_max: u16,
}

impl Default for PosConfiguration {
    /// Defaults according to the datasheet
    fn default() -> Self {
        Self {
            velocity_min: 0x0064,                     // 100
            velocity_max: 0x2710,                     // 10000
            acceleration_pos: 0x03E8,                 // 1000
            acceleration_neg: 0x03E8,                 // 1000
            deceleration_pos: 0x03E8,                 // 1000
            deceleration_neg: 0x03E8,                 // 1000
            emergency_deceleration: 0x0064,           // 100
            calibration_position: 0x00000000,         // 0
            calibration_velocity_towards_cam: 0x0064, // 100
            calibration_velocity_off_cam: 0x000A,     // 10
            target_window: 0x000A,                    // 10
            in_target_timeout: 0x03E8,                // 1000
            dead_time_compensation: 0x0032,           // 50
            modulo_factor: 0x00000000,                // 0
            modulo_tolerance_window: 0x00000000,      // 0
            position_lag_max: 0x0000,                 // 0
        }
    }
}

impl PosConfiguration {
    pub async fn write_config<'a>(
        &self,
        device: &EthercrabSubDevicePreoperational<'a>,
    ) -> Result<(), anyhow::Error> {
        device.sdo_write(0x8020, 0x01, self.velocity_min).await?;
        device.sdo_write(0x8020, 0x02, self.velocity_max).await?;
        device
            .sdo_write(0x8020, 0x03, self.acceleration_pos)
            .await?;
        device
            .sdo_write(0x8020, 0x04, self.acceleration_neg)
            .await?;
        device
            .sdo_write(0x8020, 0x05, self.deceleration_pos)
            .await?;
        device
            .sdo_write(0x8020, 0x06, self.deceleration_neg)
            .await?;
        device
            .sdo_write(0x8020, 0x07, self.emergency_deceleration)
            .await?;
        device
            .sdo_write(0x8020, 0x08, self.calibration_position)
            .await?;
        device
            .sdo_write(0x8020, 0x09, self.calibration_velocity_towards_cam)
            .await?;
        device
            .sdo_write(0x8020, 0x0A, self.calibration_velocity_off_cam)
            .await?;
        device.sdo_write(0x8020, 0x0B, self.target_window).await?;
        device
            .sdo_write(0x8020, 0x0C, self.in_target_timeout)
            .await?;
        device
            .sdo_write(0x8020, 0x0D, self.dead_time_compensation)
            .await?;
        device.sdo_write(0x8020, 0x0E, self.modulo_factor).await?;
        device
            .sdo_write(0x8020, 0x0F, self.modulo_tolerance_window)
            .await?;
        device
            .sdo_write(0x8020, 0x10, self.position_lag_max)
            .await?;
        Ok(())
    }
}

/// Start type values for POS Features
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartType {
    /// Idle
    Idle = 0,
    /// Absolute positioning
    Absolute = 1,
    /// Relative positioning
    Relative = 2,
    /// Endless plus (continuous movement in positive direction)
    EndlessPlus = 3,
    /// Endless minus (continuous movement in negative direction)
    EndlessMinus = 4,
    /// Additive positioning
    Additive = 6,
    /// Modulo current
    ModuloCurrent = 1029,
    /// Modulo minus
    ModuloMinus = 773,
    /// Modulo plus
    ModuloPlus = 517,
    /// Modulo short
    ModuloShort = 261,
    /// Calibration (Hardware sync)
    CalibrationHardwareSync = 24832,
    /// Calibration (Plc cam)
    CalibrationPlcCam = 24576,
    /// Calibration (Clear manual)
    CalibrationClearManual = 28416,
    /// Calibration (Set manual)
    CalibrationSetManual = 28160,
    /// Calibration (Set manual auto)
    CalibrationSetManualAuto = 28161,
}

impl TryFrom<u16> for StartType {
    type Error = anyhow::Error;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Idle),
            1 => Ok(Self::Absolute),
            2 => Ok(Self::Relative),
            3 => Ok(Self::EndlessPlus),
            4 => Ok(Self::EndlessMinus),
            6 => Ok(Self::Additive),
            1029 => Ok(Self::ModuloCurrent),
            773 => Ok(Self::ModuloMinus),
            517 => Ok(Self::ModuloPlus),
            261 => Ok(Self::ModuloShort),
            24832 => Ok(Self::CalibrationHardwareSync),
            24576 => Ok(Self::CalibrationPlcCam),
            28416 => Ok(Self::CalibrationClearManual),
            28160 => Ok(Self::CalibrationSetManual),
            28161 => Ok(Self::CalibrationSetManualAuto),
            _ => Err(anyhow::anyhow!("Invalid value for StartType: {}", value)),
        }
    }
}

impl From<StartType> for u16 {
    fn from(start_type: StartType) -> Self {
        start_type as Self
    }
}

#[derive(Debug, Clone)]
/// POS Features for Index 8021
pub struct PosFeatures {
    /// # 0x8021:01
    /// Start type
    ///
    /// default: `0x0001` (1dec) = Absolute
    pub start_type: StartType,

    /// # 0x8021:11
    /// Time information
    /// permitted values:
    /// - `0` = Elapsed time
    ///
    /// current drive time since start of the travel command
    ///
    /// default: `0x00` (0dec)
    pub time_information: u8, // Using u8 for BIT2 type

    /// # 0x8021:13
    /// Invert calibration cam search direction
    /// Inversion of the direction of rotation towards the cam
    ///
    /// default: `0x01` (1dec)
    pub invert_calibration_cam_search_direction: bool,

    /// # 0x8021:14
    /// Invert sync impulse search direction
    /// Inversion of the direction of rotation away from the cam
    ///
    /// default: `0x00` (0dec)
    pub invert_sync_impulse_search_direction: bool,

    /// # 0x8021:15
    /// Emergency stop on position lag error
    /// Triggers an emergency stop if the maximum following
    /// error is exceeded
    ///
    /// default: `0x00` (0dec)
    pub emergency_stop_on_position_lag_error: bool,

    /// # 0x8021:16
    /// Enhanced diag history
    /// Provides detailed messages about the status of the
    /// positioning interface in the diag history
    ///
    /// default: `0x00` (0dec)
    pub enhanced_diag_history: bool,
}

impl Default for PosFeatures {
    /// Defaults according to the datasheet
    fn default() -> Self {
        Self {
            start_type: StartType::Absolute,
            time_information: 0x00,
            invert_calibration_cam_search_direction: false,
            invert_sync_impulse_search_direction: false,
            emergency_stop_on_position_lag_error: false,
            enhanced_diag_history: false,
        }
    }
}

impl PosFeatures {
    pub async fn write_config<'a>(
        &self,
        device: &EthercrabSubDevicePreoperational<'a>,
    ) -> Result<(), anyhow::Error> {
        device
            .sdo_write(0x8021, 0x01, u16::from(self.start_type))
            .await?;
        device
            .sdo_write(0x8021, 0x11, self.time_information)
            .await?;
        device
            .sdo_write(0x8021, 0x13, self.invert_calibration_cam_search_direction)
            .await?;
        device
            .sdo_write(0x8021, 0x14, self.invert_sync_impulse_search_direction)
            .await?;
        device
            .sdo_write(0x8021, 0x15, self.emergency_stop_on_position_lag_error)
            .await?;
        device
            .sdo_write(0x8021, 0x16, self.enhanced_diag_history)
            .await?;
        Ok(())
    }
}

/// Operation mode for EL7031
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EL70x1OperationMode {
    /// Automatic
    Automatic = 0,
    /// Direct velocity
    DirectVelocity = 1,
    /// Velocity controller
    VelocityController = 2,
    /// Position controller
    PositionController = 3,
}

impl std::fmt::Debug for EL70x1OperationMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Automatic => write!(f, "Automatic (0)"),
            Self::DirectVelocity => write!(f, "DirectVelocity (1)"),
            Self::VelocityController => write!(f, "VelocityController (2)"),
            Self::PositionController => write!(f, "PositionController (3)"),
        }
    }
}

impl TryFrom<u8> for EL70x1OperationMode {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Automatic),
            1 => Ok(Self::DirectVelocity),
            2 => Ok(Self::VelocityController),
            3 => Ok(Self::PositionController),
            _ => Err(anyhow::anyhow!(
                "Invalid value for EL7031OperationMode: {}",
                value
            )),
        }
    }
}

impl From<EL70x1OperationMode> for u8 {
    fn from(value: EL70x1OperationMode) -> Self {
        value as Self
    }
}

/// Speed range for EL7031
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EL70x1SpeedRange {
    /// 1000 full steps/second
    Steps1000 = 0,
    /// 2000 full steps/second
    Steps2000 = 1,
    /// 4000 full steps/second
    Steps4000 = 2,
    /// 8000 full steps/second
    Steps8000 = 3,
    /// 16000 full steps/second
    Steps16000 = 4,
    /// 32000 full steps/second
    Steps32000 = 5,
}

impl std::fmt::Debug for EL70x1SpeedRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Steps1000 => write!(f, "1000 steps/s (0)"),
            Self::Steps2000 => write!(f, "2000 steps/s (1)"),
            Self::Steps4000 => write!(f, "4000 steps/s (2)"),
            Self::Steps8000 => write!(f, "8000 steps/s (3)"),
            Self::Steps16000 => write!(f, "16000 steps/s (4)"),
            Self::Steps32000 => write!(f, "32000 steps/s (5)"),
        }
    }
}

impl TryFrom<u8> for EL70x1SpeedRange {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Steps1000),
            1 => Ok(Self::Steps2000),
            2 => Ok(Self::Steps4000),
            3 => Ok(Self::Steps8000),
            4 => Ok(Self::Steps16000),
            5 => Ok(Self::Steps32000),
            _ => Err(anyhow::anyhow!(
                "Invalid value for EL7031SpeedRange: {}",
                value
            )),
        }
    }
}

impl From<EL70x1SpeedRange> for u8 {
    fn from(value: EL70x1SpeedRange) -> Self {
        value as Self
    }
}

/// Info data selection for EL7031
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EL70x1InfoData {
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

impl std::fmt::Debug for EL70x1InfoData {
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

impl TryFrom<u8> for EL70x1InfoData {
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
            101 => Ok(Self::InternalTemperature),
            103 => Ok(Self::ControlVoltage),
            104 => Ok(Self::MotorSupplyVoltage),
            150 => Ok(Self::DriveStatusWord),
            151 => Ok(Self::DriveState),
            152 => Ok(Self::DrivePositionLagLow),
            153 => Ok(Self::DrivePositionLagHigh),
            _ => Err(anyhow::anyhow!(
                "Invalid value for EL7031InfoData: {}",
                value
            )),
        }
    }
}

impl From<EL70x1InfoData> for u8 {
    fn from(value: EL70x1InfoData) -> Self {
        value as Self
    }
}

/// Input function for EL7031
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EL70x1InputFunction {
    /// Normal input
    NormalInput = 0,
    /// Hardware Enable
    HardwareEnable = 1,
    /// Plc cam
    PlcCam = 2,
    /// Auto start
    AutoStart = 3,
}

impl std::fmt::Debug for EL70x1InputFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NormalInput => write!(f, "NormalInput (0)"),
            Self::HardwareEnable => write!(f, "HardwareEnable (1)"),
            Self::PlcCam => write!(f, "PlcCam (2)"),
            Self::AutoStart => write!(f, "AutoStart (3)"),
        }
    }
}

impl TryFrom<u8> for EL70x1InputFunction {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::NormalInput),
            1 => Ok(Self::HardwareEnable),
            2 => Ok(Self::PlcCam),
            3 => Ok(Self::AutoStart),
            _ => Err(anyhow::anyhow!(
                "Invalid value for EL7031InputFunction: {}",
                value
            )),
        }
    }
}

impl From<EL70x1InputFunction> for u8 {
    fn from(value: EL70x1InputFunction) -> Self {
        value as Self
    }
}

#[derive(Debug, Clone)]
pub struct EL7031_0030AnalogInputChannelConfiguration {
    // 80n0:01 User Scaling is Active
    pub enable_user_scale: bool,

    // 80n0:06
    // Enable filter, which makes PLC-cycle-synchronous
    // data exchange unnecessary
    pub enable_filter: bool,

    // 80n0:07
    // limit 1 enabled
    pub enable_limit_1: bool,

    // 80n0:08
    // limit2 enabled
    pub enable_limit_2: bool,

    // 80n0:11
    // User Scaling Offset
    pub user_scale_offset: i16,

    // 80n0:12
    // Gain of the user scaling
    // The gain has a fixed-point-representation with the factor
    // 2^-16 The value 1 corresponds to 65535 and is limited to +/- 0x7FFFFF
    pub user_scale_gain: i32,

    // 80n0:13
    // First limit value for setting the status bits
    pub limit_1: i16,

    // 80n0:14
    // Second limit value for setting the status bits
    pub limit_2: i16,

    // 80n0:15
    /*
       This object determines the digital filter settings if it is
       active via Enable filter (Index base_index:06 [} 320]).
       The possible settings are numbered consecutively.
       2: IIR 1
       3: IIR 2
       ...
       9: IIR 8
    */
    pub filter_settings: EL7031_0030AnalogInputFilterSettings,
}

impl EL7031_0030AnalogInputChannelConfiguration {
    pub async fn write_channel_config<'a>(
        &self,
        device: &EthercrabSubDevicePreoperational<'a>,
        base_index: u16,
    ) -> Result<(), anyhow::Error> {
        device
            .sdo_write(base_index, 0x01, self.enable_user_scale)
            .await?;
        device
            .sdo_write(base_index, 0x06, self.enable_filter)
            .await?;
        device
            .sdo_write(base_index, 0x07, self.enable_limit_1)
            .await?;
        device
            .sdo_write(base_index, 0x08, self.enable_limit_2)
            .await?;
        device
            .sdo_write(base_index, 0x11, self.user_scale_offset)
            .await?;
        device
            .sdo_write(base_index, 0x12, self.user_scale_gain)
            .await?;
        device.sdo_write(base_index, 0x13, self.limit_1).await?;
        device.sdo_write(base_index, 0x14, self.limit_2).await?;
        device
            .sdo_write(base_index, 0x15, u16::from(self.filter_settings))
            .await?;
        Ok(())
    }
}

impl Default for EL7031_0030AnalogInputChannelConfiguration {
    fn default() -> Self {
        Self {
            enable_user_scale: false,
            enable_filter: true,
            enable_limit_1: false,
            enable_limit_2: false,
            user_scale_offset: 0,
            user_scale_gain: 65536,
            limit_1: 0,
            limit_2: 0,
            filter_settings: EL7031_0030AnalogInputFilterSettings::IIR1,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EL7031_0030AnalogInputFilterSettings {
    IIR1,
    IIR2,
    IIR3,
    IIR4,
    IIR5,
    IIR6,
    IIR7,
    IIR8,
}

impl From<EL7031_0030AnalogInputFilterSettings> for u16 {
    fn from(filter_settings: EL7031_0030AnalogInputFilterSettings) -> Self {
        match filter_settings {
            EL7031_0030AnalogInputFilterSettings::IIR1 => 2,
            EL7031_0030AnalogInputFilterSettings::IIR2 => 3,
            EL7031_0030AnalogInputFilterSettings::IIR3 => 4,
            EL7031_0030AnalogInputFilterSettings::IIR4 => 5,
            EL7031_0030AnalogInputFilterSettings::IIR5 => 6,
            EL7031_0030AnalogInputFilterSettings::IIR6 => 7,
            EL7031_0030AnalogInputFilterSettings::IIR7 => 8,
            EL7031_0030AnalogInputFilterSettings::IIR8 => 9,
        }
    }
}
