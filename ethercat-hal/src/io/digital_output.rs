use std::{fmt, future::Future, pin::Pin, sync::Arc};

use tokio::sync::RwLock;

pub struct DigitalOutput {
    pub write: Box<dyn Fn(bool) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>,
    pub state:
        Box<dyn Fn() -> Pin<Box<dyn Future<Output = DigitalOutputState> + Send>> + Send + Sync>,
}

impl fmt::Debug for DigitalOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DigitalOutput")
    }
}

impl DigitalOutput {
    pub fn new<PORT>(
        device: Arc<RwLock<dyn DigitalOutputDevice<PORT>>>,
        port: PORT,
    ) -> DigitalOutput
    where
        PORT: Clone + Send + Sync + 'static,
    {
        // build async write closure
        let port1 = port.clone();
        let device1 = device.clone();
        let write = Box::new(
            move |value: bool| -> Pin<Box<dyn Future<Output = ()> + Send>> {
                let device_clone = device1.clone();
                let port_clone = port1.clone();
                Box::pin(async move {
                    let mut device = device_clone.write().await;
                    device.digital_output_write(port_clone, value);
                })
            },
        );

        // build async get closure
        let port2 = port.clone();
        let device2 = device.clone();
        let state = Box::new(
            move || -> Pin<Box<dyn Future<Output = DigitalOutputState> + Send>> {
                let device2 = device2.clone();
                let port_clone = port2.clone();
                Box::pin(async move {
                    let device = device2.read().await;
                    device.digital_output_state(port_clone)
                })
            },
        );
        DigitalOutput { write, state }
    }
}

#[derive(Debug, Clone)]
pub struct DigitalOutputState {
    /// Nanosecond timestamp
    pub output_ts: u64,
    /// Output value
    /// true: high
    /// false: low
    pub output: DigitalOutputOutput,
}

#[derive(Debug, Clone)]
pub struct DigitalOutputOutput {
    pub value: bool,
}

pub trait DigitalOutputDevice<PORT>: Send + Sync
where
    PORT: Clone,
{
    fn digital_output_write(&mut self, port: PORT, value: bool);
    fn digital_output_state(&self, port: PORT) -> DigitalOutputState;
}
