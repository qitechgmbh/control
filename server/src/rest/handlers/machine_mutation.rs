use super::mutation::MachineMutationBody;
use super::mutation::MutationResponse;
use crate::{
    app_state::SharedState,
    rest::util::{ResponseUtil, ResponseUtilError},
};
use axum::{Json, body::Body, extract::State, http::Response};
use serde_json::Value;
use std::sync::Arc;

#[axum::debug_handler]
pub async fn post_machine_mutate(
    State(app_state): State<Arc<SharedState>>,
    Json(body): Json<MachineMutationBody<Value>>,
) -> Response<Body> {
    let result = _post_machine_mutate(State(app_state), Json(body)).await;
    match result {
        Ok(_) => ResponseUtil::ok(MutationResponse::success()),
        Err(e) => ResponseUtilError::Error(e).into(),
    }
}

async fn _post_machine_mutate(
    State(app_state): State<Arc<SharedState>>,
    Json(body): Json<MachineMutationBody<Value>>,
) -> Result<(), anyhow::Error> {
    tracing::info!(
        "Mutating machine machine={} data={:?}",
        body.machine_identification_unique,
        body.data,
    );

    let span = tracing::info_span!("machine_mutate", machine = %body.machine_identification_unique);
    let _span = span.enter();

    match app_state
        .api_machines
        .lock()
        .await
        .get(&body.machine_identification_unique)
    {
        Some(sender) => {
            let res = sender
                .clone()
                .send(machines::MachineMessage::HttpApiJsonRequest(
                    body.data.clone(),
                ))
                .await;
            match res {
                Ok(_) => (),
                Err(e) => tracing::error!(
                    "[{}::_post_machine_mutate] Sending MachineMessage::HttpApiJsonRequest to {} failed {}",
                    module_path!(),
                    body.machine_identification_unique,
                    e
                ),
            };
            Ok(())
        }

        None => Err(anyhow::anyhow!(
            "[{}::_post_machine_mutate] Machine api_mutate error {} {}",
            module_path!(),
            "No Machine found with id: ",
            body.machine_identification_unique
        )),
    }
}
