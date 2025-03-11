use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct AnalogInput {
    pub state:
        Box<dyn Fn() -> Pin<Box<dyn Future<Output = AnalogInputState> + Send>> + Send + Sync>,
}

impl fmt::Debug for AnalogInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AnalogInput")
    }
}

impl AnalogInput {
    pub fn new<PORT>(device: Arc<RwLock<dyn AnalogInputDevice<PORT>>>, port: PORT) -> AnalogInput
    where
        PORT: Clone + Send + Sync + 'static,
    {
        // build async get closure
        let port2 = port.clone();
        let device2 = device.clone();
        let state = Box::new(
            move || -> Pin<Box<dyn Future<Output = AnalogInputState> + Send>> {
                let device2 = device2.clone();
                let port_clone = port2.clone();
                Box::pin(async move {
                    let device = device2.read().await;
                    device.analog_output_state(port_clone)
                })
            },
        );
        AnalogInput { state }
    }
}

#[derive(Debug, Clone)]
pub struct AnalogInputState {
    /// Nanosecond timestamp
    pub input_ts: u64,
    /// Output value from 0.0 to 1.0
    /// Voltage depends on the device
    pub input: AnalogInputInput,
}

#[derive(Debug, Clone)]
pub struct AnalogInputInput {
    /// from -1.0 to 1.0
    pub normalized: f32,
    /// depends on the device
    /// unit can be Volts or Milliamps
    ///
    /// - EL300x: -10V to 10V
    /// - EL304x: 0mA to 20mA
    /// - EL305x: 4ma to 20mA
    /// - EL306x: 0V to 10V
    /// - EL3062-0030: 0V to 30V
    pub absolute: f32,
}

pub trait AnalogInputDevice<PORTS>: Send + Sync {
    fn analog_output_state(&self, port: PORTS) -> AnalogInputState;
}
