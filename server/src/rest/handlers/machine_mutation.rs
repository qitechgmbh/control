use crate::{
    app_state::AppState,
    rest::util::{ResponseUtil, ResponseUtilError},
};
use axum::{Json, body::Body, extract::State, http::Response};
use control_core::rest::mutation::{MachineMutationBody, MutationResponse};
use serde_json::Value;
use std::sync::Arc;

#[axum::debug_handler]
pub async fn post_machine_mutate(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<MachineMutationBody<Value>>,
) -> Response<Body> {
    let result = _post_machine_mutate(State(app_state), Json(body)).await;
    match result {
        Ok(_) => ResponseUtil::ok(MutationResponse::success()),
        Err(e) => ResponseUtilError::Error(e).into(),
    }
}

async fn _post_machine_mutate(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<MachineMutationBody<Value>>,
) -> Result<(), anyhow::Error> {
    // lock machines
    let machines_guard = app_state.machines.read().await;

    // find machine with given identification in hashmap
    let machine = machines_guard
        .get(&body.machine_identification_unique)
        .ok_or(anyhow::anyhow!(
            "[{}::_post_machine_mutate] Machine not found {:?}",
            module_path!(),
            body.machine_identification_unique,
        ))?;

    // check machine for error
    let machine = match machine {
        Ok(m) => m,
        Err(e) => {
            return Err(anyhow::anyhow!(
                "[{}::_post_machine_mutate] Machine has error: {}",
                module_path!(),
                e
            ));
        }
    };

    // log
    tracing::info!(
        "Mutating machine machine={} data={:?}",
        body.machine_identification_unique,
        body.data,
    );
    let span = tracing::info_span!("machine_mutate", machine = %body.machine_identification_unique);
    let _span = span.enter();

    // lock machine
    let mut machine_guard = machine.lock().await;

    // write data to machine
    machine_guard.api_mutate(body.data).map_err(|e| {
        anyhow::anyhow!(
            "[{}::_post_machine_mutate] Machine api_mutate error: {}",
            module_path!(),
            e
        )
    })?;

    Ok(())
}
