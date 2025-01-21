use anyhow::{Error, Result};
use api::init::init_api;
use app_state::APP_STATE;
use env_logger::Env;
use ethercat::init::init_ethercat;

pub mod api;

pub mod app_state;

pub mod ethercat;

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    init_ethercat(APP_STATE.clone()).await;
    init_api(APP_STATE.clone()).await?;

    Ok(())
}
