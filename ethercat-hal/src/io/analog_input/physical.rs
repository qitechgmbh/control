use units::f64::{ElectricCurrent, ElectricPotential};

#[derive(Debug, Clone)]
pub enum AnalogInputValue {
    Potential(ElectricPotential),
    Current(ElectricCurrent),
}

#[derive(Debug, Clone)]
pub enum AnalogInputRange {
    Potential {
        min: ElectricPotential,
        max: ElectricPotential,
        min_raw: i16,
        max_raw: i16,
    },
    Current {
        min: ElectricCurrent,
        max: ElectricCurrent,
        min_raw: i16,
        max_raw: i16,
    },
}

impl AnalogInputRange {
    pub const fn get_min_raw(&self) -> i16 {
        match self {
            Self::Potential { min_raw, .. } => *min_raw,
            Self::Current { min_raw, .. } => *min_raw,
        }
    }

    pub const fn get_max_raw(&self) -> i16 {
        match self {
            Self::Potential { max_raw, .. } => *max_raw,
            Self::Current { max_raw, .. } => *max_raw,
        }
    }

    pub fn raw_to_normalized(&self, raw_value: i16) -> f64 {
        let range = (self.get_max_raw() as i32 - self.get_min_raw() as i32) as f64;
        (raw_value as i32 - self.get_min_raw() as i32) as f64 / range
    }

    pub fn raw_to_physical(&self, raw_value: i16) -> AnalogInputValue {
        let normalized = self.raw_to_normalized(raw_value);
        match self {
            Self::Potential { min, max, .. } => {
                let value = *min + (*max - *min).abs() * normalized;
                AnalogInputValue::Potential(value)
            }
            Self::Current { min, max, .. } => {
                let value = *min + (*max - *min).abs() * normalized;
                AnalogInputValue::Current(value)
            }
        }
    }

    /// Convert a normalized value (0 to 1.0) to a physical value
    pub fn normalized_to_physical(&self, normalized: f32) -> AnalogInputValue {
        match self {
            Self::Potential { min, max, .. } => {
                let value = *min + (*max - *min).abs() * normalized as f64;
                AnalogInputValue::Potential(value)
            }
            Self::Current { min, max, .. } => {
                let value = *min + (*max - *min).abs() * normalized as f64;
                AnalogInputValue::Current(value)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use core::f64;

    use super::*;
    use approx::assert_relative_eq;
    use units::electric_current::milliampere;
    use units::electric_potential::volt;
    use units::f64::{ElectricCurrent, ElectricPotential};

    #[test]
    fn test_analog_input_getter_voltage() {
        let analog_input_getter = AnalogInputRange::Potential {
            min: ElectricPotential::new::<volt>(-10.0),
            max: ElectricPotential::new::<volt>(10.0),
            min_raw: i16::MIN,
            max_raw: i16::MAX,
        };

        // -32768 raw = -10V
        let value = analog_input_getter.raw_to_physical(i16::MIN);
        match value {
            AnalogInputValue::Potential(v) => {
                assert_relative_eq!(v.get::<volt>(), -10.0, epsilon = f64::EPSILON);
            }
            _ => panic!("Expected a potential value"),
        }

        // 0 raw ~ 0V
        let value = analog_input_getter.raw_to_physical(0);
        match value {
            AnalogInputValue::Potential(v) => {
                assert_relative_eq!(v.get::<volt>(), 0.0, epsilon = 0.01);
            }
            _ => panic!("Expected a potential value"),
        }

        // 32767 raw = 10V
        let value = analog_input_getter.raw_to_physical(i16::MAX);
        match value {
            AnalogInputValue::Potential(v) => {
                assert_relative_eq!(v.get::<volt>(), 10.0, epsilon = f64::EPSILON);
            }
            _ => panic!("Expected a potential value"),
        }
    }

    #[test]
    fn test_analog_input_getter_current() {
        let analog_input_getter = AnalogInputRange::Current {
            min: ElectricCurrent::new::<milliampere>(4.0),
            max: ElectricCurrent::new::<milliampere>(20.0),
            min_raw: 0,
            max_raw: i16::MAX,
        };

        // 0 raw = 4mA
        let value = analog_input_getter.raw_to_physical(0);
        match value {
            AnalogInputValue::Current(v) => {
                assert_relative_eq!(v.get::<milliampere>(), 4.0, epsilon = f64::EPSILON);
            }
            _ => panic!("Expected a current value"),
        }

        // 16383 raw ~ 12mA
        let value = analog_input_getter.raw_to_physical(i16::MAX / 2);
        match value {
            AnalogInputValue::Current(v) => {
                assert_relative_eq!(v.get::<milliampere>(), 12.0, epsilon = 0.01);
            }
            _ => panic!("Expected a current value"),
        }

        // 32767 raw = 20mA
        let value = analog_input_getter.raw_to_physical(i16::MAX);
        match value {
            AnalogInputValue::Current(v) => {
                assert_relative_eq!(v.get::<milliampere>(), 20.0, epsilon = f64::EPSILON);
            }
            _ => panic!("Expected a current value"),
        }
    }
}
