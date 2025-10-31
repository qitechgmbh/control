use super::analog_input::physical::AnalogInputRange;
use super::analog_input::{AnalogInput, AnalogInputDevice, AnalogInputInput};
use smol::lock::RwLock;
use std::sync::Arc;

use std::sync::Mutex;

#[derive(Clone)]
pub struct AnalogInputDummyPort;

pub struct AnalogInputDummy {
    state: Arc<Mutex<AnalogInputInput>>,
    range: AnalogInputRange,
}

impl AnalogInputDevice<AnalogInputDummyPort> for AnalogInputDummy {
    fn get_input(&self, _port: AnalogInputDummyPort) -> AnalogInputInput {
        let guard = self.state.lock().unwrap();
        guard.clone()
    }

    fn analog_input_range(&self) -> AnalogInputRange {
        self.range.clone()
    }
}

impl AnalogInputDummy {
    pub fn new(range: AnalogInputRange) -> Self {
        let state = Arc::new(Mutex::new(AnalogInputInput {
            normalized: 0.0,
            wiring_error: false,
        }));
        Self { state, range }
    }

    pub fn analog_input(&self) -> AnalogInput {
        let device: Arc<RwLock<dyn AnalogInputDevice<AnalogInputDummyPort>>> =
            Arc::new(RwLock::new(Self {
                state: self.state.clone(),
                range: self.range.clone(),
            }));
        AnalogInput::new(device, AnalogInputDummyPort)
    }

    pub fn set_input(&mut self, input: AnalogInputInput) {
        {
            let mut input_guard = self.state.lock().unwrap();
            *input_guard = input;
        }
    }

    pub fn get_input(&self) -> AnalogInputInput {
        let input_guard = self.state.lock().unwrap();
        input_guard.clone()
    }
}

#[cfg(test)]
mod tests {
    use units::{electric_potential::volt, f64::ElectricPotential};

    use super::*;

    #[test]
    fn test_analog_input_dummy() {
        let mut dummy = AnalogInputDummy::new(AnalogInputRange::Potential {
            min: ElectricPotential::new::<volt>(0.0),
            max: ElectricPotential::new::<volt>(10.0),
            min_raw: 0,
            max_raw: i16::MAX,
        });
        let input = AnalogInputInput {
            normalized: 0.5,
            wiring_error: false,
        };
        dummy.set_input(input.clone());

        let analog_input = dummy.analog_input();
        let normalized = analog_input.get_normalized();
        assert_eq!(normalized, 0.5);
    }
}
