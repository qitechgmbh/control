use std::fmt::Debug;

use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::response::Response as AxumResponse;
use qitech_framework::MachineIdentification;
use qitech_framework::MachineInstanceIdentification;
use qitech_framework_hub::ActorContext;
use serde::Deserialize;
use serde::Serialize;

use crate::api::legacy::adapter;
use crate::api::legacy::types::MachineIdentificationUnique;

#[derive(Debug, Deserialize)]
pub struct Request {
    pub machine_identification_unique: MachineIdentificationUnique,
    pub data: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct MutationResponse {
    pub success: bool,
    pub error: Option<String>,
}

impl MutationResponse {
    pub fn success() -> Self {
        Self {
            success: true,
            error: None,
        }
    }

    pub fn error(error: impl Into<String>) -> Self {
        Self {
            success: false,
            error: Some(error.into()),
        }
    }
}

pub async fn post(State(ctx): State<ActorContext>, Json(body): Json<Request>) -> AxumResponse {
    let ident = MachineInstanceIdentification {
        machine: MachineIdentification {
            vendor_id: body
                .machine_identification_unique
                .machine_identification
                .vendor,
            machine_id: body
                .machine_identification_unique
                .machine_identification
                .machine,
        },
        serial: body.machine_identification_unique.serial,
    };

    let Some(adapter) = adapter::get(ident.machine) else {
        return Json(MutationResponse::error("no_such_machine")).into_response();
    };

    let requests = match (adapter.convert_request)(ident, body.data) {
        Ok(requests) => requests,
        Err(error) => {
            return Json(MutationResponse::error(error.to_string())).into_response();
        }
    };

    // Sequential, fail-fast: a compound legacy mutation (e.g. autotune start) may need its writes
    // applied in order before a later request in the batch depends on them.
    for request in requests {
        match ctx.send_request(request).await {
            Ok(Ok(())) => {}

            Ok(Err(error)) => {
                return Json(MutationResponse::error(error.to_string())).into_response();
            }

            Err(error) => {
                tracing::error!(%error, "failed to send runtime request");
                return Json(MutationResponse::error("No runtime is currently connected"))
                    .into_response();
            }
        }
    }

    Json(MutationResponse::success()).into_response()
}

/*
pub struct RuntimeRequestHttpError(pub RuntimeRequestError);

impl IntoResponse for RuntimeRequestHttpError {
    fn into_response(self) -> AxumResponse {
        fn error_response(
            status: StatusCode,
            error: &'static str,
            message: impl Into<String>,
        ) -> Response {
            (
                status,
                Json(json!({
                    "error": error,
                    "message": message.into(),
                })),
            )
                .into_response()
        }

        fn resource_access_response(error: ResourceAccessError) -> Response {
            match error {
                ResourceAccessError::MachineNotFound => error_response(
                    StatusCode::NOT_FOUND,
                    "machine_not_found",
                    "The requested machine was not found",
                ),

                ResourceAccessError::ResourceNotFound { kind, path } => error_response(
                    StatusCode::NOT_FOUND,
                    "resource_not_found",
                    format!("Resource of kind `{kind}` at `{path}` was not found"),
                ),

                ResourceAccessError::TypeMismatch { expected, actual } => error_response(
                    StatusCode::BAD_REQUEST,
                    "resource_type_mismatch",
                    format!("Expected resource type `{expected}`, but got `{actual}`"),
                ),
            }
        }

        match self.0 {
            RuntimeRequestError::WriteMachineDeviceInfo(error) => error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "machine_device_info_write_failed",
                error.to_string(),
            ),

            RuntimeRequestError::MachineSetConfigProperty(error) => match error {
                MachineSetConfigProperty::ResourceAccess(error) => resource_access_response(error),

                MachineSetConfigProperty::WriteError(error) => match error {
                    ConfigPropertyWriteError::ValueTypeMismatch(err) => error_response(
                        StatusCode::BAD_REQUEST,
                        "config_property_value_type_mismatch",
                        err.to_string(),
                    ),

                    ConfigPropertyWriteError::NotWritable => error_response(
                        StatusCode::CONFLICT,
                        "config_property_not_writable",
                        "The requested configuration property is not writable",
                    ),

                    ConfigPropertyWriteError::ConstraintViolation(err) => error_response(
                        StatusCode::UNPROCESSABLE_ENTITY,
                        "config_property_constraint_violation",
                        err.to_string(),
                    ),
                },
            },

            RuntimeRequestError::MachineExecuteCommand(error) => match error {
                MachineExecuteCommandError::ResourceAccess(error) => {
                    resource_access_response(error)
                }

                MachineExecuteCommandError::ExecuteError(error) => error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "command_execution_failed",
                    error.to_string(),
                ),
            },

            RuntimeRequestError::MachineSubscribe(error) => match error {
                MachineSubscribeError::ResourceAccess(error) => resource_access_response(error),

                MachineSubscribeError::ProviderNotFound => error_response(
                    StatusCode::NOT_FOUND,
                    "provider_not_found",
                    "The requested provider was not found",
                ),

                MachineSubscribeError::SubscriberNotFound => error_response(
                    StatusCode::NOT_FOUND,
                    "subscriber_not_found",
                    "The requested subscriber was not found",
                ),

                MachineSubscribeError::AlreadySubscribed => error_response(
                    StatusCode::CONFLICT,
                    "already_subscribed",
                    "The subscriber is already subscribed",
                ),

                MachineSubscribeError::UnsupportedMachine => error_response(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "unsupported_machine",
                    "The machine does not support subscriptions",
                ),

                MachineSubscribeError::TooManySubscriptions => error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "too_many_subscriptions",
                    "The subscription limit has been reached",
                ),
            },

            RuntimeRequestError::MachineUnsubscribe(error) => match error {
                MachineUnsubscribeError::SubscriptionNotFound => error_response(
                    StatusCode::NOT_FOUND,
                    "subscription_not_found",
                    "The requested subscription was not found",
                ),
            },
        }
    }
}
*/
