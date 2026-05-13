use std::fmt;
use units::electric_current::milliampere;

use super::analog_input::AnalogInput;
use super::analog_input::physical::AnalogInputValue;

/// AS008 flow sensor converter (4–20 mA → l/min).
///
/// Formula: Q [l/min] = 3.125 × (I [mA] − 4)
/// Range: 0 l/min at 4 mA, 50 l/min at 20 mA
pub struct As008Flow {
    input: AnalogInput,
}

impl fmt::Debug for As008Flow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "As008Flow")
    }
}

impl As008Flow {
    pub fn new(input: AnalogInput) -> Self {
        Self { input }
    }

    /// Returns flow in liters per minute, or `None` if the input reports a wiring error.
    pub fn get_flow_lpm(&self) -> Option<f64> {
        if self.input.get_wiring_error() {
            return None;
        }
        let current_ma = match self.input.get_physical() {
            AnalogInputValue::Current(c) => c.get::<milliampere>(),
            _ => return None,
        };
        Some(3.125 * (current_ma - 4.0))
    }
}

/// AS008 temperature sensor converter (4–20 mA → °C).
///
/// Formula: T [°C] = 9.375 × (I [mA] − 4) − 25
/// Range: −25 °C at 4 mA, 125 °C at 20 mA
pub struct As008Temp {
    input: AnalogInput,
}

impl fmt::Debug for As008Temp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "As008Temp")
    }
}

impl As008Temp {
    pub fn new(input: AnalogInput) -> Self {
        Self { input }
    }

    /// Returns temperature in degrees Celsius, or `None` if the input reports a wiring error.
    pub fn get_temperature_celsius(&self) -> Option<f64> {
        if self.input.get_wiring_error() {
            return None;
        }
        let current_ma = match self.input.get_physical() {
            AnalogInputValue::Current(c) => c.get::<milliampere>(),
            _ => return None,
        };
        Some(9.375 * (current_ma - 4.0) - 25.0)
    }
}
