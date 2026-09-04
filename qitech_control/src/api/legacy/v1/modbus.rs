use axum::Extension;
use axum::Json;
use axum::response::IntoResponse;
use qitech_framework::ModbusRtuAssignment;
use qitech_framework::runtime::modbus_rtu;
use serde::Deserialize;

use crate::api::legacy;
use crate::api::legacy::LegacySharedState;
use crate::api::legacy::types::ModbusDeviceAssignment;
use crate::api::legacy::v1::machine_mutate::MutationResponse;

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
        Some(assignment) => modbus_rtu::write_assignment(ModbusRtuAssignment {
            port: body.port,
            machine: assignment.machine_identification_unique.into(),
            slave_id: assignment.slave_id,
        }),
        None => modbus_rtu::remove_assignment(&body.port),
    };

    if let Err(e) = result {
        return Json(MutationResponse::error(e.to_string()));
    }

    broadcast_devices(&mut state_legacy);
    Json(MutationResponse::success())
}

fn broadcast_devices(state_legacy: &mut LegacySharedState) {
    let devices = modbus_rtu::list_modbus_devices()
        .into_iter()
        .map(legacy::ModbusDeviceMetadata::from)
        .collect();

    state_legacy
        .ns_main
        .update(|ns| ns.set_modbus_devices(devices));
}
