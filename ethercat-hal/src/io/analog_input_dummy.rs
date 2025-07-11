use super::analog_input::physical::AnalogInputRange;
use super::analog_input::{AnalogInput, AnalogInputInput, AnalogInputState};
use std::{future::Future, pin::Pin, sync::Arc};

use std::sync::Mutex;

pub struct AnalogInputDummy {
    state: Arc<Mutex<AnalogInputState>>,
    range: AnalogInputRange,
}

impl AnalogInputDummy {
    pub fn new(range: AnalogInputRange) -> Self {
        let state = Arc::new(Mutex::new(AnalogInputState {
            input: AnalogInputInput {
                normalized: 0.0,
                wiring_error: false,
            },
        }));
        Self { state, range }
    }

    pub fn analog_input(&mut self) -> AnalogInput {
        let state_arc = self.state.clone();
        let state = Box::new(
            move || -> Pin<Box<dyn Future<Output = AnalogInputState> + Send>> {
                let state_arc = state_arc.clone();
                Box::pin(async move { state_arc.lock().unwrap().clone() })
            },
        );
        AnalogInput {
            state,
            range: self.range.clone(),
        }
    }

    pub fn set_state(&mut self, state: AnalogInputState) {
        let mut guard = self.state.lock().unwrap();
        *guard = state;
    }

    pub fn set_input(&mut self, input: AnalogInputInput) {
        {
            let mut guard = self.state.lock().unwrap();
            guard.input = input;
        }
    }

    pub fn get_input(&self) -> AnalogInputInput {
        let guard = self.state.lock().unwrap();
        guard.input.clone()
    }

    pub fn get_state(&self) -> AnalogInputState {
        let guard = self.state.lock().unwrap();
        guard.clone()
    }
}

#[cfg(test)]
mod tests {
    use uom::si::{electric_potential::volt, f64::ElectricPotential};

    use super::*;

    #[test]
    fn test_analog_input_dummy() {
        let mut dummy = AnalogInputDummy::new(AnalogInputRange::Potential {
            min: ElectricPotential::new::<volt>(0.0),
            max: ElectricPotential::new::<volt>(10.0),
            min_raw: 0,
            max_raw: i16::MAX,
        });
        let state = AnalogInputState {
            input: AnalogInputInput {
                normalized: 0.5,
                wiring_error: false,
            },
        };
        dummy.set_input(state.input.clone());

        let analog_input = dummy.analog_input();
        let state = smol::block_on(async { (analog_input.state)().await });
        assert_eq!(state.input.normalized, 0.5);
    }
}
