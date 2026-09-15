use qitech_framework::machine::ActResult;
use qitech_lib::units::thermodynamic_temperature::degree_celsius;
use serde::Serialize;

use super::AquaPathV1;
use super::reservoir::Side;

#[derive(Serialize, Debug, Clone)]
pub struct NoticeEvent {
    pub title: String,
    pub message: String,
}

/// Handlers the framework invokes when a config property is written or a command runs.
/// Per-reservoir handlers are generic over the [`Side`] they act on.
impl AquaPathV1 {
    pub fn on_target_temperature_changed<S: Side>(&mut self) -> ActResult {
        let min_settable = self.min_settable_temperature().get::<degree_celsius>();
        S::reservoir(self).apply_target_temperature(min_settable);
        Ok(())
    }

    pub fn on_fan_max_revolutions_changed<S: Side>(&mut self) -> ActResult {
        S::reservoir(self).apply_fan_max_revolutions();
        Ok(())
    }

    pub fn on_heating_tolerance_changed<S: Side>(&mut self) -> ActResult {
        S::reservoir(self).apply_heating_tolerance();
        Ok(())
    }

    pub fn on_cooling_tolerance_changed<S: Side>(&mut self) -> ActResult {
        S::reservoir(self).apply_cooling_tolerance();
        Ok(())
    }

    pub fn on_pid_changed<S: Side>(&mut self) -> ActResult {
        S::reservoir(self).apply_pid();
        Ok(())
    }

    pub fn on_thermal_flow_settle_duration_changed<S: Side>(&mut self) -> ActResult {
        if self.allows_safety_config_changes() {
            S::reservoir(self).apply_thermal_flow_settle_duration();
        }
        Ok(())
    }

    pub fn on_pump_cooldown_min_temperature_changed<S: Side>(&mut self) -> ActResult {
        if self.allows_safety_config_changes() {
            S::reservoir(self).apply_pump_cooldown_min_temperature();
        }
        Ok(())
    }

    pub fn on_ambient_temperature_calibration_changed(&mut self) -> ActResult {
        let ambient = self.ambient_temperature_calibration_config.get();
        self.set_ambient_temperature_calibration(ambient);
        Ok(())
    }

    pub fn cmd_start_pump<S: Side>(&mut self) -> ActResult {
        S::reservoir(self).set_should_pump(true);
        Ok(())
    }

    pub fn cmd_stop_pump<S: Side>(&mut self) -> ActResult {
        S::reservoir(self).set_should_pump(false);
        Ok(())
    }
}
