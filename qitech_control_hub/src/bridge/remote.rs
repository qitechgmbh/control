use std::{sync::Arc, time::Duration};

use property::{ExportedPropertySet, PropertySetView};
use tokio::{
    io::{self, AsyncReadExt},
    net::{UnixListener, UnixStream},
    select,
    time::timeout,
};

use crate::{PropertyMessage, SharedState};

pub struct Config {
    pub socket_path: String,
}

pub async fn run(mut state: SharedState, config: Config) -> io::Result<()> {
    if std::fs::exists(&config.socket_path)? {
        std::fs::remove_file(&config.socket_path)?;
    }

    let listener = UnixListener::bind(config.socket_path)?;

    loop {
        let stream = select! {
            biased;

            _ = state.shutdown_rx.changed() => {
                println!("shutdown_signal changed, shutting down");
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

async fn handle_client(state: &mut SharedState, mut stream: UnixStream) -> io::Result<bool> {
    let mut registry = state.registry.read().await.clone();
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

        let snapshot: ExportedPropertySet = match postcard::from_bytes(&buf) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Failed to deserialize message: {e}");
                return Ok(false);
            },
        };

        super::update_registry(
            PropertySetView::exported(&snapshot),
            &mut registry, 
            state,
        ).await;

        if let Err(_) = sender.send(PropertyMessage::Exported(Arc::new(snapshot))) {
            eprintln!("Failed to broadcast property set, all channels closed. Exiting...");
            return Ok(true);
        }
    }
}
