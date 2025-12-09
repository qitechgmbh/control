use anyhow::Result;
use axum::routing::post;
use std::sync::Arc;
use std::thread;
use tower_http::cors::CorsLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;

use super::handlers::machine_mutation::post_machine_mutate;
use super::handlers::write_machine_device_identification::post_write_machine_device_identification;
use crate::app_state::SharedState;
use crate::socketio::init::init_socketio;

use crate::rest::handlers::metrics::metrics_router;

async fn init_api(app_state: Arc<SharedState>) -> Result<()> {
    let cors = CorsLayer::permissive();
    let socketio_layer = init_socketio(app_state.clone()).await;

    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::DEBUG))
        .on_request(DefaultOnRequest::new().level(Level::TRACE))
        .on_response(DefaultOnResponse::new().level(Level::TRACE));

    let app = axum::Router::new()
        .route(
            "/api/v1/write_machine_device_identification",
            post(post_write_machine_device_identification),
        )
        .route("/api/v1/machine/mutate", post(post_machine_mutate))
        .nest("/api/v1/metrics", metrics_router())
        .layer(socketio_layer)
        .layer(cors)
        .layer(trace_layer)
        .with_state(app_state.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001")
        .await
        .expect("Failed to bind to port 3001");

    tracing::info!("HTTP server running on 0.0.0.0:3001");

    axum::serve(listener, app)
        .await
        .map_err(|e| anyhow::anyhow!("Server error: {}", e))
}

/// Starts the API server in its own thread with a single-threaded Tokio runtime
pub fn start_api_thread(app_state: Arc<SharedState>) -> std::thread::JoinHandle<()> {
    thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime");

        if let Err(err) = rt.block_on(init_api(app_state.clone())) {
            eprintln!("API server exited with error: {err:?}");
        }
    })
}
