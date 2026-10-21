use std::time::Duration;

use qitech_framework::EnumProperty;
use qitech_lib::units::ThermodynamicTemperature;
use qitech_lib::units::VolumeRate;
use qitech_lib::units::thermodynamic_temperature::degree_celsius;
use qitech_lib::units::volume_rate::liter_per_minute;

/// Which segment of the fan ramp the cooling output is currently on.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, EnumProperty)]
pub enum CoolingMode {
    #[default]
    Low,
    Ramp,
    Max,
}

#[derive(Debug, Clone, Copy)]
pub struct PidGains {
    pub kp: f64,
    pub ki: f64,
    pub kd: f64,
}

/// Shape of the fan ramp, expressed as how far above target the loop sits.
#[derive(Debug, Clone, Copy)]
pub struct CoolingRampConfig {
    pub tolerance: ThermodynamicTemperature,
    pub near_band: ThermodynamicTemperature,
    pub full_band: ThermodynamicTemperature,
    pub min_rpm: f64,
}

impl Default for CoolingRampConfig {
    fn default() -> Self {
        Self {
            tolerance: ThermodynamicTemperature::new::<degree_celsius>(0.8),
            near_band: ThermodynamicTemperature::new::<degree_celsius>(2.0),
            full_band: ThermodynamicTemperature::new::<degree_celsius>(4.0),
            min_rpm: 20.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ControllerConfig {
    /// Flow below this is treated as no flow at all by every thermal interlock.
    pub min_flow_for_thermal: VolumeRate,
    /// How long a freshly started pump may report no flow before the low-flow trip arms.
    pub pump_startup_grace_period: Duration,
    /// How long flow may stay below the minimum, once armed, before the pump is stopped.
    pub low_flow_grace_period: Duration,
    /// Serves both thermal holds: how long flow must be steady before the heater may
    /// start, and how long the pump keeps circulating after the heater stops.
    pub thermal_flow_settle_duration: Duration,
    pub heating_element_power: f64,
    pub heating_pwm_period: Duration,
    /// Minimum dwell between relay switches, to stop the contacts chattering.
    pub relay_min_on_time: Duration,
    pub relay_min_off_time: Duration,
    /// Error beyond which the heater skips the duty cycle and runs continuously.
    pub heating_full_power_error: ThermodynamicTemperature,
    pub heating_tolerance: ThermodynamicTemperature,
    pub pump_cooldown_min_temperature: ThermodynamicTemperature,
    /// Hard limits: crossing either cuts the matching actuator regardless of demand.
    pub min_temperature: ThermodynamicTemperature,
    pub max_temperature: ThermodynamicTemperature,
    pub cooling: CoolingRampConfig,
}

impl Default for ControllerConfig {
    fn default() -> Self {
        Self {
            min_flow_for_thermal: VolumeRate::new::<liter_per_minute>(0.2),
            pump_startup_grace_period: Duration::from_secs(15),
            low_flow_grace_period: Duration::from_secs(5),
            thermal_flow_settle_duration: Duration::from_secs(10),
            heating_element_power: 700.0,
            heating_pwm_period: Duration::from_secs(12),
            relay_min_on_time: Duration::from_secs(5),
            relay_min_off_time: Duration::from_secs(5),
            heating_full_power_error: ThermodynamicTemperature::new::<degree_celsius>(4.0),
            heating_tolerance: ThermodynamicTemperature::new::<degree_celsius>(0.4),
            pump_cooldown_min_temperature: ThermodynamicTemperature::new::<degree_celsius>(45.0),
            min_temperature: ThermodynamicTemperature::new::<degree_celsius>(10.0),
            max_temperature: ThermodynamicTemperature::new::<degree_celsius>(80.0),
            cooling: CoolingRampConfig::default(),
        }
    }
}
