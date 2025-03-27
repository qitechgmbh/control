use crate::coe::Configuration;
use crate::pdo::PredefinedPdoAssignment;
use crate::types::EthercrabSubDevicePreoperational;
use std::marker::PhantomData; // needed to avoid compile error

/// 0x8000 CoE
#[derive(Debug, Clone)]
pub struct EL30XXConfiguration<T,A,B> where T:PredefinedPdoAssignment<A,B> {
    /// # 0x8000:01
    /// Enable user scale
    ///
    /// default: `false`
    pub enable_user_scale: bool,

    /// # 0x8000:02
    /// Presentation
    ///
    /// default: `Signed`
    pub presentation: EL30XXPresentation,

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
    pub filter_settings: EL30XXFilterSettings,

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
    pub pdo_assignment: T,

    // PhantomData ensures A and B are "used" without being stored
    pub _marker_a: PhantomData<A>, // needed to avoid compile error
    pub _marker_b: PhantomData<B>, // needed to avoid compile error
}





impl<T, A, B> Configuration for EL30XXConfiguration<T, A, B>
where
    T: PredefinedPdoAssignment<A, B> + Configuration,
    A: Configuration, // Ensures pdo_assignment can call write_config()
    B: Configuration
{
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


impl<T, A, B> Default for EL30XXConfiguration<T, A, B>
where
    T: PredefinedPdoAssignment<A, B>,
    T: Default, // Ensure `T` has a default value
{
    fn default() -> Self {
        Self {
            enable_user_scale: false,
            presentation: EL30XXPresentation::Signed,
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
            filter_settings: EL30XXFilterSettings::FIR50Hz,
            user_calibration_offset: 0,
            user_calibration_gain: 16384,
            pdo_assignment: T::default(), // Use the default for `T`
            _marker_a: PhantomData, // needed to avoid compile error
            _marker_b: PhantomData // needed to avoid compile error   
        }
    }
}



#[derive(Debug, Clone, Copy)]
pub enum EL30XXPresentation {
    Signed,
    Unsigned,
    SignedMagnitude,
}

pub enum EL30XXValue {
    Signed(i16),
    Unsigned(u16),
    SignedMagnitude(i16),
}

impl From<EL30XXPresentation> for u8 {
    fn from(presentation: EL30XXPresentation) -> Self {
        match presentation {
            EL30XXPresentation::Signed => 0,
            EL30XXPresentation::Unsigned => 1,
            EL30XXPresentation::SignedMagnitude => 2,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EL30XXFilterSettings {
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

impl From<EL30XXFilterSettings> for u16 {
    fn from(filter_settings: EL30XXFilterSettings) -> Self {
        match filter_settings {
            EL30XXFilterSettings::FIR50Hz => 0,
            EL30XXFilterSettings::FIR60Hz => 1,
            EL30XXFilterSettings::IIR1 => 2,
            EL30XXFilterSettings::IIR2 => 3,
            EL30XXFilterSettings::IIR3 => 4,
            EL30XXFilterSettings::IIR4 => 5,
            EL30XXFilterSettings::IIR5 => 6,
            EL30XXFilterSettings::IIR6 => 7,
            EL30XXFilterSettings::IIR7 => 8,
            EL30XXFilterSettings::IIR8 => 9,
        }
    }
}