use super::EthercatDeviceProcessing;
use super::{NewEthercatDevice, SubDeviceIdentityTuple};
use crate::io::analog_input::physical::AnalogInputRange;
use crate::pdo::RxPdo;
use crate::pdo::TxPdo;
use crate::{
    coe::{ConfigurableDevice, Configuration},
    helpers::signing_converter_u16::U16SigningConverter,
    pdo::{
        PredefinedPdoAssignment,
        analog_input::{AiCompact, AiStandard},
    },
    shared_config::el30xx::{EL30XXChannelConfiguration, EL30XXPresentation},
};
use crate::{
    helpers::ethercrab_types::EthercrabSubDevicePreoperational,
    io::analog_input::{AnalogInputDevice, AnalogInputInput},
};
use ethercat_hal_derive::{EthercatDevice, RxPdo, TxPdo};
use units::electric_current::milliampere;
use units::f64::ElectricCurrent;

#[derive(Debug, Clone)]
pub struct EL3024Configuration {
    pub pdo_assignment: EL3024PredefinedPdoAssignment,
    // Input1+ and Input1-
    pub channel1: EL30XXChannelConfiguration,
    // Input2+ and Input2-
    pub channel2: EL30XXChannelConfiguration,
    // Input3+ and Input3-
    pub channel3: EL30XXChannelConfiguration,
    // Input4+ and Input4-
    pub channel4: EL30XXChannelConfiguration,
}

#[derive(EthercatDevice)]
pub struct EL3024 {
    pub configuration: EL3024Configuration,
    pub txpdo: EL3024TxPdo,
    pub rxpdo: EL3024RxPdo,
    is_used: bool,
}

impl EthercatDeviceProcessing for EL3024 {}

impl std::fmt::Debug for EL3024 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EL3024")
    }
}

impl Default for EL3024PredefinedPdoAssignment {
    fn default() -> Self {
        Self::Standard
    }
}

impl Default for EL3024Configuration {
    fn default() -> Self {
        Self {
            pdo_assignment: EL3024PredefinedPdoAssignment::Standard,
            channel1: EL30XXChannelConfiguration::default(),
            channel2: EL30XXChannelConfiguration::default(),
            channel3: EL30XXChannelConfiguration::default(),
            channel4: EL30XXChannelConfiguration::default(),
        }
    }
}

impl NewEthercatDevice for EL3024 {
    fn new() -> Self {
        let configuration = EL3024Configuration::default(); // Initialize first
        Self {
            configuration: configuration.clone(),
            txpdo: configuration.pdo_assignment.txpdo_assignment(),
            rxpdo: configuration.pdo_assignment.rxpdo_assignment(),
            is_used: false,
        }
    }
}

impl AnalogInputDevice<EL3024Port> for EL3024 {
    fn get_input(&self, port: EL3024Port) -> AnalogInputInput {
        let raw_value = match port {
            EL3024Port::AI1 => match &self.txpdo {
                EL3024TxPdo {
                    ai_standard_channel1: Some(ai_standard_channel1),
                    ..
                } => ai_standard_channel1.value,
                EL3024TxPdo {
                    ai_compact_channel1: Some(ai_compact_channel1),
                    ..
                } => ai_compact_channel1.value,
                _ => panic!("Invalid TxPdo assignment"),
            },
            EL3024Port::AI2 => match &self.txpdo {
                EL3024TxPdo {
                    ai_standard_channel2: Some(ai_standard_channel2),
                    ..
                } => ai_standard_channel2.value,
                EL3024TxPdo {
                    ai_compact_channel2: Some(ai_compact_channel2),
                    ..
                } => ai_compact_channel2.value,
                _ => panic!("Invalid TxPdo assignment"),
            },
            EL3024Port::AI3 => match &self.txpdo {
                EL3024TxPdo {
                    ai_standard_channel3: Some(ai_standard_channel3),
                    ..
                } => ai_standard_channel3.value,
                EL3024TxPdo {
                    ai_compact_channel3: Some(ai_compact_channel3),
                    ..
                } => ai_compact_channel3.value,
                _ => panic!("Invalid TxPdo assignment"),
            },
            EL3024Port::AI4 => match &self.txpdo {
                EL3024TxPdo {
                    ai_standard_channel4: Some(ai_standard_channel4),
                    ..
                } => ai_standard_channel4.value,
                EL3024TxPdo {
                    ai_compact_channel4: Some(ai_compact_channel4),
                    ..
                } => ai_compact_channel4.value,
                _ => panic!("Invalid TxPdo assignment"),
            },
        };
        let raw_value = U16SigningConverter::load_raw(raw_value);

        let presentation = match port {
            EL3024Port::AI1 => self.configuration.channel1.presentation,
            EL3024Port::AI2 => self.configuration.channel2.presentation,
            EL3024Port::AI3 => self.configuration.channel3.presentation,
            EL3024Port::AI4 => self.configuration.channel4.presentation,
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
        AnalogInputRange::Current {
            min: ElectricCurrent::new::<milliampere>(4.0),
            max: ElectricCurrent::new::<milliampere>(20.0),
            min_raw: 0,
            max_raw: 32767,
        }
    }
}

impl ConfigurableDevice<EL3024Configuration> for EL3024 {
    async fn write_config<'maindevice>(
        &mut self,
        device: &EthercrabSubDevicePreoperational<'maindevice>,
        config: &EL3024Configuration,
    ) -> Result<(), anyhow::Error> {
        config.write_config(device).await?;
        self.configuration = config.clone();
        self.txpdo = config.pdo_assignment.txpdo_assignment();
        Ok(())
    }

    fn get_config(&self) -> EL3024Configuration {
        self.configuration.clone()
    }
}

#[derive(Debug, Clone)]
pub enum EL3024Port {
    AI1,
    AI2,
    AI3,
    AI4,
}

#[derive(Debug, Clone, TxPdo)]
pub struct EL3024TxPdo {
    #[pdo_object_index(0x1A00)]
    pub ai_standard_channel1: Option<AiStandard>,
    #[pdo_object_index(0x1A01)]
    pub ai_compact_channel1: Option<AiCompact>,

    #[pdo_object_index(0x1A02)]
    pub ai_standard_channel2: Option<AiStandard>,
    #[pdo_object_index(0x1A03)]
    pub ai_compact_channel2: Option<AiCompact>,

    #[pdo_object_index(0x1A04)]
    pub ai_standard_channel3: Option<AiStandard>,
    #[pdo_object_index(0x1A05)]
    pub ai_compact_channel3: Option<AiCompact>,

    #[pdo_object_index(0x1A06)]
    pub ai_standard_channel4: Option<AiStandard>,
    #[pdo_object_index(0x1A07)]
    pub ai_compact_channel4: Option<AiCompact>,
}

#[derive(Debug, Clone, RxPdo)]
pub struct EL3024RxPdo {}

impl Configuration for EL3024Configuration {
    async fn write_config<'a>(
        &self,
        device: &EthercrabSubDevicePreoperational<'a>,
    ) -> Result<(), anyhow::Error> {
        // Write configuration for Channel 1
        self.channel1.write_channel_config(device, 0x8000).await?;

        // Write configuration for Channel 2
        self.channel2.write_channel_config(device, 0x8010).await?;
        // Write configuration for Channel 3
        self.channel3.write_channel_config(device, 0x8020).await?;

        // Write configuration for Channel 4
        self.channel4.write_channel_config(device, 0x8030).await?;
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
pub enum EL3024PredefinedPdoAssignment {
    Standard,
    Compact,
}

impl PredefinedPdoAssignment<EL3024TxPdo, EL3024RxPdo> for EL3024PredefinedPdoAssignment {
    fn txpdo_assignment(&self) -> EL3024TxPdo {
        match self {
            Self::Standard => EL3024TxPdo {
                ai_standard_channel1: Some(AiStandard::default()),
                ai_compact_channel1: None,
                ai_standard_channel2: Some(AiStandard::default()),
                ai_compact_channel2: None,
                ai_standard_channel3: Some(AiStandard::default()),
                ai_compact_channel3: None,
                ai_standard_channel4: Some(AiStandard::default()),
                ai_compact_channel4: None,
            },
            Self::Compact => EL3024TxPdo {
                ai_standard_channel1: None,
                ai_compact_channel1: Some(AiCompact::default()),
                ai_standard_channel2: None,
                ai_compact_channel2: Some(AiCompact::default()),
                ai_standard_channel3: None,
                ai_compact_channel3: Some(AiCompact::default()),
                ai_standard_channel4: None,
                ai_compact_channel4: Some(AiCompact::default()),
            },
        }
    }

    fn rxpdo_assignment(&self) -> EL3024RxPdo {
        match self {
            Self::Standard => EL3024RxPdo {},
            Self::Compact => EL3024RxPdo {},
        }
    }
}

pub const EL3024_VENDOR_ID: u32 = 2;
pub const EL3024_PRODUCT_ID: u32 = 198193234;
pub const EL3024_REVISION_A: u32 = 1245184;
pub const EL3024_IDENTITY_A: SubDeviceIdentityTuple =
    (EL3024_VENDOR_ID, EL3024_PRODUCT_ID, EL3024_REVISION_A);
