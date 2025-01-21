use std::io::Error;
use std::sync::Arc;

use axum::routing::get;

use crate::app_state::AppState;

use super::ethercat::get_ethercat;

pub async fn init_api(app_state: Arc<AppState>) -> Result<(), Error> {
    // make axum server to serve the data on /ethercat
    let app = axum::Router::new()
        .route("/api/v1/ethercat", get(get_ethercat))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
