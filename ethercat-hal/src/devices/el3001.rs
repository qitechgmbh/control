use super::SubDeviceIdentityTuple;
use crate::{
    coe::{ConfigurableDevice, Configuration},
    pdo::{
        el30xx::{AiCompact, AiStandard},
        PdoPreset, TxPdo,
    },
    signing::Integer16,
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
    pub configuration: EL3001Configuration,
}

impl std::fmt::Debug for EL3001 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EL3001")
    }
}

impl EL3001 {
    pub fn new() -> Self {
        let configuration = EL3001Configuration::default();
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
            EL3001Presentation::Unsigned => raw_value.into_unsigned() as i16,
            EL3001Presentation::Signed => raw_value.into_signed(),
            EL3001Presentation::SignedMagnitude => raw_value.into_signed_magnitude(),
        };
        let normalized = f32::from(value) / f32::from(i16::MAX);
        AnalogInputState {
            input_ts: self.input_ts,
            input: AnalogInputInput { normalized },
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

/// 0x8000 CoE
#[derive(Debug, Clone)]
pub struct EL3001Configuration {
    /// # 0x8000:01
    /// Enable user scale
    ///
    /// default: `false`
    pub enable_user_scale: bool,

    /// # 0x8000:02
    /// Presentation
    ///
    /// default: `Signed`
    pub presentation: EL3001Presentation,

    /// # 0x8000:05
    /// Siemens bits
    ///
    /// default: `false`
    pub siemens_bits: bool,

    /// # 0x8000:06
    /// Enable filter
    ///
    /// default: `true`
    pub enable_filter: bool,

    /// # 0x8000:07
    /// Enable limit 1
    ///
    /// default: `false`
    pub enable_limit_1: bool,

    /// # 0x8000:08
    /// Enable limit 2
    ///
    /// default: `false`
    pub enable_limit_2: bool,

    /// # 0x8000:0A
    /// Enable user calibration
    ///
    /// default: `false`
    pub enable_user_calibration: bool,

    /// # 0x8000:0B
    /// Enable vendor calibration
    ///
    /// default: `true`
    pub enable_vendor_calibration: bool,

    /// # 0x8000:0E
    /// Swap limit bits
    ///
    /// default: `false`
    pub swap_limit_bits: bool,

    /// # 0x8000:11
    /// User scale offset
    ///
    /// default: `0`
    pub user_scale_offset: i16,

    /// # 0x8000:12
    /// User scale gain
    ///
    /// default: `65536`
    pub user_scale_gain: i32,

    /// # 0x8000:13
    /// Limit 1
    ///
    /// default: `0`
    pub limit_1: i16,

    /// # 0x8000:14
    /// Limit 2
    ///
    /// default: `0`
    pub limit_2: i16,

    /// # 0x8000:15
    /// Filter settings
    ///
    /// default: `50 Hz FIR`
    pub filter_settings: EL3001FilterSettings,

    /// # 0x8000:17
    /// User calibration offset
    ///
    /// default: `0`
    pub user_calibration_offset: i16,

    /// # 0x8000:18
    /// User calibration gain
    ///
    /// default: `16384`
    pub user_calibration_gain: i16,

    /// # 0x1400 & 0x1600
    pub pdo_assignment: EL3001PdoPreset,
}

impl Configuration for EL3001Configuration {
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

impl Default for EL3001Configuration {
    fn default() -> Self {
        Self {
            enable_user_scale: false,
            presentation: EL3001Presentation::Signed,
            siemens_bits: false,
            enable_filter: true,
            enable_limit_1: false,
            enable_limit_2: false,
            enable_user_calibration: false,
            enable_vendor_calibration: true,
            swap_limit_bits: false,
            user_scale_offset: 0,
            user_scale_gain: 65536,
            limit_1: 0,
            limit_2: 0,
            filter_settings: EL3001FilterSettings::FIR50Hz,
            user_calibration_offset: 0,
            user_calibration_gain: 16384,
            pdo_assignment: EL3001PdoPreset::Standard,
        }
    }
}

#[derive(Debug, Clone)]
pub enum EL3001PdoPreset {
    Standard,
    Compact,
}

impl PdoPreset<EL3001TxPdo, EL3001RxPdo> for EL3001PdoPreset {
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

#[derive(Debug, Clone, Copy)]
pub enum EL3001Presentation {
    Signed,
    Unsigned,
    SignedMagnitude,
}

pub enum EL3001Value {
    Signed(i16),
    Unsigned(u16),
    SignedMagnitude(i16),
}

impl From<EL3001Presentation> for u8 {
    fn from(presentation: EL3001Presentation) -> Self {
        match presentation {
            EL3001Presentation::Signed => 0,
            EL3001Presentation::Unsigned => 1,
            EL3001Presentation::SignedMagnitude => 2,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EL3001FilterSettings {
    FIR50Hz,
    FIR60Hz,
    IIR1,
    IIR2,
    IIR3,
    IIR4,
    IIR5,
    IIR6,
    IIR7,
    IIR8,
}

impl From<EL3001FilterSettings> for u16 {
    fn from(filter_settings: EL3001FilterSettings) -> Self {
        match filter_settings {
            EL3001FilterSettings::FIR50Hz => 0,
            EL3001FilterSettings::FIR60Hz => 1,
            EL3001FilterSettings::IIR1 => 2,
            EL3001FilterSettings::IIR2 => 3,
            EL3001FilterSettings::IIR3 => 4,
            EL3001FilterSettings::IIR4 => 5,
            EL3001FilterSettings::IIR5 => 6,
            EL3001FilterSettings::IIR6 => 7,
            EL3001FilterSettings::IIR7 => 8,
            EL3001FilterSettings::IIR8 => 9,
        }
    }
}

pub const EL3001_VENDOR_ID: u32 = 0x2;
pub const EL3001_PRODUCT_ID: u32 = 0x0bb93052;
pub const EL3001_REVISION_A: u32 = 0x00160000;
pub const EL3001_IDENTITY_A: SubDeviceIdentityTuple =
    (EL3001_VENDOR_ID, EL3001_PRODUCT_ID, EL3001_REVISION_A);
