use crate::{
    coe::{ConfigurableDevice, Configuration},
    helpers::ethercrab_types::EthercrabSubDevicePreoperational,
    pdo::PredefinedPdoAssignment,
    shared_config::el70x1::{
        EL70x1InfoData, EL70x1InputFunction, EL70x1OperationMode, EL70x1SpeedRange,
        EncConfiguration, PosConfiguration, PosFeatures, StmMotorConfiguration,
    },
};

use super::{EL7037, pdo::EL7037PredefinedPdoAssignment};

/// STM controller configuration for EL7037.
///
/// Index 0x8011 on the EL7037 only exposes subindices 0x01 (Kp) and 0x02 (Ki)
/// (max subindex = 0x02), unlike the EL7031 which has additional subindices.
/// There is also no second controller at 0x8013 on this terminal.
#[derive(Debug, Clone)]
pub struct StmControllerConfiguration {
    /// # 0x8011:01
    /// Kp control factor (unit: 0.001)
    ///
    /// default: `0x0190` (400dec)
    pub kp_factor: u16,

    /// # 0x8011:02
    /// Ki control factor (unit: 0.001)
    ///
    /// default: `0x0004` (4dec)
    pub ki_factor: u16,
}

impl Default for StmControllerConfiguration {
    fn default() -> Self {
        Self {
            kp_factor: 0x0190,
            ki_factor: 0x0004,
        }
    }
}

impl StmControllerConfiguration {
    pub async fn write_config<'a>(
        &self,
        device: &EthercrabSubDevicePreoperational<'a>,
    ) -> Result<(), anyhow::Error> {
        device.sdo_write(0x8011, 0x01, self.kp_factor).await?;
        device.sdo_write(0x8011, 0x02, self.ki_factor).await?;
        Ok(())
    }
}

/// STM features for EL7037.
///
/// Writes operation_mode (0x8012:01) explicitly. The shared [`crate::shared_config::el70x1::StmFeatures`]
/// does not write this subindex; the EL7037 requires it to be set for velocity/position modes.
#[derive(Debug, Clone)]
pub struct StmFeatures {
    /// # 0x8012:01
    /// Operating mode
    /// - `0` = Automatic
    /// - `1` = Velocity direct
    /// - `3` = Position controller
    /// - `4` = Ext. Velocity mode
    /// - `5` = Ext. Position mode
    /// - `6` = Velocity sensorless
    ///
    /// default: `0x00` = Automatic
    pub operation_mode: EL70x1OperationMode,

    /// # 0x8012:05
    /// Speed range
    ///
    /// default: `0x01` = 2000 full steps/second
    pub speed_range: EL70x1SpeedRange,

    /// # 0x8012:09
    /// Invert motor polarity
    ///
    /// default: `false`
    pub invert_motor_polarity: bool,

    /// # 0x8012:11
    /// Select info data 1
    ///
    /// default: `MotorCurrentCoilA`
    pub select_info_data_1: EL70x1InfoData,

    /// # 0x8012:19
    /// Select info data 2
    ///
    /// default: `MotorCurrentCoilB`
    pub select_info_data_2: EL70x1InfoData,

    /// # 0x8012:30
    /// Invert digital input 1
    ///
    /// default: `false`
    pub invert_digital_input_1: bool,

    /// # 0x8012:31
    /// Invert digital input 2
    ///
    /// default: `false`
    pub invert_digital_input_2: bool,

    /// # 0x8012:32
    /// Function for input 1
    ///
    /// default: `PlcCam`
    pub function_for_input_1: EL70x1InputFunction,

    /// # 0x8012:36
    /// Function for input 2
    ///
    /// default: `PlcCam`
    pub function_for_input_2: EL70x1InputFunction,
}

impl Default for StmFeatures {
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

/// Configuration for EL7037 Stepper Motor Terminal
#[derive(Debug, Clone)]
pub struct EL7037Configuration {
    /// Encoder configuration
    pub encoder: EncConfiguration,

    /// STM motor configuration
    pub stm_motor: StmMotorConfiguration,

    /// STM controller configuration (index 0x8011, subindices 0x01–0x02 only)
    pub stm_controller: StmControllerConfiguration,

    /// STM features
    pub stm_features: StmFeatures,

    /// POS configuration
    pub pos_configuration: PosConfiguration,

    /// POS features
    pub pos_features: PosFeatures,

    pub pdo_assignment: EL7037PredefinedPdoAssignment,
}

impl Default for EL7037Configuration {
    /// Defaults according to the datasheet
    fn default() -> Self {
        Self {
            encoder: EncConfiguration::default(),
            stm_motor: StmMotorConfiguration::default(),
            stm_controller: StmControllerConfiguration::default(),
            stm_features: StmFeatures::default(),
            pos_configuration: PosConfiguration::default(),
            pos_features: PosFeatures::default(),
            pdo_assignment: EL7037PredefinedPdoAssignment::default(),
        }
    }
}

impl Configuration for EL7037Configuration {
    async fn write_config<'a>(
        &self,
        device: &EthercrabSubDevicePreoperational<'a>,
    ) -> Result<(), anyhow::Error> {
        self.encoder.write_config(device).await?;
        self.stm_motor.write_config(device).await?;
        self.stm_controller.write_config(device).await?;
        self.stm_features.write_config(device).await?;
        self.pos_configuration.write_config(device).await?;
        self.pos_features.write_config(device).await?;
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

impl ConfigurableDevice<EL7037Configuration> for EL7037 {
    async fn write_config<'maindevice>(
        &mut self,
        device: &EthercrabSubDevicePreoperational<'maindevice>,
        config: &EL7037Configuration,
    ) -> Result<(), anyhow::Error> {
        config.write_config(device).await?;
        self.configuration = config.clone();
        self.txpdo = config.pdo_assignment.txpdo_assignment();
        self.rxpdo = config.pdo_assignment.rxpdo_assignment();
        Ok(())
    }

    fn get_config(&self) -> EL7037Configuration {
        self.configuration.clone()
    }
}
