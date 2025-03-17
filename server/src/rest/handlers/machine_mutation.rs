use super::MutationResponse;
use crate::{
    app_state::AppState,
    ethercat::device_identification::MachineIdentificationUnique,
    rest::util::{ResponseUtil, ResponseUtilError},
};
use axum::{body::Body, extract::State, http::Response, Json};
use serde_json::Value;
use std::sync::Arc;

#[derive(Debug, serde::Deserialize)]
pub struct WriteMachineBody<T> {
    pub machine_identification_unique: MachineIdentificationUnique,
    pub data: T,
}

#[axum::debug_handler]
pub async fn post_machine_mutate(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<WriteMachineBody<Value>>,
) -> Response<Body> {
    let result = _post_machine_mutate(State(app_state), Json(body)).await;
    match result {
        Ok(_) => ResponseUtil::ok(MutationResponse::success()),
        Err(e) => ResponseUtilError::Error(e).into(),
    }
}

async fn _post_machine_mutate(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<WriteMachineBody<Value>>,
) -> Result<(), anyhow::Error> {
    let ethercat_setup_guard = app_state.ethercat_setup.read().await;
    let ethercat_setup = ethercat_setup_guard.as_ref().ok_or(anyhow::anyhow!(
        "[{}::_post_machine_mutate] No setup",
        module_path!()
    ))?;

    // find machine with given identification in hashmap
    let machine = ethercat_setup
        .machines
        .get(&body.machine_identification_unique)
        .ok_or(anyhow::anyhow!(
            "[{}::_post_machine_mutate] Machine not found {}, all machines: {:?}",
            module_path!(),
            body.machine_identification_unique,
            ethercat_setup.machines.keys()
        ))?;

    // check if machine has error
    let machine = machine.as_ref().or(Err(anyhow::anyhow!(
        "[{}::_post_machine_mutate] Machine has error",
        module_path!()
    )))?;

    // write data to machine
    let mut machine = machine.write().await;
    machine.api_mutate(body.data).or_else(|e| {
        Err(anyhow::anyhow!(
            "[{}::_post_machine_mutate] Machine api_mutate error: {}",
            module_path!(),
            e
        ))
    })?;

    Ok(())
}
