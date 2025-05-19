use crate::{app_state::AppState, rest::util::ResponseUtil};
use axum::{Json, extract::State, http::Response};
use control_core::{
    ethercat::eeprom_identification::write_machine_device_identification,
    machines::identification::{DeviceHardwareIdentificationEthercat, DeviceMachineIdentification},
    rest::mutation::MutationResponse,
};
use std::sync::Arc;

#[derive(serde::Deserialize, Debug)]
pub struct Body {
    pub device_machine_identification: DeviceMachineIdentification,
    pub hardware_identification_ethercat: DeviceHardwareIdentificationEthercat,
}

#[axum::debug_handler]
pub async fn post_write_machine_device_identification(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<Body>,
) -> Response<axum::body::Body> {
    let ethercat_setup_guard = app_state.ethercat_setup.read().await;
    let ethercat_setup = match ethercat_setup_guard.as_ref() {
        Some(setup) => setup,
        None => return ResponseUtil::error("EthercatSetup not initialized"),
    };

    let subdevice = match ethercat_setup.group.subdevice(
        &ethercat_setup.maindevice,
        body.hardware_identification_ethercat.subdevice_index,
    ) {
        Ok(subdevice) => subdevice,
        Err(_) => return ResponseUtil::not_found("SubDevice not found"),
    };

    match write_machine_device_identification(
        &subdevice,
        &ethercat_setup.maindevice,
        &body.device_machine_identification,
    )
    .await
    {
        Ok(_) => {}
        Err(e) => return ResponseUtil::error(e.to_string().as_str()),
    }

    ResponseUtil::ok(MutationResponse::success())
}
