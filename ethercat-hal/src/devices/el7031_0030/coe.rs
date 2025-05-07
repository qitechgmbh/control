use crate::{
    coe::{ConfigurableDevice, Configuration},
    pdo::PredefinedPdoAssignment,
    shared_config::{
        el30xx::EL30XXChannelConfiguration,
        el70x1::{
            EncConfiguration, PosConfiguration, PosFeatures, StmControllerConfiguration,
            StmFeatures, StmMotorConfiguration,
        },
    },
    types::EthercrabSubDevicePreoperational,
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
    pub analog_input_channel_1: EL30XXChannelConfiguration,

    /// Analog Channel 2
    pub analog_input_channel_2: EL30XXChannelConfiguration,

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
            analog_input_channel_1: EL30XXChannelConfiguration::default(),
            analog_input_channel_2: EL30XXChannelConfiguration::default(),
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
        self.configuration = config.clone();
        self.txpdo.write_config(device).await?;
        self.rxpdo.write_config(device).await?;
        config.write_config(device).await?;
        Ok(())
    }

    fn get_config(&self) -> EL7031_0030Configuration {
        self.configuration.clone()
    }
}
