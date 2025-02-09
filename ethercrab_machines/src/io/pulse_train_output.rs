use std::{future::Future, pin::Pin, sync::Arc};

use tokio::sync::RwLock;

pub struct PulseTrainOutput {
    pub write: Box<
        dyn Fn(PulseTrainOutputWrite) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync,
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
            move |value: PulseTrainOutputWrite| -> Pin<Box<dyn Future<Output = ()> + Send>> {
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
    pub input: PulseTrainOutputRead,
    pub output: PulseTrainOutputWrite,
}

#[derive(Debug, Clone)]
pub struct PulseTrainOutputRead {
    // 0x1A01
    pub status_select_end_counter: bool,
    pub status_ramp_active: bool,
    pub status_input_t: bool,
    pub status_input_z: bool,
    pub status_error: bool,
    pub status_sync_error: bool,
    pub status_txpdo_toggle: bool,
    // 0x1A05
    pub set_counter_done: bool,
    pub counter_underflow: bool,
    pub counter_overflow: bool,
    pub counter_value: u64,
}

#[derive(Debug, Clone)]
pub struct PulseTrainOutputWrite {
    // 0x1601
    pub frequency_select: bool,
    pub disble_ramp: bool,
    pub go_counter: bool,
    pub frequency_value: u32,
    // 0x1607 UDINT
    pub target_counter_value: u64,
    // 0x1605
    pub set_counter: bool,
    pub set_counter_value: u64,
}

pub trait PulseTrainOutputDevice<PORT>: Send + Sync
where
    PORT: Clone,
{
    fn pulse_train_output_write(&mut self, port: PORT, value: PulseTrainOutputWrite);
    fn pulse_train_output_state(&self, port: PORT) -> PulseTrainOutputState;
}
