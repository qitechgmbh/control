use std::io::Error;
use std::sync::Arc;

use socketioxide::layer::SocketIoLayer;
use tower_http::cors::CorsLayer;

use crate::app_state::AppState;

// use super::handlers::x::post_x;
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
        // .route("/api/v1/x", post(post_x))
        .layer(socketio_layer)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
