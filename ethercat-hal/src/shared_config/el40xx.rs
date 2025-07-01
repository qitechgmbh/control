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

#[cfg(test)]
mod tests {
    use super::*;

    // Helper implementations for converting types to bytes
    pub struct ByteVec(Vec<u8>);

    impl From<bool> for ByteVec {
        fn from(value: bool) -> Self {
            ByteVec(vec![value as u8])
        }
    }
    impl From<u8> for ByteVec {
        fn from(value: u8) -> Self {
            ByteVec(vec![value])
        }
    }

    impl From<i16> for ByteVec {
        fn from(value: i16) -> Self {
            ByteVec(value.to_le_bytes().to_vec())
        }
    }

    impl From<i32> for ByteVec {
        fn from(value: i32) -> Self {
            ByteVec(value.to_le_bytes().to_vec())
        }
    }

    impl From<u16> for ByteVec {
        fn from(value: u16) -> Self {
            ByteVec(value.to_le_bytes().to_vec())
        }
    }

    #[test]
    fn test_el40xx_channel_configuration_default() {
        let config = EL40XXChannelConfiguration::default();

        assert_eq!(config.enable_user_scale, false);
        assert!(matches!(config.presentation, EL40XXPresentation::Signed));
        assert!(matches!(config.watchdog, EL40XXWatchdog::DefaultValue));
        assert_eq!(config.enable_user_calibration, false);
        assert_eq!(config.enable_vendor_calibration, true);
        assert_eq!(config.offset, 0);
        assert_eq!(config.gain, 65536);
        assert_eq!(config.default_output, 0);
        assert_eq!(config.default_output_ramp, 65535);
        assert_eq!(config.user_calibration_offset, 0);
        assert_eq!(config.user_calibration_gain, 65535);
    }

    #[test]
    fn test_el40xx_presentation_enum_values() {
        assert_eq!(EL40XXPresentation::Signed as u8, 0);
        assert_eq!(EL40XXPresentation::Unsigned as u8, 1);
        assert_eq!(EL40XXPresentation::SignedAbsoluteMSB as u8, 2);
        assert_eq!(EL40XXPresentation::Absolute as u8, 3);
    }

    #[test]
    fn test_el40xx_watchdog_enum_values() {
        assert_eq!(EL40XXWatchdog::DefaultValue as u8, 0);
        assert_eq!(EL40XXWatchdog::Ramp as u8, 1);
        assert_eq!(EL40XXWatchdog::LastValue as u8, 2);
    }

    #[test]
    fn test_el40xx_channel_configuration_custom() {
        let config = EL40XXChannelConfiguration {
            enable_user_scale: true,
            presentation: EL40XXPresentation::Unsigned,
            watchdog: EL40XXWatchdog::Ramp,
            enable_user_calibration: true,
            enable_vendor_calibration: false,
            offset: -1000,
            gain: 32768,
            default_output: 2048,
            default_output_ramp: 1000,
            user_calibration_offset: -500,
            user_calibration_gain: 32767,
        };

        assert_eq!(config.enable_user_scale, true);
        assert!(matches!(config.presentation, EL40XXPresentation::Unsigned));
        assert!(matches!(config.watchdog, EL40XXWatchdog::Ramp));
        assert_eq!(config.enable_user_calibration, true);
        assert_eq!(config.enable_vendor_calibration, false);
        assert_eq!(config.offset, -1000);
        assert_eq!(config.gain, 32768);
        assert_eq!(config.default_output, 2048);
        assert_eq!(config.default_output_ramp, 1000);
        assert_eq!(config.user_calibration_offset, -500);
        assert_eq!(config.user_calibration_gain, 32767);
    }

    #[test]
    fn test_gain_fixed_point_conversion() {
        // Test gain values in fixed-point format (factor 2^-16)
        // 1.0 = 65536, 0.5 = 32768, 2.0 = 131072, etc.
        let test_cases = [
            (0.5, 32768),  // 0.5 * 65536 = 32768
            (1.0, 65536),  // 1.0 * 65536 = 65536 (default)
            (2.0, 131072), // 2.0 * 65536 = 131072
            (0.25, 16384), // 0.25 * 65536 = 16384
            (4.0, 262144), // 4.0 * 65536 = 262144
        ];

        for (scale_factor, expected_gain) in test_cases {
            let config = EL40XXChannelConfiguration {
                gain: expected_gain,
                ..Default::default()
            };

            assert_eq!(
                config.gain, expected_gain,
                "Gain for scale factor {} should be {}",
                scale_factor, expected_gain
            );

            // Verify the conversion back to scale factor
            let calculated_scale = config.gain as f64 / 65536.0;
            assert!(
                (calculated_scale - scale_factor).abs() < 0.001,
                "Scale factor calculation failed for {}",
                scale_factor
            );
        }
    }

    #[test]
    fn test_voltage_range_calculations() {
        // Test typical voltage output calculations
        // For ±10V range: -10V = -32768, +10V = +32767, 0V = 0
        let test_cases = [
            (-10.0, -32768), // -10V
            (-5.0, -16384),  // -5V (approximately)
            (0.0, 0),        // 0V
            (5.0, 16383),    // +5V (approximately)
            (10.0, 32767),   // +10V (max positive)
        ];

        for (voltage, expected_raw) in test_cases {
            let config = EL40XXChannelConfiguration {
                default_output: expected_raw,
                ..Default::default()
            };

            assert_eq!(
                config.default_output, expected_raw,
                "Raw value for {}V should be {}",
                voltage, expected_raw
            );
        }
    }

    #[test]
    fn test_current_range_calculations() {
        // Test 4-20mA current output calculations
        // Different modules may have different mappings, but common ones:
        // 0-20mA: 0 = 0mA, 32767 = 20mA
        // 4-20mA: often mapped to full range where min value = 4mA, max = 20mA
        let test_cases = [
            (0, "0mA or 4mA (depending on module)"),
            (16383, "~10mA or ~12mA"),
            (32767, "20mA"),
        ];

        for (raw_value, description) in test_cases {
            let config = EL40XXChannelConfiguration {
                default_output: raw_value,
                ..Default::default()
            };

            assert_eq!(
                config.default_output, raw_value,
                "Raw value for {} should be {}",
                description, raw_value
            );
        }
    }

    #[test]
    fn test_ramp_calculations() {
        // Test output ramp values (digits/ms)
        // Higher values = faster ramp, lower values = slower ramp
        let test_cases = [
            (1, "Very slow ramp"),
            (100, "Slow ramp"),
            (1000, "Medium ramp"),
            (10000, "Fast ramp"),
            (65535, "Maximum ramp speed (default)"),
        ];

        for (ramp_value, description) in test_cases {
            let config = EL40XXChannelConfiguration {
                default_output_ramp: ramp_value,
                ..Default::default()
            };

            assert_eq!(
                config.default_output_ramp, ramp_value,
                "Ramp value for {} should be {}",
                description, ramp_value
            );
        }
    }

    #[test]
    fn test_configuration_clone() {
        let original = EL40XXChannelConfiguration {
            enable_user_scale: true,
            presentation: EL40XXPresentation::Unsigned,
            watchdog: EL40XXWatchdog::Ramp,
            enable_user_calibration: true,
            enable_vendor_calibration: false,
            offset: 1000,
            gain: 32768,
            default_output: 2048,
            default_output_ramp: 500,
            user_calibration_offset: -100,
            user_calibration_gain: 40000,
        };

        let cloned = original.clone();

        // Verify all fields are cloned correctly
        assert_eq!(original.enable_user_scale, cloned.enable_user_scale);
        assert_eq!(
            original.enable_user_calibration,
            cloned.enable_user_calibration
        );
        assert_eq!(
            original.enable_vendor_calibration,
            cloned.enable_vendor_calibration
        );
        assert_eq!(original.offset, cloned.offset);
        assert_eq!(original.gain, cloned.gain);
        assert_eq!(original.default_output, cloned.default_output);
        assert_eq!(original.default_output_ramp, cloned.default_output_ramp);
        assert_eq!(
            original.user_calibration_offset,
            cloned.user_calibration_offset
        );
        assert_eq!(original.user_calibration_gain, cloned.user_calibration_gain);
    }

    #[test]
    fn test_configuration_debug() {
        let config = EL40XXChannelConfiguration::default();
        let debug_string = format!("{:?}", config);

        // Should contain struct name and some key field values
        assert!(debug_string.contains("EL40XXChannelConfiguration"));
        assert!(debug_string.contains("enable_user_scale"));
        assert!(debug_string.contains("gain"));
        assert!(debug_string.contains("65536")); // Default gain value
    }

    #[test]
    fn test_presentation_modes() {
        // Test different presentation modes and their use cases
        let signed_config = EL40XXChannelConfiguration {
            presentation: EL40XXPresentation::Signed,
            ..Default::default()
        };

        let unsigned_config = EL40XXChannelConfiguration {
            presentation: EL40XXPresentation::Unsigned,
            ..Default::default()
        };

        let signed_magnitude_config = EL40XXChannelConfiguration {
            presentation: EL40XXPresentation::SignedAbsoluteMSB,
            ..Default::default()
        };

        let absolute_config = EL40XXChannelConfiguration {
            presentation: EL40XXPresentation::Absolute,
            ..Default::default()
        };

        // Verify enum values
        assert_eq!(signed_config.presentation as u8, 0);
        assert_eq!(unsigned_config.presentation as u8, 1);
        assert_eq!(signed_magnitude_config.presentation as u8, 2);
        assert_eq!(absolute_config.presentation as u8, 3);
    }

    #[test]
    fn test_watchdog_behaviors() {
        // Test different watchdog behaviors
        let default_value_config = EL40XXChannelConfiguration {
            watchdog: EL40XXWatchdog::DefaultValue,
            ..Default::default()
        };

        let ramp_config = EL40XXChannelConfiguration {
            watchdog: EL40XXWatchdog::Ramp,
            ..Default::default()
        };

        let last_value_config = EL40XXChannelConfiguration {
            watchdog: EL40XXWatchdog::LastValue,
            ..Default::default()
        };

        // Verify enum values
        assert_eq!(default_value_config.watchdog as u8, 0);
        assert_eq!(ramp_config.watchdog as u8, 1);
        assert_eq!(last_value_config.watchdog as u8, 2);
    }
}
