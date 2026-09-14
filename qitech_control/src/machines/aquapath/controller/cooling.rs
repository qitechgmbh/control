use std::time::Instant;

use qitech_lib::units::AngularVelocity;
use qitech_lib::units::angular_velocity::revolution_per_minute;
use qitech_lib::units::thermodynamic_temperature::degree_celsius;

use super::config::ControllerConfig;
use super::config::CoolingMode;
use super::config::CoolingRampConfig;
use super::io::DwellRelay;
use super::io::FanOutput;

/// Owns the cooling relay and the fan drive behind it.
pub struct CoolingController {
    relay: DwellRelay,
    fan: FanOutput,
    allowed: bool,
    mode: Option<CoolingMode>,
    revolutions: AngularVelocity,
    max_revolutions: AngularVelocity,
}

impl CoolingController {
    pub fn new(relay: DwellRelay, fan: FanOutput, max_revolutions: AngularVelocity) -> Self {
        Self {
            relay,
            fan,
            allowed: false,
            mode: None,
            revolutions: AngularVelocity::new::<revolution_per_minute>(0.0),
            max_revolutions,
        }
    }

    pub fn is_on(&self) -> bool {
        self.relay.is_on()
    }

    pub fn is_allowed(&self) -> bool {
        self.allowed
    }

    pub fn allow(&mut self) {
        self.allowed = true;
    }

    pub fn disallow(&mut self) {
        self.allowed = false;
    }

    pub fn mode(&self) -> Option<CoolingMode> {
        self.mode
    }

    pub fn revolutions(&self) -> AngularVelocity {
        self.revolutions
    }

    pub fn set_max_revolutions(&mut self, revolutions: AngularVelocity) {
        self.max_revolutions = revolutions;
    }

    /// Runs the fan at the point on the ramp matching how far above target the loop is.
    pub fn drive(&mut self, temp_offset: f64, now: Instant, config: &ControllerConfig) {
        self.relay.request(
            true,
            now,
            config.relay_min_on_time,
            config.relay_min_off_time,
        );

        // The dwell may still be holding the relay open; don't spin a fan with no supply.
        if !self.relay.is_on() {
            return;
        }

        let max_rpm = self.max_revolutions.get::<revolution_per_minute>();
        let (rpm, mode) = target_rpm(temp_offset, max_rpm, config.cooling);
        let rpm = rpm.clamp(0.0, max_rpm);

        self.fan.set_rpm(rpm);
        self.revolutions = AngularVelocity::new::<revolution_per_minute>(rpm);
        self.mode = Some(mode);
    }

    /// Normal, dwell-limited shutdown. The fan is only parked if the relay really opened.
    pub fn request_off(&mut self, now: Instant, config: &ControllerConfig) {
        let switched = self.relay.request(
            false,
            now,
            config.relay_min_on_time,
            config.relay_min_off_time,
        );
        if switched {
            self.park_fan();
        }
    }

    /// Opens the relay immediately and parks the fan, ignoring the dwell.
    pub fn force_off(&mut self, now: Instant) {
        self.relay.force_off(now);
        self.park_fan();
    }

    fn park_fan(&mut self) {
        self.fan.set_rpm(0.0);
        self.mode = None;
        self.revolutions = AngularVelocity::new::<revolution_per_minute>(0.0);
    }
}

/// Maps "how many degrees above target" onto a fan speed and the ramp segment it fell in.
fn target_rpm(temp_offset: f64, max_rpm: f64, config: CoolingRampConfig) -> (f64, CoolingMode) {
    let full_band = config.full_band.get::<degree_celsius>();
    let near_band = config.near_band.get::<degree_celsius>();
    let tolerance = config.tolerance.get::<degree_celsius>();

    if temp_offset >= full_band {
        // Far above target: cool aggressively.
        return (max_rpm, CoolingMode::Max);
    }

    if temp_offset >= near_band {
        // Mid-range: ramp from 60% to 100% of max.
        let t = (temp_offset - near_band) / (full_band - near_band);
        return ((0.6 + 0.4 * t) * max_rpm, CoolingMode::Ramp);
    }

    // Near target but still above tolerance: low, smooth cooling, ramped from the
    // minimum up to 60% of max to reduce overshoot and chatter.
    let t = ((temp_offset - tolerance) / (near_band - tolerance)).clamp(0.0, 1.0);
    let lower = config.min_rpm.min(max_rpm);
    let upper = 0.6 * max_rpm;
    (lower + (upper - lower) * t, CoolingMode::Low)
}
