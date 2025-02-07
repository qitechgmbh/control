use std::io::Error;
use std::sync::Arc;
use std::time::Duration;

use crate::app_state::AppState;
use axum::extract::Path;
use axum::{
    body::Body,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
};
use include_dir::{include_dir, Dir, File};
use mime_guess::{mime, Mime};
use socketioxide::layer::SocketIoLayer;
use tower_http::cors::CorsLayer;

// use super::handlers::x::post_x;
use tower_http::trace::TraceLayer;

static FRONTEND_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../frontend/dist");

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
        .route(
            "/{*path}",
            get(|path| async { serve_asset(Some(path)).await }),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    axum::serve(listener, app).await?;

    open::that("http://localhost:3001")?;

    Ok(())
}

async fn serve_asset(path: Option<Path<String>>) -> impl IntoResponse {
    let serve_file =
        |file: &File, mime_type: Option<Mime>, cache: Duration, code: Option<StatusCode>| {
            Response::builder()
                .status(code.unwrap_or(StatusCode::OK))
                .header(
                    header::CONTENT_TYPE,
                    mime_type.unwrap_or(mime::TEXT_HTML).to_string(),
                )
                .header(
                    header::CACHE_CONTROL,
                    format!("max-age={}", cache.as_secs_f32()),
                )
                .body(Body::from(file.contents().to_owned()))
                .unwrap()
        };

    match path {
        Some(Path(path)) => serve_file(
            match FRONTEND_DIR.get_file(&path) {
                Some(file) => file,
                None => {
                    return Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(Body::empty())
                        .unwrap()
                }
            },
            mime_guess::from_path(&path).first(),
            Duration::from_secs(60 * 60 * 24 * 365),
            None,
        ),
        None => panic!(),
    }
}
