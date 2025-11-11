use super::{EthercatDeviceProcessing, NewEthercatDevice, SubDeviceIdentityTuple};
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
use units::{electric_potential::volt, f64::ElectricPotential};

#[derive(EthercatDevice)]
pub struct EL3001 {
    pub txpdo: EL3001TxPdo,
    pub configuration: EL3001Configuration,
    is_used: bool,
}

impl EthercatDeviceProcessing for EL3001 {}

impl std::fmt::Debug for EL3001 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EL3001")
    }
}

impl Default for EL3001PredefinedPdoAssignment {
    fn default() -> Self {
        Self::Standard
    }
}

impl NewEthercatDevice for EL3001 {
    fn new() -> Self {
        let configuration: EL3001Configuration = EL3001Configuration::default();
        Self {
            txpdo: configuration.pdo_assignment.txpdo_assignment(),
            configuration,
            is_used: false,
        }
    }
}

impl AnalogInputDevice<EL3001Port> for EL3001 {
    fn get_input(&self, port: EL3001Port) -> AnalogInputInput {
        let raw_value = match port {
            EL3001Port::AI1 => match &self.txpdo {
                EL3001TxPdo {
                    ai_standard: Some(ai_standard),
                    ..
                } => ai_standard.value,
                EL3001TxPdo {
                    ai_compact: Some(ai_compact),
                    ..
                } => ai_compact.value,
                _ => panic!("Invalid TxPdo assignment"),
            },
        };
        let channel_config = match port {
            EL3001Port::AI1 => &self.configuration.channel_1,
        };
        let raw_value = U16SigningConverter::load_raw(raw_value);
        let value: i16 = match channel_config.presentation {
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
            min: ElectricPotential::new::<volt>(-10.0),
            max: ElectricPotential::new::<volt>(10.0),
            min_raw: i16::MIN,
            max_raw: i16::MAX,
        }
    }
}

impl ConfigurableDevice<EL3001Configuration> for EL3001 {
    async fn write_config<'maindevice>(
        &mut self,
        device: &EthercrabSubDevicePreoperational<'maindevice>,
        config: &EL3001Configuration,
    ) -> Result<(), anyhow::Error> {
        config.write_config(device).await?;
        self.configuration = config.clone();
        self.txpdo = config.pdo_assignment.txpdo_assignment();
        Ok(())
    }

    fn get_config(&self) -> EL3001Configuration {
        self.configuration.clone()
    }
}

#[derive(Debug, Clone)]
pub enum EL3001Port {
    AI1,
}

#[derive(Debug, Clone, TxPdo)]
pub struct EL3001TxPdo {
    #[pdo_object_index(0x1A00)]
    pub ai_standard: Option<AiStandard>,
    #[pdo_object_index(0x1A01)]
    pub ai_compact: Option<AiCompact>,
}

#[derive(Debug, Clone, RxPdo)]
pub struct EL3001RxPdo {}

#[derive(Debug, Clone, Default)]
pub struct EL3001Configuration {
    pub pdo_assignment: EL3001PredefinedPdoAssignment,
    pub channel_1: EL30XXChannelConfiguration,
}

impl Configuration for EL3001Configuration {
    async fn write_config<'a>(
        &self,
        device: &EthercrabSubDevicePreoperational<'a>,
    ) -> Result<(), anyhow::Error> {
        self.channel_1.write_channel_config(device, 0x8000).await?;
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
pub enum EL3001PredefinedPdoAssignment {
    Standard,
    Compact,
}

impl PredefinedPdoAssignment<EL3001TxPdo, EL3001RxPdo> for EL3001PredefinedPdoAssignment {
    fn txpdo_assignment(&self) -> EL3001TxPdo {
        match self {
            Self::Standard => EL3001TxPdo {
                ai_standard: Some(AiStandard::default()),
                ai_compact: None,
            },
            Self::Compact => EL3001TxPdo {
                ai_standard: None,
                ai_compact: Some(AiCompact::default()),
            },
        }
    }

    fn rxpdo_assignment(&self) -> EL3001RxPdo {
        match self {
            Self::Standard => EL3001RxPdo {},
            Self::Compact => EL3001RxPdo {},
        }
    }
}

pub const EL3001_VENDOR_ID: u32 = 0x2;
pub const EL3001_PRODUCT_ID: u32 = 0x0bb93052;
pub const EL3001_REVISION_A: u32 = 0x00160000;
pub const EL3001_IDENTITY_A: SubDeviceIdentityTuple =
    (EL3001_VENDOR_ID, EL3001_PRODUCT_ID, EL3001_REVISION_A);
