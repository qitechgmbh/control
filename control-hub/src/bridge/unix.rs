use std::{path::Path, sync::Arc, time::Duration};

use machine_core::property::PropertyBatch;
use tokio::{
    io::{self, AsyncReadExt},
    net::{UnixListener, UnixStream},
    select,
    time::timeout,
};

use crate::{SharedState, bridge::Bridge};

pub struct UnixBridge {
    listener: UnixListener,
}

impl UnixBridge {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        if std::fs::exists(&path)? {
            std::fs::remove_file(&path)?;
        }

        let listener = UnixListener::bind(&path)?;

        Ok(Self { listener })
    }
}

impl Bridge for UnixBridge {
    fn run(self, mut state: SharedState) -> impl Future<Output = io::Result<()>> + Send {
        let listener = self.listener;

        async move {
            loop {
                let stream = tokio::select! {
                    biased;

                    _ = state.shutdown_rx.changed() => {
                        println!("Received shutdown signal, shutting down...");
                        return Ok(());
                    }

                    res = listener.accept() => {
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

                let should_exit = handle_client(&mut state, stream).await?;

                if should_exit {
                    return Ok(());
                }
            }
        }
    }
}

async fn handle_client(state: &mut SharedState, mut stream: UnixStream) -> io::Result<bool> {
    let mut registry = (*state.machine_registry.load_full()).clone();
    let sender = state.snapshot_tx.clone();

    loop {
        let len = select! {
            biased;

            _ = state.shutdown_rx.changed() => {
                println!("shutdown_signal changed, shutting down");
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

        let result = match timeout(
            Duration::from_millis(2500), 
            stream.read_exact(&mut buf)
        ).await {
            Ok(v) => v,
            Err(_) => {
                eprintln!("Timeout while waiting for data");
                return Ok(false);
            },
        };

        if let Err(e) = result {
            eprintln!("Failed to read data: {e}");
            return Ok(false);
        }

        let batch: PropertyBatch = match postcard::from_bytes(&buf) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Failed to deserialize message: {e}");
                return Ok(false);
            },
        };

        if super::update_registry(&batch, &mut registry) {
            state.machine_registry.swap(Arc::new(registry.clone()));
        }

        if sender.send(Arc::new(batch)).is_err() {
            eprintln!("Failed to broadcast property set, all channels closed. Exiting...");
            return Ok(true);
        }
    }
}
