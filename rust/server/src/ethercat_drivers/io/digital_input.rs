use parking_lot::RwLock;
use std::sync::Arc;

type Value = bool;

pub struct DigitalInput {
    pub state: Box<dyn Fn() -> DigitalInputState + Send + Sync>,
}

impl DigitalInput {
    pub fn new<PORT>(device: Arc<RwLock<dyn DigitalInputDevice<PORT>>>, port: PORT) -> DigitalInput
    where
        PORT: Clone + Send + Sync + 'static,
    {
        // build async get closure
        let port2 = port.clone();
        let device2 = device.clone();
        let state = Box::new(move || {
            let device = device2.read();
            device.digital_input_state(port2.clone())
        });

        DigitalInput { state }
    }
}

#[derive(Debug, Clone)]
pub struct DigitalInputState {
    /// Nanosecond timestamp
    pub input_ts: u64,
    /// Input value
    /// true: high
    /// false: low
    pub value: Value,
}

pub trait DigitalInputDevice<PORTS>: Send + Sync
where
    PORTS: Clone,
{
    fn digital_input_state(&self, port: PORTS) -> DigitalInputState;
}
