use super::MutationResponse;
use crate::{
    app_state::AppState,
    ethercat::device_identification::{
        write_machine_device_identification, MachineDeviceIdentification,
    },
    rest::util::QuickResponse,
};
use axum::{body::Body, extract::State, http::Response, Json};
use std::sync::Arc;

#[axum::debug_handler]
pub async fn post_write_machine_device_identification(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<MachineDeviceIdentification>,
) -> Response<Body> {
    let ethercat_setup_guard = app_state.ethercat_setup.read().await;
    let ethercat_setup = match ethercat_setup_guard.as_ref() {
        Some(setup) => setup,
        None => return QuickResponse::error("EthercatSetup not initialized"),
    };

    let subdevice = match ethercat_setup
        .group
        .subdevice(&ethercat_setup.maindevice, body.subdevice_index)
    {
        Ok(subdevice) => subdevice,
        Err(_) => return QuickResponse::not_found("SubDevice not found"),
    };

    match write_machine_device_identification(&subdevice, &ethercat_setup.maindevice, &body).await {
        Ok(_) => {}
        Err(e) => return QuickResponse::error(e.to_string().as_str()),
    }

    QuickResponse::ok(MutationResponse::success())
}
