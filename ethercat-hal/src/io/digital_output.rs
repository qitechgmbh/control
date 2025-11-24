use std::{fmt, sync::Arc};

use smol::lock::RwLock;

/// Digital Output (DO) device
///
/// Writes digital values (true or false) to the device.
pub struct DigitalOutput {
    /// Write a value to the digital output
    set_output: Box<dyn Fn(DigitalOutputOutput) + Send + Sync>,

    /// Read the state of the digital output
    get_output: Box<dyn Fn() -> DigitalOutputOutput + Send + Sync>,
}

impl fmt::Debug for DigitalOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DigitalOutput")
    }
}

impl DigitalOutput {
    pub fn new<PORT>(device: Arc<RwLock<dyn DigitalOutputDevice<PORT>>>, port: PORT) -> Self
    where
        PORT: Clone + Send + Sync + 'static,
    {
        // build sync write closure
        let port1 = port.clone();
        let device1 = device.clone();
        let set_output = Box::new(move |value: DigitalOutputOutput| {
            let mut device = smol::block_on(device1.write());
            device.set_output(port1.clone(), value);
        });

        // build sync get closure
        let port2 = port;
        let device2 = device.clone();
        let get_output = Box::new(move || -> DigitalOutputOutput {
            let device = smol::block_on(device2.read());
            device.get_output(port2.clone())
        });

        Self {
            set_output,
            get_output,
        }
    }

    /// Set the digital output value
    pub fn set(&self, enabled: bool) {
        (self.set_output)(enabled.into());
    }

    /// Get the current output value
    pub fn get(&self) -> bool {
        let output = (self.get_output)();
        output.into()
    }
}

/// Output value
/// true: high
/// false: low
#[derive(Debug, Clone)]
pub struct DigitalOutputOutput(pub bool);

impl From<bool> for DigitalOutputOutput {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

impl From<DigitalOutputOutput> for bool {
    fn from(value: DigitalOutputOutput) -> Self {
        value.0
    }
}

pub trait DigitalOutputDevice<PORT>: Send + Sync
where
    PORT: Clone,
{
    fn set_output(&mut self, port: PORT, value: DigitalOutputOutput);
    fn get_output(&self, port: PORT) -> DigitalOutputOutput;
}
