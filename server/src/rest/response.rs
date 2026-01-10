use axum::{Json, body::Body, http::StatusCode};
use serde_json::json;

pub enum ApiError {
    ErrBadRequest(String),
    ErrNotFound(String),
    ErrInternal(String),
}

impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let json = match self {
            Self::ErrBadRequest(ref e) => serde_json::to_string(&json!({ "error_bad_request": e })),
            Self::ErrNotFound(ref e) => serde_json::to_string(&json!({ "error_not_found": e })),
            Self::ErrInternal(ref e) => serde_json::to_string(&json!({ "error_internal": e })),
        };

        let body = match json {
            Ok(s) => Body::from(s),
            Err(_) => Body::empty(),
        };

        let status = match self {
            Self::ErrBadRequest(_) => StatusCode::BAD_REQUEST,
            Self::ErrNotFound(_) => StatusCode::NOT_FOUND,
            Self::ErrInternal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        axum::response::Response::builder()
            .status(status)
            .header("Content-Type", "application/json")
            .body(body)
            .expect("Failed to build error response")
    }
}

pub type Result<T> = axum::response::Result<Json<T>, ApiError>;

pub fn json<T>(t: T) -> Result<T> {
    Result::Ok(Json(t))
}

pub fn bad_request<E: ToString>(e: E) -> ApiError {
    ApiError::ErrBadRequest(e.to_string())
}

pub fn not_found<E: ToString>(e: E) -> ApiError {
    ApiError::ErrNotFound(e.to_string())
}

pub fn internal_error<E: ToString>(e: E) -> ApiError {
    ApiError::ErrInternal(e.to_string())
}
