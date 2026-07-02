use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::api::{property::{LIMIT_DEFAULT, LIMIT_MAXIMUM}, types::{Aggregation, AggregationOperation, Interval, Ordering, TimeSpan}};

#[derive(Deserialize)]
pub struct PropertyQuery {
    /// Start timestamp.
    #[serde(default, with = "clickhouse::serde::chrono::datetime64::millis::option")]
    pub from: Option<DateTime<Utc>>,

    /// End timestamp.
    #[serde(default, with = "clickhouse::serde::chrono::datetime64::millis::option")]
    pub to: Option<DateTime<Utc>>,

    /// Read the last x duration (s, m, d, h supported).
    pub last: Option<Interval>,

    /// Interval for aggregation
    pub interval: Option<Interval>,

    /// Aggregation method
    #[serde(rename = "aggregate")]
    pub operation: Option<AggregationOperation>,

    /// Result ordering
    pub ordering: Option<Ordering>,

    /// Maximum number of samples to return.
    pub limit: Option<u64>,

    // /// Response format.
    // pub format: Option<ResponseFormat>,
}

impl PropertyQuery {
    pub fn get_time_span(
        &self,
        now: DateTime<Utc>,
    ) -> Result<TimeSpan, String> {
        match (self.last.clone(), self.from, self.to) {
            // last only
            (Some(last), None, None) => {
                let duration = last.to_duration();
                Ok(TimeSpan::new(Some(now - duration), None))
            }
            // from and to
            (None, Some(from), Some(to)) => Ok(TimeSpan::new(Some(from), Some(to))),
            // from only
            (None, Some(from), None) => Ok(TimeSpan::new(Some(from), None)),
            // to only
            (None, None, Some(to)) => Ok(TimeSpan::new(None, Some(to))) ,
            // nothing
            (None, None, None) => Ok(TimeSpan::new(None, None)),
            // invalid combinations
            (Some(_), _, _) => {
                Err("'last' cannot be combined with 'from' or 'to'".into())
            }
        }
    }

    pub fn get_aggregation(&self) -> Result<Option<Aggregation>, String> {
        match (self.operation, self.interval.clone()) {
            // missing interval
            (Some(_), None) => Err("'aggregate' requires 'interval'".into()),
            // missing aggregate
            (None, Some(_)) => {
                Err("'interval' requires 'aggregate'".into())
            }
            // ok
            (Some(operation), Some(interval)) => {
                Ok(Some(Aggregation { operation, interval }))
            },
            // no aggregate requested
            _ => Ok(None),
        }
    }

    pub fn get_ordering(&self) -> Ordering {
        self.ordering.unwrap_or(Ordering::Ascending)
    }

    pub fn get_limit(&self) -> u64 {
        self.limit.unwrap_or(LIMIT_DEFAULT).min(LIMIT_MAXIMUM)
    }
}
