use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use physical::{AnalogInputRange, AnalogInputValue};
use smol::lock::RwLock;

pub mod physical;

/// Analog Input (AI) device
///
/// Reads normalized (-1.0 to 1.0) values from the device. These values con be converted to a moltage or mA
/// depending on the type of device and its range.
pub struct AnalogInput {
    /// Read the state of the analog input
    pub state:
        Box<dyn Fn() -> Pin<Box<dyn Future<Output = AnalogInputState> + Send>> + Send + Sync>,
    pub range: AnalogInputRange,
}

impl fmt::Debug for AnalogInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AnalogInput")
    }
}

/// Implement on device that have analog inputs
impl AnalogInput {
    pub fn new<PORT>(device: Arc<RwLock<dyn AnalogInputDevice<PORT>>>, port: PORT) -> AnalogInput
    where
        PORT: Clone + Send + Sync + 'static,
    {
        // Get `AnalogInputRange` from device
        let device_for_range = Arc::clone(&device);
        let range = smol::block_on(async {
            let device = device_for_range.read().await;
            device.analog_input_range()
        });

        // build async get closure
        let state = Box::new(
            move || -> Pin<Box<dyn Future<Output = AnalogInputState> + Send>> {
                let device2 = Arc::clone(&device);
                let port_clone = port.clone();
                Box::pin(async move {
                    let device = device2.read().await;
                    device.analog_output_state(port_clone)
                })
            },
        );

        AnalogInput { state, range }
    }
}

#[derive(Debug, Clone)]
pub struct AnalogInputState {
    pub input: AnalogInputInput,
}

#[derive(Debug, Clone)]
pub struct AnalogInputInput {
    /// from -1.0 to 1.0
    /// Can be converted to voltage or mA knowning the type and range of the device
    pub normalized: f32,
}

impl AnalogInputInput {
    /// Convert to physical value
    pub fn get_physical(&self, range: &AnalogInputRange) -> AnalogInputValue {
        range.normalized_to_physical(self.normalized)
    }
}

pub trait AnalogInputDevice<PORTS>: Send + Sync {
    fn analog_output_state(&self, port: PORTS) -> AnalogInputState;
    fn analog_input_range(&self) -> AnalogInputRange;
}
