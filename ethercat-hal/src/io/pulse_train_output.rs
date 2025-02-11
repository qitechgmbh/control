use std::{future::Future, pin::Pin, sync::Arc};

use tokio::sync::RwLock;

pub struct PulseTrainOutput {
    pub write: Box<
        dyn Fn(PulseTrainOutputOutput) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync,
    >,
    pub state:
        Box<dyn Fn() -> Pin<Box<dyn Future<Output = PulseTrainOutputState> + Send>> + Send + Sync>,
}

impl PulseTrainOutput {
    pub fn new<PORT>(
        device: Arc<RwLock<dyn PulseTrainOutputDevice<PORT>>>,
        port: PORT,
    ) -> PulseTrainOutput
    where
        PORT: Clone + Send + Sync + 'static,
    {
        // build async write closure
        let port1 = port.clone();
        let device1 = device.clone();
        let write = Box::new(
            move |value: PulseTrainOutputOutput| -> Pin<Box<dyn Future<Output = ()> + Send>> {
                let device_clone = device1.clone();
                let port_clone = port1.clone();
                Box::pin(async move {
                    let mut device = device_clone.write().await;
                    device.pulse_train_output_write(port_clone, value);
                })
            },
        );

        // build async get closure
        let port2 = port.clone();
        let device2 = device.clone();
        let state = Box::new(
            move || -> Pin<Box<dyn Future<Output = PulseTrainOutputState> + Send>> {
                let device2 = device2.clone();
                let port_clone = port2.clone();
                Box::pin(async move {
                    let device = device2.read().await;
                    device.pulse_train_output_state(port_clone)
                })
            },
        );

        PulseTrainOutput { write, state }
    }
}

#[derive(Debug, Clone)]
pub struct PulseTrainOutputState {
    /// Nanosecond timestamp
    pub output_ts: u64,
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
