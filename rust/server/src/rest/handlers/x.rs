use crate::{app_state::AppState, ethercat::util::find_device, rest::util::QuickResponse};
use axum::{body::Body, extract::State, http::Response, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize, Clone)]
pub struct XRequest {
    pub address: u16,
}

#[derive(Debug, Serialize, Clone)]
struct XResponse {
    pub x2000: u8,
}

// #[axum::debug_handler]
// pub async fn post_x(
//     State(app_state): State<Arc<AppState>>,
//     Json(payload): Json<XRequest>,
// ) -> Response<Body> {
//     let master_guard = app_state.as_ref().ethercat_master.read();
//     let master = match master_guard.as_ref() {
//         Some(device) => device,
//         None => return QuickResponse::error("MainDevice not initialized"),
//     };

//     let mut ethercat_group_guard = app_state.ethercat_group.write();
//     let group = match ethercat_group_guard.as_mut() {
//         Some(group) => group,
//         None => return QuickResponse::error("SubDeviceGroup not initialized"),
//     };

//     let device = match find_device(group, master, payload.address).await {
//         Some(device) => device,
//         None => return QuickResponse::error("Device not found"),
//     };

//     let res = device.sdo_read::<u8>(0x2000, 0x00).await;
//     let x2000 = match res {
//         Ok(x) => x,
//         Err(e) => return QuickResponse::error(&format!("{:?}", e)),
//     };

//     QuickResponse::ok(XResponse { x2000: x2000 })
// }
