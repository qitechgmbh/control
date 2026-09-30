use axum::Json;
use axum::body::Body;
use axum::extract::State;
use axum::http::Response;
<<<<<<< HEAD
use qitech_framework::RuntimeRequestKind;
use serde::Deserialize;

use crate::api::legacy::LegacyApiState;
use crate::api::legacy::response_util::ResponseUtil;
use crate::api::legacy::types::LegacyMachineIdentificationUnique;
use crate::api::legacy::v1::machine_mutate::MutationResponse;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct DeviceMachineIdentification {
    pub machine_identification_unique: LegacyMachineIdentificationUnique,
    pub role: u16,
}

#[derive(Deserialize, Debug)]
pub struct Request {
    pub device_machine_identification: DeviceMachineIdentification,
    pub hardware_identification_ethercat: DeviceHardwareIdentificationEthercat,
=======
use axum::http::StatusCode;
use axum::response::IntoResponse;
use qitech_framework::MachineInstanceIdentification;
use qitech_framework::RuntimeRequestKind;
use qitech_framework::ident::DeviceMachineAssignment;
use qitech_framework_hub::ActorContext;
use serde::Deserialize;
use tokio::sync::mpsc;

#[derive(Deserialize, Debug)]
pub struct Request {
    pub ident_device: DeviceMachineAssignment,
    pub ident_hardware: DeviceHardwareIdentificationEthercat,
>>>>>>> 8e49141 (Jse control v2 (#1680))
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct DeviceHardwareIdentificationEthercat {
    pub subdevice_index: usize,
}

pub async fn post(
<<<<<<< HEAD
    State(state): State<LegacyApiState>,
    Json(body): Json<Request>,
) -> Response<Body> {
    let res = state
        .ctx
        .send_request(RuntimeRequestKind::WriteMachineDeviceInfo {
            machine_ident: body
                .device_machine_identification
                .machine_identification_unique
                .into(),
            role: body.device_machine_identification.role,
            subdevice_index: body.hardware_identification_ethercat.subdevice_index,
        })
        .await;

    if let Err(e) = res {
        ResponseUtil::error(&format!("{:?}", e))
    } else {
        ResponseUtil::ok(MutationResponse::success())
    }
=======
    State(state): State<(ActorContext, mpsc::Sender<MachineInstanceIdentification>)>,
    Json(body): Json<Request>,
) -> Response<Body> {
    let res = state
        .0
        .send_request(RuntimeRequestKind::WriteMachineDeviceInfo {
            machine_ident: body.ident_device.machine,
            role: body.ident_device.role,
            subdevice_index: body.ident_hardware.subdevice_index,
        });

    _ = res;

    (StatusCode::OK, ()).into_response()
>>>>>>> 8e49141 (Jse control v2 (#1680))
}
