use std::{fmt::Debug, time::Instant};
use smol::channel::{Receiver, Sender};

use crate::{AsyncThreadMessage, MachineConnection, MachineMessage};
use crate::machine_identification::MachineIdentificationUnique;

type Namespace = super::Winder2Namespace;

const MAX_CONNECTIONS: usize = 2;

#[derive(Debug)]
pub struct MachineBase 
{
    pub machine_identification_unique: MachineIdentificationUnique,

    pub api_receiver: Receiver<MachineMessage>,
    pub api_sender:   Sender<MachineMessage>,
    pub main_sender:  Option<Sender<AsyncThreadMessage>>,

    connected_machines_buf: [MachineConnection; MAX_CONNECTIONS],
    connected_machines_len: usize,

    // socketio
    pub namespace: Namespace,
    pub last_measurement_emit: Instant,

    /// Will be initialized as false and set to true by emit_state
    /// This way we can signal to the client that the first state emission is a default state
    pub emitted_default_state: bool,
}