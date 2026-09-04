use std::collections::HashMap;

use qitech_framework::ConfigPropertyEvent;
use qitech_framework::ConfigPropertyWriteOutcome;
use qitech_framework::MachineIdentification;
use qitech_framework::MachineInstanceIdentification;
use qitech_framework::MachineSchema;
use qitech_framework::MachinesReport;
use qitech_framework::StatePropertyEvent;
use socketioxide::extract::SocketRef;

use crate::api::legacy::adapter;
use crate::api::legacy::socketio::events::SocketIOEvent;
use crate::api::types::ConfigPropertyInfo;
use crate::api::types::MachineInstance;
use crate::api::types::MeasurementInfo;
use crate::api::types::StatePropertyInfo;

#[derive(Default, Clone)]
pub struct MachineNamespaceManager {
    registry: HashMap<MachineInstanceIdentification, Entry>,
}

impl MachineNamespaceManager {
    pub fn register(&mut self, ident: MachineInstanceIdentification, schema: &MachineSchema) {
        if self.registry.contains_key(&ident) {
            return;
        }

        let mut config_properties = HashMap::new();
        for name in schema.config_properties.keys() {
            config_properties.insert(name.clone(), None);
        }

        let mut state_properties = HashMap::new();
        for name in schema.state_properties.keys() {
            state_properties.insert(name.clone(), None);
        }

        let mut measurements = HashMap::new();
        for name in schema.measurements.keys() {
            measurements.insert(name.clone(), None);
        }

        self.registry.insert(
            ident,
            Entry {
                sockets: Default::default(),
                instance: MachineInstance {
                    config_properties,
                    state_properties,
                    measurements,
                },
                last_state: None,
            },
        );
    }

    pub fn update(&mut self, report: &MachinesReport) {
        for record in &report.config_property_records {
            let Some(entry) = self.registry.get_mut(&record.machine) else {
                // no machine registered under that uid
                continue;
            };

            let Some(info) = entry.instance.config_properties.get_mut(&record.path) else {
                // not defined in schema
                continue;
            };

            if let ConfigPropertyEvent::Registered {
                default,
                capability,
                constraints,
            } = record.event.clone()
            {
                *info = Some(ConfigPropertyInfo {
                    value: default.clone(),
                    default,
                    capability,
                    constraints,
                    records: Vec::default(),
                });

                continue;
            }

            let info = info.as_mut().expect("Property should be registered now...");

            match record.event.clone() {
                ConfigPropertyEvent::Registered { .. } => {
                    unreachable!()
                }

                ConfigPropertyEvent::DefaultChanged(value) => {
                    info.default = value;
                }

                ConfigPropertyEvent::CapabilityChanged(value) => {
                    info.capability = value;
                }

                ConfigPropertyEvent::ConstraintsChanged(value) => {
                    info.constraints = value;
                }

                ConfigPropertyEvent::Written { value, outcome, .. } => {
                    if !matches!(
                        outcome,
                        ConfigPropertyWriteOutcome::Accepted { changed: true }
                    ) {
                        continue;
                    }

                    info.value = value;
                }
            }
        }

        for record in &report.state_property_records {
            let Some(entry) = self.registry.get_mut(&record.machine) else {
                // no machine registered under that uid
                continue;
            };

            let Some(info) = entry.instance.state_properties.get_mut(&record.path) else {
                // not defined in schema
                continue;
            };

            if let StatePropertyEvent::Registered { value } = record.event.clone() {
                *info = Some(StatePropertyInfo {
                    value,
                    records: Default::default(),
                });

                continue;
            }

            match record.event.clone() {
                StatePropertyEvent::Registered { value }
                | StatePropertyEvent::ValueChanged { value } => {
                    *info = Some(StatePropertyInfo {
                        value,
                        records: Default::default(),
                    })
                }
            }
        }

        for snapshot in &report.measurement_snapshots {
            let Some(entry) = self.registry.get_mut(&snapshot.machine) else {
                // no machine registered under that uid
                continue;
            };

            let Some(info) = entry.instance.measurements.get_mut(&snapshot.path) else {
                // not defined in schema
                continue;
            };

            *info = Some(MeasurementInfo {
                value: snapshot.value,
            });
        }

        for (ident, entry) in &mut self.registry {
            let Some(adapter) = adapter::get(ident.machine) else {
                continue;
            };

            // --- emit state event ---
            if let Some(data) = (adapter.init_state_event)(&entry.instance, false)
                && entry.last_state.as_ref() != Some(&data)
            {
                let event = SocketIOEvent::new("StateEvent", data.clone());
                Self::broadcast(&mut entry.sockets, event);
                entry.last_state = Some(data);
            }

            // --- emit live values ---
            if let Some(data) = (adapter.init_measurements_event)(&entry.instance) {
                let event = SocketIOEvent::new("LiveValuesEvent", data);
                Self::broadcast(&mut entry.sockets, event);
            }
        }
    }

    pub fn add_socket(&mut self, socket: SocketRef) {
        let ident = match machine_namespace_path_to_ident(socket.ns()) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("Failed to parse NamespaceId: {}", e);

                if let Err(e) = socket.disconnect() {
                    tracing::error!("Failed to disconnect Socket: {}", e);
                }

                return;
            }
        };

        let Some(entry) = self.registry.get_mut(&ident) else {
            tracing::error!("Failed to add socket {ident}: No such machine");

            if let Err(e) = socket.disconnect() {
                tracing::error!("Failed to dsiconnect Socket: {}", e);
            }

            return;
        };

        tracing::info!("Adding new main namespace socket!");

        // --- send the current state ---
        if let Some(adapter) = adapter::get(ident.machine)
            && let Some(data) = (adapter.init_state_event)(&entry.instance, true)
        {
            let event = SocketIOEvent::new("StateEvent", data);

            if let Err(e) = socket.emit("event", &event) {
                tracing::error!("Failed to send state to new socket: {e}");
                return;
            }
        }

        // --- store the socket ---
        entry.sockets.push(socket);
    }

    fn broadcast(sockets: &mut Vec<SocketRef>, event: SocketIOEvent) {
        sockets.retain(|socket| {
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

#[derive(Default, Clone)]
pub struct Entry {
    sockets: Vec<SocketRef>,
    instance: MachineInstance,
    last_state: Option<serde_json::Value>,
}

pub fn machine_namespace_path_to_ident(s: &str) -> Result<MachineInstanceIdentification, String> {
    if let Some(machine_path) = s.strip_prefix("/machine/") {
        let parts: Vec<&str> = machine_path.split('/').collect();
        if parts.len() == 3 {
            let vendor_id = parts[0]
                .parse::<u16>()
                .map_err(|_| "Invalid vendor id".to_string())?;
            let machine_id = parts[1]
                .parse::<u16>()
                .map_err(|_| "Invalid machine id".to_string())?;
            let serial = parts[2]
                .parse::<u16>()
                .map_err(|_| "Invalid serial id".to_string())?;

            return Ok(MachineInstanceIdentification {
                machine: MachineIdentification {
                    vendor_id,
                    machine_id,
                },
                serial,
            });
        }
    }

    Err(format!("Invalid namespace path: {}", s))
}
