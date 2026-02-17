use units::Angle;
use units::angle::degree;

/// Configuration for tension arm monitoring
#[derive(Debug, Clone)]
pub struct TensionArmMonitorConfig {
    pub enabled: bool,
    pub min_angle: Angle,
    pub max_angle: Angle,
}

impl Default for TensionArmMonitorConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            min_angle: Angle::new::<degree>(10.0),
            max_angle: Angle::new::<degree>(170.0),
        }
    }
}

/// Configuration for voltage monitoring
#[derive(Debug, Clone)]
pub struct VoltageMonitorConfig {
    pub enabled: bool,
    pub min_voltage: f64,
    pub max_voltage: f64,
    /// Delay for voltage readings in millimeters of filament travel
    pub delay_mm: f64,
}

impl Default for VoltageMonitorConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            min_voltage: 2.0,
            max_voltage: 8.0,
            delay_mm: 0.0,
        }
    }
}

/// Configuration for sleep timer
#[derive(Debug, Clone)]
pub struct SleepTimerConfig {
    pub enabled: bool,
    pub timeout_seconds: u64,
}

impl Default for SleepTimerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            timeout_seconds: 900, // 15 minutes
        }
    }
}
