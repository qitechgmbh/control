use std::{io, sync::Arc};
use axum::routing::get;

use crate::SharedState;

mod utils;
mod types;
mod property;
mod machines;
mod machine;
mod excel;

pub struct Config {
    pub address: String,
}

pub async fn run(state: SharedState, config: Config) -> io::Result<()> {
    let app = axum::Router::new()
        .route("/api/v2/machines", get(machines::handle))
        .route("/api/v2/machines/{name}/{serial}", get(machine::handle))
        .route("/api/v2/machines/{name}/{serial}/{property_name}", get(property::handle))
        .with_state(Arc::new(state));

    let listener = tokio::net::TcpListener::bind(&config.address).await?;
    println!("[RestApi] Listening on {}", &config.address);
    axum::serve(listener, app).await?;
    Ok(())
}


