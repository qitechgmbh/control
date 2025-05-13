use std::{sync::Arc, time::Instant};

use api::DreMachineNamespace;
use control_core::machines::Machine;
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
