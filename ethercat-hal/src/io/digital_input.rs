use std::{fmt, future::Future, pin::Pin, sync::Arc};

use smol::lock::RwLock;

/// Digital Input (DI) device
///
/// Reads digital values (true or false) from the device.
pub struct DigitalInput {
    /// Read the state of the digital input
    pub state:
        Box<dyn Fn() -> Pin<Box<dyn Future<Output = DigitalInputState> + Send>> + Send + Sync>,
}

impl fmt::Debug for DigitalInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DigitalInput")
    }
}

/// Implement on device that have digital inputs
impl DigitalInput {
    pub fn new<PORT>(device: Arc<RwLock<dyn DigitalInputDevice<PORT>>>, port: PORT) -> DigitalInput
    where
        PORT: Clone + Send + Sync + 'static,
    {
        // build async get closure
        let port2 = port.clone();
        let device2 = device.clone();
        let state = Box::new(
            move || -> Pin<Box<dyn Future<Output = DigitalInputState> + Send>> {
                let device2 = device2.clone();
                let port_clone = port2.clone();
                Box::pin(async move {
                    let device = device2.read().await;
                    device.digital_input_state(port_clone)
                })
            },
        );

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
    pub input: DigitalInputInput,
}

#[derive(Debug, Clone)]
pub struct DigitalInputInput {
    pub value: bool,
}

pub trait DigitalInputDevice<PORTS>: Send + Sync
where
    PORTS: Clone,
{
    fn digital_input_state(&self, port: PORTS) -> DigitalInputState;
}
