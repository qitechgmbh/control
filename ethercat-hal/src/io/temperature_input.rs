use smol::lock::RwLock;

use crate::pdo::basic::Limit;
use std::{fmt, sync::Arc};

/// Temperature Input (TI) device
///
/// Reads temperature values from the device.
pub struct TemperatureInput {
    /// Read the state of the temperature input
    get_input: Box<dyn Fn() -> TemperatureInputInput + Send + Sync>,
}

impl fmt::Debug for TemperatureInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DigitalInput")
    }
}

impl TemperatureInput {
    pub fn new<PORTS>(device: Arc<RwLock<dyn TemperatureInputDevice<PORTS>>>, port: PORTS) -> Self
    where
        PORTS: Clone + Send + Sync + 'static,
    {
        // build sync get closure
        let port2 = port;
        let device2 = device.clone();
        let get_input = Box::new(move || {
            let device2 = device2.clone();
            let port_clone = port2.clone();

            let device = device2.read_blocking();
            device.get_input(port_clone)
        });
        Self { get_input }
    }

    /// Get the current temperature in degrees Celsius
    pub fn get_temperature(&self) -> Result<f64, TemperatureInputError> {
        let input = (self.get_input)();
        if input.overvoltage {
            Err(TemperatureInputError::OverVoltage)
        } else if input.undervoltage {
            Err(TemperatureInputError::UnderVoltage)
        } else {
            Ok(input.temperature as f64)
        }
    }
}

pub enum TemperatureInputError {
    /// Over-voltage error
    OverVoltage,

    /// Under-voltage error
    UnderVoltage,
}

#[derive(Debug, Clone)]
pub struct TemperatureInputState {
    /// Input value
    pub input: TemperatureInputInput,
}

#[derive(Debug, Clone)]
pub struct TemperatureInputInput {
    /// Temperature in degrees Celsius (°C) with a resolution of 0.1°C
    pub temperature: f32,

    /// Under-voltage error
    pub undervoltage: bool,

    /// Over-voltage error
    pub overvoltage: bool,

    /// Configured limit 1
    pub limit1: Limit,

    /// Configured limit 2
    pub limit2: Limit,

    /// Error flag
    pub error: bool,

    /// if the TxPdo state is valid
    pub txpdo_state: bool,

    /// if the TxPdo is toggled
    pub txpdo_toggle: bool,
}

pub trait TemperatureInputDevice<PORTS>: Send + Sync {
    fn get_input(&self, port: PORTS) -> TemperatureInputInput;
}
