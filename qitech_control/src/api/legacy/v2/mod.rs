use axum::Router;
use axum::routing::post;
use qitech_framework_hub::ActorContext;

pub mod machine_mutate;

pub fn router() -> Router<ActorContext> {
    Router::new()
        // .route(
        //     "/write_machine_device_identification",
        //     post(write_machine_device_identification::post),
        // )
        .route("/machine/mutate", post(machine_mutate::post))
}
