use std::sync::Arc;

use crate::app_state::AppState;

use super::setup::launch_pdu_loop;

pub async fn init_ethercat(app_state: Arc<AppState>) {
    let interface = "en10";

    // scan ethercat task
    tokio::spawn(launch_pdu_loop(&interface, app_state.clone()));
}
