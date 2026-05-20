use std::fmt;
use units::electric_current::milliampere;

use super::analog_input::AnalogInput;
use super::analog_input::physical::AnalogInputValue;

/// AS006 flow sensor converter (4–20 mA → l/min).
///
/// Formula: Q [l/min] = 0.938 × (I [mA] − 4)
/// Range: 0 l/min at 4 mA, 15 l/min at 20 mA
pub struct As006Flow {
    input: AnalogInput,
}

impl fmt::Debug for As006Flow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "As006Flow")
    }
}

impl As006Flow {
    pub fn new(input: AnalogInput) -> Self {
        Self { input }
    }

    /// Returns measured current in mA, or `None` on wiring/protocol errors.
    pub fn get_current_ma(&self) -> Option<f64> {
        if self.input.get_wiring_error() {
            return None;
        }
        match self.input.get_physical() {
            AnalogInputValue::Current(c) => Some(c.get::<milliampere>()),
            _ => None,
        }
    }

    /// Returns flow in liters per minute, or `None` if the input reports a wiring error.
    pub fn get_flow_lpm(&self) -> Option<f64> {
        let current_ma = self.get_current_ma()?;
        Some(0.938 * (current_ma - 4.0))
    }
}

/// AS006 temperature sensor converter (4–20 mA → °C).
///
/// Formula: T [°C] = 9.375 × (I [mA] − 4) − 25
/// Range: −25 °C at 4 mA, 125 °C at 20 mA
pub struct As006Temp {
    input: AnalogInput,
}

impl fmt::Debug for As006Temp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "As006Temp")
    }
}

impl As006Temp {
    pub fn new(input: AnalogInput) -> Self {
        Self { input }
    }

    /// Returns measured current in mA, or `None` on wiring/protocol errors.
    pub fn get_current_ma(&self) -> Option<f64> {
        if self.input.get_wiring_error() {
            return None;
        }
        match self.input.get_physical() {
            AnalogInputValue::Current(c) => Some(c.get::<milliampere>()),
            _ => None,
        }
    }

    /// Returns temperature in degrees Celsius, or `None` if the input reports a wiring error.
    pub fn get_temperature_celsius(&self) -> Option<f64> {
        let current_ma = self.get_current_ma()?;
        Some(9.375 * (current_ma - 4.0) - 25.0)
    }
}
