use axum::{
    body::Body,
    http::{Response, StatusCode},
};
use serde_json::json;

pub fn build_error_response(message: &str) -> Response<Body> {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header("Content-Type", "application/json")
        .body(Body::from(
            serde_json::to_string(&json!({ "error": message })).unwrap(),
        ))
        .unwrap()
}
