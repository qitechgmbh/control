use std::collections::HashMap;

use qitech_framework::EtherCATStatus;
use qitech_framework::MachineInstanceIdentification;
use socketioxide::extract::SocketRef;

use crate::api::legacy;
use crate::api::legacy::socketio::events::EthercatDevicesEvent;
use crate::api::legacy::socketio::events::EthercatSetupDone;
use crate::api::legacy::socketio::events::MachineObj;
use crate::api::legacy::socketio::events::MachinesEvent;
use crate::api::legacy::socketio::events::SocketIOEvent;

// --- main namespace ---
#[derive(Default, Clone)]
pub struct MainNamespaceManager {
    sockets: Vec<SocketRef>,
    machines: HashMap<MachineInstanceIdentification, MachineObj>,
    ecat_state: Option<&'static str>,
    ecat_devices: Option<Vec<legacy::EtherCATDeviceMetadata>>,
}

impl MainNamespaceManager {
    pub fn add_socket(&mut self, socket: SocketRef) {
        tracing::info!("Adding new main namespace socket!");

        // --- send the last recorded ether cat state ---
        if let Some(state) = self.ecat_state {
            let event = SocketIOEvent::new(
                "EthercatStateEvent",
                EthercatDevicesEvent::State(state.to_string()),
            );

            if let Err(e) = socket.emit("event", &event) {
                tracing::error!("Failed to send message to new socket: {e}");
                return;
            }
        }

        // --- send the ecat devices if already recorded ---
        if let Some(devices) = self.ecat_devices.clone() {
            let event = SocketIOEvent::new(
                "EthercatDevicesEvent",
                EthercatDevicesEvent::Done(EthercatSetupDone { devices }),
            );

            if let Err(e) = socket.emit("event", &event) {
                tracing::error!("Failed to send message to new socket: {e}");
                return;
            }
        }

        // --- store the socket ---
        self.sockets.push(socket);
    }

    pub fn set_ecat_state(&mut self, state: EtherCATStatus) {
        let state = match state {
            EtherCATStatus::NoInterface => "no interface",
            EtherCATStatus::Boot => "booting",
            EtherCATStatus::Init => "init",
            EtherCATStatus::PreOp => "preop",
            EtherCATStatus::PreopPdi => "preoppdi",
            EtherCATStatus::Op => "op",
        };

        let event = SocketIOEvent::new(
            "EthercatStateEvent",
            EthercatDevicesEvent::State(state.to_string()),
        );

        self.broadcast(event);
        self.ecat_state = Some(state);
    }

    pub fn set_ecat_devices(&mut self, devices: Vec<legacy::EtherCATDeviceMetadata>) {
        let event = SocketIOEvent::new(
            "EthercatDevicesEvent",
            EthercatDevicesEvent::Done(EthercatSetupDone {
                devices: devices.clone(),
            }),
        );

        self.broadcast(event);
        self.ecat_devices = Some(devices);
    }

    pub fn add_machine(
        &mut self,
        ident: MachineInstanceIdentification,
        result: Result<(), String>,
    ) {
        self.machines.insert(
            ident,
            MachineObj {
                machine_identification_unique: legacy::MachineIdentificationUnique {
                    machine_identification: legacy::types::MachineIdentification {
                        vendor: ident.machine.vendor_id,
                        machine: ident.machine.machine_id,
                    },
                    serial: ident.serial,
                },
                error: result.err(),
            },
        );

        let event = SocketIOEvent::new(
            "MachinesEvent",
            MachinesEvent {
                machines: self.machines.values().cloned().collect(),
            },
        );

        tracing::info!("Added Machine: {ident}");
        self.broadcast(event);
    }

    fn broadcast(&mut self, event: SocketIOEvent) {
        self.sockets.retain(|socket| {
            match socket.emit("event", &event) {
                Ok(()) => true,

                // should not happen
                Err(socketioxide::SendError::Serialize(err)) => {
                    panic!("Who fucked up the payload?: {err}");
                }

                // channel full or disconnected
                Err(socketioxide::SendError::Socket(_)) => false,
            }
        });
    }
}
