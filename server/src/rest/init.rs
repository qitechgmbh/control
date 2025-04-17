use super::handlers::machine_mutation::post_machine_mutate;
use super::handlers::write_machine_device_identification::post_write_machine_device_identification;
use crate::app_state::AppState;
use crate::panic::{send_panic, PanicDetails};
use crate::socketio::init::init_socketio;
use anyhow::anyhow;
use axum::routing::post;
use smol::channel::Sender;
use std::sync::Arc;
use std::thread::JoinHandle;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

pub fn init_api(
    thread_panic_tx: Sender<PanicDetails>,
    app_state: Arc<AppState>,
) -> Result<JoinHandle<()>, anyhow::Error> {
    std::thread::Builder::new()
        .name("ApiThread".to_string())
        .spawn(|| {
            send_panic("ApiThread", thread_panic_tx);

            let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
            rt.spawn(async {
                // allow all CORS requests
                let cors = CorsLayer::permissive();

                //setup logging
                // TODO: this codes makes etehrcrab crash
                // tracing_subscriber::fmt()
                //     .with_max_level(tracing::Level::DEBUG)
                //     .init();

                // creat socketio layer
                let socketio_layer = init_socketio().await;

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

                let listener = tokio::net::TcpListener::bind("0.0.0.0:3001")
                    .await
                    .expect("Failed to bind to port 3001");
                axum::serve(listener, app).await.expect("Failed to serve");
            });

            loop {
                std::thread::park();
            }
        })
        .map_err(|e| anyhow!("Failed to spawn API thread: {}", e))
}
