use crate::{
    coe::{ConfigurableDevice, Configuration}, pdo::{
        el30xx::{AiCompact, AiStandard},
        PredefinedPdoAssignment, TxPdo,
    }, shared_config::el30xx::{EL30XXConfiguration, EL30XXPresentation}, signing::Integer16
};
use crate::{
    io::analog_input::{AnalogInputDevice, AnalogInputInput, AnalogInputState},
    types::EthercrabSubDevicePreoperational,
};
use ethercat_hal_derive::{Device, RxPdo, TxPdo};

use super::{NewDevice, SubDeviceIdentityTuple};


#[derive(Device)]
pub struct EL3024 {
    pub input_ts: u64,
    pub txpdo: EL3024TxPdo,
    pub configuration: EL30XXConfiguration<EL3024PdoPreset,EL3024TxPdo,EL3024RxPdo>,
}

impl std::fmt::Debug for EL3024 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EL3024")
    }
}

impl Default for EL3024PdoPreset {
    fn default() -> Self {
       Self::Standard
    }
}

impl NewDevice for EL3024 {
    fn new() -> Self {
        let configuration: EL30XXConfiguration<EL3024PdoPreset,EL3024TxPdo,EL3024RxPdo> = EL30XXConfiguration::default();
        Self {
            input_ts: 0,
            txpdo: configuration.pdo_assignment.txpdo_assignment(),
            configuration,
        }
    }
}

impl AnalogInputDevice<EL3024Port> for EL3024 {
    fn analog_output_state(&self, port: EL3024Port) -> AnalogInputState {
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
        let raw_value = Integer16::from(raw_value);
        let value: i16 = match self.configuration.presentation {
            EL30XXPresentation::Unsigned => raw_value.into_unsigned() as i16,
            EL30XXPresentation::Signed => raw_value.into_signed(),
            EL30XXPresentation::SignedMagnitude => raw_value.into_signed_magnitude(),
        };
        let normalized = f32::from(value) / f32::from(i16::MAX);
        AnalogInputState {
            input_ts: self.input_ts,
            input: AnalogInputInput { normalized },
        }
    }
}

impl ConfigurableDevice<EL30XXConfiguration<EL3024PdoPreset,EL3024TxPdo,EL3024RxPdo>> for EL3024 {
    async fn write_config<'maindevice>(
        &mut self,
        device: &EthercrabSubDevicePreoperational<'maindevice>,
        config: &EL30XXConfiguration<EL3024PdoPreset,EL3024TxPdo,EL3024RxPdo>,
    ) -> Result<(), anyhow::Error> {
        config.write_config(device).await?;
        self.configuration = config.clone();
        self.txpdo = config.pdo_assignment.txpdo_assignment();
        Ok(())
    }

    fn get_config(&self) -> EL30XXConfiguration<EL3024PdoPreset,EL3024TxPdo,EL3024RxPdo> {
        self.configuration.clone()
    }
}

#[derive(Debug, Clone)]
pub enum EL3024Port {
    AI1,
    AI2,
    AI3,
    AI4
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


impl Configuration for EL30XXConfiguration<EL3024PdoPreset,EL3024TxPdo,EL3024RxPdo> {
    async fn write_config<'a>(
        &self,
        device: &EthercrabSubDevicePreoperational<'a>,
    ) -> Result<(), anyhow::Error> {
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
pub enum EL3024PdoPreset {
    Standard,
    Compact,
}



impl PredefinedPdoAssignment<EL3024TxPdo, EL3024RxPdo> for EL3024PdoPreset {
    fn txpdo_assignment(&self) -> EL3024TxPdo {
        match self {
            EL3024PdoPreset::Standard => EL3024TxPdo {
                ai_standard_channel1: Some(AiStandard::default()),
                ai_compact_channel1: None,
                ai_standard_channel2: Some(AiStandard::default()),
                ai_compact_channel2: None,
                ai_standard_channel3: Some(AiStandard::default()),
                ai_compact_channel3: None,
                ai_standard_channel4: Some(AiStandard::default()),
                ai_compact_channel4: None,
            },
            EL3024PdoPreset::Compact => EL3024TxPdo {
                ai_standard_channel1: None,
                ai_compact_channel1:  Some(AiCompact::default()),
                ai_standard_channel2: None,
                ai_compact_channel2:  Some(AiCompact::default()),
                ai_standard_channel3: None,
                ai_compact_channel3:  Some(AiCompact::default()),
                ai_standard_channel4: None,
                ai_compact_channel4:  Some(AiCompact::default())
            },
        }
    }

    fn rxpdo_assignment(&self) -> EL3024RxPdo {
        match self {
            EL3024PdoPreset::Standard => EL3024RxPdo {},
            EL3024PdoPreset::Compact => EL3024RxPdo {},
        }
    }
}


pub const EL3024_VENDOR_ID: u32 = 2;
pub const EL3024_PRODUCT_ID: u32 = 198193234;
pub const EL3024_REVISION_A: u32 = 1245184;
pub const EL3024_IDENTITY_A: SubDeviceIdentityTuple =
    (EL3024_VENDOR_ID, EL3024_PRODUCT_ID, EL3024_REVISION_A);
