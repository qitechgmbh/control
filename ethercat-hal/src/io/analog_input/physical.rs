use uom::si::f64::{ElectricCurrent, ElectricPotential};

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
    },
    Current {
        min: ElectricCurrent,
        max: ElectricCurrent,
    },
}

impl AnalogInputRange {
    /// Convert a normalized value (-1.0 to 1.0) to a physical value
    pub fn normalized_to_physical(&self, normalized: f32) -> AnalogInputValue {
        // map -1/1 to 0/1
        let clipped = normalized as f64 / 2.0 + 0.5;

        match self {
            AnalogInputRange::Potential { min, max } => {
                let value = *min + (*max - *min).abs() * clipped;
                AnalogInputValue::Potential(value)
            }
            AnalogInputRange::Current { min, max } => {
                let value = *min + (*max - *min).abs() * clipped;
                AnalogInputValue::Current(value)
            }
        }
    }
}

// test using AnalogInputRange not AnalogInputGetter
#[cfg(test)]
mod tests {
    use core::f64;

    use super::*;
    use approx::assert_relative_eq;
    use uom::si::electric_current::milliampere;
    use uom::si::electric_potential::volt;
    use uom::si::f64::{ElectricCurrent, ElectricPotential};

    #[test]
    fn test_analog_input_getter_voltage() {
        let analog_input_getter = AnalogInputRange::Potential {
            min: ElectricPotential::new::<volt>(-10.0),
            max: ElectricPotential::new::<volt>(10.0),
        };

        // Check that normalized -1.0 is -10V
        let value = analog_input_getter.normalized_to_physical(-1.0);
        match value {
            AnalogInputValue::Potential(v) => {
                assert_relative_eq!(v.get::<volt>(), -10.0, epsilon = f64::EPSILON);
            }
            _ => panic!("Expected a potential value"),
        }

        // Check that normalized 0.0 is 0V
        let value = analog_input_getter.normalized_to_physical(0.0);
        match value {
            AnalogInputValue::Potential(v) => {
                assert_relative_eq!(v.get::<volt>(), 0.0, epsilon = f64::EPSILON);
            }
            _ => panic!("Expected a potential value"),
        }

        // Check that normalized 0.5 is 5V
        let value = analog_input_getter.normalized_to_physical(0.5);
        match value {
            AnalogInputValue::Potential(v) => {
                assert_relative_eq!(v.get::<volt>(), 5.0, epsilon = f64::EPSILON);
            }
            _ => panic!("Expected a potential value"),
        }

        // Check that normalized 1.0 is 10V
        let value = analog_input_getter.normalized_to_physical(1.0);
        match value {
            AnalogInputValue::Potential(v) => {
                assert_relative_eq!(v.get::<volt>(), 10.0, epsilon = f64::EPSILON);
            }
            _ => panic!("Expected a potential value"),
        }
    }

    #[test]
    // 4mA to 20mA
    fn test_analog_input_getter_current() {
        let analog_input_getter = AnalogInputRange::Current {
            min: ElectricCurrent::new::<milliampere>(4.0),
            max: ElectricCurrent::new::<milliampere>(20.0),
        };

        // Check that normalized -1.0 is 4mA
        let value = analog_input_getter.normalized_to_physical(-1.0);
        match value {
            AnalogInputValue::Current(v) => {
                assert_relative_eq!(v.get::<milliampere>(), 4.0, epsilon = f64::EPSILON);
            }
            _ => panic!("Expected a current value"),
        }

        // Check that normalized 0.0 is 12mA
        let value = analog_input_getter.normalized_to_physical(0.0);
        match value {
            AnalogInputValue::Current(v) => {
                assert_relative_eq!(v.get::<milliampere>(), 12.0, epsilon = f64::EPSILON);
            }
            _ => panic!("Expected a current value"),
        }

        // Check that normalized 0.5 is 16mA
        let value = analog_input_getter.normalized_to_physical(0.5);
        match value {
            AnalogInputValue::Current(v) => {
                assert_relative_eq!(v.get::<milliampere>(), 16.0, epsilon = f64::EPSILON);
            }
            _ => panic!("Expected a current value"),
        }

        // Check that normalized 1.0 is 20mA
        let value = analog_input_getter.normalized_to_physical(1.0);
        match value {
            AnalogInputValue::Current(v) => {
                assert_relative_eq!(v.get::<milliampere>(), 20.0, epsilon = f64::EPSILON);
            }
            _ => panic!("Expected a current value"),
        }
    }
}
