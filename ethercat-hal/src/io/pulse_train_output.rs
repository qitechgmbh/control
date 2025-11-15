use std::{fmt, sync::Arc};

use smol::lock::RwLock;

/// Pulse Train Output (PTO) device
///
/// Generates digital puleses with a given frequency (not PWM) and counts them.
pub struct PulseTrainOutput {
    /// Write to the pulse train output
    set_output: Box<dyn Fn(PulseTrainOutputOutput) + Send + Sync>,
    /// Read the state of the pulse train output
    get_output: Box<dyn Fn() -> PulseTrainOutputOutput + Send + Sync>,
    get_input: Box<dyn Fn() -> PulseTrainOutputInput + Send + Sync>,
}

impl fmt::Debug for PulseTrainOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PulseTrainOutput")
    }
}

impl<'device> PulseTrainOutput {
    pub fn new<PORT, DEVICE>(device: Arc<RwLock<DEVICE>>, port: PORT) -> Self
    where
        PORT: Clone + Copy + Send + Sync + 'static,
        DEVICE: PulseTrainOutputDevice<PORT> + Send + Sync + 'static,
    {
        // build sync write closure
        let device1 = device.clone();
        let set_output = Box::new(move |value: PulseTrainOutputOutput| {
            let mut device = smol::block_on(device1.write());
            device.set_output(port, value);
        });

        // build sync get closure
        let device2 = device.clone();
        let get_output = Box::new(move || -> PulseTrainOutputOutput {
            let device = smol::block_on(device2.read());
            device.get_output(port)
        });

        // build sync get closure
        let device2 = device;
        let get_input = Box::new(move || -> PulseTrainOutputInput {
            let device = smol::block_on(device2.read());
            device.get_input(port)
        });

        Self {
            set_output,
            get_output,
            get_input,
        }
    }

    /// Set the frequency value
    pub fn set_frequency(&mut self, frequency: i32) {
        let mut output = (self.get_output)();
        output.frequency_value = frequency;
        (self.set_output)(output);
    }

    /// Get the current frequency value
    pub fn get_frequency(&self) -> i32 {
        let output = (self.get_output)();
        output.frequency_value
    }

    /// Get the current encoder position (counter value)
    pub fn get_position(&self) -> u32 {
        let input = (self.get_input)();
        input.counter_value
    }
}

#[derive(Debug, Clone)]
pub struct PulseTrainOutputState {
    pub input: PulseTrainOutputInput,
    pub output: PulseTrainOutputOutput,
}

#[derive(Debug, Clone)]
pub struct PulseTrainOutputInput {
    pub select_end_counter: bool,
    pub ramp_active: bool,
    pub input_t: bool,
    pub input_z: bool,
    pub error: bool,
    pub sync_error: bool,
    pub counter_underflow: bool,
    pub counter_overflow: bool,
    pub counter_value: u32,
    pub set_counter_done: bool,
}

#[derive(Debug, Clone)]
pub struct PulseTrainOutputOutput {
    pub disble_ramp: bool,
    pub frequency_value: i32,
    pub target_counter_value: u32,
    pub set_counter: bool,
    pub set_counter_value: u32,
}

pub trait PulseTrainOutputDevice<PORT>: Send + Sync
where
    PORT: Clone,
{
    fn set_output(&mut self, port: PORT, value: PulseTrainOutputOutput);
    fn get_output(&self, port: PORT) -> PulseTrainOutputOutput;
    fn get_input(&self, port: PORT) -> PulseTrainOutputInput;
}
