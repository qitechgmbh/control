use anyhow::Result;
use axum::Json;
use axum::body::Body;
use axum::extract::State;
use axum::http::Response;
use axum::routing::post;
use machine_implementations::MachineMessage;
use machine_implementations::machine_identification::{DeviceHardwareIdentificationEthercat, DeviceMachineIdentification, QiTechMachineIdentificationUnique};
use response_util::{ResponseUtil, ResponseUtilError};
use rest_api::rest_api_router;
use serde::Serialize;
use serde_json::Value;
use socketio::init::init_socketio;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;
use std::fmt::Debug;

use crate::SharedAppState;
pub mod socketio;
pub mod rest_api;
pub mod response;
pub mod response_util;

#[derive(Debug, Serialize, Clone)]
pub struct MutationResponse {
    pub success: bool,
    pub error: Option<String>,
}

impl MutationResponse {
    pub const fn success() -> Self {
        Self {
            success: true,
            error: None,
        }
    }
    pub const fn error(error: String) -> Self {
        Self {
            success: false,
            error: Some(error),
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct MachineMutationBody<T>
where
    T: Debug,
{
    pub machine_identification_unique: QiTechMachineIdentificationUnique,
    pub data: T,
}



#[derive(serde::Deserialize, Debug)]
pub struct MachineDeviceInfoRequest {
    pub device_machine_identification: DeviceMachineIdentification,
    pub hardware_identification_ethercat: DeviceHardwareIdentificationEthercat,
}

pub async fn post_write_machine_device_identification(
    State(app_state): State<Arc<SharedAppState>>,
    Json(body): Json<MachineDeviceInfoRequest>,
) -> Response<axum::body::Body> {
    /*
    let res = app_state
        .rt_machine_creation_channel
        .send(HotThreadMessage::WriteMachineDeviceInfo(
            body,
        ))
        .await;

    match res {
        Ok(_) => (),
        Err(e) => tracing::error!(
            "Failed to send HotThreadMessage::WriteMachineDeviceInfo {}",
            e
        ),
    }*/

    ResponseUtil::ok(MutationResponse::success())
}


async fn post_machine_mutate(
    State(app_state): State<Arc<SharedAppState>>,
    Json(body): Json<MachineMutationBody<Value>>,
) -> Response<Body> {
    tracing::info!(
        "Mutating machine machine={} data={:?}",
        body.machine_identification_unique,
        body.data,
    );

    let span = tracing::info_span!("machine_mutate", machine = %body.machine_identification_unique);
    let _span = span.enter();

    let res = match app_state
        .machines_with_channel
        .get(&body.machine_identification_unique)
    {
        Some(sender) => {
            let res = sender
                .clone()
                .send(MachineMessage::HttpApiJsonRequest(
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
    };

    match res {
        Ok(_) => ResponseUtil::ok(MutationResponse::success()),
        Err(e) => ResponseUtilError::Error(e).into(),
    }
}


pub async fn init_api(app_state: Arc<SharedAppState>) -> Result<()> {
    let cors = CorsLayer::permissive();
    let socketio_layer = init_socketio(app_state.clone()).await;

    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::DEBUG))
        .on_request(DefaultOnRequest::new().level(Level::TRACE))
        .on_response(DefaultOnResponse::new().level(Level::TRACE));

    let app = axum::Router::new()
        .route(
            "/api/v1/write_machine_device_identification",
            post(post_write_machine_device_identification),
        )
        .route("/api/v1/machine/mutate", post(post_machine_mutate))
        .nest("/api/v2", rest_api_router())
        .layer(socketio_layer)
        .layer(cors)
        .layer(trace_layer)
        .with_state(app_state.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001")
        .await
        .expect("Failed to bind to port 3001");

    tracing::info!("HTTP server running on 0.0.0.0:3001");

    axum::serve(listener, app)
        .await
        .map_err(|e| anyhow::anyhow!("Server error: {}", e))
}
