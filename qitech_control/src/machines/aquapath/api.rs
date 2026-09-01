use std::time::Instant;

use crate::machines::aquapath::AquaPathV1;

use super::{AquaPathV1Mode, controller::CoolingMode};
use qitech_framework::machine::{ActResult, ConfigProperty, Measurement, StateProperty};
use qitech_lib::units::{
    angular_velocity::revolution_per_minute, thermodynamic_temperature::degree_celsius,
    volume_rate::liter_per_minute,
};
use serde::Serialize;

pub struct Measurements {
    pub left_flow: Measurement<f64>,
    pub right_flow: Measurement<f64>,

    pub left_temperature: Measurement<f64>,
    pub right_temperature: Measurement<f64>,

    pub left_revolutions: Measurement<f64>,
    pub right_revolutions: Measurement<f64>,

    pub left_power: Measurement<f64>,
    pub right_power: Measurement<f64>,

    pub left_total_energy: Measurement<f64>,
    pub right_total_energy: Measurement<f64>,
}

pub struct StateProperties {
    pub mode_state: ModeState,    
    pub left_heating_startup_wait_active: StateProperty<bool>,
    pub right_heating_startup_wait_active: StateProperty<bool>,

    pub left_pump_cooldown_active: StateProperty<bool>,
    pub right_pump_cooldown_active: StateProperty<bool>,

    pub left_should_flow: StateProperty<bool>,
    pub right_should_flow: StateProperty<bool>,

    pub left_heating: StateProperty<bool>,
    pub right_heating: StateProperty<bool>,

    pub left_pump_cooldown_remaining: StateProperty<f64>,
    pub right_pump_cooldown_remaining: StateProperty<f64>,

    pub left_heating_startup_wait_remaining: StateProperty<f64>,
    pub right_heating_startup_wait_remaining: StateProperty<f64>,

    pub left_cooling_mode: StateProperty<Option<CoolingMode>>,
    pub right_cooling_mode: StateProperty<Option<CoolingMode>>,

    pub left_thermal_safety_state: ThermalSafetyState,
    pub right_thermal_safety_state: ThermalSafetyState,
}

pub struct ConfigProperties {
    pub left_target_temperature: ConfigProperty<f64>,
    pub right_target_temperature: ConfigProperty<f64>,
    pub ambient_temperature_calibration: ConfigProperty<f64>,
    pub left_fan_max_revolutions: ConfigProperty<f64>,
    pub right_fan_max_revolutions: ConfigProperty<f64>,
    pub left_tolerance_config: ToleranceState,
    pub right_tolerance_config: ToleranceState,
    pub left_pid_config: PidState,
    pub right_pid_config: PidState,
    pub left_thermal_flow_settle_duration: ConfigProperty<f64>,
    pub right_thermal_flow_settle_duration: ConfigProperty<f64>,
    pub left_pump_cooldown_min_temperature: ConfigProperty<f64>,
    pub right_pump_cooldown_min_temperature: ConfigProperty<f64>,
}

#[derive(Serialize, Debug, Clone)]
pub struct NoticeEvent {
    pub title: String,
    pub message: String,
}

pub struct ModeState {
    pub mode: StateProperty<AquaPathV1Mode>,
}

pub struct ToleranceState {
    pub heating: ConfigProperty<f64>,
    pub cooling: ConfigProperty<f64>,
}

pub struct PidState {
    pub kp: ConfigProperty<f64>,
    pub ki: ConfigProperty<f64>,
    pub kd: ConfigProperty<f64>,
}

pub struct ThermalSafetyState {
    pub thermal_delay: StateProperty<f64>,
    pub cooldown_min_temperature: StateProperty<f64>,
}

impl AquaPathV1 {
    pub fn on_right_target_temparature_changed(&mut self) -> ActResult {
        _ = self.set_target_temperature(
            self.config_props.right_target_temperature.get(),
            super::AquaPathSideType::Right,
        );
        Ok(())
    }

    pub fn on_left_target_temparature_changed(&mut self) -> ActResult {
        _ = self.set_target_temperature(
            self.config_props.left_target_temperature.get(),
            super::AquaPathSideType::Left,
        );
        Ok(())
    }

    pub fn on_set_right_revolutions(&mut self) -> ActResult {
        self.set_max_revolutions(
            self.config_props.right_fan_max_revolutions.get(),
            super::AquaPathSideType::Right,
        );
        Ok(())
    }

    pub fn on_set_left_revolutions(&mut self) -> ActResult {
        self.set_max_revolutions(
            self.config_props.left_fan_max_revolutions.get(),
            super::AquaPathSideType::Left,
        );
        Ok(())
    }

    pub fn on_set_right_heating_tolerance(&mut self) -> ActResult {
        self.set_heating_tolerance(
            self.config_props.right_tolerance_config.heating.get(),
            super::AquaPathSideType::Right,
        );
        Ok(())
    }

    pub fn on_set_left_heating_tolerance(&mut self) -> ActResult {
        self.set_heating_tolerance(
            self.config_props.left_tolerance_config.heating.get(),
            super::AquaPathSideType::Left,
        );
        Ok(())
    }

    pub fn on_set_right_cooling_tolerance(&mut self) -> ActResult {
        self.set_cooling_tolerance(
            self.config_props.right_tolerance_config.cooling.get(),
            super::AquaPathSideType::Right,
        );
        Ok(())
    }

    pub fn on_set_left_cooling_tolerance(&mut self) -> ActResult {
        self.set_cooling_tolerance(
            self.config_props.left_tolerance_config.cooling.get(),
            super::AquaPathSideType::Left,
        );
        Ok(())
    }

    pub fn on_set_left_pid(&mut self) -> ActResult {
        self.set_pid(super::AquaPathSideType::Left);
        Ok(())
    }

    pub fn on_set_right_pid(&mut self) -> ActResult {
        self.set_pid(super::AquaPathSideType::Right);
        Ok(())
    }

    pub fn on_set_left_thermal_flow_settle_duration(&mut self) -> ActResult {
        self.set_thermal_flow_settle_duration(
            self.config_props.left_thermal_flow_settle_duration.get(),
            super::AquaPathSideType::Left,
        );
        Ok(())
    }

    pub fn on_set_right_thermal_flow_settle_duration(&mut self) -> ActResult {
        // TODO change config_props
        self.set_thermal_flow_settle_duration(
            self.config_props.right_thermal_flow_settle_duration.get(),
            super::AquaPathSideType::Right,
        );
        Ok(())
    }

    pub fn on_set_left_cooldown_min_temp(&mut self) -> ActResult {
        self.set_pump_cooldown_min_temperature(
            self.config_props.left_pump_cooldown_min_temperature.get(),
            super::AquaPathSideType::Left,
        );
        Ok(())
    }

    pub fn on_set_right_cooldown_min_temp(&mut self) -> ActResult {
        self.set_pump_cooldown_min_temperature(
            self.config_props.right_pump_cooldown_min_temperature.get(),
            super::AquaPathSideType::Right,
        );
        Ok(())
    }

    pub fn on_set_ambient_temperature_calibration(&mut self) -> ActResult {
        self.set_ambient_temperature_calibration(
            self.config_props.ambient_temperature_calibration.get(),
        );
        Ok(())
    }
}

impl AquaPathV1 {
    pub fn update_measurements(&mut self) {
        //self.left_controller.
        let left_flow = self.left_controller.current_flow.get::<liter_per_minute>();
        let right_flow = self.right_controller.current_flow.get::<liter_per_minute>();

        let left_temp = self
            .left_controller
            .current_temperature
            .get::<degree_celsius>();
        let right_temp = self
            .right_controller
            .current_temperature
            .get::<degree_celsius>();

        let left_revolutions = self
            .left_controller
            .get_current_revolutions()
            .get::<revolution_per_minute>();
        let right_revolutions = self
            .right_controller
            .get_current_revolutions()
            .get::<revolution_per_minute>();

        self.measurements.left_flow.set(left_flow);
        self.measurements.right_flow.set(right_flow);

        self.measurements.left_temperature.set(left_temp);
        self.measurements.right_temperature.set(right_temp);

        self.measurements.left_revolutions.set(left_revolutions);
        self.measurements.right_revolutions.set(right_revolutions);

        self.measurements.left_power.set(self.left_controller.power);
        self.measurements
            .right_power
            .set(self.right_controller.power);

        self.measurements
            .left_total_energy
            .set(self.left_controller.get_total_energy());
        self.measurements
            .right_total_energy
            .set(self.right_controller.get_total_energy());
    }

    pub fn update_states(&mut self, now: Instant) {
        let mode = self.mode.clone();

        let left_heating_startup_wait_active =
            self.left_controller.is_heating_startup_wait_active(now);
        let right_heating_startup_wait_active =
            self.right_controller.is_heating_startup_wait_active(now);

        self.state_props.mode_state.mode.set(mode);
        self.state_props
            .left_heating_startup_wait_active
            .set(left_heating_startup_wait_active);
        self.state_props
            .right_heating_startup_wait_active
            .set(right_heating_startup_wait_active);

        self.state_props
            .left_pump_cooldown_active
            .set(self.left_controller.is_pump_cooldown_active(now));
        self.state_props
            .right_pump_cooldown_active
            .set(self.right_controller.is_pump_cooldown_active(now));

        self.state_props
            .left_should_flow
            .set(self.left_controller.should_pump);
        self.state_props
            .right_should_flow
            .set(self.right_controller.should_pump);

        self.state_props
            .left_heating
            .set(self.left_controller.temperature.heating);
        self.state_props
            .right_heating
            .set(self.right_controller.temperature.heating);

        self.state_props.left_pump_cooldown_remaining.set(
            self.left_controller
                .get_pump_cooldown_remaining(now)
                .as_secs_f64(),
        );

        self.state_props.right_pump_cooldown_remaining.set(
            self.right_controller
                .get_pump_cooldown_remaining(now)
                .as_secs_f64(),
        );

        self.state_props.left_heating_startup_wait_remaining.set(
            self.left_controller
                .get_heating_startup_wait_remaining(now)
                .as_secs_f64(),
        );

        self.state_props.right_heating_startup_wait_remaining.set(
            self.right_controller
                .get_heating_startup_wait_remaining(now)
                .as_secs_f64(),
        );

        self.state_props
            .left_cooling_mode
            .set(self.left_controller.cooling_mode.clone());

        self.state_props
            .right_cooling_mode
            .set(self.right_controller.cooling_mode.clone());

        self.state_props
            .left_thermal_safety_state
            .thermal_delay
            .set(
                self.left_controller
                    .get_thermal_flow_settle_duration()
                    .as_secs_f64(),
            );

        self.state_props
            .left_thermal_safety_state
            .cooldown_min_temperature
            .set(
                self.left_controller
                    .get_pump_cooldown_min_temperature()
                    .get::<degree_celsius>(),
            );

        self.state_props
            .right_thermal_safety_state
            .thermal_delay
            .set(
                self.right_controller
                    .get_thermal_flow_settle_duration()
                    .as_secs_f64(),
            );
        self.state_props
            .right_thermal_safety_state
            .cooldown_min_temperature
            .set(
                self.right_controller
                    .get_pump_cooldown_min_temperature()
                    .get::<degree_celsius>(),
            );
    }
}
