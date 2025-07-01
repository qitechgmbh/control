use super::EthercatDeviceProcessing;
use super::{NewEthercatDevice, SubDeviceIdentityTuple};
use crate::pdo::RxPdo;
use crate::pdo::TxPdo;
use crate::{
    coe::{ConfigurableDevice, Configuration},
    pdo::{PredefinedPdoAssignment, el40xx::AnalogOutput},
    shared_config::el40xx::EL40XXChannelConfiguration,
};
use crate::{
    helpers::ethercrab_types::EthercrabSubDevicePreoperational,
    io::analog_output::{AnalogOutputDevice, AnalogOutputOutput, AnalogOutputState},
};
use ethercat_hal_derive::{EthercatDevice, RxPdo, TxPdo};

#[derive(Debug, Clone)]
pub struct EL4002Configuration {
    pub pdo_assignment: EL4002PredefinedPdoAssignment,
    // Output1+ and Output1-
    pub channel1: EL40XXChannelConfiguration,
    // Output2+ and Output2-
    pub channel2: EL40XXChannelConfiguration,
}

#[derive(EthercatDevice)]
pub struct EL4002 {
    pub configuration: EL4002Configuration,
    pub txpdo: EL4002TxPdo,
    pub rxpdo: EL4002RxPdo,
    is_used: bool,
}

impl EthercatDeviceProcessing for EL4002 {}

impl std::fmt::Debug for EL4002 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EL4002")
    }
}

impl Default for EL4002PredefinedPdoAssignment {
    fn default() -> Self {
        Self::Standard
    }
}

impl Default for EL4002Configuration {
    fn default() -> Self {
        Self {
            pdo_assignment: EL4002PredefinedPdoAssignment::Standard,
            channel1: EL40XXChannelConfiguration::default(),
            channel2: EL40XXChannelConfiguration::default(),
        }
    }
}

impl NewEthercatDevice for EL4002 {
    fn new() -> Self {
        let configuration = EL4002Configuration::default();
        Self {
            configuration: configuration.clone(),
            txpdo: configuration.pdo_assignment.txpdo_assignment(),
            rxpdo: configuration.pdo_assignment.rxpdo_assignment(),
            is_used: false,
        }
    }
}

impl AnalogOutputDevice<EL4002Port> for EL4002 {
    fn analog_output_state(&self, port: EL4002Port) -> AnalogOutputState {
        let raw_value = match port {
            EL4002Port::AO1 => {
                self.rxpdo
                    .channel1
                    .as_ref()
                    .expect("Channel 1 should be configured")
                    .value
            }
            EL4002Port::AO2 => {
                self.rxpdo
                    .channel2
                    .as_ref()
                    .expect("Channel 2 should be configured")
                    .value
            }
        };

        // Convert raw i16 value to normalized output (0.0 to 1.0)
        let normalized = (raw_value as f32) / (i16::MAX as f32);

        AnalogOutputState {
            output: AnalogOutputOutput(normalized),
        }
    }

    fn analog_output_write(&mut self, port: EL4002Port, output: AnalogOutputOutput) {
        // Convert normalized value (0.0 to 1.0) to raw i16 value
        let raw_value = (output.0.clamp(0.0, 1.0) * (i16::MAX as f32)) as i16;

        match port {
            EL4002Port::AO1 => {
                if let Some(channel1) = &mut self.rxpdo.channel1 {
                    channel1.value = raw_value;
                }
            }
            EL4002Port::AO2 => {
                if let Some(channel2) = &mut self.rxpdo.channel2 {
                    channel2.value = raw_value;
                }
            }
        }
    }
}

impl ConfigurableDevice<EL4002Configuration> for EL4002 {
    async fn write_config<'maindevice>(
        &mut self,
        device: &EthercrabSubDevicePreoperational<'maindevice>,
        config: &EL4002Configuration,
    ) -> Result<(), anyhow::Error> {
        config.write_config(device).await?;
        self.configuration = config.clone();
        self.rxpdo = config.pdo_assignment.rxpdo_assignment();
        Ok(())
    }

    fn get_config(&self) -> EL4002Configuration {
        self.configuration.clone()
    }
}

#[derive(Debug, Clone)]
pub enum EL4002Port {
    AO1,
    AO2,
}

#[derive(Debug, Clone, TxPdo)]
pub struct EL4002TxPdo {}

#[derive(Debug, Clone, RxPdo)]
pub struct EL4002RxPdo {
    #[pdo_object_index(0x1600)]
    pub channel1: Option<AnalogOutput>,
    #[pdo_object_index(0x1601)]
    pub channel2: Option<AnalogOutput>,
}

impl Configuration for EL4002Configuration {
    async fn write_config<'a>(
        &self,
        device: &EthercrabSubDevicePreoperational<'a>,
    ) -> Result<(), anyhow::Error> {
        // Write configuration for Channel 1
        self.channel1.write_channel_config(device, 0x8000).await?;

        // Write configuration for Channel 2
        self.channel2.write_channel_config(device, 0x8010).await?;

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

#[derive(Debug, Clone)]
pub enum EL4002PredefinedPdoAssignment {
    Standard,
    Compact,
}

impl PredefinedPdoAssignment<EL4002TxPdo, EL4002RxPdo> for EL4002PredefinedPdoAssignment {
    fn txpdo_assignment(&self) -> EL4002TxPdo {
        EL4002TxPdo {}
    }

    fn rxpdo_assignment(&self) -> EL4002RxPdo {
        match self {
            EL4002PredefinedPdoAssignment::Standard => EL4002RxPdo {
                channel1: Some(AnalogOutput::default()),
                channel2: Some(AnalogOutput::default()),
            },
            EL4002PredefinedPdoAssignment::Compact => EL4002RxPdo {
                channel1: Some(AnalogOutput::default()),
                channel2: Some(AnalogOutput::default()),
            },
        }
    }
}

pub const EL4002_VENDOR_ID: u32 = 2;
pub const EL4002_PRODUCT_ID: u32 = 262210642;
pub const EL4002_REVISION_A: u32 = 1441792;
pub const EL4002_REVISION_B: u32 = 1376256;

pub const EL4002_IDENTITY_A: SubDeviceIdentityTuple =
    (EL4002_VENDOR_ID, EL4002_PRODUCT_ID, EL4002_REVISION_A);

pub const EL4002_IDENTITY_B: SubDeviceIdentityTuple =
    (EL4002_VENDOR_ID, EL4002_PRODUCT_ID, EL4002_REVISION_B);
