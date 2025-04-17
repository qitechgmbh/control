use app_state::APP_STATE;
use env_logger::Env;
use ethercat::init::init_ethercat;
use rest::init::init_api;
use socketio::init::init_socketio;

pub mod app_state;
pub mod ethercat;
pub mod machines;
pub mod rest;
pub mod socketio;

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let socketio_layer = smol::block_on(init_socketio());
    let rt = smol::Executor::new();
    let _ = rt.spawn(async {
        init_api(APP_STATE.clone(), socketio_layer).await.unwrap();
    });

    init_ethercat(APP_STATE.clone());
}
