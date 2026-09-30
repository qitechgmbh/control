use axum::Router;
use axum::routing::post;
<<<<<<< HEAD

use crate::api::legacy::LegacyApiState;

pub mod machine_mutate;
pub mod modbus;
pub mod write_machine_device_identification;

pub fn router() -> Router<LegacyApiState> {
=======
use qitech_framework::MachineInstanceIdentification;
use qitech_framework_hub::ActorContext;
use tokio::sync::mpsc;

pub mod machine_mutate;
pub mod modbus;
mod response_util;
pub mod write_machine_device_identification;

pub fn router() -> Router<(ActorContext, mpsc::Sender<MachineInstanceIdentification>)> {
>>>>>>> 8e49141 (Jse control v2 (#1680))
    Router::new()
        .route(
            "/write_machine_device_identification",
            post(write_machine_device_identification::post),
        )
        .route("/machine/mutate", post(machine_mutate::post))
        .route("/modbus/scan", post(modbus::scan))
        .route(
            "/write_modbus_device_assignment",
            post(modbus::write_assignment),
        )
}
