use axum::Extension;
use axum::Json;
use axum::Router;
use axum::body::Body;
use axum::http::StatusCode;
use axum::routing;
use qitech_framework::MachineIdentification;
use qitech_framework::MachineInstanceIdentification;
use qitech_framework::machine::MachineDescriptor;
use serde::Serialize;
use serde_json::json;

use crate::api::legacy::LegacyApiState;
use crate::api::legacy::types::LegacyMachineIdentificationUnique;
use crate::machines::ExtruderV1;
use crate::machines::ExtruderV2;
use crate::machines::LaserV1;
use crate::machines::WinderV1_7031_Spool;
use crate::machines::WinderV1_Regular;

mod machine;
mod machines;

pub fn router() -> Router<LegacyApiState> {
    Router::new()
        .route("/machine", routing::get(machines::get))
        .merge(make_machine_router(WinderV1_Regular::IDENTIFICATION))
        .merge(make_machine_router(WinderV1_7031_Spool::IDENTIFICATION))
        .merge(make_machine_router(ExtruderV1::IDENTIFICATION))
        .merge(make_machine_router(ExtruderV2::IDENTIFICATION))
        .merge(make_machine_router(LaserV1::IDENTIFICATION))
}

fn make_machine_router(id: MachineIdentification) -> Router<LegacyApiState> {
    let slug = slug(id);
    let path = format!("/machine/{slug}/{{serial}}");

    println!("Exposing: {}", path);

    Router::new()
        .route(&path, routing::get(machine::get))
        .route(&path, routing::post(machine::post))
        .layer(Extension(id))
}

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

// --- entry ---
#[derive(Serialize, Debug, PartialEq)]
pub struct MachineEntry {
    legacy_id: LegacyMachineIdentificationUnique,
    serial: u16,
    vendor: String,
    slug: String,
    error: Option<String>,
}

impl MachineEntry {
    pub fn from_ident(ident: MachineInstanceIdentification) -> Self {
        let vendor = if ident.machine.vendor_id == 1 {
            "QiTech"
        } else {
            "N/A"
        }
        .to_string();

        let slug = match ident.machine {
            WinderV1_Regular::IDENTIFICATION => "winder_v1",
            WinderV1_7031_Spool::IDENTIFICATION => "winder_v1_7031",
            ExtruderV1::IDENTIFICATION => "extruder_v1",
            ExtruderV2::IDENTIFICATION => "extruder_v2",
            LaserV1::IDENTIFICATION => "laser_v1",
            _ => "N/A",
        }
        .to_string();

        MachineEntry {
            legacy_id: ident.into(),
            serial: ident.serial,
            vendor,
            slug,
            error: None,
        }
    }
}

pub fn slug(ident: MachineIdentification) -> String {
    match ident {
        WinderV1_Regular::IDENTIFICATION => "winder_v1",
        WinderV1_7031_Spool::IDENTIFICATION => "winder_v1_7031",
        ExtruderV1::IDENTIFICATION => "extruder_v1",
        ExtruderV2::IDENTIFICATION => "extruder_v2",
        LaserV1::IDENTIFICATION => "laser_v1",
        _ => "N/A",
    }
    .to_string()
}
