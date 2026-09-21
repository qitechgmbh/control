use axum::Router;
use axum::routing::post;
use qitech_framework::MachineInstanceIdentification;
use qitech_framework_hub::ActorContext;
use tokio::sync::mpsc;

pub mod machine_mutate;
mod response_util;
pub mod write_machine_device_identification;

pub fn router() -> Router<(ActorContext, mpsc::Sender<MachineInstanceIdentification>)> {
    Router::new()
        .route(
            "/write_machine_device_identification",
            post(write_machine_device_identification::post),
        )
        .route("/machine/mutate", post(machine_mutate::post))
}
