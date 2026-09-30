use axum::Json;
use axum::body::Body;
use axum::extract::State;
use axum::http::Response;
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
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct DeviceHardwareIdentificationEthercat {
    pub subdevice_index: usize,
}

pub async fn post(
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
}
