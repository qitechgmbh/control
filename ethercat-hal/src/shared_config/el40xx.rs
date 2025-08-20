use crate::helpers::ethercrab_types::EthercrabSubDevicePreoperational;

#[derive(Debug, Clone)]
pub struct EL40XXChannelConfiguration {
    /// Enable user scale (0x80n0:01) - Default: false (0x00)
    pub enable_user_scale: bool,

    /// Presentation mode (0x80n0:02) - Default: Signed (0x00)
    pub presentation: EL40XXPresentation,

    /// Watchdog behavior (0x80n0:05) - Default: DefaultValue (0x00)
    pub watchdog: EL40XXWatchdog,

    /// Enable user calibration (0x80n0:07) - Default: false (0x00)
    pub enable_user_calibration: bool,

    /// Enable vendor calibration (0x80n0:08) - Default: true (0x01)
    pub enable_vendor_calibration: bool,

    /// User scaling offset (0x80n0:11) - Default: 0x0000
    pub offset: i16,

    /// User scaling gain (0x80n0:12) - Default: 0x00010000 (65536dec)
    /// Fixed-point format with factor 2^-16, where value 1.0 = 65536
    pub gain: i32,

    /// Default output value (0x80n0:13) - Default: 0x0000
    pub default_output: i16,

    /// Default output ramp (0x80n0:14) - Default: 0xFFFF (65535dec)
    /// Value in digits/ms
    pub default_output_ramp: u16,

    /// User calibration offset (0x80n0:15) - Default: 0x0000
    pub user_calibration_offset: i16,

    /// User calibration gain (0x80n0:16) - Default: 0xFFFF (65535dec)
    pub user_calibration_gain: u16,
}

#[derive(Debug, Clone)]
pub enum EL40XXPresentation {
    /// Signed presentation (DEFAULT) - Two's complement format
    /// Range: -32768 to +32767
    Signed,

    /// Unsigned presentation
    /// Range: 0 to +65535
    Unsigned,

    /// Absolute value with MSB as sign - Magnitude-sign format
    /// Range: -32768 to +32767 (not two's complement)
    SignedAbsoluteMSB,

    /// Absolute value - Negative numbers output as positive
    Absolute,
}

impl From<EL40XXPresentation> for u8 {
    fn from(presentation: EL40XXPresentation) -> Self {
        match presentation {
            EL40XXPresentation::Signed => 0,
            EL40XXPresentation::Unsigned => 1,
            EL40XXPresentation::SignedAbsoluteMSB => 2,
            EL40XXPresentation::Absolute => 3,
        }
    }
}

#[derive(Debug, Clone)]
pub enum EL40XXWatchdog {
    /// Default watchdog value (0x80n0:13) is active (DEFAULT)
    DefaultValue,

    /// Watchdog ramp (0x80n0:14) for moving to default value is active
    Ramp,

    /// Last output value - maintains last process data on watchdog drop
    LastValue,
}

impl From<EL40XXWatchdog> for u8 {
    fn from(presentation: EL40XXWatchdog) -> Self {
        match presentation {
            EL40XXWatchdog::DefaultValue => 0,
            EL40XXWatchdog::Ramp => 1,
            EL40XXWatchdog::LastValue => 2,
        }
    }
}

impl Default for EL40XXChannelConfiguration {
    fn default() -> Self {
        Self {
            enable_user_scale: false,                 // 0x80n0:01 = 0x00 (0dec)
            presentation: EL40XXPresentation::Signed, // 0x80n0:02 = 0x00 (0dec) - DEFAULT
            watchdog: EL40XXWatchdog::DefaultValue,   // 0x80n0:05 = 0x00 (0dec)
            enable_user_calibration: false,           // 0x80n0:07 = 0x00 (0dec)
            enable_vendor_calibration: true,          // 0x80n0:08 = 0x01 (1dec)
            offset: 0,                                // 0x80n0:11 = 0x0000 (0dec)
            gain: 65536,                              // 0x80n0:12 = 0x00010000 (65536dec)
            default_output: 0,                        // 0x80n0:13 = 0x0000 (0dec)
            default_output_ramp: 65535,               // 0x80n0:14 = 0xFFFF (65535dec)
            user_calibration_offset: 0,               // 0x80n0:15 = 0x0000 (0dec)
            user_calibration_gain: 65535,             // 0x80n0:16 = 0xFFFF (65535dec)
        }
    }
}

impl EL40XXChannelConfiguration {
    pub async fn write_channel_config<'a>(
        &self,
        device: &EthercrabSubDevicePreoperational<'a>,
        base_index: u16,
    ) -> Result<(), anyhow::Error> {
        tracing::info!("write_channel_config");

        // Write all configuration parameters according to the documentation table
        device
            .sdo_write(base_index, 0x01, self.enable_user_scale as u8)
            .await?;
        device
            .sdo_write(base_index, 0x02, self.presentation.clone() as u8)
            .await?;
        device
            .sdo_write(base_index, 0x05, self.watchdog.clone() as u8)
            .await?;
        device
            .sdo_write(base_index, 0x07, self.enable_user_calibration as u8)
            .await?;
        device
            .sdo_write(base_index, 0x08, self.enable_vendor_calibration as u8)
            .await?;
        device.sdo_write(base_index, 0x11, self.offset).await?;
        device.sdo_write(base_index, 0x12, self.gain).await?;
        device
            .sdo_write(base_index, 0x13, self.default_output)
            .await?;
        device
            .sdo_write(base_index, 0x14, self.default_output_ramp)
            .await?;
        device
            .sdo_write(base_index, 0x15, self.user_calibration_offset)
            .await?;
        device
            .sdo_write(base_index, 0x16, self.user_calibration_gain)
            .await?;

        Ok(())
    }
}
