use crate::ethercat_drivers::{
    actor::Actor, io::digital_output::DigitalOutput, utils::traits::ArcRwLock,
};

/// Set a digital output high and low with a given interval
pub struct StepperDriverMaxSpeed {
    pulse: DigitalOutput,
}

impl StepperDriverMaxSpeed {
    pub fn new(output: DigitalOutput) -> Self {
        Self { pulse: output }
    }
}

impl Actor for StepperDriverMaxSpeed {
    fn act(&mut self, _now_ts: u64) {
        let state = (self.pulse.state)();
        match state.value {
            true => (self.pulse.write)(false),
            false => (self.pulse.write)(true),
        }
    }
}

impl ArcRwLock for StepperDriverMaxSpeed {}
