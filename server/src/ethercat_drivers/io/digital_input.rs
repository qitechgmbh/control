use std::{future::Future, pin::Pin, sync::Arc};
use tokio::sync::RwLock;

type Value = bool;

pub struct DigitalInput {
    pub state:
        Box<dyn Fn() -> Pin<Box<dyn Future<Output = DigitalInputState> + Send>> + Send + Sync>,
}

#[derive(Debug, Clone)]
pub struct DigitalInputState {
    pub output_ts: u64,
    pub value: Value,
}

pub trait DigitalInputDevice<PORTS> {
    fn digital_input_state(&self, port: PORTS) -> DigitalInputState;
    fn digital_input(device: Arc<RwLock<Self>>, port: PORTS) -> DigitalInput
    where
        Self: Send + Sync + 'static,
        PORTS: Clone + Send + Sync + 'static,
    {
        // build async get closure
        let device2 = device.clone();
        let port2 = port.clone();
        let state = Box::new(move || {
            let device2 = device2.clone();
            let port2 = port2.clone();
            Box::pin(async move {
                let device2_guard = device2.read().await;
                device2_guard.digital_input_state(port2.clone())
            }) as Pin<Box<dyn Future<Output = DigitalInputState> + Send + 'static>>
        });

        DigitalInput {
            // write,
            state,
        }
    }
}
