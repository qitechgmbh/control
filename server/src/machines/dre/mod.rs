use std::{sync::Arc, time::Instant};
use api::{DiameterEvent, DreEvents, DreMachineNamespace};
use control_core::{machines::Machine, socketio::namespace::NamespaceCacheingLogic};
use smol::lock::RwLock;
use crate::serial::devices::dre::Dre;

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct DreMachine {
    // drivers
    dre: Arc<RwLock<Dre>>,

    // socketio
    namespace: DreMachineNamespace,
    last_measurement_emit: Instant,
}

impl Machine for DreMachine {}

impl DreMachine{
    pub async fn emit_dre_data(&mut self) {
        let dre_data = self.dre.read().await.get_data().await;

        let diameter_event = DiameterEvent{
            dre_data
        };
        self.namespace.emit_cached(DreEvents::DiameterEvent(diameter_event.build()));

    }
}