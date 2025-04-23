use anyhow::Error;
use smol::lock::RwLock;
use std::{fmt, future::Future, pin::Pin, sync::Arc};

pub struct SerialInterface {
    pub has_message: Box<dyn Fn() -> Pin<Box<dyn Future<Output = bool> + Send>> + Send + Sync>,
    pub write_message: Box<
        dyn Fn(Vec<u8>) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send>> + Send + Sync,
    >,
    pub read_message:
        Box<dyn Fn() -> Pin<Box<dyn Future<Output = Option<Vec<u8>>> + Send>> + Send + Sync>,
}

impl fmt::Debug for SerialInterface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SerialInterface")
    }
}

/// Implement on device that have digital inputs
impl SerialInterface {
    pub fn new<PORT>(
        device: Arc<RwLock<dyn SerialInterfaceDevice<PORT>>>,
        port: PORT,
    ) -> SerialInterface
    where
        PORT: Clone + Send + Sync + 'static,
    {
        // build async get closure
        let mut port2 = port.clone();
        let mut device2 = device.clone();

        let read_message = Box::new(
            move || -> Pin<Box<dyn Future<Output = Option<Vec<u8>>> + Send>> {
                let device2 = device2.clone();
                let port_clone = port2.clone();

                Box::pin(async move {
                    let mut device = device2.write().await;
                    device.serial_interface_read_message(port_clone)
                })
            },
        );

        port2 = port.clone();
        device2 = device.clone();

        let write_message = Box::new(
            move |message: Vec<u8>| -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send>> {
                let device2 = device2.clone();
                let port_clone = port2.clone();
                let message2 = message.to_owned();

                Box::pin(async move {
                    let mut device = device2.write().await;
                    device.serial_interface_write_message(port_clone, message2)
                })
            },
        );

        port2 = port.clone();
        device2 = device.clone();

        let has_message = Box::new(move || -> Pin<Box<dyn Future<Output = bool> + Send>> {
            let device2 = device2.clone();
            let port_clone = port2.clone();

            Box::pin(async move {
                let mut device = device2.write().await;
                device.serial_interface_has_messages(port_clone)
            })
        });

        SerialInterface {
            has_message,
            write_message,
            read_message,
        }
    }
}

pub trait SerialInterfaceDevice<PORTS>: Send + Sync
where
    PORTS: Clone,
{
    fn serial_interface_read_message(&mut self, port: PORTS) -> Option<Vec<u8>>;
    fn serial_interface_write_message(
        &mut self,
        port: PORTS,
        message: Vec<u8>,
    ) -> Result<(), Error>;
    fn serial_interface_has_messages(&mut self, port: PORTS) -> bool;
}
