use smol::lock::RwLock;

use crate::pdo::basic::Limit;
use std::{fmt, future::Future, pin::Pin, sync::Arc};

/// Temperature Input (TI) device
///
/// Reads temperature values from the device.
pub struct TemperatureInput {
    /// Read the state of the temperature input
    pub state:
        Box<dyn Fn() -> Pin<Box<dyn Future<Output = TemperatureInputState> + Send>> + Send + Sync>,
}

impl fmt::Debug for TemperatureInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DigitalInput")
    }
}

impl TemperatureInput {
    pub fn new<PORTS>(
        device: Arc<RwLock<dyn TemperatureInputDevice<PORTS>>>,
        port: PORTS,
    ) -> TemperatureInput
    where
        PORTS: Clone + Send + Sync + 'static,
    {
        // build async get closure
        let port2 = port.clone();
        let device2 = device.clone();
        let state = Box::new(
            move || -> Pin<Box<dyn Future<Output = TemperatureInputState> + Send>> {
                let device2 = device2.clone();
                let port_clone = port2.clone();
                Box::pin(async move {
                    let device = device2.read().await;
                    device.temperature_input_state(port_clone)
                })
            },
        );
        TemperatureInput { state }
    }
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
    fn temperature_input_state(&self, port: PORTS) -> TemperatureInputState;
}
