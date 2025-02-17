use std::sync::Arc;
use tokio::sync::RwLock;

use std::future::Future;
use std::pin::Pin;

pub struct AnalogOutput {
    pub write: Box<dyn Fn(f32) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>,
    pub state:
        Box<dyn Fn() -> Pin<Box<dyn Future<Output = AnalogOutputState> + Send>> + Send + Sync>,
}

impl AnalogOutput {
    pub fn new<PORT>(device: Arc<RwLock<dyn AnalogOutputDevice<PORT>>>, port: PORT) -> AnalogOutput
    where
        PORT: Clone + Send + Sync + 'static,
    {
        // build async write closure
        let port1 = port.clone();
        let device1 = device.clone();
        let write = Box::new(
            move |value: f32| -> Pin<Box<dyn Future<Output = ()> + Send>> {
                let device_clone = device1.clone();
                let port_clone = port1.clone();
                Box::pin(async move {
                    let mut device = device_clone.write().await;
                    device.analog_output_write(port_clone, value);
                })
            },
        );

        // build async get closure
        let port2 = port.clone();
        let device2 = device.clone();
        let state = Box::new(
            move || -> Pin<Box<dyn Future<Output = AnalogOutputState> + Send>> {
                let device2 = device2.clone();
                let port_clone = port2.clone();
                Box::pin(async move {
                    let device = device2.read().await;
                    device.analog_output_state(port_clone)
                })
            },
        );
        AnalogOutput { write, state }
    }
}

#[derive(Debug, Clone)]
pub struct AnalogOutputState {
    /// Nanosecond timestamp
    pub output_ts: u64,
    /// Output value from 0.0 to 1.0
    /// Voltage depends on the device
    pub output: AnalogOutputOutput,
}

#[derive(Debug, Clone)]
pub struct AnalogOutputOutput {
    pub value: f32,
}

pub trait AnalogOutputDevice<PORTS>: Send + Sync {
    fn analog_output_write(&mut self, port: PORTS, value: f32);
    fn analog_output_state(&self, port: PORTS) -> AnalogOutputState;
}
