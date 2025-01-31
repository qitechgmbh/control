use std::{future::Future, pin::Pin, sync::Arc};

pub struct DigitalOutput {
    pub write: Box<dyn Fn(bool) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>,
    pub state:
        Box<dyn Fn() -> Pin<Box<dyn Future<Output = DigitalOutputState> + Send>> + Send + Sync>,
}

pub struct DigitalOutputState {
    pub output_ts: u64,
    pub value: bool,
}

pub trait DigitalOutputDevice<PORTS> {
    fn digital_output_write(&mut self, port: PORTS, value: bool);
    fn digital_output_state(&self, port: PORTS) -> DigitalOutputState;
    fn digital_output(device: Arc<tokio::sync::RwLock<Self>>, port: PORTS) -> DigitalOutput
    where
        Self: Send + Sync + 'static,
        PORTS: Clone + Send + Sync + 'static,
    {
        // build async write closure
        let device1 = device.clone();
        let port1 = port.clone();
        let write = Box::new(move |value| {
            let device1 = device1.clone();
            let port1 = port1.clone();
            Box::pin(async move {
                let mut device1_guard = device1.write().await;
                device1_guard.digital_output_write(port1.clone(), value)
            }) as Pin<Box<dyn Future<Output = ()> + Send + 'static>>
        });

        // build async get closure
        let device2 = device.clone();
        let port2 = port.clone();
        let state = Box::new(move || {
            let device2 = device2.clone();
            let port2 = port2.clone();
            Box::pin(async move {
                let device2_guard = device2.read().await;
                device2_guard.digital_output_state(port2.clone())
            }) as Pin<Box<dyn Future<Output = DigitalOutputState> + Send + 'static>>
        });

        DigitalOutput { write, state }
    }
}
