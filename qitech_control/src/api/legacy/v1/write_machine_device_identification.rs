use axum::Json;
use axum::body::Body;
use axum::extract::State;
use axum::http::Response;
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
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct DeviceHardwareIdentificationEthercat {
    pub subdevice_index: usize,
}

pub async fn post(
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
}
