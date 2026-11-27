//! Pieces shared by the ClickHouse-backed endpoints (history, export, markers, storage).

use axum::response::IntoResponse;
use axum::response::Response as AxumResponse;
use serde::Deserialize;

use crate::api::legacy::response_util::ResponseUtil;

/// Upper bound on measurement + target names per request.
pub const MAX_NAMES: usize = 64;

/// Finite measurement samples of one machine in a time range, as `timestamp, name, v`.
/// Binds, in order: identity, names, from (Unix ms, inclusive), to (Unix ms, exclusive).
pub const RAW_SAMPLES: &str = "
    SELECT timestamp, name, assumeNotNull(value) AS v
    FROM machine_measurements
    WHERE identity = ?
      AND name IN ?
      AND timestamp >= fromUnixTimestamp64Milli(?)
      AND timestamp <  fromUnixTimestamp64Milli(?)
      AND value IS NOT NULL
      AND isFinite(assumeNotNull(value))";

/// Result of a ClickHouse-backed handler; the error becomes `{ "error": … }`.
pub type ApiResult = Result<AxumResponse, ApiError>;

#[derive(Debug)]
pub enum ApiError {
    /// The request itself is invalid.
    BadRequest(&'static str),
    /// A ClickHouse query failed while doing `action` (e.g. `history query`).
    Db {
        action: &'static str,
        error: clickhouse::error::Error,
    },
}

impl ApiError {
    /// `map_err` adapter: `.await.map_err(ApiError::db("marker add"))?`.
    pub fn db(action: &'static str) -> impl FnOnce(clickhouse::error::Error) -> Self {
        move |error| Self::Db { action, error }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> AxumResponse {
        match self {
            Self::BadRequest(message) => ResponseUtil::error(message),
            Self::Db { action, error } => {
                tracing::error!(%error, "{action} failed");
                ResponseUtil::error(&format!("{action} failed: {error}"))
            }
        }
    }
}

/// A time range in Unix ms, `from` inclusive, `to` exclusive.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct TimeRange {
    pub from: i64,
    pub to: i64,
}

impl TimeRange {
    pub fn validate(&self) -> Result<(), ApiError> {
        if self.to <= self.from {
            return Err(ApiError::BadRequest("`to` must be greater than `from`"));
        }
        Ok(())
    }

    pub fn span(&self) -> i64 {
        self.to - self.from
    }
}

/// Rejects requests that ask for more than [`MAX_NAMES`] series.
pub fn check_name_count(count: usize) -> Result<(), ApiError> {
    if count > MAX_NAMES {
        return Err(ApiError::BadRequest("too many names requested"));
    }
    Ok(())
}
