use super::handlers::machine_mutation::post_machine_mutate;
use super::handlers::write_machine_device_identification::post_write_machine_device_identification;
use crate::app_state::AppState;
use crate::panic::{PanicDetails, send_panic};
use crate::socketio::init::init_socketio;
use anyhow::anyhow;
use axum::routing::post;
use smol::channel::Sender;
use std::sync::Arc;
use std::thread::JoinHandle;
use tower_http::cors::CorsLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;

pub fn init_api(
    thread_panic_tx: Sender<PanicDetails>,
    app_state: Arc<AppState>,
) -> Result<JoinHandle<()>, anyhow::Error> {
    std::thread::Builder::new()
        .name("api".to_string())
        .spawn(|| {
            send_panic(thread_panic_tx);

            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_io()
                .enable_time()
                .build()
                .expect("Failed to create runtime");

            rt.block_on(async {
                // allow all CORS requests
                let cors = CorsLayer::permissive();

                // creat socketio layer
                let socketio_layer = init_socketio(&app_state).await;

                // Create a more detailed trace layer for HTTP requests
                let trace_layer = TraceLayer::new_for_http()
                    .make_span_with(
                        DefaultMakeSpan::new()
                            .level(Level::DEBUG)
                            .include_headers(false),
                    )
                    .on_request(DefaultOnRequest::new().level(Level::TRACE))
                    .on_response(
                        DefaultOnResponse::new()
                            .level(Level::TRACE)
                            .include_headers(false),
                    );

                // make axum server to serve the data on /ethercat
                let app = axum::Router::new()
                    .route(
                        "/api/v1/write_machine_device_identification",
                        post(post_write_machine_device_identification),
                    )
                    .route("/api/v1/machine/mutate", post(post_machine_mutate))
                    .layer(socketio_layer)
                    .layer(cors)
                    .layer(trace_layer)
                    .with_state(app_state);

                let listener = tokio::net::TcpListener::bind("0.0.0.0:3001")
                    .await
                    .expect("Failed to bind to port 3001");

                tracing::info!("Starting HTTP server on 0.0.0.0:3001");
                axum::serve(listener, app).await.expect("Failed to serve");
            });
        })
        .map_err(|e| anyhow!("Failed to spawn API thread: {}", e))
}
