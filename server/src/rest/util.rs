use axum::{
    body::Body,
    http::{Response, StatusCode},
};
use serde_json::json;

pub struct ResponseUtil {}

impl ResponseUtil {
    pub fn error(message: &str) -> Response<Body> {
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header("Content-Type", "application/json")
            .body(Body::from(
                serde_json::to_string(&json!({ "error": message })).unwrap(),
            ))
            .unwrap()
    }

    pub fn ok<T: serde::Serialize>(data: T) -> Response<Body> {
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&data).unwrap()))
            .unwrap()
    }

    pub fn not_found(message: &str) -> Response<Body> {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("Content-Type", "application/json")
            .body(Body::from(
                serde_json::to_string(&json!({ "error": message })).unwrap(),
            ))
            .unwrap()
    }
}

pub enum ResponseUtilError {
    Error(anyhow::Error),
    NotFound(anyhow::Error),
}

impl From<ResponseUtilError> for Response<Body> {
    fn from(error: ResponseUtilError) -> Self {
        match error {
            ResponseUtilError::Error(e) => ResponseUtil::error(&e.to_string()),
            ResponseUtilError::NotFound(e) => ResponseUtil::not_found(&e.to_string()),
        }
    }
}
