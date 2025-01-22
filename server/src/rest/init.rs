use std::io::Error;
use std::sync::Arc;

use socketioxide::layer::SocketIoLayer;
use tower_http::cors::CorsLayer;

use crate::app_state::AppState;

pub async fn init_api(
    app_state: Arc<AppState>,
    socketio_layer: SocketIoLayer,
) -> Result<(), Error> {
    // allow all CORS requests
    let cors = CorsLayer::permissive();

    // make axum server to serve the data on /ethercat
    let app = axum::Router::new()
        // .route("/api/v1/ethercat", get(get_ethercat))
        .layer(socketio_layer)
        .layer(cors)
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
