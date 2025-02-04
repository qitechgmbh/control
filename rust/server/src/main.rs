use anyhow::{Error, Result};
use app_state::APP_STATE;
use env_logger::Env;
use ethercat::init::init_ethercat;
use rest::init::init_api;
use socketio::init::init_socketio;

pub mod app_state;
pub mod ethercat;
pub mod ethercat_drivers;
pub mod rest;
pub mod socketio;

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let socketio_layer = init_socketio().await;
    init_ethercat(APP_STATE.clone());
    init_api(APP_STATE.clone(), socketio_layer).await?;

    Ok(())
}
