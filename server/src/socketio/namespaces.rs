use std::sync::Arc;
use std::collections::HashMap;

use control_core::socketio::{namespace::NamespaceInterface, namespace_id::NamespaceId};
use control_core::machines::identification::MachineIdentificationUnique;
use socketioxide::extract::SocketRef;

use crate::app_state;

use super::main_namespace::MainRoom;

/// Manages Socket.IO namespaces and handles timing issues during server startup.
/// 
/// The main challenge this module solves is a race condition that occurs when
/// clients reconnect after server restart:
/// 
/// 1. Server starts and Socket.IO layer becomes available
/// 2. Clients attempt to reconnect to machine namespaces (e.g., /machine/1/2/1)  
/// 3. But machines haven't been initialized from EtherCAT setup yet (takes ~2 seconds)
/// 4. Without pending connection handling, these early connections would fail
/// 5. Even after machines become available, failed connections miss subsequent events
/// 
/// The solution uses a pending connections queue that:
/// - Temporarily stores socket connections for machines that don't exist yet
/// - Processes the queue after machine initialization completes
/// - Ensures all clients receive events regardless of connection timing

pub struct Namespaces {
    pub main_namespace: MainRoom,
    /// Store pending connections for machine namespaces that don't exist yet.
    /// 
    /// This is necessary because during server startup, there's a timing window where:
    /// 1. Socket.IO server starts accepting connections
    /// 2. Clients attempt to connect to machine namespaces (e.g., /machine/1/2/1)
    /// 3. But machines haven't been initialized yet from EtherCAT setup
    /// 
    /// Without this mechanism, early socket connections would fail and never receive
    /// events even after machines become available. This HashMap queues socket
    /// connections until the corresponding machine is initialized.
    pub pending_machine_connections: HashMap<MachineIdentificationUnique, Vec<SocketRef>>,
}

impl Namespaces {
    pub fn new() -> Self {
        Self {
            main_namespace: MainRoom::new(),
            pending_machine_connections: HashMap::new(),
        }
    }

    pub async fn apply_mut(
        &mut self,
        namespace_id: NamespaceId,
        app_state: &Arc<app_state::AppState>,
        callback: impl FnOnce(Result<&mut dyn NamespaceInterface, anyhow::Error>),
    ) {
        match namespace_id {
            NamespaceId::Main => callback(Ok(&mut self.main_namespace.0)),
            NamespaceId::Machine(machine_identification_unique) => {
                // lock machines
                let machines_guard = app_state.machines.read().await;

                // get machine
                let machine = match machines_guard.get(&machine_identification_unique) {
                    Some(machine) => machine,
                    None => {
                        callback(Err(anyhow::anyhow!(
                            "[{}::Namespaces::appply_mut] Machine {:?} not found",
                            module_path!(),
                            machine_identification_unique
                        )));
                        return;
                    }
                };

                // check if machine has error
                let machine = match machine {
                    Ok(machine) => machine,
                    Err(err) => {
                        callback(Err(anyhow::anyhow!(
                            "[{}::Namespaces::appply_mut] Machine {:?} has error: {}",
                            module_path!(),
                            machine_identification_unique,
                            err
                        )));
                        return;
                    }
                };

                let mut machines_guard = machine.lock().await;

                let namespace = machines_guard.api_event_namespace();
                callback(Ok(namespace));
            }
        }
    }

    /// Adds a socket connection to the pending queue for a machine that doesn't exist yet.
    /// 
    /// This function is called when a client attempts to connect to a machine namespace
    /// before the machine has been initialized. Instead of rejecting the connection,
    /// we queue it here so it can be processed once the machine becomes available.
    /// 
    /// # Arguments
    /// * `machine_identification_unique` - The unique identifier for the machine
    /// * `socket` - The socket reference to queue for later subscription
    /// 
    /// # Why this is needed
    /// During server restart, clients may reconnect before EtherCAT initialization
    /// completes. Without queuing, these early connections would be lost and clients
    /// would miss subsequent events even after successful reconnection.
    pub fn add_pending_machine_connection(&mut self, machine_identification_unique: MachineIdentificationUnique, socket: SocketRef) {
        log::info!(
            "Adding pending connection for machine {:?}, socket {}",
            machine_identification_unique,
            socket.id
        );
        
        self.pending_machine_connections
            .entry(machine_identification_unique)
            .or_insert_with(Vec::new)
            .push(socket);
    }

    /// Processes all pending socket connections for machines that are now available.
    /// 
    /// This function should be called after machine initialization completes (typically
    /// after EtherCAT setup). It iterates through all queued socket connections and
    /// subscribes them to their respective machine namespaces if the machines are
    /// now available.
    /// 
    /// # Arguments
    /// * `app_state` - Application state containing the initialized machines
    /// 
    /// # Process
    /// 1. Check each machine in the pending connections queue
    /// 2. If the machine is now available, subscribe all pending sockets
    /// 3. Re-emit cached events to bring sockets up to current state
    /// 4. Remove processed connections from the queue
    /// 
    /// # Why this is critical
    /// Without this step, sockets that connected early would remain in limbo -
    /// connected but not subscribed to events. This ensures that all clients
    /// receive machine events regardless of connection timing.
    pub async fn process_pending_connections(&mut self, app_state: &Arc<app_state::AppState>) {
        let mut to_remove = Vec::new();
        
        for (machine_identification_unique, sockets) in &self.pending_machine_connections {
            log::info!(
                "Processing {} pending connections for machine {:?}",
                sockets.len(),
                machine_identification_unique
            );

            // Check if machine is now available
            let machines_guard = app_state.machines.read().await;
            if let Some(Ok(machine)) = machines_guard.get(machine_identification_unique) {
                let mut machine_guard = machine.lock().await;
                let namespace = machine_guard.api_event_namespace();
                
                // Subscribe all pending sockets and re-emit cached events
                for socket in sockets {
                    log::info!(
                        "Subscribing previously pending socket {} to machine {:?}",
                        socket.id,
                        machine_identification_unique
                    );
                    namespace.subscribe(socket.clone());
                    namespace.reemit(socket.clone());
                }
                
                to_remove.push(machine_identification_unique.clone());
            }
        }
        
        // Remove processed connections
        for machine_identification_unique in to_remove {
            self.pending_machine_connections.remove(&machine_identification_unique);
        }
    }
}
