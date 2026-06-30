use std::{io, sync::Arc};
use tokio::sync::{mpsc};
use machine_core::property::{PropertySet, PropertySetView};
use crate::{PropertyMessage, SharedState};

pub async fn run(state: SharedState, mut rx: mpsc::Receiver<Arc<PropertySet>>) -> io::Result<()> {
    let sender = state.snapshot_tx.clone();

    let mut registry = (*state.machine_registry.load_full()).clone();

    loop {
        let Some(set) = rx.recv().await else {
            // channel closed
            return Ok(());
        };

        if super::update_registry(PropertySetView::native(&set), &mut registry) {
            state.machine_registry.swap(Arc::new(registry.clone()));
        }

        if let Err(_) = sender.send(PropertyMessage::Native(set)) {
            eprintln!("Failed to broadcast property set, all channels closed. Exiting...");
            return Ok(());
        }
    }
}
