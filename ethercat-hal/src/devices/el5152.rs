use super::{EthercatDeviceProcessing, NewEthercatDevice, SubDeviceIdentityTuple};
use crate::coe::{ConfigurableDevice, Configuration};
use crate::helpers::ethercrab_types::EthercrabSubDevicePreoperational;
use crate::io::encoder_input::{
    EncoderInputCounter, EncoderInputDevice, EncoderInputFrequency, EncoderInputPeriod,
};

use crate::pdo::{
    PredefinedPdoAssignment, RxPdo, TxPdo,
    el5152::{
        El5152EncoderControl, El5152EncoderFrequency, El5152EncoderPeriod, El5152EncoderStatus,
    },
};

use ethercat_hal_derive::EthercatDevice;

/// EL5152 2-channel incremental encoder interface
///
/// 24V HTL, 100 kHz, dual channel

#[derive(EthercatDevice)]
pub struct EL5152 {
    pub configuration: EL5152Configuration,
    pub rxpdo: EL5152RxPdo,
    pub txpdo: EL5152TxPdo,
    is_used: bool,
}

impl EthercatDeviceProcessing for EL5152 {}

#[derive(Debug, Clone)]
pub struct EL5152Configuration {
    pub pdo_assignment: EL5152PredefinedPdoAssignment,
    pub channel1: EL5152ChannelConfiguration,
    pub channel2: EL5152ChannelConfiguration,
}

#[derive(Debug, Clone)]
pub enum EL5152PredefinedPdoAssignment {
    Period,
    Frequency,
}

impl std::fmt::Debug for EL5152 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EL5152")
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EL5152Port {
    ENC1,
    ENC2,
}

impl Default for EL5152Configuration {
    fn default() -> Self {
        tracing::info!("default");
        Self {
            pdo_assignment: EL5152PredefinedPdoAssignment::Period,
            channel1: EL5152ChannelConfiguration::default(),
            channel2: EL5152ChannelConfiguration::default(),
        }
    }
}

impl NewEthercatDevice for EL5152 {
    fn new() -> Self {
        tracing::info!("new");

        let configuration: EL5152Configuration = EL5152Configuration::default();
        tracing::info!("configuration {:?}", configuration);

        Self {
            configuration: configuration.clone(),
            rxpdo: configuration.pdo_assignment.rxpdo_assignment(),
            txpdo: configuration.pdo_assignment.txpdo_assignment(),
            is_used: false,
        }
    }
}

impl ConfigurableDevice<EL5152Configuration> for EL5152 {
    async fn write_config<'maindevice>(
        &mut self,
        device: &EthercrabSubDevicePreoperational<'maindevice>,
        config: &EL5152Configuration,
    ) -> Result<(), anyhow::Error> {
        config.write_config(device).await?;
        self.configuration = config.clone();
        self.txpdo = config.pdo_assignment.txpdo_assignment();
        self.rxpdo = config.pdo_assignment.rxpdo_assignment();
        Ok(())
    }

    fn get_config(&self) -> EL5152Configuration {
        self.configuration.clone()
    }
}

impl EncoderInputDevice<EL5152Port> for EL5152 {
    fn get_counter_value(&self, port: EL5152Port) -> Result<EncoderInputCounter, anyhow::Error> {
        tracing::info!("get_counter_value");

        let value = match port {
            EL5152Port::ENC1 => self
                .txpdo
                .status_channel1
                .as_ref()
                .map_or(0, |status| status.counter_value),
            EL5152Port::ENC2 => self
                .txpdo
                .status_channel2
                .as_ref()
                .map_or(0, |status| status.counter_value),
        };
        Ok(EncoderInputCounter { value })
    }

    fn get_frequency(
        &self,
        port: EL5152Port,
    ) -> Result<Option<EncoderInputFrequency>, anyhow::Error> {
        tracing::info!("get_frequency");

        let frequency = match port {
            EL5152Port::ENC1 => {
                self.txpdo
                    .frequency_channel1
                    .as_ref()
                    .map(|f| EncoderInputFrequency {
                        value: f.frequency_value,
                    })
            }
            EL5152Port::ENC2 => {
                self.txpdo
                    .frequency_channel2
                    .as_ref()
                    .map(|f| EncoderInputFrequency {
                        value: f.frequency_value,
                    })
            }
        };
        Ok(frequency)
    }

    fn get_period(&self, port: EL5152Port) -> Result<Option<EncoderInputPeriod>, anyhow::Error> {
        tracing::info!("get_period");

        let period = match port {
            EL5152Port::ENC1 => self
                .txpdo
                .period_channel1
                .as_ref()
                .map(|p| EncoderInputPeriod {
                    value: p.period_value,
                }),
            EL5152Port::ENC2 => self
                .txpdo
                .period_channel2
                .as_ref()
                .map(|p| EncoderInputPeriod {
                    value: p.period_value,
                }),
        };
        Ok(period)
    }

    fn set_counter(&mut self, port: EL5152Port, value: u32) -> Result<(), anyhow::Error> {
        tracing::info!("set_counter");

        match port {
            EL5152Port::ENC1 => {
                if let Some(control) = self.rxpdo.control_channel1.as_mut() {
                    control.set_counter_value = value;
                    control.set_counter = true;
                }
            }
            EL5152Port::ENC2 => {
                if let Some(control) = self.rxpdo.control_channel2.as_mut() {
                    control.set_counter_value = value;
                    control.set_counter = true;
                }
            }
        }
        Ok(())
    }
}

impl Configuration for EL5152Configuration {
    async fn write_config<'a>(
        &self,
        device: &EthercrabSubDevicePreoperational<'a>,
    ) -> Result<(), anyhow::Error> {
        tracing::info!("config");
        // Configure channel 1
        self.channel1.write_channel_config(device, 0x8000).await?;

        // Configure channel 2
        self.channel2.write_channel_config(device, 0x8010).await?;
        // Write PDO assignments
        self.pdo_assignment
            .txpdo_assignment()
            .write_config(device)
            .await?;
        self.pdo_assignment
            .rxpdo_assignment()
            .write_config(device)
            .await?;

        // Free RUN
        device.sdo_write(0x1C32, 0x1, 0x00u16).await?;
        device.sdo_write(0x1C33, 0x1, 0x00u16).await?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct EL5152RxPdo {
    pub control_channel1: Option<El5152EncoderControl>,
    pub control_channel2: Option<El5152EncoderControl>,
}

impl crate::coe::Configuration for EL5152RxPdo {
    async fn write_config<'a>(
        &self,
        device: &EthercrabSubDevicePreoperational<'a>,
    ) -> Result<(), anyhow::Error> {
        tracing::info!("EL5152RxPdo");

        device.sdo_write(0x1C12, 0, 0u8).await?;
        let mut len: u8 = 0;
        if self.control_channel1.is_some() {
            len += 1;
            device.sdo_write(0x1C12, len, 0x1600u16).await?;
        }
        if self.control_channel2.is_some() {
            len += 1;
            device.sdo_write(0x1C12, len, 0x1602u16).await?;
        }
        device.sdo_write(0x1C12, 0, len).await?;
        Ok(())
    }
}
impl crate::pdo::RxPdo for EL5152RxPdo {
    ///Implemented by the ethercat_hal_derive::RxPdo derive macro
    fn get_objects(&self) -> Box<[Option<&dyn crate::pdo::RxPdoObject>]> {
        Box::new([
            self.control_channel1
                .as_ref()
                .map(|o| o as &dyn crate::pdo::RxPdoObject),
            self.control_channel2
                .as_ref()
                .map(|o| o as &dyn crate::pdo::RxPdoObject),
        ])
    }
}

#[derive(Debug, Clone)]
pub struct EL5152TxPdo {
    pub status_channel1: Option<El5152EncoderStatus>,
    pub period_channel1: Option<El5152EncoderPeriod>,
    pub frequency_channel1: Option<El5152EncoderFrequency>,

    pub status_channel2: Option<El5152EncoderStatus>,
    pub period_channel2: Option<El5152EncoderPeriod>,
    pub frequency_channel2: Option<El5152EncoderFrequency>,
}

impl crate::coe::Configuration for EL5152TxPdo {
    ///Implemented by the ethercat_hal_derive::TxPdo derive macro
    async fn write_config<'a>(
        &self,
        device: &EthercrabSubDevicePreoperational<'a>,
    ) -> Result<(), anyhow::Error> {
        tracing::info!("EL5152TxPdo");
        let mut len: u8 = 0;
        if self.status_channel1.is_some() {
            len += 1;
            device.sdo_write(0x1C13, len, 0x1A00u16).await?;
        }
        if self.period_channel1.is_some() {
            len += 1;
            device.sdo_write(0x1C13, len, 0x1A02u16).await?;
        }
        if self.frequency_channel1.is_some() {
            len += 1;
            device.sdo_write(0x1C13, len, 0x1A03u16).await?;
        }
        if self.status_channel2.is_some() {
            len += 1;
            device.sdo_write(0x1C13, len, 0x1A04u16).await?;
        }
        if self.period_channel2.is_some() {
            len += 1;
            device.sdo_write(0x1C13, len, 0x1A06u16).await?;
        }
        if self.frequency_channel2.is_some() {
            len += 1;
            device.sdo_write(0x1C13, len, 0x1A07u16).await?;
        }
        device.sdo_write(0x1C13, 0, len).await?;
        Ok(())
    }
}
impl crate::pdo::TxPdo for EL5152TxPdo {
    ///Implemented by the ethercat_hal_derive::TxPdo derive macro
    fn get_objects(&self) -> Box<[Option<&dyn crate::pdo::TxPdoObject>]> {
        Box::new([
            self.status_channel1
                .as_ref()
                .map(|o| o as &dyn crate::pdo::TxPdoObject),
            self.period_channel1
                .as_ref()
                .map(|o| o as &dyn crate::pdo::TxPdoObject),
            self.frequency_channel1
                .as_ref()
                .map(|o| o as &dyn crate::pdo::TxPdoObject),
            self.status_channel2
                .as_ref()
                .map(|o| o as &dyn crate::pdo::TxPdoObject),
            self.period_channel2
                .as_ref()
                .map(|o| o as &dyn crate::pdo::TxPdoObject),
            self.frequency_channel2
                .as_ref()
                .map(|o| o as &dyn crate::pdo::TxPdoObject),
        ])
    }
    ///Implemented by the ethercat_hal_derive::TxPdo derive macro
    fn get_objects_mut(&mut self) -> Box<[Option<&mut dyn crate::pdo::TxPdoObject>]> {
        Box::new([
            self.status_channel1
                .as_mut()
                .map(|o| o as &mut dyn crate::pdo::TxPdoObject),
            self.period_channel1
                .as_mut()
                .map(|o| o as &mut dyn crate::pdo::TxPdoObject),
            self.frequency_channel1
                .as_mut()
                .map(|o| o as &mut dyn crate::pdo::TxPdoObject),
            self.status_channel2
                .as_mut()
                .map(|o| o as &mut dyn crate::pdo::TxPdoObject),
            self.period_channel2
                .as_mut()
                .map(|o| o as &mut dyn crate::pdo::TxPdoObject),
            self.frequency_channel2
                .as_mut()
                .map(|o| o as &mut dyn crate::pdo::TxPdoObject),
        ])
    }
}
impl PredefinedPdoAssignment<EL5152TxPdo, EL5152RxPdo> for EL5152PredefinedPdoAssignment {
    fn rxpdo_assignment(&self) -> EL5152RxPdo {
        tracing::info!("rxpdo_assignment");

        EL5152RxPdo {
            control_channel1: Some(El5152EncoderControl::default()),
            control_channel2: Some(El5152EncoderControl::default()),
        }
    }

    fn txpdo_assignment(&self) -> EL5152TxPdo {
        tracing::info!("txpdo_assignment");

        match self {
            Self::Period => EL5152TxPdo {
                status_channel1: Some(El5152EncoderStatus::default()),
                period_channel1: Some(El5152EncoderPeriod::default()),
                frequency_channel1: None,

                status_channel2: Some(El5152EncoderStatus::default()),
                period_channel2: Some(El5152EncoderPeriod::default()),
                frequency_channel2: None,
            },
            Self::Frequency => EL5152TxPdo {
                status_channel1: Some(El5152EncoderStatus::default()),
                period_channel1: None,
                frequency_channel1: Some(El5152EncoderFrequency::default()),

                status_channel2: Some(El5152EncoderStatus::default()),
                period_channel2: None,
                frequency_channel2: Some(El5152EncoderFrequency::default()),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct EL5152ChannelConfiguration {
    // 80n0:3
    pub enable_counter: u8,
    // 80n0:8
    pub disable_filter: u8,
    // 80n0:A
    pub enable_micro_increment: u8,
    // 80n0:E
    pub reversion_rotation: u8,
    // 80n0:F 0-µs, 1-ms
    pub frequency_based_window: u8,
    // 80n0:11
    pub frequency_window: u16,
    // 80n0:13
    pub frequency_scaling: u16,
    // 80n0:14
    pub period_scaling: u16,
    // 80n0:15 100: "0.01 Hz"
    pub frequency_resolution: u16,
    // 80n0:16 100: "100 ns"
    pub period_resolution: u16,
    // 80n0:17
    pub frequency_wait_time: u16,
}

impl EL5152ChannelConfiguration {
    pub async fn write_channel_config<'a>(
        &self,
        device: &EthercrabSubDevicePreoperational<'a>,
        base_index: u16,
    ) -> Result<(), anyhow::Error> {
        tracing::info!("write_channel_config {:?}", base_index);

        device
            .sdo_write(base_index, 0x03, self.enable_counter)
            .await?;
        device
            .sdo_write(base_index, 0x08, self.disable_filter)
            .await?;
        device
            .sdo_write(base_index, 0x0A, self.enable_micro_increment)
            .await?;
        device
            .sdo_write(base_index, 0x0E, self.reversion_rotation)
            .await?;
        device
            .sdo_write(base_index, 0x0F, self.frequency_based_window)
            .await?;
        device
            .sdo_write(base_index, 0x11, self.frequency_window)
            .await?;
        device
            .sdo_write(base_index, 0x13, self.frequency_scaling)
            .await?;
        device
            .sdo_write(base_index, 0x14, self.period_scaling)
            .await?;
        device
            .sdo_write(base_index, 0x15, self.frequency_resolution)
            .await?;
        device
            .sdo_write(base_index, 0x16, self.period_resolution)
            .await?;
        device
            .sdo_write(base_index, 0x17, self.frequency_wait_time)
            .await?;

        Ok(())
    }
}

impl Default for EL5152ChannelConfiguration {
    fn default() -> Self {
        Self {
            enable_counter: 0x00u8,
            disable_filter: 0x00u8,
            enable_micro_increment: 0x00u8,
            reversion_rotation: 0x00u8,
            frequency_based_window: 0x00u8,
            frequency_window: 0x2710u16,
            frequency_scaling: 0x0064u16,
            period_scaling: 0x0064u16,
            frequency_resolution: 0x0064u16,
            period_resolution: 0x0064u16,
            frequency_wait_time: 0x0640u16,
        }
    }
}

pub const EL5152_VENDOR_ID: u32 = 0x2;
pub const EL5152_PRODUCT_ID: u32 = 0x14203052;
pub const EL5152_REVISION_A: u32 = 0x140000;
pub const EL5152_IDENTITY_A: SubDeviceIdentityTuple =
    (EL5152_VENDOR_ID, EL5152_PRODUCT_ID, EL5152_REVISION_A);
