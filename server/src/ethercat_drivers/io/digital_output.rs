use std::sync::Arc;

use parking_lot::RwLock;

pub struct DigitalOutput {
    pub write: Box<dyn Fn(bool) -> () + Send + Sync>,
    pub state: Box<dyn Fn() -> DigitalOutputState + Send + Sync>,
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
        let write = Box::new(move |value| {
            let mut device = device1.write();
            device.digital_output_write(port1.clone(), value)
        });

        // build async get closure
        let port2 = port.clone();
        let device2 = device.clone();
        let state = Box::new(move || {
            let device = device2.read();
            device.digital_output_state(port2.clone())
        });

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
    pub value: bool,
}

pub trait DigitalOutputDevice<PORT>: Send + Sync
where
    PORT: Clone,
{
    fn digital_output_write(&mut self, port: PORT, value: bool);
    fn digital_output_state(&self, port: PORT) -> DigitalOutputState;
}
