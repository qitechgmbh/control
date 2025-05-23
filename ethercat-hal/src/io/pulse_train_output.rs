use std::{fmt, future::Future, pin::Pin, sync::Arc};

use smol::lock::RwLock;

/// Pulse Train Output (PTO) device
///
/// Generates digital puleses with a given frequency (not PWM) and counts them.
pub struct PulseTrainOutput {
    /// Write to the pulse train output
    pub write: Box<
        dyn Fn(PulseTrainOutputOutput) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync,
    >,
    /// Read the state of the pulse train output
    pub state:
        Box<dyn Fn() -> Pin<Box<dyn Future<Output = PulseTrainOutputState> + Send>> + Send + Sync>,
}

impl fmt::Debug for PulseTrainOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PulseTrainOutput")
    }
}

impl<'device> PulseTrainOutput {
    pub fn new<PORT, DEVICE>(device: Arc<RwLock<DEVICE>>, port: PORT) -> PulseTrainOutput
    where
        PORT: Clone + Copy + Send + Sync + 'static,
        DEVICE: PulseTrainOutputDevice<PORT> + Send + Sync + 'static,
    {
        // build async write closure
        let device1 = device.clone();
        let write = Box::new(
            move |value: PulseTrainOutputOutput| -> Pin<Box<dyn Future<Output = ()> + Send>> {
                let device_clone = device1.clone();
                Box::pin(async move {
                    let mut device = device_clone.write().await;
                    device.pulse_train_output_write(port, value);
                })
            },
        );

        // build async get closure
        let device2 = device.clone();
        let state = Box::new(
            move || -> Pin<Box<dyn Future<Output = PulseTrainOutputState> + Send>> {
                let device2 = device2.clone();
                Box::pin(async move {
                    let device = device2.read().await;
                    device.pulse_train_output_state(port)
                })
            },
        );

        PulseTrainOutput { write, state }
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
    fn pulse_train_output_write(&mut self, port: PORT, value: PulseTrainOutputOutput);
    fn pulse_train_output_state(&self, port: PORT) -> PulseTrainOutputState;
}
