use std::time::Instant;

use units::thermodynamic_temperature::degree_celsius;

use crate::gluetex::HeatingZone;

use super::zone::TemperatureController;

#[derive(Debug)]
pub struct HeatingBank {
    pub zones: [TemperatureController; 6],
    pub enabled: bool,
}

impl HeatingBank {
    pub fn new(zones: [TemperatureController; 6]) -> Self {
        Self {
            zones,
            enabled: false,
        }
    }

    pub fn zone_mut(&mut self, zone: HeatingZone) -> &mut TemperatureController {
        &mut self.zones[zone.index()]
    }

    pub fn zone(&self, zone: HeatingZone) -> &TemperatureController {
        &self.zones[zone.index()]
    }

    pub fn update_all(&mut self, now: Instant) {
        for controller in &mut self.zones {
            controller.update(now);
        }
    }

    pub fn any_autotuning(&self) -> bool {
        self.zones.iter().any(TemperatureController::is_autotuning)
    }

    pub fn any_over_temperature(&self) -> bool {
        self.zones
            .iter()
            .any(TemperatureController::is_over_temperature)
    }

    pub fn over_temperature_zone_mask(&self) -> u8 {
        let mut mask = 0u8;
        for (i, zone) in self.zones.iter().enumerate() {
            if zone.is_over_temperature() {
                mask |= 1 << i;
            }
        }
        mask
    }

    pub fn allow_all_heating(&mut self) {
        for controller in &mut self.zones {
            controller.allow_heating();
        }
    }

    pub fn disallow_all_heating(&mut self) {
        for controller in &mut self.zones {
            controller.disallow_heating();
        }
    }

    pub fn zone_temperature_celsius(&self, index: usize) -> f64 {
        self.zones[index]
            .heating
            .temperature
            .get::<degree_celsius>()
    }
}
