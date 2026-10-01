use axum::Router;
use axum::routing::post;
use crate::api::legacy::LegacyApiState;

pub mod machine_mutate;
pub mod modbus;
pub mod write_machine_device_identification;

pub fn router() -> Router<LegacyApiState> {
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
