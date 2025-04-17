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

    let runtime = tokio::runtime::Runtime::new().unwrap();
    let socketio_layer = runtime.block_on(init_socketio());
    runtime.spawn(async {
        init_api(APP_STATE.clone(), socketio_layer)
            .await
            .expect("Failed to run init_api()");
    });

    let init_ethercat_result = init_ethercat(APP_STATE.clone());
    init_ethercat_result.expect("Failed to run init_ethercat()");
}
