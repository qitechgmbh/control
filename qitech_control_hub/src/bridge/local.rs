use std::{io, sync::Arc};
use tokio::sync::{mpsc};

use property::{PropertySet, PropertySetView};
use crate::{PropertyMessage, SharedState};

pub async fn run(state: SharedState, mut rx: mpsc::Receiver<Arc<PropertySet>>) -> io::Result<()> {
    let sender = state.snapshot_tx.clone();

    let mut registry = state.registry.read().await.clone();

    loop {
        let Some(set) = rx.recv().await else {
            // channel closed
            return Ok(());
        };
        
        super::update_registry(
            PropertySetView::native_dirty(&set),
            &mut registry, 
            &state,
        ).await;

        if let Err(_) = sender.send(PropertyMessage::Native(set)) {
            eprintln!("Failed to broadcast property set, all channels closed. Exiting...");
            return Ok(());
        }
    }
}
