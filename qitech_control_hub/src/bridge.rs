use std::sync::Arc;

use property::ExportedPropertySet;
use tokio::{
    io::{self, AsyncReadExt},
    net::{UnixListener, UnixStream},
    select,
    signal::unix::{Signal, SignalKind, signal}, sync::broadcast,
};

use crate::shared_state::{MachineRegistry, PropertyType, SharedState};

pub struct ControlBridge {
    state: SharedState,

    /// local copy of the registry we can update 
    /// to only lock for writes when the local one
    /// has changed
    registry: MachineRegistry,

    sigabrt: Signal,
    listener: UnixListener,
    sender: broadcast::Sender<Arc<ExportedPropertySet>>,
}

impl ControlBridge {
    pub fn new<'a, S: Into<&'a str>>(state: SharedState, socket_path: S) -> io::Result<Self> {
        let socket_path = socket_path.into();
        let registry = state.machine_registry.blocking_read().clone();

        if std::fs::exists(socket_path)? {
            std::fs::remove_file(socket_path)?;
        }

        let listener = UnixListener::bind(socket_path)?;
        let sigabrt = signal(SignalKind::terminate())?;
        let sender = state.snapshot_tx.clone();

        Ok(Self {
            state,
            registry,
            sigabrt,
            listener,
            sender
        })
    }

    pub async fn run(mut self) -> io::Result<()> {
        loop {
            let stream = select! {
                biased;

                _ = self.sigabrt.recv() => {
                    println!("Received SIGABRT");
                    return Ok(());
                }

                res = self.listener.accept() => {
                    let (stream, _) = match res {
                        Ok(v) => v,
                        Err(e) => {
                            eprintln!("Failed to accept connection: {e}");
                            continue;
                        }
                    };

                    stream
                }
            };

            let should_exit = self.handle_client(stream).await?;
            
            if should_exit {
                return Ok(());
            }
        }
    }

    async fn handle_client(&mut self, mut stream: UnixStream) -> io::Result<bool> {
        loop {
            let len = select! {
                biased;

                _ = self.sigabrt.recv() => {
                    println!("Received SIGABRT");
                    return Ok(true);
                }

                result = stream.read_u32_le() => {
                    match result {
                        Ok(v) => v,
                        Err(e) => {
                            eprintln!("Failed to read len: {e}");
                            return Err(e);
                        },
                    }
                }
            };

            let mut buf = vec![0u8; len as usize];
            select! {
                biased;

                _ = self.sigabrt.recv() => {
                    println!("Received SIGABRT");
                    return Ok(true);
                }

                result = stream.read_exact(&mut buf) => {
                    if let Err(e) = result {
                        eprintln!("Failed to read data: {e}");
                        continue;
                    }
                }
            };

            let snapshot: ExportedPropertySet = match postcard::from_bytes(&buf) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Failed to deserialize message: {e}");
                    return Ok(false);
                },
            };

            let mut did_registry_change = false;

            for entry in &snapshot.float {
                let props = self.registry.entry(entry.ident).or_default();

                if props.contains_key(&entry.name) {
                    continue;
                }

                let old = props.insert(entry.name.clone(), PropertyType::Float);
                did_registry_change = true;
                assert!(old.is_none());
            }

            for entry in &snapshot.int {
                let props = self.registry.entry(entry.ident).or_default();

                if props.contains_key(&entry.name) {
                    continue;
                }

                let old = props.insert(entry.name.clone(), PropertyType::Integer);
                did_registry_change = true;
                assert!(old.is_none());
            }

            for entry in &snapshot.float {
                let props = self.registry.entry(entry.ident).or_default();

                if props.contains_key(&entry.name) {
                    continue;
                }

                let old = props.insert(entry.name.clone(), PropertyType::Boolean);
                did_registry_change = true;
                assert!(old.is_none());
            }

            for entry in &snapshot.float {
                let props = self.registry.entry(entry.ident).or_default();

                if props.contains_key(&entry.name) {
                    continue;
                }

                let old = props.insert(entry.name.clone(), PropertyType::String);
                did_registry_change = true;
                assert!(old.is_none());
            }

            if did_registry_change {
                // only invoke a registry sync when necessary
                let mut registry = self.state.machine_registry.write().await;
                *registry = self.registry.clone();
            }

            if let Err(_) = self.sender.send(Arc::new(snapshot)) {
                eprintln!("Failed to broadcast property set, all channels closed. Exiting...");
                return Ok(true);
            }
        }
    }
}
