use super::{EthercatDeviceProcessing, NewEthercatDevice, SubDeviceIdentityTuple};
use crate::pdo::RxPdo;
use crate::{
    coe::{ConfigurableDevice, Configuration},
    helpers::signing_converter_u16::U16SigningConverter,
    io::analog_input::physical::AnalogInputRange,
    pdo::{
        PredefinedPdoAssignment, TxPdo,
        analog_input::{AiCompact, AiStandard},
    },
    shared_config::el30xx::{EL30XXChannelConfiguration, EL30XXPresentation},
};
use crate::{
    helpers::ethercrab_types::EthercrabSubDevicePreoperational,
    io::analog_input::{AnalogInputDevice, AnalogInputInput},
};
use ethercat_hal_derive::{EthercatDevice, RxPdo, TxPdo};
use uom::si::{electric_potential::volt, f64::ElectricPotential};

#[derive(Debug, Clone)]
pub struct EL3062_0030Configuration {
    pub pdo_assignment: EL3062_0030PredefinedPdoAssignment,
    pub channel_1: EL30XXChannelConfiguration,
    pub channel_2: EL30XXChannelConfiguration,
}
#[derive(EthercatDevice)]
pub struct EL3062_0030 {
    pub configuration: EL3062_0030Configuration,
    pub txpdo: EL3062_0030TxPdo,
    pub rxpdo: EL3062_0030RxPdo,
    is_used: bool,
}

impl EthercatDeviceProcessing for EL3062_0030 {}

impl std::fmt::Debug for EL3062_0030 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EL3062_0030")
    }
}

impl Default for EL3062_0030Configuration {
    fn default() -> Self {
        Self {
            pdo_assignment: EL3062_0030PredefinedPdoAssignment::Standard,
            channel_1: EL30XXChannelConfiguration::default(),
            channel_2: EL30XXChannelConfiguration::default(),
        }
    }
}

impl NewEthercatDevice for EL3062_0030 {
    fn new() -> Self {
        let configuration: EL3062_0030Configuration = EL3062_0030Configuration::default();
        Self {
            configuration: configuration.clone(),
            txpdo: configuration.pdo_assignment.txpdo_assignment(),
            rxpdo: configuration.pdo_assignment.rxpdo_assignment(),
            is_used: false,
        }
    }
}

impl AnalogInputDevice<EL3062_0030Port> for EL3062_0030 {
    fn get_input(&self, port: EL3062_0030Port) -> AnalogInputInput {
        let raw_value = match port {
            EL3062_0030Port::AI1 => match &self.txpdo {
                EL3062_0030TxPdo {
                    ai_standard_channel1: Some(ai_standard_channel1),
                    ..
                } => ai_standard_channel1.value,
                EL3062_0030TxPdo {
                    ai_compact_channel1: Some(ai_compact_channel1),
                    ..
                } => ai_compact_channel1.value,
                _ => panic!("Invalid TxPdo assignment"),
            },
            EL3062_0030Port::AI2 => match &self.txpdo {
                EL3062_0030TxPdo {
                    ai_standard_channel2: Some(ai_standard_channel2),
                    ..
                } => ai_standard_channel2.value,
                EL3062_0030TxPdo {
                    ai_compact_channel2: Some(ai_compact_channel2),
                    ..
                } => ai_compact_channel2.value,
                _ => panic!("Invalid TxPdo assignment"),
            },
        };
        let raw_value = U16SigningConverter::load_raw(raw_value);

        let presentation = match port {
            EL3062_0030Port::AI1 => &self.configuration.channel_1.presentation,
            EL3062_0030Port::AI2 => &self.configuration.channel_2.presentation,
        };
        let value: i16 = match presentation {
            EL30XXPresentation::Unsigned => raw_value.as_unsigned() as i16,
            EL30XXPresentation::Signed => raw_value.as_signed(),
            EL30XXPresentation::SignedMagnitude => raw_value.as_signed_magnitude(),
        };
        let normalized = f32::from(value) / f32::from(i16::MAX);
        AnalogInputInput {
            normalized,
            wiring_error: false,
        }
    }

    fn analog_input_range(&self) -> AnalogInputRange {
        AnalogInputRange::Potential {
            min: ElectricPotential::new::<volt>(0.0),
            max: ElectricPotential::new::<volt>(30.0),
            min_raw: 0,
            max_raw: i16::MAX,
        }
    }
}

impl ConfigurableDevice<EL3062_0030Configuration> for EL3062_0030 {
    async fn write_config<'maindevice>(
        &mut self,
        device: &EthercrabSubDevicePreoperational<'maindevice>,
        config: &EL3062_0030Configuration,
    ) -> Result<(), anyhow::Error> {
        config.write_config(device).await?;
        self.configuration = config.clone();
        self.txpdo = config.pdo_assignment.txpdo_assignment();
        Ok(())
    }

    fn get_config(&self) -> EL3062_0030Configuration {
        self.configuration.clone()
    }
}

#[derive(Debug, Clone)]
pub enum EL3062_0030Port {
    AI1,
    AI2,
}

#[derive(Debug, Clone, TxPdo)]
pub struct EL3062_0030TxPdo {
    #[pdo_object_index(0x1A00)]
    pub ai_standard_channel1: Option<AiStandard>,
    #[pdo_object_index(0x1A01)]
    pub ai_compact_channel1: Option<AiCompact>,

    #[pdo_object_index(0x1A02)]
    pub ai_standard_channel2: Option<AiStandard>,
    #[pdo_object_index(0x1A03)]
    pub ai_compact_channel2: Option<AiCompact>,
}

#[derive(Debug, Clone, RxPdo)]
pub struct EL3062_0030RxPdo {}

impl Configuration for EL3062_0030Configuration {
    async fn write_config<'a>(
        &self,
        device: &EthercrabSubDevicePreoperational<'a>,
    ) -> Result<(), anyhow::Error> {
        self.channel_1.write_channel_config(device, 0x8000).await?;
        self.channel_2.write_channel_config(device, 0x8010).await?;

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
pub enum EL3062_0030PredefinedPdoAssignment {
    Standard,
    Compact,
}

impl PredefinedPdoAssignment<EL3062_0030TxPdo, EL3062_0030RxPdo>
    for EL3062_0030PredefinedPdoAssignment
{
    fn txpdo_assignment(&self) -> EL3062_0030TxPdo {
        match self {
            EL3062_0030PredefinedPdoAssignment::Standard => EL3062_0030TxPdo {
                ai_standard_channel1: Some(AiStandard::default()),
                ai_compact_channel1: None,
                ai_standard_channel2: Some(AiStandard::default()),
                ai_compact_channel2: None,
            },
            EL3062_0030PredefinedPdoAssignment::Compact => EL3062_0030TxPdo {
                ai_standard_channel1: None,
                ai_compact_channel1: Some(AiCompact::default()),
                ai_standard_channel2: None,
                ai_compact_channel2: Some(AiCompact::default()),
            },
        }
    }

    fn rxpdo_assignment(&self) -> EL3062_0030RxPdo {
        match self {
            EL3062_0030PredefinedPdoAssignment::Standard => EL3062_0030RxPdo {},
            EL3062_0030PredefinedPdoAssignment::Compact => EL3062_0030RxPdo {},
        }
    }
}

pub const EL3062_0030_VENDOR_ID: u32 = 0x2;
pub const EL3062_0030_PRODUCT_ID: u32 = 0x0bf63052;
pub const EL3062_0030_REVISION_A: u32 = 0x0017001e;
pub const EL3062_0030_IDENTITY_A: SubDeviceIdentityTuple = (
    EL3062_0030_VENDOR_ID,
    EL3062_0030_PRODUCT_ID,
    EL3062_0030_REVISION_A,
);
