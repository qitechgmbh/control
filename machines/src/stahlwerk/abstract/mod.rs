use std::{fmt::Debug, sync::Arc, time::Instant};

use crate::{MachineMessage, machine_identification::MachineIdentificationUnique};
use smol::channel::{Receiver, Sender};
use tracing::instrument;
use crate::AsyncThreadMessage;

use control_core::socketio::namespace::{
    CacheableEvents, 
    Namespace, 
    NamespaceCacheingLogic
};

#[derive(Debug)]
pub struct BaseMachine
{
    pub machine_uid: MachineIdentificationUnique,
    pub main_sender: Option<Sender<AsyncThreadMessage>>,

    pub namespace: NamespaceImpl,

    // socketio
    pub last_measurement_emit: Instant,

    // api
    pub api_sender:   Sender<MachineMessage>,
    pub api_receiver: Receiver<MachineMessage>,

    pub emitted_default_state: bool,
}

#[derive(Debug)]
pub struct NamespaceImpl
{
    pub namespace: Option<Namespace>,
}

impl NamespaceImpl
{
    pub fn new(namespace: Option<Namespace>) -> Self
    {
        Self { namespace }
    }
}

impl<E: CacheableEvents<E>> NamespaceCacheingLogic<E> for NamespaceImpl
{
    #[instrument(skip_all)]
    fn emit(&mut self, events: E) 
    {
        let event     = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();

        match &mut self.namespace 
        {
            Some(ns) => ns.emit(event, &buffer_fn),
            None     => (),
        }
    }
}