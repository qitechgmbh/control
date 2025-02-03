use parking_lot::RwLock;
use std::sync::Arc;

pub struct AnalogOutput {
    pub write: Box<dyn Fn(f32) -> () + Send + Sync>,
    pub state: Box<dyn Fn() -> AnalogOutputState + Send + Sync>,
}

impl AnalogOutput {
    pub fn new<PORT>(device: Arc<RwLock<dyn AnalogOutputDevice<PORT>>>, port: PORT) -> AnalogOutput
    where
        PORT: Clone + Send + Sync + 'static,
    {
        // build async write closure
        let port1 = port.clone();
        let device1 = device.clone();
        let write = Box::new(move |value| {
            let mut device = device1.write();
            device.analog_output_write(port1.clone(), value)
        });

        // build async get closure
        let port2 = port.clone();
        let device2 = device.clone();
        let state = Box::new(move || {
            let device = device2.read();
            device.analog_output_state(port2.clone())
        });

        AnalogOutput { write, state }
    }
}

#[derive(Debug, Clone)]
pub struct AnalogOutputState {
    /// Nanosecond timestamp
    pub output_ts: u64,
    /// Output value from 0.0 to 1.0
    /// Voltage depends on the device
    pub value: f32,
}

pub trait AnalogOutputDevice<PORTS>: Send + Sync {
    fn analog_output_write(&mut self, port: PORTS, value: f32);
    fn analog_output_state(&self, port: PORTS) -> AnalogOutputState;
}
