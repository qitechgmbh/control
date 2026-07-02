use std::sync::Arc;

use machine_core::property::PropertyBatch;
use tokio::{io, sync::mpsc};

use crate::{SharedState, bridge::Bridge};

pub struct EmbeddedBridge {
    rx: mpsc::Receiver<Arc<PropertyBatch>>,
}

impl Bridge for EmbeddedBridge {
    fn run(mut self, state: SharedState) -> impl Future<Output = io::Result<()>> + Send {
        let mut registry = (*state.machine_registry.load_full()).clone();
        let sender = state.snapshot_tx.clone();

        async move {
            loop {
                let Some(batch) = self.rx.recv().await else {
                    // channel closed
                    return Ok(());
                };

                if super::update_registry(&batch, &mut registry) {
                    state.machine_registry.swap(Arc::new(registry.clone()));
                }

                if sender.send(batch).is_err() {
                    eprintln!("Failed to broadcast property set, all channels closed. Exiting...");
                    return Ok(());
                }
            }
        }

    }
}
