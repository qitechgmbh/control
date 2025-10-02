use super::{EthercatDeviceProcessing, NewEthercatDevice, SubDeviceIdentityTuple};
use crate::coe::Configuration;
use crate::helpers::ethercrab_types::EthercrabSubDevicePreoperational;
use crate::io::analog_output::{AnalogOutputDevice, AnalogOutputOutput};
use crate::pdo::PredefinedPdoAssignment;
use crate::pdo::RxPdo;
use crate::pdo::TxPdo;
use crate::pdo::el40xx::AnalogOutput;
use crate::shared_config::el40xx::EL40XXChannelConfiguration;
use ethercat_hal_derive::{EthercatDevice, RxPdo, TxPdo};
/// EL4002 2-channel analog output device
///
/// 0-10V DC, 12-bit resolution

#[derive(EthercatDevice)]
pub struct EL4002 {
    pub configuration: EL4002Configuration,
    pub rxpdo: EL4002RxPdo,
    pub txpdo: EL4002TxPdo,
    is_used: bool,
}

impl EthercatDeviceProcessing for EL4002 {}

#[derive(Debug, Clone)]
pub struct EL4002Configuration {
    pub pdo_assignment: EL4002PredefinedPdoAssignment,
    pub channel1: EL40XXChannelConfiguration,
    pub channel2: EL40XXChannelConfiguration,
}

#[derive(Debug, Clone)]
pub enum EL4002PredefinedPdoAssignment {
    Standard,
}

impl std::fmt::Debug for EL4002 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EL4002")
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EL4002Port {
    AO1,
    AO2,
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
        let configuration: EL4002Configuration = EL4002Configuration::default();
        Self {
            configuration: configuration.clone(),
            rxpdo: configuration.pdo_assignment.rxpdo_assignment(),
            txpdo: configuration.pdo_assignment.txpdo_assignment(),
            is_used: false,
        }
    }
}
fn normalize_voltage_to_int(value: f32) -> i16 {
    // Clamp the value between 0.0 and 10.0
    let clamped = value.clamp(0.0, 10.0);

    // Normalize to 0.0-1.0 range
    let normalized = clamped / 10.0;
    (normalized * 32767.0) as i16
}

impl AnalogOutputDevice<EL4002Port> for EL4002 {
    fn set_output(&mut self, port: EL4002Port, value: AnalogOutputOutput) {
        let value = normalize_voltage_to_int(value.0);
        match port {
            EL4002Port::AO1 => {
                if let Some(channel) = self.rxpdo.ao_channel1.as_mut() {
                    channel.value = value
                }
            }
            EL4002Port::AO2 => {
                if let Some(channel) = self.rxpdo.ao_channel2.as_mut() {
                    channel.value = value
                }
            }
        }
    }

    fn get_output(&self, _port: EL4002Port) -> AnalogOutputOutput {
        AnalogOutputOutput(0.0) // Default value, should not be used
    }
}

impl EL4002 {
    pub async fn write_config<'a>(
        &mut self,
        subdevice: &EthercrabSubDevicePreoperational<'a>,
    ) -> Result<(), anyhow::Error> {
        tracing::info!("el4002");
        self.configuration
            .channel1
            .write_channel_config(subdevice, 0x8000)
            .await?;
        self.configuration
            .channel1
            .write_channel_config(subdevice, 0x8010)
            .await?;

        self.configuration
            .pdo_assignment
            .txpdo_assignment()
            .write_config(subdevice)
            .await?;
        self.configuration
            .pdo_assignment
            .rxpdo_assignment()
            .write_config(subdevice)
            .await?;
        Ok(())
    }
}

#[derive(Debug, Clone, RxPdo)]
pub struct EL4002RxPdo {
    #[pdo_object_index(0x1600)]
    pub ao_channel1: Option<AnalogOutput>,
    #[pdo_object_index(0x1601)]
    pub ao_channel2: Option<AnalogOutput>,
}

#[derive(Debug, Clone, TxPdo)]
pub struct EL4002TxPdo {}

impl PredefinedPdoAssignment<EL4002TxPdo, EL4002RxPdo> for EL4002PredefinedPdoAssignment {
    fn rxpdo_assignment(&self) -> EL4002RxPdo {
        match self {
            Self::Standard => EL4002RxPdo {
                ao_channel1: Some(AnalogOutput::default()),
                ao_channel2: Some(AnalogOutput::default()),
            },
        }
    }

    fn txpdo_assignment(&self) -> EL4002TxPdo {
        EL4002TxPdo {}
    }
}
pub const EL4002_VENDOR_ID: u32 = 0x2;
pub const EL4002_PRODUCT_ID: u32 = 0xfa23052;
pub const EL4002_REVISION_A: u32 = 0x140000;
pub const EL4002_IDENTITY_A: SubDeviceIdentityTuple =
    (EL4002_VENDOR_ID, EL4002_PRODUCT_ID, EL4002_REVISION_A);
