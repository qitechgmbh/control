use crate::{app_state::SharedState, rest::util::ResponseUtil};
use axum::{Json, extract::State, http::Response};
use machines::machine_identification::{
    DeviceHardwareIdentificationEthercat, DeviceMachineIdentification,
};

use std::sync::Arc;

use super::mutation::MutationResponse;

#[derive(serde::Deserialize, Debug)]
pub struct MachineDeviceInfoRequest {
    pub device_machine_identification: DeviceMachineIdentification,
    pub hardware_identification_ethercat: DeviceHardwareIdentificationEthercat,
}

#[axum::debug_handler]
pub async fn post_write_machine_device_identification(
    State(app_state): State<Arc<SharedState>>,
    Json(body): Json<MachineDeviceInfoRequest>,
) -> Response<axum::body::Body> {
    let res = app_state
        .rt_machine_creation_channel
        .send(crate::app_state::HotThreadMessage::WriteMachineDeviceInfo(
            body,
        ))
        .await;

    match res {
        Ok(_) => (),
        Err(e) => tracing::error!(
            "Failed to send HotThreadMessage::WriteMachineDeviceInfo {}",
            e
        ),
    }

    ResponseUtil::ok(MutationResponse::success())
}
