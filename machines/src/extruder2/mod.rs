#[cfg(not(feature = "mock-machine"))]
use std::time::Instant;

#[cfg(not(feature = "mock-machine"))]
use api::ExtruderV3Namespace;
#[cfg(not(feature = "mock-machine"))]
use smol::channel::Receiver;
#[cfg(not(feature = "mock-machine"))]
use smol::channel::Sender;

use serde::{Deserialize, Serialize};

#[cfg(not(feature = "mock-machine"))]
use units::electric_current::ampere;

#[cfg(not(feature = "mock-machine"))]
use units::electric_potential::volt;

#[cfg(not(feature = "mock-machine"))]
use crate::MACHINE_EXTRUDER_V2;

#[cfg(not(feature = "mock-machine"))]
use crate::{AsyncThreadMessage, Machine};

#[cfg(not(feature = "mock-machine"))]
use crate::{
    MachineMessage, VENDOR_QITECH,
    extruder1::{
        screw_speed_controller::ScrewSpeedController, temperature_controller::TemperatureController,
    },
    machine_identification::{MachineIdentification, MachineIdentificationUnique},
};

pub mod act;
pub mod api;
pub mod emit;
pub mod mock;
pub mod new;
pub mod temperature_controller;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum ExtruderV3Mode {
    Standby,
    Heat,
    Extrude,
}

pub enum HeatingType {
    Nozzle,
    Front,
    Back,
    Middle,
}

/// Watchdog state for monitoring heating progress
#[cfg(not(feature = "mock-machine"))]
#[derive(Debug, Clone)]
pub struct HeatingWatchdogZone {
    /// Temperature when heating started for this zone
    start_temperature: Option<f64>,
    /// Timestamp when heating started
    heating_start_time: Option<Instant>,
    /// Whether a fault has been detected
    fault_detected: bool,
    /// Last second that was logged
    last_logged_second: Option<u64>,
}

#[cfg(not(feature = "mock-machine"))]
impl Default for HeatingWatchdogZone {
    fn default() -> Self {
        Self {
            start_temperature: None,
            heating_start_time: None,
            fault_detected: false,
            last_logged_second: None,
        }
    }
}

#[cfg(not(feature = "mock-machine"))]
impl HeatingWatchdogZone {
    fn check(
        &mut self,
        controller: &TemperatureController,
        zone_name: &str,
        now: Instant,
        min_temperature_delta_c: f64,
        watchdog_timeout_secs: u64,
    ) -> bool {
        use std::time::Duration;
        use units::thermodynamic_temperature::degree_celsius;

        let current_temp = controller.heating.temperature.get::<degree_celsius>();
        let target_temp = controller.heating.target_temperature.get::<degree_celsius>();
        let is_heating_active = controller.heating.heating;

        // Check if heating should be active 
        let should_be_heating = target_temp > current_temp;

        // Start monitoring when heating becomes active
        if should_be_heating && is_heating_active {
            if self.start_temperature.is_none() {
                self.start_temperature = Some(current_temp);
                self.heating_start_time = Some(now);
                self.fault_detected = false;
                tracing::info!(
                    "Heating watchdog started for {}: start_temp={:.1}°C, target_temp={:.1}°C",
                    zone_name,
                    current_temp,
                    target_temp
                );
            }
        } else {
            // Reset if heating is no longer active
            if self.start_temperature.is_some() {
                tracing::info!(
                    "Heating watchdog reset for {}: heating no longer active",
                    zone_name
                );
                *self = HeatingWatchdogZone::default();
            }
            return false; // No fault when not monitoring
        }

        // Check for fault if we're monitoring
        if let (Some(start_temp), Some(start_time)) = (self.start_temperature, self.heating_start_time)
        {
            let elapsed = now.duration_since(start_time);
            let temp_increase = current_temp - start_temp;

            // Check if timeout exceeded without sufficient temperature increase
            if elapsed >= Duration::from_secs(watchdog_timeout_secs)
                && temp_increase < min_temperature_delta_c
                && !self.fault_detected
            {
                self.fault_detected = true;
                tracing::warn!(
                    "Heating watchdog fault detected for {}: elapsed={:.1}s, temp_increase={:.2}°C, start_temp={:.1}°C, current_temp={:.1}°C, target_temp={:.1}°C",
                    zone_name,
                    elapsed.as_secs_f64(),
                    temp_increase,
                    start_temp,
                    current_temp,
                    target_temp
                );
                return true; // Fault detected
            }

            // Log progress periodically
            let current_second = elapsed.as_secs();
            if current_second > 0 && current_second % 10 == 0 {
                let should_log = match self.last_logged_second {
                    Some(last) => current_second > last,
                    None => true,
                };

                if should_log {
                    tracing::debug!(
                        "Heating watchdog progress for {}: elapsed={:.1}s, temp_increase={:.2}°C",
                        zone_name,
                        elapsed.as_secs_f64(),
                        temp_increase
                    );
                    self.last_logged_second = Some(current_second);
                }
            }
        }

        false // No fault
    }
}

/// Watchdog state for all heating zones
#[cfg(not(feature = "mock-machine"))]
#[derive(Debug, Clone)]
pub struct HeatingWatchdog {
    pub front: HeatingWatchdogZone,
    pub middle: HeatingWatchdogZone,
    pub back: HeatingWatchdogZone,
    pub nozzle: HeatingWatchdogZone,
}

#[cfg(not(feature = "mock-machine"))]
impl Default for HeatingWatchdog {
    fn default() -> Self {
        Self {
            front: HeatingWatchdogZone::default(),
            middle: HeatingWatchdogZone::default(),
            back: HeatingWatchdogZone::default(),
            nozzle: HeatingWatchdogZone::default(),
        }
    }
}

#[cfg(not(feature = "mock-machine"))]
#[derive(Debug)]
pub struct ExtruderV3 {
    api_receiver: Receiver<MachineMessage>,
    api_sender: Sender<MachineMessage>,
    main_sender: Option<Sender<AsyncThreadMessage>>,

    machine_identification_unique: MachineIdentificationUnique,
    namespace: ExtruderV3Namespace,

    last_measurement_emit: Instant,
    last_status_hash: Option<u64>,
    mode: ExtruderV3Mode,

    screw_speed_controller: ScrewSpeedController,
    temperature_controller_front: TemperatureController,
    temperature_controller_middle: TemperatureController,
    temperature_controller_back: TemperatureController,
    temperature_controller_nozzle: TemperatureController,

    /// Energy tracking for total consumption calculation
    total_energy_kwh: f64,
    last_energy_calculation_time: Option<Instant>,

    /// will be initalized as false and set to true by `emit_state`
    /// This way we can signal to the client that the first state emission is a default state
    emitted_default_state: bool,

    /// Heating safeguard configuration
    heating_safeguard_enabled: bool,

    /// Heating watchdog state for each zone
    heating_watchdog: HeatingWatchdog,

    /// Current heating fault state
    heating_fault_state: crate::extruder2::api::HeatingFaultState,
}

#[cfg(not(feature = "mock-machine"))]
impl Machine for ExtruderV3 {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}

#[cfg(not(feature = "mock-machine"))]
impl std::fmt::Display for ExtruderV3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ExtruderV3")
    }
}

#[cfg(not(feature = "mock-machine"))]
impl ExtruderV3 {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: MACHINE_EXTRUDER_V2,
    };
}

#[cfg(not(feature = "mock-machine"))]
impl ExtruderV3 {
    /// Calculate combined power consumption in watts
    fn calculate_combined_power(&mut self) -> f64 {
        let motor_power = {
            let motor_status = &self.screw_speed_controller.inverter.motor_status;
            let voltage = motor_status.voltage.get::<volt>();
            let current = motor_status.current.get::<ampere>();
            voltage * current
        };
        let nozzle_power = self
            .temperature_controller_nozzle
            .get_heating_element_wattage();
        let front_power = self
            .temperature_controller_front
            .get_heating_element_wattage();
        let back_power = self
            .temperature_controller_back
            .get_heating_element_wattage();
        let middle_power = self
            .temperature_controller_middle
            .get_heating_element_wattage();

        motor_power + nozzle_power + front_power + back_power + middle_power
    }

    /// Update total energy consumption in kWh
    fn update_total_energy(&mut self, current_power_watts: f64, now: Instant) {
        if let Some(last_time) = self.last_energy_calculation_time {
            let time_delta_hours = now.duration_since(last_time).as_secs_f64() / 3600.0;
            let energy_delta_kwh = (current_power_watts / 1000.0) * time_delta_hours;
            self.total_energy_kwh += energy_delta_kwh;
        }
        self.last_energy_calculation_time = Some(now);
    }

    // Functions without emit_state remain here

    // Set all relays to ZERO
    // We don't need a function to enable again though, as the act loop will detect the mode
    fn turn_heating_off(&mut self) {
        self.temperature_controller_back.disable();
        self.temperature_controller_front.disable();
        self.temperature_controller_middle.disable();
        self.temperature_controller_nozzle.disable();
    }

    fn switch_to_standby(&mut self) {
        match self.mode {
            ExtruderV3Mode::Standby => (),
            ExtruderV3Mode::Heat => {
                self.turn_heating_off();
                self.screw_speed_controller.reset_pid();
            }
            ExtruderV3Mode::Extrude => {
                self.turn_heating_off();
                self.screw_speed_controller.turn_motor_off();
                self.screw_speed_controller.reset_pid();
            }
        };
        self.mode = ExtruderV3Mode::Standby;
    }

    fn switch_to_heat(&mut self) {
        match self.mode {
            ExtruderV3Mode::Standby => self.enable_heating(),
            ExtruderV3Mode::Heat => (),
            ExtruderV3Mode::Extrude => {
                self.screw_speed_controller.turn_motor_off();
                self.screw_speed_controller.reset_pid();
            }
        }
        self.mode = ExtruderV3Mode::Heat;
    }

    fn switch_to_extrude(&mut self) {
        match self.mode {
            ExtruderV3Mode::Standby => {
                self.screw_speed_controller.turn_motor_on();
                self.enable_heating();
                self.screw_speed_controller.reset_pid();
            }
            ExtruderV3Mode::Heat => {
                self.screw_speed_controller.turn_motor_on();
                self.enable_heating();
                self.screw_speed_controller.reset_pid();
            }
            ExtruderV3Mode::Extrude => (),
        }
        self.mode = ExtruderV3Mode::Extrude;
    }

    fn switch_mode(&mut self, mode: ExtruderV3Mode) {
        if self.mode == mode {
            return;
        }

        match mode {
            ExtruderV3Mode::Standby => self.switch_to_standby(),
            ExtruderV3Mode::Heat => self.switch_to_heat(),
            ExtruderV3Mode::Extrude => self.switch_to_extrude(),
        }
    }

    fn reset_inverter(&mut self) {
        self.screw_speed_controller.inverter.reset_inverter();
    }

    /// Update heating watchdog for all zones
    /// Monitors temperature increases and detects heating faults
    fn update_heating_watchdog(&mut self, now: Instant) {
        // Watchdog parameters
        // These defaults are intentionally conservative to avoid false positives
        // on older machines or in cold environments. If we do not see at least
        // ~5°C increase within one minute, we treat this as a fault.
        const MIN_TEMPERATURE_DELTA_C: f64 = 5.0; // Minimum temperature increase in °C
        const WATCHDOG_TIMEOUT_SECS: u64 = 60; // Timeout in seconds

        // Only monitor when in Heat or Extrude mode
        if self.mode == ExtruderV3Mode::Standby {
            // Reset all watchdog states when in standby
            self.heating_watchdog.front = HeatingWatchdogZone::default();
            self.heating_watchdog.middle = HeatingWatchdogZone::default();
            self.heating_watchdog.back = HeatingWatchdogZone::default();
            self.heating_watchdog.nozzle = HeatingWatchdogZone::default();
            return;
        }

        // Check all zones
        let mut fault_detected = false;
        if self.heating_watchdog.front.check(
            &self.temperature_controller_front,
            "front",
            now,
            MIN_TEMPERATURE_DELTA_C,
            WATCHDOG_TIMEOUT_SECS,
        ) {
            fault_detected = true;
        }
        if self.heating_watchdog.middle.check(
            &self.temperature_controller_middle,
            "middle",
            now,
            MIN_TEMPERATURE_DELTA_C,
            WATCHDOG_TIMEOUT_SECS,
        ) {
            fault_detected = true;
        }
        if self.heating_watchdog.back.check(
            &self.temperature_controller_back,
            "back",
            now,
            MIN_TEMPERATURE_DELTA_C,
            WATCHDOG_TIMEOUT_SECS,
        ) {
            fault_detected = true;
        }
        if self.heating_watchdog.nozzle.check(
            &self.temperature_controller_nozzle,
            "nozzle",
            now,
            MIN_TEMPERATURE_DELTA_C,
            WATCHDOG_TIMEOUT_SECS,
        ) {
            fault_detected = true;
        }

        // If fault detected, automatically set to standby and record fault
        if fault_detected {
            // Determine which zone(s) have faults
            let fault_zones: Vec<&str> = [
                (self.heating_watchdog.front.fault_detected, "front"),
                (self.heating_watchdog.middle.fault_detected, "middle"),
                (self.heating_watchdog.back.fault_detected, "back"),
                (self.heating_watchdog.nozzle.fault_detected, "nozzle"),
            ]
            .iter()
            .filter_map(|(has_fault, zone)| if *has_fault { Some(*zone) } else { None })
            .collect();

            // In theory fault_detected guarantees at least one zone here, but use a
            // defensive default to avoid panicking on an empty list in case the logic
            // above is changed in the future.
            let first_fault_zone = fault_zones
                .first()
                .copied()
                .unwrap_or("unknown");

            if first_fault_zone == "unknown" {
                tracing::error!(
                    "Heating watchdog reported fault_detected=true but no individual zone was marked as faulty"
                );
            }

            let fault_zone_str = if fault_zones.is_empty() {
                "unknown".to_string()
            } else {
                fault_zones.join(", ")
            };
            tracing::error!(
                "Heating fault detected in zone(s): {} - automatically setting extruder to standby",
                fault_zone_str
            );

            // Set fault state
            self.heating_fault_state.fault_zone = Some(first_fault_zone.to_string());
            self.heating_fault_state.fault_acknowledged = false;

            self.switch_to_standby();
            self.emit_state();
        }
    }
}
