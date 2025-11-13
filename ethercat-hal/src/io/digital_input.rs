use std::{fmt, sync::Arc};

use smol::lock::RwLock;

/// Digital Input (DI) device
///
/// Reads digital values (true or false) from the device.
pub struct DigitalInput {
    /// Read the state of the digital input
    get_input: Box<dyn Fn() -> Result<DigitalInputInput, anyhow::Error> + Send + Sync>,
}

impl fmt::Debug for DigitalInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DigitalInput")
    }
}

/// Implement on device that have digital inputs
impl DigitalInput {
    pub fn new<PORT>(device: Arc<RwLock<dyn DigitalInputDevice<PORT>>>, port: PORT) -> Self
    where
        PORT: Clone + Send + Sync + 'static,
    {
        // build sync get closure
        let port2 = port;
        let device2 = device.clone();
        let get_input = Box::new(move || {
            let device = smol::block_on(device2.read());
            device.get_input(port2.clone())
        });

        Self { get_input }
    }

    /// Get the current value of the digital input
    pub fn get_value(&self) -> Result<bool, anyhow::Error> {
        let input = (self.get_input)()?;
        Ok(input.value)
    }
}

#[derive(Debug, Clone)]
pub struct DigitalInputInput {
    pub value: bool,
}

pub trait DigitalInputDevice<PORTS>: Send + Sync
where
    PORTS: Clone,
{
    fn get_input(&self, port: PORTS) -> Result<DigitalInputInput, anyhow::Error>;
}
