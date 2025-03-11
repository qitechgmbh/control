use super::Actor;
use crate::io::pulse_train_output::{
    PulseTrainOutput, PulseTrainOutputInput, PulseTrainOutputOutput,
};
use std::{future::Future, pin::Pin, sync::Arc};
use tokio::sync::RwLock;

/// Set a digital output high and low with a given interval
#[derive(Debug)]
pub struct StepperDriverPulseTrain {
    pulse: PulseTrainOutput,

    // track position
    counter_cleared_last: bool,
    last_target_position: u32,
    position: i64,
}

impl<'ptodevice> StepperDriverPulseTrain {
    pub fn new(output: PulseTrainOutput) -> Self {
        Self {
            pulse: output,
            counter_cleared_last: false,
            last_target_position: 0,
            position: 0,
        }
    }
}

impl<'ptodevice> Actor for StepperDriverPulseTrain {
    fn act(&mut self, _now_ts: u64) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let state = (self.pulse.state)().await;
            let mut output = state.output.clone();
            log::info!("state: {:#?}", state);
            log::info!("position: {:#?}", self.position);

            // self.counter_to_position(&state.input, &mut output);

            output.frequency_value = 6000;

            (self.pulse.write)(output).await;
        })
    }
}

impl<'ptodevice> StepperDriverPulseTrain {
    /// since the internal counter of 32bit is a bit small, we use this counter instead
    fn counter_to_position(
        &mut self,
        input: &PulseTrainOutputInput,
        output: &mut PulseTrainOutputOutput,
    ) {
        self.position += input.counter_value as i64;
        match self.counter_cleared_last {
            true => {
                output.set_counter = false;
                self.counter_cleared_last = false;
            }
            false => {
                output.set_counter_value = 0;
                output.set_counter = true;
                // reduce target positon by the amount since
                self.counter_cleared_last = true;
            }
        }
    }
}

impl<'ptodevice> From<StepperDriverPulseTrain> for Arc<RwLock<StepperDriverPulseTrain>> {
    fn from(actor: StepperDriverPulseTrain) -> Self {
        Arc::new(RwLock::new(actor))
    }
}
