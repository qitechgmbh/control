use super::{NewDevice, SubDeviceIdentityTuple};
use crate::{
    coe::{ConfigurableDevice, Configuration}, pdo::{
        el30xx::{AiCompact, AiStandard},
        PredefinedPdoAssignment, TxPdo,
    }, shared_config::el30xx::{EL30XXConfiguration, EL30XXFilterSettings, EL30XXPresentation}, signing::Integer16
};
use crate::{
    io::analog_input::{AnalogInputDevice, AnalogInputInput, AnalogInputState},
    types::EthercrabSubDevicePreoperational,
};
use ethercat_hal_derive::{Device, RxPdo, TxPdo};

#[derive(Device)]
pub struct EL3001 {
    pub input_ts: u64,
    pub txpdo: EL3001TxPdo,
    pub configuration: EL30XXConfiguration<EL3001PdoPreset,EL3001TxPdo,EL3001RxPdo>,
}

impl std::fmt::Debug for EL3001 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EL3001")
    }
}



impl Default for EL3001PdoPreset {
    fn default() -> Self {
       Self::Standard
    }
}

impl NewDevice for EL3001 {
    fn new() -> Self {
        let configuration:EL30XXConfiguration<EL3001PdoPreset,EL3001TxPdo,EL3001RxPdo> = EL30XXConfiguration::default();
        Self {
            input_ts: 0,
            txpdo: configuration.pdo_assignment.txpdo_assignment(),
            configuration,
        }
    }
}

impl AnalogInputDevice<EL3001Port> for EL3001 {
    fn analog_output_state(&self, port: EL3001Port) -> AnalogInputState {
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

impl ConfigurableDevice<EL30XXConfiguration<EL3001PdoPreset,EL3001TxPdo,EL3001RxPdo>> for EL3001 {
    async fn write_config<'maindevice>(
        &mut self,
        device: &EthercrabSubDevicePreoperational<'maindevice>,
        config: &EL30XXConfiguration<EL3001PdoPreset,EL3001TxPdo,EL3001RxPdo>,
    ) -> Result<(), anyhow::Error> {
        config.write_config(device).await?;
        self.configuration = config.clone();
        self.txpdo = config.pdo_assignment.txpdo_assignment();
        Ok(())
    }

    fn get_config(&self) -> EL30XXConfiguration<EL3001PdoPreset,EL3001TxPdo,EL3001RxPdo> {
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

impl Configuration for EL30XXConfiguration<EL3001PdoPreset,EL3001TxPdo,EL3001RxPdo> {
    async fn write_config<'a>(
        &self,
        device: &EthercrabSubDevicePreoperational<'a>,
    ) -> Result<(), anyhow::Error> {
        device
            .sdo_write(0x8000, 0x01, self.enable_user_scale)
            .await?;
        device
            .sdo_write(0x8000, 0x02, u8::from(self.presentation))
            .await?;
        device.sdo_write(0x8000, 0x05, self.siemens_bits).await?;
        device.sdo_write(0x8000, 0x06, self.enable_filter).await?;
        device.sdo_write(0x8000, 0x07, self.enable_limit_1).await?;
        device.sdo_write(0x8000, 0x08, self.enable_limit_2).await?;
        device
            .sdo_write(0x8000, 0x0A, self.enable_user_calibration)
            .await?;
        device
            .sdo_write(0x8000, 0x0B, self.enable_vendor_calibration)
            .await?;
        device.sdo_write(0x8000, 0x0E, self.swap_limit_bits).await?;
        device
            .sdo_write(0x8000, 0x11, self.user_scale_offset)
            .await?;
        device.sdo_write(0x8000, 0x12, self.user_scale_gain).await?;
        device.sdo_write(0x8000, 0x13, self.limit_1).await?;
        device.sdo_write(0x8000, 0x14, self.limit_2).await?;
        device
            .sdo_write(0x8000, 0x15, u16::from(self.filter_settings))
            .await?;
        device
            .sdo_write(0x8000, 0x17, self.user_calibration_offset)
            .await?;
        device
            .sdo_write(0x8000, 0x18, self.user_calibration_gain)
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


#[derive(Debug, Clone)]
pub enum EL3001PdoPreset {
    Standard,
    Compact,
}

impl PredefinedPdoAssignment<EL3001TxPdo, EL3001RxPdo> for EL3001PdoPreset {
    fn txpdo_assignment(&self) -> EL3001TxPdo {
        match self {
            EL3001PdoPreset::Standard => EL3001TxPdo {
                ai_standard: Some(AiStandard::default()),
                ai_compact: None,
            },
            EL3001PdoPreset::Compact => EL3001TxPdo {
                ai_standard: None,
                ai_compact: Some(AiCompact::default()),
            },
        }
    }

    fn rxpdo_assignment(&self) -> EL3001RxPdo {
        match self {
            EL3001PdoPreset::Standard => EL3001RxPdo {},
            EL3001PdoPreset::Compact => EL3001RxPdo {},
        }
    }
}



pub const EL3001_VENDOR_ID: u32 = 0x2;
pub const EL3001_PRODUCT_ID: u32 = 0x0bb93052;
pub const EL3001_REVISION_A: u32 = 0x00160000;
pub const EL3001_IDENTITY_A: SubDeviceIdentityTuple =
    (EL3001_VENDOR_ID, EL3001_PRODUCT_ID, EL3001_REVISION_A);
