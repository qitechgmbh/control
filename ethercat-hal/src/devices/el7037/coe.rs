use crate::{
    coe::{ConfigurableDevice, Configuration},
    helpers::ethercrab_types::EthercrabSubDevicePreoperational,
    pdo::PredefinedPdoAssignment,
    shared_config::el70x1::{
        EL70x1InfoData, EncConfiguration, PosConfiguration, PosFeatures, StmControllerConfiguration,
        StmFeatures, StmMotorConfiguration,
    },
};

use super::{pdo::EL7037PredefinedPdoAssignment, EL7037};

/// Configuration for EL7037 Stepper Motor Terminal
#[derive(Debug, Clone)]
pub struct EL7037Configuration {
    /// Encoder configuration
    pub encoder: EncConfiguration,

    /// STM motor configuration
    pub stm_motor: StmMotorConfiguration,

    /// STM controller configuration
    pub stm_controller_1: StmControllerConfiguration,

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
            stm_controller_1: StmControllerConfiguration::default(),
            stm_features: StmFeatures {
                select_info_data_1: EL70x1InfoData::MotorLoad,
                select_info_data_2: EL70x1InfoData::MotorDcCurrent,
                ..Default::default()
            },
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
        // EL7037 uses 10 mV units for nominal_voltage (0x8010:03),
        // unlike EL7031 which uses 1 mV. Write fields individually.
        // NOTE: 0x8010:03 (nominal_voltage) is skipped — EL7037 rejects SDO writes.
        //       The terminal default (5000 = 50V in 10mV units) is acceptable.
        device.sdo_write(0x8010, 0x01, self.stm_motor.max_current).await?;
        device.sdo_write(0x8010, 0x02, self.stm_motor.reduced_current).await?;
        device.sdo_write(0x8010, 0x04, self.stm_motor.motor_coil_resistance).await?;
        device.sdo_write(0x8010, 0x05, self.stm_motor.motor_emf).await?;
        device.sdo_write(0x8010, 0x06, self.stm_motor.motor_full_steps).await?;
        device.sdo_write(0x8010, 0x09, self.stm_motor.start_velocity).await?;
        device.sdo_write(0x8010, 0x10, self.stm_motor.drive_on_delay_time).await?;
        device.sdo_write(0x8010, 0x11, self.stm_motor.drive_off_delay_time).await?;
        self.stm_controller_1.write_config(device, 0x8011).await?;
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
