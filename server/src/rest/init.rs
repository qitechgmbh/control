use super::handlers::machine_mutation::post_machine_mutate;
use super::handlers::write_machine_device_identification::post_write_machine_device_identification;
use crate::app_state::AppState;
use axum::routing::post;
use socketioxide::layer::SocketIoLayer;
use std::io::Error;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

pub async fn init_api(
    app_state: Arc<AppState>,
    socketio_layer: SocketIoLayer,
) -> Result<(), Error> {
    // allow all CORS requests
    let cors = CorsLayer::permissive();

    //setup logging
    // TODO: this codes makes etehrcrab crash
    // tracing_subscriber::fmt()
    //     .with_max_level(tracing::Level::DEBUG)
    //     .init();

    // make axum server to serve the data on /ethercat
    let app = axum::Router::new()
        .route(
            "/api/v1/write_machine_device_identification",
            post(post_write_machine_device_identification),
        )
        .route("/api/v1/machine/mutate", post(post_machine_mutate))
        .layer(socketio_layer)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    axum::serve(listener, app).await?;

    open::that("http://localhost:3001")?;

    Ok(())
}
