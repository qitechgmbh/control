use smol::future::block_on;
use smol::lock::RwLock;
use std::fmt;
use std::sync::Arc;

/// Analog Output (AO) device
///
/// We can write values in clip space (0 to 1) to the device. The voltage output depends on the
/// device and its range.
pub struct AnalogOutput {
    /// Write a value to the analog output
    pub set_output: Box<dyn Fn(AnalogOutputOutput) + Send + Sync>,

    /// Read the state of the analog output
    pub get_output: Box<dyn Fn() -> AnalogOutputOutput + Send + Sync>,
}

impl fmt::Debug for AnalogOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AnalogOutput")
    }
}

/// Implement on device that have analog outputs
impl AnalogOutput {
    pub fn new<PORT>(device: Arc<RwLock<dyn AnalogOutputDevice<PORT>>>, port: PORT) -> Self
    where
        PORT: Clone + Send + Sync + 'static,
    {
        // build sync write closure
        let port1 = port.clone();
        let device1 = device.clone();
        let set_output = Box::new(move |value: AnalogOutputOutput| {
            let mut device = block_on(device1.write());
            device.set_output(port1.clone(), value);
        });

        // build sync get closure
        let port2 = port;
        let device2 = device.clone();
        let get_output = Box::new(move || -> AnalogOutputOutput {
            let device = block_on(device2.read());
            device.get_output(port2.clone())
        });

        Self {
            set_output,
            get_output,
        }
    }

    /// Set the analog output value
    pub fn set(&self, value: f32) {
        (self.set_output)(value.into());
    }

    /// Get the current output value
    pub fn get(&self) -> f32 {
        let output = (self.get_output)();
        output.into()
    }
}

#[derive(Debug, Clone)]
pub struct AnalogOutputOutput(pub f32);

impl From<f32> for AnalogOutputOutput {
    fn from(value: f32) -> Self {
        Self(value)
    }
}

impl From<AnalogOutputOutput> for f32 {
    fn from(value: AnalogOutputOutput) -> Self {
        value.0
    }
}

pub trait AnalogOutputDevice<PORTS>: Send + Sync {
    fn set_output(&mut self, port: PORTS, value: AnalogOutputOutput);
    fn get_output(&self, port: PORTS) -> AnalogOutputOutput;
}
