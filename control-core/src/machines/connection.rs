use crate::downcast::Downcast;
use crate::machines::identification::MachineIdentificationUnique;
use crate::machines::manager::MachineManager;
use crate::socketio::{event::GenericEvent, namespace::Namespace};
use serde::Serialize;
use smol::block_on;
use smol::lock::RwLock;
use std::sync::Arc;
use std::sync::Weak;

use super::Machine;
use anyhow::anyhow;
use smol::{channel::Sender, lock::Mutex};
use socketioxide::extract::SocketRef;

#[derive(Debug)]
pub enum MachineConnection<M: Machine + ?Sized>
where
    Self: Sized,
{
    Error(anyhow::Error),
    Disconnected,
    Connected(Arc<Mutex<M>>),
}

#[derive(Debug)]
pub struct MachineSlot<M: Machine + ?Sized>
where
    Self: Sized,
{
    pub machine_connection: MachineConnection<M>,
    pub namespace: Arc<Mutex<Namespace>>,
}

pub type MachineConnectionGeneric = MachineConnection<dyn Machine>;

pub type MachineSlotGeneric = MachineSlot<dyn Machine>;

impl<M: Machine> Clone for MachineConnection<M> {
    fn clone(&self) -> Self {
        match self {
            Self::Connected(m) => Self::Connected(m.clone()),
            Self::Error(e) => Self::Error(anyhow!(e.to_string())),
            Self::Disconnected => Self::Disconnected,
        }
    }
}

impl<M: Machine + ?Sized> MachineConnection<M> {
    pub fn to_error(&self) -> Option<anyhow::Error> {
        match self {
            MachineConnection::Error(err) => Some(anyhow!(err.to_string())),
            Self::Disconnected => Some(anyhow!("Machine is disconnected")),
            _ => None,
        }
    }

    pub fn to_machine(&self) -> Option<Arc<Mutex<M>>> {
        match self {
            MachineConnection::Connected(machine) => Some(machine.clone()),
            _ => None,
        }
    }
}

impl<M: Machine + ?Sized> MachineSlot<M> {
    pub fn new(socket_queue_tx: Sender<(SocketRef, Arc<GenericEvent>)>) -> Self {
        let namespace = Namespace::new(socket_queue_tx);
        Self {
            machine_connection: MachineConnection::Disconnected,
            namespace: Arc::new(Mutex::new(namespace)),
        }
    }

    pub fn is_connected(&self) -> bool {
        match self.machine_connection {
            MachineConnection::Connected(_) => true,
            _ => false,
        }
    }
}

pub trait CrossConnectableMachine<
    F: CrossConnectableMachine<F, T>,
    T: CrossConnectableMachine<T, F>,
>: Machine
{
    fn get_cross_connection(&mut self) -> &mut MachineCrossConnection<F, T>;
}

#[derive(Debug)]
pub struct MachineCrossConnection<
    F: CrossConnectableMachine<F, T>,
    T: CrossConnectableMachine<T, F>,
> {
    machine_manager: Weak<RwLock<MachineManager>>,
    machine_identification_unique: MachineIdentificationUnique,
    connected_machine: Weak<Mutex<MachineSlot<T>>>,
    _make_compiler_happy: std::marker::PhantomData<F>,
}

#[derive(Serialize, Debug, Clone)]
pub struct MachineCrossConnectionState {
    /// Connected Machine
    pub machine_identification_unique: Option<MachineIdentificationUnique>,
    pub is_available: bool,
}

impl<F: CrossConnectableMachine<F, T>, T: CrossConnectableMachine<T, F>>
    MachineCrossConnection<F, T>
{
    pub fn new(
        manager: Weak<RwLock<MachineManager>>,
        machine_identification_unique: &MachineIdentificationUnique,
    ) -> Self {
        MachineCrossConnection {
            machine_manager: manager,
            connected_machine: Weak::new(),
            machine_identification_unique: machine_identification_unique.clone(),
            _make_compiler_happy: std::marker::PhantomData,
        }
    }

    pub fn set_connected_machine(
        &mut self,
        machine_identification_unique: &MachineIdentificationUnique,
    ) {
        if let Some(machine_manager) = self.machine_manager.upgrade() {
            let machine_manager = block_on(machine_manager.read());

            let slot: Arc<Mutex<MachineSlotGeneric>> =
                match machine_manager.get(&machine_identification_unique) {
                    Some(c) => c,
                    None => return,
                };

            let slot: Arc<Mutex<MachineSlot<T>>> = match Downcast::downcast(&slot) {
                Ok(s) => s,
                Err(_) => return,
            };

            self.connected_machine = Arc::downgrade(&slot);
        } else {
            panic!("The machine manager has died, cannot do anything at this point");
        }
    }

    pub fn reverse_connect(&self) {
        if let Some(slot) = self.connected_machine.upgrade() {
            let slot = &mut slot.lock_blocking();

            if let MachineConnection::Connected(machine) = &slot.machine_connection {
                let mut machine = machine.lock_blocking();
                let cross_connection = machine.get_cross_connection();

                if !cross_connection.is_connected() {
                    cross_connection.set_connected_machine(&self.machine_identification_unique);
                }
            };
        }
    }

    pub fn disconnect(&mut self) {
        self.connected_machine = Weak::new();
    }

    pub fn reverse_disconnect(&mut self) {
        if let Some(slot) = self.connected_machine.upgrade() {
            let slot = slot.lock_blocking();

            if let MachineConnection::Connected(machine) = &slot.machine_connection {
                let mut machine = machine.lock_blocking();
                let cross_connection = machine.get_cross_connection();
                cross_connection.disconnect();
            };
        }
    }

    pub fn is_connected(&self) -> bool {
        let arc = self.connected_machine.upgrade();
        arc.is_some()
    }

    pub fn to_state(&self) -> MachineCrossConnectionState {
        if let Some(slot) = self.connected_machine.upgrade() {
            let connection = &slot.lock_blocking().machine_connection;

            if let MachineConnection::Connected(machine) = connection {
                let machine = machine.lock_blocking();

                return MachineCrossConnectionState {
                    machine_identification_unique: Some(
                        machine.get_machine_identification_unique(),
                    ),
                    is_available: true,
                };
            }
        }

        MachineCrossConnectionState {
            machine_identification_unique: None,
            is_available: false,
        }
    }
}
