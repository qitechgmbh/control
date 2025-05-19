use crate::helpers::ethercrab_types::EthercrabSubDevicePreoperational;

impl EL30XXChannelConfiguration {
    pub async fn write_channel_config<'a>(
        &self,
        device: &EthercrabSubDevicePreoperational<'a>,
        base_index: u16,
    ) -> Result<(), anyhow::Error> {
        device
            .sdo_write(base_index, 0x01, self.enable_user_scale)
            .await?;
        device
            .sdo_write(base_index, 0x02, u8::from(self.presentation))
            .await?;
        device
            .sdo_write(base_index, 0x05, self.siemens_bits)
            .await?;
        device
            .sdo_write(base_index, 0x06, self.enable_filter)
            .await?;
        device
            .sdo_write(base_index, 0x07, self.enable_limit_1)
            .await?;
        device
            .sdo_write(base_index, 0x08, self.enable_limit_2)
            .await?;
        device
            .sdo_write(base_index, 0x0A, self.enable_user_calibration)
            .await?;
        device
            .sdo_write(base_index, 0x0B, self.enable_vendor_calibration)
            .await?;
        device
            .sdo_write(base_index, 0x0E, self.swap_limit_bits)
            .await?;
        device
            .sdo_write(base_index, 0x11, self.user_scale_offset)
            .await?;
        device
            .sdo_write(base_index, 0x12, self.user_scale_gain)
            .await?;
        device.sdo_write(base_index, 0x13, self.limit_1).await?;
        device.sdo_write(base_index, 0x14, self.limit_2).await?;
        device
            .sdo_write(base_index, 0x15, u16::from(self.filter_settings))
            .await?;
        device
            .sdo_write(base_index, 0x17, self.user_calibration_offset)
            .await?;
        device
            .sdo_write(base_index, 0x18, self.user_calibration_gain)
            .await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct EL30XXChannelConfiguration {
    // 80n0:01 User Scaling is Active
    pub enable_user_scale: bool,

    // 80n0:02
    // 0: Signed presentation
    // 1: Unsigned presentation
    // 2: Absolute value with MSB as sign
    // Signed amount representation
    pub presentation: EL30XXPresentation,

    // 80n0:05
    // the S5 bits are displayed in the three low-order bits
    pub siemens_bits: bool,

    // 80n0:06
    // Enable filter, which makes PLC-cycle-synchronous
    // data exchange unnecessary
    pub enable_filter: bool,

    // 80n0:07
    // limit 1 enabled
    pub enable_limit_1: bool,

    // 80n0:08
    // limit2 enabled
    pub enable_limit_2: bool,

    // 80n0:0A
    // enabling of user_calibration
    pub enable_user_calibration: bool,

    //  80n0:0B
    // enabling of vendor_calibration
    pub enable_vendor_calibration: bool,

    // 80n0:0E
    // Swap Limit Bits
    pub swap_limit_bits: bool,

    // 80n0:11
    // User Scaling Offset
    pub user_scale_offset: i16,

    // 80n0:12
    // Gain of the user scaling
    // The gain has a fixed-point-representation with the factor
    // 2^-16 The value 1 corresponds to 65535 and is limited to +/- 0x7FFFFF
    pub user_scale_gain: i32,

    // 80n0:13
    // First limit value for setting the status bits
    pub limit_1: i16,

    // 80n0:14
    // Second limit value for setting the status bits
    pub limit_2: i16,

    // 80n0:15
    /*
       This object determines the digital filter settings if it is
       active via Enable filter (Index base_index:06 [} 320]).
       The possible settings are numbered consecutively.
       0: 50 Hz FIR
       1: 60 Hz FIR
       2: IIR 1
       3: IIR 2
       ...
       9: IIR 8
    */
    pub filter_settings: EL30XXFilterSettings,

    // 80n0:17
    //  User calibration offset
    pub user_calibration_offset: i16,

    // 80n0:18
    // User calibration gain
    pub user_calibration_gain: i16,
}

impl Default for EL30XXChannelConfiguration {
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
