use crate::{
    coe::{ConfigurableDevice, Configuration},
    helpers::ethercrab_types::EthercrabSubDevicePreoperational,
    pdo::PredefinedPdoAssignment,
    shared_config::el70x1::{
        EL70x1InfoData, EL70x1InputFunction, EL70x1OperationMode, EL70x1SpeedRange,
        EL7031_0030AnalogInputChannelConfiguration, EncConfiguration, PosConfiguration,
        PosFeatures, StmControllerConfiguration, StmMotorConfiguration,
    },
};

use super::{EL7031_0030, pdo::EL7031_0030PredefinedPdoAssignment};

/// Configuration for EL7031_0030 Stepper Motor Terminal
#[derive(Debug, Clone)]
pub struct EL7031_0030Configuration {
    /// Encoder configuration
    pub encoder: EncConfiguration,

    /// STM motor configuration
    pub stm_motor: StmMotorConfiguration,

    /// STM controller configuration
    pub stm_controller_1: StmControllerConfiguration,

    /// STM controller configuration
    pub stm_controller_2: StmControllerConfiguration,

    /// STM features
    pub stm_features: StmFeatures,

    /// POS configuration
    pub pos_configuration: PosConfiguration,

    /// POS features
    pub pos_features: PosFeatures,

    /// Analog Channel 1
    pub analog_input_channel_1: EL7031_0030AnalogInputChannelConfiguration,

    /// Analog Channel 2
    pub analog_input_channel_2: EL7031_0030AnalogInputChannelConfiguration,

    pub pdo_assignment: EL7031_0030PredefinedPdoAssignment,
}

impl Default for EL7031_0030Configuration {
    /// Defaults according to the datasheet
    fn default() -> Self {
        Self {
            encoder: EncConfiguration::default(),
            stm_motor: StmMotorConfiguration::default(),
            stm_controller_1: StmControllerConfiguration::default(),
            stm_controller_2: StmControllerConfiguration::default(),
            stm_features: StmFeatures::default(),
            pos_configuration: PosConfiguration::default(),
            pos_features: PosFeatures::default(),
            analog_input_channel_1: EL7031_0030AnalogInputChannelConfiguration::default(),
            analog_input_channel_2: EL7031_0030AnalogInputChannelConfiguration::default(),
            pdo_assignment: EL7031_0030PredefinedPdoAssignment::default(),
        }
    }
}

impl Configuration for EL7031_0030Configuration {
    async fn write_config<'a>(
        &self,
        device: &EthercrabSubDevicePreoperational<'a>,
    ) -> Result<(), anyhow::Error> {
        self.encoder.write_config(device).await?;
        self.stm_motor.write_config(device).await?;
        self.stm_controller_1.write_config(device, 0x8011).await?;
        self.stm_controller_2.write_config(device, 0x8013).await?;
        self.stm_features.write_config(device).await?;
        self.pos_configuration.write_config(device).await?;
        self.pos_features.write_config(device).await?;
        self.analog_input_channel_1
            .write_channel_config(device, 0x8030)
            .await?;
        self.analog_input_channel_2
            .write_channel_config(device, 0x8040)
            .await?;
        self.pdo_assignment
            .txpdo_assignment()
            .write_config(device)
            .await?;
        self.pdo_assignment
            .rxpdo_assignment()
            .write_config(device)
            .await?;
        Ok(())
    }
}

impl ConfigurableDevice<EL7031_0030Configuration> for EL7031_0030 {
    async fn write_config<'maindevice>(
        &mut self,
        device: &EthercrabSubDevicePreoperational<'maindevice>,
        config: &EL7031_0030Configuration,
    ) -> Result<(), anyhow::Error> {
        config.write_config(device).await?;
        self.configuration = config.clone();
        self.txpdo = config.pdo_assignment.txpdo_assignment();
        self.rxpdo = config.pdo_assignment.rxpdo_assignment();
        Ok(())
    }

    fn get_config(&self) -> EL7031_0030Configuration {
        self.configuration.clone()
    }
}

/// StmFeatures for the EL7031-0030
///
/// Has two extra fields over the [`crate::shared_config::el70x1::StmFeatures`]
/// for digital input emulation
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

    /// # 0x8012:45
    /// Digital input emulation
    /// - `0` = Off [`EL7031_0030DigitalInputEmulation::Off`]
    /// - `1` = Uin > Limit 1 (threshold) [`EL7031_0030DigitalInputEmulation::Threshold1`]
    /// - `2` = Uin > Limit 2 (threshold) [`EL7031_0030DigitalInputEmulation::Threshold2`]
    /// - `3` = Limit 1 < Uin < Limit 2 (band) [`EL7031_0030DigitalInputEmulation::Band1`]
    /// - `4` = Limit 2 < Uin < Limit 1 (band) [`EL7031_0030DigitalInputEmulation::Band2`]
    /// - `5` = Uin > Limit 1 = 1; Uin < Limit 2 = 0 (hysteresis) [`EL7031_0030DigitalInputEmulation::Hysteresis1`]
    /// - `6` = Uin > Limit 2 = 1; Uin < Limit 1 = 0 (hysteresis) [`EL7031_0030DigitalInputEmulation::Hysteresis2`]
    pub digital_input_emulation_channel_1: EL7031_0030DigitalInputEmulation,

    /// # 0x8012:49
    /// Digital input emulation
    /// - `0` = Off [`EL7031_0030DigitalInputEmulation::Off`]
    /// - `1` = Uin > Limit 1 (threshold) [`EL7031_0030DigitalInputEmulation::Threshold1`]
    /// - `2` = Uin > Limit 2 (threshold) [`EL7031_0030DigitalInputEmulation::Threshold2`]
    /// - `3` = Limit 1 < Uin < Limit 2 (band) [`EL7031_0030DigitalInputEmulation::Band1`]
    /// - `4` = Limit 2 < Uin < Limit 1 (band) [`EL7031_0030DigitalInputEmulation::Band2`]
    /// - `5` = Uin > Limit 1 = 1; Uin < Limit 2 = 0 (hysteresis) [`EL7031_0030DigitalInputEmulation::Hysteresis1`]
    /// - `6` = Uin > Limit 2 = 1; Uin < Limit 1 = 0 (hysteresis) [`EL7031_0030DigitalInputEmulation::Hysteresis2`]
    pub digital_input_emulation_channel_2: EL7031_0030DigitalInputEmulation,
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
            digital_input_emulation_channel_1: EL7031_0030DigitalInputEmulation::default(),
            digital_input_emulation_channel_2: EL7031_0030DigitalInputEmulation::default(),
        }
    }
}

impl StmFeatures {
    pub async fn write_config<'a>(
        &self,
        device: &EthercrabSubDevicePreoperational<'a>,
    ) -> Result<(), anyhow::Error> {
        device.sdo_write(0x8012, 0x01, 0u8).await?;
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
        device
            .sdo_write(
                0x8012,
                0x45,
                u8::from(self.digital_input_emulation_channel_1.clone()),
            )
            .await?;
        device
            .sdo_write(
                0x8012,
                0x49,
                u8::from(self.digital_input_emulation_channel_2.clone()),
            )
            .await?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub enum EL7031_0030DigitalInputEmulation {
    #[default]
    Off,
    /// Uin > Limit 1 (threshold)
    Threshold1,
    /// Uin > Limit 2 (threshold)
    Threshold2,
    /// Limit 1 < Uin < Limit 2 (band)
    Band1,
    /// Limit 2 < Uin < Limit 1 (band)
    Band2,
    /// Uin > Limit 1 = 1;
    /// Uin < Limit 2 = 0
    /// (hysteresis)
    Hysteresis1,
    /// Uin > Limit 2 = 1;
    /// Uin < Limit 1 = 0
    /// (hysteresis)
    Hysteresis2,
}

impl TryFrom<u8> for EL7031_0030DigitalInputEmulation {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Off),
            1 => Ok(Self::Threshold1),
            2 => Ok(Self::Threshold2),
            3 => Ok(Self::Band1),
            4 => Ok(Self::Band2),
            5 => Ok(Self::Hysteresis1),
            6 => Ok(Self::Hysteresis2),
            _ => Err(anyhow::anyhow!(
                "Invalid value for EL7031_0030DigitalInputEmulation: {}",
                value
            )),
        }
    }
}

impl From<EL7031_0030DigitalInputEmulation> for u8 {
    fn from(value: EL7031_0030DigitalInputEmulation) -> Self {
        match value {
            EL7031_0030DigitalInputEmulation::Off => 0,
            EL7031_0030DigitalInputEmulation::Threshold1 => 1,
            EL7031_0030DigitalInputEmulation::Threshold2 => 2,
            EL7031_0030DigitalInputEmulation::Band1 => 3,
            EL7031_0030DigitalInputEmulation::Band2 => 4,
            EL7031_0030DigitalInputEmulation::Hysteresis1 => 5,
            EL7031_0030DigitalInputEmulation::Hysteresis2 => 6,
        }
    }
}
