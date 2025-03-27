use super::Actor;
use ethercat_hal::io::analog_input::AnalogInput;
use std::{future::Future, pin::Pin};
use uom::si::{
    electric_current::milliampere,
    electric_potential::volt,
    f32::{ElectricCurrent, ElectricPotential},
};

#[derive(Debug)]
pub enum AnalogInputValue {
    Potential(ElectricPotential),
    Current(ElectricCurrent),
}

#[derive(Debug)]
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

pub enum AnalogInputDevice {
    EL300x,
    EL304x,
    EL305x,
    EL306x,
    EL3062_0030,
}

impl From<AnalogInputDevice> for AnalogInputRange {
    fn from(device: AnalogInputDevice) -> Self {
        match device {
            AnalogInputDevice::EL300x => AnalogInputRange::Potential {
                min: ElectricPotential::new::<volt>(-10.0),
                max: ElectricPotential::new::<volt>(10.0),
            },
            AnalogInputDevice::EL304x => AnalogInputRange::Current {
                min: ElectricCurrent::new::<milliampere>(0.0),
                max: ElectricCurrent::new::<milliampere>(20.0),
            },
            AnalogInputDevice::EL305x => AnalogInputRange::Current {
                min: ElectricCurrent::new::<milliampere>(4.0),
                max: ElectricCurrent::new::<milliampere>(20.0),
            },
            AnalogInputDevice::EL306x => AnalogInputRange::Potential {
                min: ElectricPotential::new::<volt>(0.0),
                max: ElectricPotential::new::<volt>(10.0),
            },
            AnalogInputDevice::EL3062_0030 => AnalogInputRange::Potential {
                min: ElectricPotential::new::<volt>(0.0),
                max: ElectricPotential::new::<volt>(30.0),
            },
        }
    }
}

/// Log the state of a analog input
#[derive(Debug)]
pub struct AnalogInputGetter {
    input: AnalogInput,
    normalized: Option<f32>,
    pub range: AnalogInputRange,
}

impl AnalogInputGetter {
    pub fn new(input: AnalogInput, range: AnalogInputRange) -> Self {
        Self {
            input,
            normalized: None,
            range,
        }
    }

    /// Value from -1.0 to 1.0
    pub fn get_normalized(&self) -> Option<f32> {
        self.normalized
    }

    fn normalized_to_physical(&self, normalized: f32) -> Option<AnalogInputValue> {
        // map -1/1 to 0/1
        let clipped = normalized / 2.0 + 0.5;

        match self.range {
            AnalogInputRange::Potential { min, max } => {
                let value = min + (max - min).abs() * clipped;
                Some(AnalogInputValue::Potential(value))
            }
            AnalogInputRange::Current { min, max } => {
                let value = min + (max - min).abs() * clipped;
                Some(AnalogInputValue::Current(value))
            }
        }
    }

    pub fn get_physical(&self) -> Option<AnalogInputValue> {
        let normalized = match self.get_normalized() {
            Some(value) => value,
            None => return None,
        };

        self.normalized_to_physical(normalized)
    }
}

impl Actor for AnalogInputGetter {
    fn act(&mut self, _now_ts: u64) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let state = (self.input.state)().await;
            self.normalized = Some(state.input.normalized);
        })
    }
}

#[cfg(test)]
mod tests {
    use core::f32;

    use super::*;
    use approx::assert_relative_eq;
    use ethercat_hal::io::analog_input_dummy::AnalogInputDummy;

    #[test]
    fn test_analog_input_getter_voltage() {
        let analog_input_getter = AnalogInputGetter::new(
            AnalogInputDummy::new().analog_input(),
            AnalogInputRange::Potential {
                min: ElectricPotential::new::<volt>(-10.0),
                max: ElectricPotential::new::<volt>(10.0),
            },
        );

        // Check that normalized -1.0 is -10V
        let value = analog_input_getter.normalized_to_physical(-1.0).unwrap();
        match value {
            AnalogInputValue::Potential(v) => {
                assert_relative_eq!(v.get::<volt>(), -10.0, epsilon = f32::EPSILON);
            }
            _ => panic!("Expected a potential value"),
        }

        // Check that normalized 0.0 is 0V
        let value = analog_input_getter.normalized_to_physical(0.0).unwrap();
        match value {
            AnalogInputValue::Potential(v) => {
                assert_relative_eq!(v.get::<volt>(), 0.0, epsilon = f32::EPSILON);
            }
            _ => panic!("Expected a potential value"),
        }

        // Check that normalized 0.5 is 5V
        let value = analog_input_getter.normalized_to_physical(0.5).unwrap();
        match value {
            AnalogInputValue::Potential(v) => {
                assert_relative_eq!(v.get::<volt>(), 5.0, epsilon = f32::EPSILON);
            }
            _ => panic!("Expected a potential value"),
        }

        // Check that normalized 1.0 is 10V
        let value = analog_input_getter.normalized_to_physical(1.0).unwrap();
        match value {
            AnalogInputValue::Potential(v) => {
                assert_relative_eq!(v.get::<volt>(), 10.0, epsilon = f32::EPSILON);
            }
            _ => panic!("Expected a potential value"),
        }
    }

    #[test]
    // 4mA to 20mA
    fn test_analog_input_getter_current() {
        let analog_input_getter = AnalogInputGetter::new(
            AnalogInputDummy::new().analog_input(),
            AnalogInputRange::Current {
                min: ElectricCurrent::new::<milliampere>(4.0),
                max: ElectricCurrent::new::<milliampere>(20.0),
            },
        );

        // Check that normalized -1.0 is 4mA
        let value = analog_input_getter.normalized_to_physical(-1.0).unwrap();
        match value {
            AnalogInputValue::Current(v) => {
                assert_relative_eq!(v.get::<milliampere>(), 4.0, epsilon = f32::EPSILON);
            }
            _ => panic!("Expected a current value"),
        }

        // Check that normalized 0.0 is 12mA
        let value = analog_input_getter.normalized_to_physical(0.0).unwrap();
        match value {
            AnalogInputValue::Current(v) => {
                assert_relative_eq!(v.get::<milliampere>(), 12.0, epsilon = f32::EPSILON);
            }
            _ => panic!("Expected a current value"),
        }

        // Check that normalized 0.5 is 16mA
        let value = analog_input_getter.normalized_to_physical(0.5).unwrap();
        match value {
            AnalogInputValue::Current(v) => {
                assert_relative_eq!(v.get::<milliampere>(), 16.0, epsilon = f32::EPSILON);
            }
            _ => panic!("Expected a current value"),
        }

        // Check that normalized 1.0 is 20mA
        let value = analog_input_getter.normalized_to_physical(1.0).unwrap();
        match value {
            AnalogInputValue::Current(v) => {
                assert_relative_eq!(v.get::<milliampere>(), 20.0, epsilon = f32::EPSILON);
            }
            _ => panic!("Expected a current value"),
        }
    }
}
