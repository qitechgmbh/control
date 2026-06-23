use std::{io, time::Duration};
mod exporter;
use clickhouse::Client;
use tokio::sync::broadcast;

use crate::shared_state::SharedState;

mod bridge;
use bridge::ControlBridge;

mod rest_api;
mod shared_state;
mod registry;

// TODO: use ENV vars for this
const SOCKET_PATH: &str = "/tmp/qitech_ctrl_hub.sock";
const DB_URL: &str = "http://localhost:8123";
const DB_USER: &str = "default";
// const DB_PWD: &str = "";
const DB_NAME: &str = "qitech_ctrl";

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut client = Client::default()
        .with_url(DB_URL)
        .with_user(DB_USER)
        // .with_password("")
        .with_database(DB_NAME);

    // TODO: create dedicated type for registry
    let mut registry = Default::default();
    registry::refresh(&mut client, &mut registry).await;

    let (tx, rx) = broadcast::channel(1024);

    let state = SharedState {
        client: client.clone(),
        snapshot_tx: tx,
        machine_registry: Default::default(),
    };

    // bridge sub system
    let bridge = ControlBridge::new(state.clone(), SOCKET_PATH)?;
    tokio::spawn(bridge.run());

    // exporter sub system
    let config = exporter::Config { export_interval: Duration::from_secs_f64(2.0) };
    tokio::spawn(exporter::run(client, rx, config));

    // rest api sub system
    let config = rest_api::Config { address: "0.0.0.0:3000" };
    tokio::spawn(rest_api::run(state.clone(), config));
    
    // grafana live sub system
    // TODO: implement 

    Ok(())
}
