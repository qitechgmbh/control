use std::collections::HashMap;

use axum::Extension;
use axum::Json;
use axum::response::IntoResponse;
use qitech_framework::runtime::modbus_rtu;
use serde::Deserialize;

use crate::api::legacy;
use crate::api::legacy::LegacySharedState;
use crate::api::legacy::types::ModbusDeviceAssignment;
use crate::api::legacy::v1::machine_mutate::MutationResponse;
use crate::modbus::assignments;
use crate::modbus::assignments::ModbusRtuAssignment;

#[derive(Debug, Deserialize)]
pub struct WriteAssignmentRequest {
    pub port: String,
    /// `None` unassigns the port.
    pub device_machine_identification: Option<ModbusDeviceAssignment>,
}

pub async fn scan(Extension(mut state_legacy): Extension<LegacySharedState>) -> impl IntoResponse {
    broadcast_devices(&mut state_legacy);
    Json(MutationResponse::success())
}

pub async fn write_assignment(
    Extension(mut state_legacy): Extension<LegacySharedState>,
    Json(body): Json<WriteAssignmentRequest>,
) -> impl IntoResponse {
    let result = match body.device_machine_identification {
        Some(assignment) => assignments::write(ModbusRtuAssignment {
            port: body.port,
            machine: assignment.machine_identification_unique.into(),
            slave_id: assignment.slave_id,
        }),
        None => assignments::remove(&body.port),
    };

    if let Err(e) = result {
        return Json(MutationResponse::error(e.to_string()));
    }

    broadcast_devices(&mut state_legacy);
    Json(MutationResponse::success())
}

pub(crate) fn broadcast_devices(state_legacy: &mut LegacySharedState) {
    let devices = list_devices();

    state_legacy
        .ns_main
        .update(|ns| ns.set_modbus_devices(devices));
}

/// The union of physically discovered ports and stored assignments: a port that is assigned but
/// currently unplugged still shows up (`present: false`) so it can be reassigned or unassigned
/// from the UI.
fn list_devices() -> Vec<legacy::ModbusDeviceMetadata> {
    let mut assignments: HashMap<String, ModbusRtuAssignment> = assignments::read()
        .into_iter()
        .map(|a| (a.port.clone(), a))
        .collect();

    let mut devices: Vec<legacy::ModbusDeviceMetadata> = modbus_rtu::list_serial_ports()
        .into_iter()
        .map(|p| {
            // --- an assignment may be stored under any of this port's udev aliases; report it
            // under the key it was written with, so re-assigning overwrites instead of
            // duplicating ---
            let port = p
                .aliases
                .iter()
                .find(|alias| assignments.contains_key(*alias))
                .cloned()
                .unwrap_or(p.port);

            legacy::ModbusDeviceMetadata {
                assignment: assignments.remove(&port).map(assignment_to_wire),
                port,
                present: true,
                device_node: Some(p.device_node),
                by_id: p.by_id,
                description: p.description,
                usb_vid: p.usb_vid,
                usb_pid: p.usb_pid,
                usb_serial: p.usb_serial,
            }
        })
        .collect();

    // --- assignments left over have no matching physical port right now ---
    devices.extend(
        assignments
            .into_values()
            .map(|a| legacy::ModbusDeviceMetadata {
                port: a.port.clone(),
                present: false,
                device_node: None,
                by_id: None,
                description: None,
                usb_vid: None,
                usb_pid: None,
                usb_serial: None,
                assignment: Some(assignment_to_wire(a)),
            }),
    );

    devices
}

fn assignment_to_wire(assignment: ModbusRtuAssignment) -> ModbusDeviceAssignment {
    ModbusDeviceAssignment {
        machine_identification_unique: assignment.machine.into(),
        slave_id: assignment.slave_id,
    }
}
