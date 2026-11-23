use axum::Router;
use axum::routing::post;

use crate::api::legacy::LegacyApiState;

pub mod history;
pub mod machine_mutate;
pub mod markers;
pub mod modbus;
pub mod write_machine_device_identification;

pub fn router() -> Router<LegacyApiState> {
    Router::new()
        .route(
            "/write_machine_device_identification",
            post(write_machine_device_identification::post),
        )
        .route("/machine/mutate", post(machine_mutate::post))
        .route("/machine/history", post(history::post))
        .route("/machine/markers", post(markers::list))
        .route("/machine/markers/add", post(markers::add))
        .route("/machine/markers/remove", post(markers::remove))
        .route("/machine/markers/clear", post(markers::clear))
        .route("/modbus/scan", post(modbus::scan))
        .route(
            "/write_modbus_device_assignment",
            post(modbus::write_assignment),
        )
}
